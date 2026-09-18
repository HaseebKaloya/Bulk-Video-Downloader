use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, RANGE};
use tokio_util::sync::CancellationToken;
use crate::domain::DownloadTask;
use crate::errors::AppError;

pub struct DownloadProgress {
    pub task_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub progress: f64,
    pub speed_bytes_per_sec: f64,
    pub eta_seconds: Option<u64>,
}

pub struct DownloadWorker {
    client: reqwest::Client,
}

impl DownloadWorker {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .user_agent("BulkVideoDownloader/1.0.0")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn execute_download<F>(
        &self,
        task: &DownloadTask,
        cancel_token: CancellationToken,
        progress_callback: F,
    ) -> Result<PathBuf, AppError>
    where
        F: Fn(DownloadProgress) + Send + Sync + 'static,
    {
        let out_dir = Path::new(&task.output_directory);
        std::fs::create_dir_all(out_dir).map_err(|e| AppError::FilesystemError {
            code: "FS_DIR_CREATE_FAILED".into(),
            message: format!("Failed to create destination directory {:?}: {}", out_dir, e),
            action: "Verify directory write permissions.".into(),
        })?;

        let is_ytdlp = task.provider_id.as_deref() == Some("yt_dlp")
            || crate::providers::ytdlp::YtDlpProvider::is_supported_social_url(&task.source_url);

        if is_ytdlp {
            let clean_base_name = task
                .output_filename
                .as_deref()
                .and_then(|f| Path::new(f).file_stem().and_then(|s| s.to_str()))
                .unwrap_or(&task.id);

            let quality = task.selected_quality.as_deref().unwrap_or("best");
            let format = task.selected_extension.as_deref();
            return crate::providers::ytdlp::YtDlpProvider::execute_download(
                &task.id,
                &task.source_url,
                out_dir,
                clean_base_name,
                quality,
                format,
                cancel_token,
                progress_callback,
            );
        }

        // Determine filename
        let extension = task
            .selected_extension
            .as_deref()
            .unwrap_or("mp4")
            .trim_start_matches('.');
        let raw_filename = task.output_filename.clone().unwrap_or_else(|| {
            format!(
                "{}.{}",
                task.title.as_deref().unwrap_or(&task.id),
                extension
            )
        });

        let target_file_path = out_dir.join(&raw_filename);
        let partial_file_path = out_dir.join(format!("{}.bvd-partial", raw_filename));

        // Check if destination file already completed
        if target_file_path.exists() {
            let meta = std::fs::metadata(&target_file_path).ok();
            let final_size = meta.map(|m| m.len()).unwrap_or(0);
            if let Some(expected) = task.total_bytes {
                if expected > 0 && final_size >= expected {
                    return Ok(target_file_path);
                }
            }
        }

        // Check partial file for resumption
        let mut existing_bytes: u64 = 0;
        if partial_file_path.exists() {
            if let Ok(meta) = std::fs::metadata(&partial_file_path) {
                existing_bytes = meta.len();
            }
        }

        let mut headers = HeaderMap::new();
        if existing_bytes > 0 {
            let range_header = format!("bytes={}-", existing_bytes);
            if let Ok(val) = HeaderValue::from_str(&range_header) {
                headers.insert(RANGE, val);
            }
        }

        let response = self
            .client
            .get(&task.source_url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| AppError::NetworkError {
                code: "NET_REQUEST_FAILED".into(),
                message: format!("Failed to connect to source URL: {}", e),
                action: "Check internet connection and verify source URL is reachable.".into(),
                retryable: true,
            })?;

        let (response, mut existing_bytes) = if response.status().as_u16() == 416 {
            // Partial file is out of range or server changed; reset and re-download from start
            let _ = std::fs::remove_file(&partial_file_path);
            let fresh_response = self
                .client
                .get(&task.source_url)
                .send()
                .await
                .map_err(|e| AppError::NetworkError {
                    code: "NET_REQUEST_FAILED".into(),
                    message: format!("Failed to reconnect to source URL: {}", e),
                    action: "Check internet connection.".into(),
                    retryable: true,
                })?;
            (fresh_response, 0u64)
        } else {
            (response, existing_bytes)
        };

        let status = response.status();
        if !status.is_success() && status.as_u16() != 206 {
            return Err(AppError::NetworkError {
                code: format!("HTTP_{}", status.as_u16()),
                message: format!("Server returned error status {}", status),
                action: "Verify source link validity and authorization.".into(),
                retryable: status.is_server_error() || status.as_u16() == 429,
            });
        }

        let is_partial = status.as_u16() == 206;
        let mut file = if is_partial && existing_bytes > 0 {
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(&partial_file_path)
                .map_err(|e| AppError::FilesystemError {
                    code: "FS_FILE_OPEN_FAILED".into(),
                    message: format!("Failed to open partial file {:?}: {}", partial_file_path, e),
                    action: "Verify file permissions.".into(),
                })?
        } else {
            existing_bytes = 0;
            File::create(&partial_file_path).map_err(|e| AppError::FilesystemError {
                code: "FS_FILE_CREATE_FAILED".into(),
                message: format!("Failed to create partial file {:?}: {}", partial_file_path, e),
                action: "Verify directory write permissions.".into(),
            })?
        };

        if is_partial && existing_bytes > 0 {
            file.seek(SeekFrom::Start(existing_bytes))
                .map_err(|e| AppError::FilesystemError {
                    code: "FS_SEEK_FAILED".into(),
                    message: format!("Failed to seek partial file: {}", e),
                    action: "Retry download from beginning.".into(),
                })?;
        }

        // Calculate total length
        let content_length = response.content_length();
        let total_bytes: Option<u64> = if is_partial {
            content_length.map(|cl| cl + existing_bytes).or(task.total_bytes)
        } else {
            content_length.or(task.total_bytes)
        };

        let mut downloaded_bytes = existing_bytes;
        let mut stream = response.bytes_stream();

        let mut last_progress_time = Instant::now();
        let mut bytes_since_last_progress: u64 = 0;
        let mut smoothed_speed: f64 = 0.0;

        while let Some(chunk_result) = stream.next().await {
            if cancel_token.is_cancelled() {
                file.flush().ok();
                return Err(AppError::CancellationError {
                    code: "DOWNLOAD_PAUSED".into(),
                    message: "Download was paused or cancelled by user.".into(),
                    action: "Resume download when ready.".into(),
                });
            }

            let chunk = chunk_result.map_err(|e| AppError::NetworkError {
                code: "NET_CHUNK_READ_FAILED".into(),
                message: format!("Stream interrupted: {}", e),
                action: "Download will automatically resume via retry policy.".into(),
                retryable: true,
            })?;

            file.write_all(&chunk).map_err(|e| AppError::FilesystemError {
                code: "FS_WRITE_FAILED".into(),
                message: format!("Failed to write stream chunk to disk: {}", e),
                action: "Check disk space and write permissions.".into(),
            })?;

            let chunk_len = chunk.len() as u64;
            downloaded_bytes += chunk_len;
            bytes_since_last_progress += chunk_len;

            let elapsed = last_progress_time.elapsed();
            if elapsed >= Duration::from_millis(200) {
                let instant_speed = (bytes_since_last_progress as f64) / elapsed.as_secs_f64();
                if smoothed_speed == 0.0 {
                    smoothed_speed = instant_speed;
                } else {
                    smoothed_speed = (smoothed_speed * 0.7) + (instant_speed * 0.3);
                }

                let progress = if let Some(tot) = total_bytes {
                    if tot > 0 {
                        (downloaded_bytes as f64) / (tot as f64)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                let eta = if smoothed_speed > 0.0 {
                    total_bytes.and_then(|tot| {
                        if tot > downloaded_bytes {
                            Some(((tot - downloaded_bytes) as f64 / smoothed_speed) as u64)
                        } else {
                            Some(0)
                        }
                    })
                } else {
                    None
                };

                progress_callback(DownloadProgress {
                    task_id: task.id.clone(),
                    bytes_downloaded: downloaded_bytes,
                    total_bytes,
                    progress: progress.min(1.0),
                    speed_bytes_per_sec: smoothed_speed,
                    eta_seconds: eta,
                });

                last_progress_time = Instant::now();
                bytes_since_last_progress = 0;
            }
        }

        // Flush and validate
        file.flush().map_err(|e| AppError::FilesystemError {
            code: "FS_FLUSH_FAILED".into(),
            message: format!("Failed to flush file buffers to disk: {}", e),
            action: "Check storage health.".into(),
        })?;

        drop(file);

        // Safe atomic finalization: rename .bvd-partial to target filename
        std::fs::rename(&partial_file_path, &target_file_path).map_err(|e| {
            AppError::FilesystemError {
                code: "FS_RENAME_FAILED".into(),
                message: format!(
                    "Failed to finalize file from {:?} to {:?}: {}",
                    partial_file_path, target_file_path, e
                ),
                action: "Ensure target file is not open in another program.".into(),
            }
        })?;

        // Final 100% progress event
        progress_callback(DownloadProgress {
            task_id: task.id.clone(),
            bytes_downloaded: downloaded_bytes,
            total_bytes: Some(downloaded_bytes),
            progress: 1.0,
            speed_bytes_per_sec: 0.0,
            eta_seconds: Some(0),
        });

        Ok(target_file_path)
    }
}

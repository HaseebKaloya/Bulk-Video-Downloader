use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use url::Url;
use crate::downloader::DownloadProgress;
use crate::errors::AppError;
use crate::media::MediaProcessor;
use crate::providers::{MediaFormat, MediaMetadata, MediaProvider, SourceValidation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtDlpStatus {
    pub is_available: bool,
    pub version: Option<String>,
    pub binary_path: Option<String>,
    pub message: String,
}

pub struct YtDlpProvider;

impl YtDlpProvider {
    pub fn find_binary() -> PathBuf {
        let exe_name = if cfg!(target_os = "windows") {
            "yt-dlp.exe"
        } else {
            "yt-dlp"
        };

        // 1. Check next to running executable and Tauri resource folders
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                let candidates = [
                    exe_dir.join(exe_name),
                    exe_dir.join("bin").join(exe_name),
                    exe_dir.join("resources").join("bin").join(exe_name),
                    exe_dir.join("resources").join(exe_name),
                    exe_dir.join("..").join("Resources").join("bin").join(exe_name),
                    exe_dir.join("..").join("lib").join("bin").join(exe_name),
                    exe_dir.join("..").join("bin").join(exe_name),
                    exe_dir.join("..").join("..").join("bin").join(exe_name),
                ];
                for candidate in &candidates {
                    if candidate.is_file() {
                        return candidate.clone();
                    }
                }
            }
        }

        // 2. Check project / cwd directories
        if let Ok(cwd) = std::env::current_dir() {
            let candidates = [
                cwd.join(exe_name),
                cwd.join("bin").join(exe_name),
                cwd.join("src-tauri").join("bin").join(exe_name),
                cwd.join("..").join("bin").join(exe_name),
                cwd.join("..").join("src-tauri").join("bin").join(exe_name),
            ];
            for candidate in &candidates {
                if candidate.is_file() {
                    return candidate.clone();
                }
            }
        }

        // 3. Fallback to PATH
        PathBuf::from(exe_name)
    }

    pub fn check_status() -> YtDlpStatus {
        let bin = Self::find_binary();
        let mut cmd = Command::new(&bin);
        cmd.arg("--version");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let out = cmd.output();

        match out {
            Ok(o) if o.status.success() => {
                let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
                YtDlpStatus {
                    is_available: true,
                    version: Some(version),
                    binary_path: Some(bin.to_string_lossy().to_string()),
                    message: "yt-dlp is active and ready for YouTube, TikTok, and Instagram downloads.".to_string(),
                }
            }
            _ => YtDlpStatus {
                is_available: false,
                version: None,
                binary_path: None,
                message: "yt-dlp was not detected. Click 'Install / Update yt-dlp' to enable YouTube, TikTok, and Instagram extraction.".to_string(),
            },
        }
    }

    pub fn get_host_domain(url_str: &str) -> Option<String> {
        url::Url::parse(url_str)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
    }

    pub fn is_supported_social_url(url_str: &str) -> bool {
        if let Some(host) = Self::get_host_domain(url_str) {
            host.ends_with("youtube.com")
                || host == "youtu.be"
                || host.ends_with("tiktok.com")
                || host.ends_with("instagram.com")
                || host.ends_with("twitter.com")
                || host == "x.com"
                || host.ends_with(".x.com")
                || host.ends_with("facebook.com")
                || host.ends_with("fb.watch")
                || host.ends_with("fb.com")
                || host.ends_with("vimeo.com")
                || host.ends_with("reddit.com")
                || host.ends_with("dailymotion.com")
                || host.ends_with("twitch.tv")
        } else {
            false
        }
    }

    pub fn identify_platform(url_str: &str) -> &'static str {
        if let Some(host) = Self::get_host_domain(url_str) {
            if host.ends_with("youtube.com") || host == "youtu.be" {
                "YouTube"
            } else if host.ends_with("tiktok.com") {
                "TikTok (No Watermark)"
            } else if host.ends_with("instagram.com") {
                "Instagram"
            } else if host.ends_with("twitter.com") || host == "x.com" || host.ends_with(".x.com") {
                "X / Twitter"
            } else if host.ends_with("facebook.com") || host.ends_with("fb.watch") || host.ends_with("fb.com") {
                "Facebook"
            } else if host.ends_with("vimeo.com") {
                "Vimeo"
            } else if host.ends_with("reddit.com") {
                "Reddit"
            } else if host.ends_with("dailymotion.com") {
                "Dailymotion"
            } else if host.ends_with("twitch.tv") {
                "Twitch"
            } else {
                "Streaming Video"
            }
        } else {
            "Direct Stream"
        }
    }

    pub fn execute_download<F>(
        task_id: &str,
        source_url: &str,
        output_dir: &Path,
        output_filename: &str,
        quality_preference: &str,
        format_preference: Option<&str>,
        cancel_token: CancellationToken,
        progress_callback: F,
    ) -> Result<PathBuf, AppError>
    where
        F: Fn(DownloadProgress) + Send + Sync + 'static,
    {
        let yt_bin = Self::find_binary();
        let status = Self::check_status();
        if !status.is_available {
            return Err(AppError::ProviderError {
                code: "YTDLP_NOT_FOUND".into(),
                message: "yt-dlp binary is not installed or available on this system.".into(),
                action: "Please click 'Install / Update yt-dlp' in Settings.".into(),
            });
        }

        std::fs::create_dir_all(output_dir).map_err(|e| AppError::FilesystemError {
            code: "FS_CREATE_DIR_FAILED".into(),
            message: format!("Failed to create output directory: {}", e),
            action: "Check storage permissions.".into(),
        })?;

        // Format mapping logic based on user preferences
        let is_audio_only = quality_preference == "audio"
            || quality_preference.contains("audio")
            || format_preference.map_or(false, |f| {
                matches!(f.to_lowercase().as_str(), "mp3" | "m4a" | "aac" | "wav" | "flac" | "opus")
            });

        let is_video_only = quality_preference.ends_with("_video_only")
            || format_preference.map_or(false, |f| f == "video_only");

        let raw_quality = quality_preference
            .trim_end_matches("_video_only")
            .to_lowercase();

        let output_template = output_dir
            .join(format!("{}.%(ext)s", output_filename))
            .to_string_lossy()
            .to_string();

        let format_arg = if is_audio_only {
            "bestaudio/best".to_string()
        } else if is_video_only {
            match raw_quality.as_str() {
                "highest" => "bestvideo".to_string(),
                "2160p" | "4k" => "bestvideo[height<=2160]/bestvideo".to_string(),
                "1440p" | "2k" => "bestvideo[height<=1440]/bestvideo".to_string(),
                "1080p" => "bestvideo[height<=1080]/bestvideo".to_string(),
                "720p" => "bestvideo[height<=720]/bestvideo".to_string(),
                "480p" => "bestvideo[height<=480]/bestvideo".to_string(),
                "360p" => "bestvideo[height<=360]/bestvideo".to_string(),
                _ => "bestvideo".to_string(),
            }
        } else {
            match raw_quality.as_str() {
                "highest" => "bestvideo+bestaudio/best".to_string(),
                "2160p" | "4k" => "bestvideo[height<=2160]+bestaudio/best[height<=2160]/best".to_string(),
                "1440p" | "2k" => "bestvideo[height<=1440]+bestaudio/best[height<=1440]/best".to_string(),
                "1080p" => "bestvideo[height<=1080]+bestaudio/best[height<=1080]/best".to_string(),
                "720p" => "bestvideo[height<=720]+bestaudio/best[height<=720]/best".to_string(),
                "480p" => "bestvideo[height<=480]+bestaudio/best[height<=480]/best".to_string(),
                "360p" => "bestvideo[height<=360]+bestaudio/best[height<=360]/best".to_string(),
                _ => "bestvideo+bestaudio/best".to_string(),
            }
        };

        // Check if ffmpeg is available for merging streams
        let ffmpeg_status = MediaProcessor::check_ffmpeg();
        let mut cmd = Command::new(&yt_bin);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        cmd.arg("--no-playlist")
            .arg("--newline")
            .arg("--no-colors")
            .arg("--windows-filenames")
            .arg("--continue")
            .arg("--no-warnings")
            .arg("--format")
            .arg(&format_arg)
            .arg("--output")
            .arg(&output_template)
            .arg("--progress-template")
            .arg("download:DOWNLOAD:%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s|%(progress.downloaded_bytes)s|%(progress.total_bytes)s");

        if is_audio_only {
            let audio_ext = format_preference
                .filter(|&f| matches!(f.to_lowercase().as_str(), "mp3" | "m4a" | "aac" | "wav" | "flac" | "opus"))
                .unwrap_or("mp3");

            if ffmpeg_status.is_available {
                cmd.arg("-x")
                    .arg("--extract-audio")
                    .arg("--audio-format")
                    .arg(audio_ext)
                    .arg("--audio-quality")
                    .arg("0");
            }
        } else if !is_video_only {
            if let Some(container) = format_preference {
                let c = container.to_lowercase();
                if matches!(c.as_str(), "mp4" | "mkv" | "webm") {
                    if ffmpeg_status.is_available {
                        cmd.arg("--merge-output-format").arg(&c);
                    }
                }
            }
        }

        if ffmpeg_status.is_available {
            if let Some(path) = ffmpeg_status.binary_path {
                if let Some(parent) = Path::new(&path).parent() {
                    cmd.arg("--ffmpeg-location").arg(parent);
                }
            }
        }

        cmd.arg(source_url);

        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| AppError::ProviderError {
            code: "YTDLP_SPAWN_FAILED".into(),
            message: format!("Failed to start yt-dlp process: {}", e),
            action: "Verify yt-dlp installation.".into(),
        })?;

        // Active cancellation watcher: terminates child process immediately on cancel_token trigger
        let child_id = child.id();
        let cancel_token_watcher = cancel_token.clone();
        let is_running_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let is_running_clone = is_running_flag.clone();

        std::thread::spawn(move || {
            while is_running_clone.load(std::sync::atomic::Ordering::Relaxed) {
                if cancel_token_watcher.is_cancelled() {
                    #[cfg(target_os = "windows")]
                    {
                        use std::os::windows::process::CommandExt;
                        let _ = std::process::Command::new("taskkill")
                            .args(["/F", "/T", "/PID", &child_id.to_string()])
                            .creation_flags(0x08000000)
                            .output();
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        let _ = std::process::Command::new("kill")
                            .args(["-9", &child_id.to_string()])
                            .output();
                    }
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        });

        // Capture stderr in background to surface genuine error messages and avoid pipe stalls
        let stderr_lines = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let stderr_lines_clone = stderr_lines.clone();

        if let Some(err_pipe) = child.stderr.take() {
            std::thread::spawn(move || {
                let r = BufReader::new(err_pipe);
                for line in r.lines().flatten() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        let mut l = stderr_lines_clone.lock().unwrap();
                        if l.len() >= 15 {
                            l.remove(0);
                        }
                        l.push(trimmed.to_string());
                    }
                }
            });
        }

        let stdout = child.stdout.take().ok_or_else(|| AppError::InternalError {
            code: "PIPE_FAILED".into(),
            message: "Failed to open child process stdout pipe".into(),
            action: "Retry download.".into(),
        })?;

        let reader = BufReader::new(stdout);
        let mut last_bytes: u64 = 0;
        let mut last_total: Option<u64> = None;

        for line in reader.lines() {
            if cancel_token.is_cancelled() {
                break;
            }

            if let Ok(l) = line {
                let trimmed = l.trim();
                // 1. Primary: DOWNLOAD: prefix
                if let Some(pos) = trimmed.find("DOWNLOAD:") {
                    let raw = &trimmed[pos + 9..];
                    let parts: Vec<&str> = raw.split('|').collect();
                    if parts.len() >= 5 {
                        let percent_raw = parts[0].trim().trim_end_matches('%').trim();
                        let percent_val: f64 = percent_raw.parse::<f64>().unwrap_or(0.0);
                        let progress_ratio = (percent_val / 100.0).clamp(0.0, 1.0);

                        let speed_str = parts[1].trim();
                        let speed_bytes = parse_speed_bytes(speed_str);

                        let downloaded_bytes: u64 = parts[3].trim().parse().unwrap_or(last_bytes);
                        let total_bytes: Option<u64> = parts[4].trim().parse().ok().or(last_total);

                        let eta_str = parts[2].trim();
                        let eta_seconds = parse_eta_seconds(eta_str);

                        last_bytes = downloaded_bytes;
                        if total_bytes.is_some() {
                            last_total = total_bytes;
                        }

                        progress_callback(DownloadProgress {
                            task_id: task_id.to_string(),
                            bytes_downloaded: downloaded_bytes,
                            total_bytes,
                            progress: progress_ratio,
                            speed_bytes_per_sec: speed_bytes,
                            eta_seconds,
                        });
                    }
                } else if trimmed.starts_with("[download]") && trimmed.contains('%') {
                    // 2. Fallback: standard yt-dlp [download] progress lines
                    if let Some(parsed) = parse_standard_download_line(trimmed, last_bytes, last_total) {
                        last_bytes = parsed.bytes_downloaded;
                        if parsed.total_bytes.is_some() {
                            last_total = parsed.total_bytes;
                        }

                        progress_callback(DownloadProgress {
                            task_id: task_id.to_string(),
                            bytes_downloaded: parsed.bytes_downloaded,
                            total_bytes: parsed.total_bytes,
                            progress: parsed.progress,
                            speed_bytes_per_sec: parsed.speed_bytes_per_sec,
                            eta_seconds: parsed.eta_seconds,
                        });
                    }
                }
            }
        }

        is_running_flag.store(false, std::sync::atomic::Ordering::Relaxed);

        if cancel_token.is_cancelled() {
            let _ = child.kill();
            return Err(AppError::CancellationError {
                code: "DOWNLOAD_CANCELLED".into(),
                message: "Download cancelled or paused by user.".into(),
                action: "Resume download when ready.".into(),
            });
        }

        let exit_status = child.wait().map_err(|e| AppError::ProviderError {
            code: "YTDLP_WAIT_FAILED".into(),
            message: format!("Error waiting for yt-dlp: {}", e),
            action: "Retry download.".into(),
        })?;

        if cancel_token.is_cancelled() {
            return Err(AppError::CancellationError {
                code: "DOWNLOAD_CANCELLED".into(),
                message: "Download cancelled or paused by user.".into(),
                action: "Resume download when ready.".into(),
            });
        }

        if !exit_status.success() {
            let captured = stderr_lines.lock().unwrap();
            let err_detail = captured
                .iter()
                .rev()
                .find(|s| s.contains("ERROR:") || s.contains("Error:") || s.contains("error:"))
                .or_else(|| captured.last())
                .cloned()
                .unwrap_or_else(|| "yt-dlp encountered an error extracting video.".into());

            return Err(AppError::ProviderError {
                code: "YTDLP_ERROR".into(),
                message: err_detail,
                action: "Verify source video is public and accessible.".into(),
            });
        }

        // Final 100% progress callback
        if last_bytes > 0 {
            progress_callback(DownloadProgress {
                task_id: task_id.to_string(),
                bytes_downloaded: last_bytes,
                total_bytes: last_total.or(Some(last_bytes)),
                progress: 1.0,
                speed_bytes_per_sec: 0.0,
                eta_seconds: Some(0),
            });
        }

        // Find the generated output file
        let mut final_path = PathBuf::new();
        let target_ext = format_preference.unwrap_or(if is_audio_only { "mp3" } else { "mp4" });
        let direct_candidate = output_dir.join(format!("{}.{}", output_filename, target_ext));
        if direct_candidate.exists() {
            final_path = direct_candidate;
        } else {
            let extensions = ["mp4", "mp3", "m4a", "mkv", "webm", "opus", "wav", "flac", "aac"];
            for ext in &extensions {
                let candidate = output_dir.join(format!("{}.{}", output_filename, ext));
                if candidate.exists() {
                    final_path = candidate;
                    break;
                }
            }
        }

        if !final_path.exists() {
            if let Ok(entries) = std::fs::read_dir(output_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if let Some(stem) = p.file_stem() {
                        if stem.to_string_lossy() == output_filename {
                            final_path = p;
                            break;
                        }
                    }
                }
            }
        }

        if !final_path.exists() {
            final_path = output_dir.join(format!("{}.{}", output_filename, target_ext));
        }

        Ok(final_path)
    }
}

struct ParsedProgress {
    bytes_downloaded: u64,
    total_bytes: Option<u64>,
    progress: f64,
    speed_bytes_per_sec: f64,
    eta_seconds: Option<u64>,
}

fn parse_standard_download_line(line: &str, last_bytes: u64, last_total: Option<u64>) -> Option<ParsedProgress> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let mut progress: f64 = 0.0;
    let mut speed_bytes_per_sec: f64 = 0.0;
    let mut eta_seconds: Option<u64> = None;
    let mut total_bytes: Option<u64> = last_total;
    let mut bytes_downloaded: u64 = last_bytes;

    for (i, &token) in tokens.iter().enumerate() {
        if token.ends_with('%') {
            if let Ok(p) = token.trim_end_matches('%').parse::<f64>() {
                progress = (p / 100.0).clamp(0.0, 1.0);
            }
        }
        if (token == "at" || token == "AT") && i + 1 < tokens.len() {
            speed_bytes_per_sec = parse_speed_bytes(tokens[i + 1]);
        }
        if (token == "ETA" || token == "eta") && i + 1 < tokens.len() {
            eta_seconds = parse_eta_seconds(tokens[i + 1]);
        }
        if (token == "of" || token == "OF") && i + 1 < tokens.len() {
            let total_str = tokens[i + 1].trim_start_matches('~');
            let parsed_size = parse_size_bytes(total_str);
            if parsed_size > 0 {
                total_bytes = Some(parsed_size);
                bytes_downloaded = (parsed_size as f64 * progress) as u64;
            }
        }
    }

    if progress > 0.0 || speed_bytes_per_sec > 0.0 {
        Some(ParsedProgress {
            bytes_downloaded,
            total_bytes,
            progress,
            speed_bytes_per_sec,
            eta_seconds,
        })
    } else {
        None
    }
}

fn parse_size_bytes(size_str: &str) -> u64 {
    let s = size_str.trim().to_uppercase();
    if s.contains("GIB") || s.contains("GB") {
        let num: f64 = s.split('G').next().unwrap_or("0").trim().parse().unwrap_or(0.0);
        (num * 1024.0 * 1024.0 * 1024.0) as u64
    } else if s.contains("MIB") || s.contains("MB") {
        let num: f64 = s.split('M').next().unwrap_or("0").trim().parse().unwrap_or(0.0);
        (num * 1024.0 * 1024.0) as u64
    } else if s.contains("KIB") || s.contains("KB") {
        let num: f64 = s.split('K').next().unwrap_or("0").trim().parse().unwrap_or(0.0);
        (num * 1024.0) as u64
    } else {
        s.parse::<u64>().unwrap_or(0)
    }
}

fn parse_eta_seconds(eta_str: &str) -> Option<u64> {
    let parts: Vec<&str> = eta_str.split(':').collect();
    match parts.len() {
        2 => {
            let mins: u64 = parts[0].trim().parse().ok()?;
            let secs: u64 = parts[1].trim().parse().ok()?;
            Some(mins * 60 + secs)
        }
        3 => {
            let hrs: u64 = parts[0].trim().parse().ok()?;
            let mins: u64 = parts[1].trim().parse().ok()?;
            let secs: u64 = parts[2].trim().parse().ok()?;
            Some(hrs * 3600 + mins * 60 + secs)
        }
        _ => None,
    }
}

fn parse_speed_bytes(speed_str: &str) -> f64 {
    let s = speed_str.trim().to_uppercase();
    if s.contains("GIB/S") || s.contains("GB/S") {
        let num: f64 = s.split('G').next().unwrap_or("0").trim().parse().unwrap_or(0.0);
        num * 1024.0 * 1024.0 * 1024.0
    } else if s.contains("MIB/S") || s.contains("MB/S") {
        let num: f64 = s.split('M').next().unwrap_or("0").trim().parse().unwrap_or(0.0);
        num * 1024.0 * 1024.0
    } else if s.contains("KIB/S") || s.contains("KB/S") {
        let num: f64 = s.split('K').next().unwrap_or("0").trim().parse().unwrap_or(0.0);
        num * 1024.0
    } else if s.contains("B/S") {
        let num: f64 = s.split('B').next().unwrap_or("0").trim().parse().unwrap_or(0.0);
        num
    } else {
        0.0
    }
}

#[async_trait::async_trait]
impl MediaProvider for YtDlpProvider {
    fn provider_id(&self) -> &'static str {
        "yt_dlp"
    }

    async fn validate_source(&self, source_url: &str) -> Result<SourceValidation, AppError> {
        let trimmed = source_url.trim();
        if trimmed.is_empty() {
            return Ok(SourceValidation {
                is_valid: false,
                normalized_url: String::new(),
                provider_id: self.provider_id().to_string(),
                error_message: Some("URL is empty".into()),
            });
        }

        if let Ok(parsed) = Url::parse(trimmed) {
            let scheme = parsed.scheme();
            if scheme == "http" || scheme == "https" {
                return Ok(SourceValidation {
                    is_valid: true,
                    normalized_url: parsed.to_string(),
                    provider_id: self.provider_id().to_string(),
                    error_message: None,
                });
            }
        }

        Ok(SourceValidation {
            is_valid: false,
            normalized_url: trimmed.to_string(),
            provider_id: self.provider_id().to_string(),
            error_message: Some("Invalid URL format".into()),
        })
    }

    async fn resolve_metadata(&self, source_url: &str) -> Result<MediaMetadata, AppError> {
        let yt_bin = Self::find_binary();
        let status = Self::check_status();

        let platform = Self::identify_platform(source_url);

        if !status.is_available {
            // Graceful fallback metadata when yt-dlp binary is pending installation
            return Ok(MediaMetadata {
                source_url: source_url.to_string(),
                provider_id: self.provider_id().to_string(),
                title: format!("{} Video", platform),
                description: None,
                thumbnail_url: None,
                duration_seconds: None,
                author: Some(platform.to_string()),
                formats: vec![
                    MediaFormat {
                        format_id: "best".into(),
                        resolution: "Best Quality".into(),
                        extension: "mp4".into(),
                        bitrate: None,
                        fps: None,
                        estimated_size: None,
                        supports_resume: true,
                        audio_only: false,
                    },
                    MediaFormat {
                        format_id: "audio".into(),
                        resolution: "Audio (MP3)".into(),
                        extension: "mp3".into(),
                        bitrate: None,
                        fps: None,
                        estimated_size: None,
                        supports_resume: true,
                        audio_only: true,
                    },
                ],
            });
        }

        let mut cmd = Command::new(&yt_bin);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let out = cmd
            .arg("--dump-json")
            .arg("--no-playlist")
            .arg(source_url)
            .output();

        if let Ok(o) = out {
            if o.status.success() {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&o.stdout) {
                    let title = val
                        .get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("video_download")
                        .to_string();

                    let thumbnail_url = val
                        .get("thumbnail")
                        .and_then(|t| t.as_str())
                        .map(|t| t.to_string());

                    let duration_seconds = val.get("duration").and_then(|d| d.as_u64());

                    let author = val
                        .get("uploader")
                        .or_else(|| val.get("channel"))
                        .and_then(|a| a.as_str())
                        .map(|a| a.to_string());

                    let formats = vec![
                        MediaFormat {
                            format_id: "best".into(),
                            resolution: "Best Quality (Auto)".into(),
                            extension: "mp4".into(),
                            bitrate: None,
                            fps: None,
                            estimated_size: None,
                            supports_resume: true,
                            audio_only: false,
                        },
                        MediaFormat {
                            format_id: "1080p".into(),
                            resolution: "1080p Full HD".into(),
                            extension: "mp4".into(),
                            bitrate: Some(4000),
                            fps: Some(30),
                            estimated_size: None,
                            supports_resume: true,
                            audio_only: false,
                        },
                        MediaFormat {
                            format_id: "720p".into(),
                            resolution: "720p HD".into(),
                            extension: "mp4".into(),
                            bitrate: Some(2500),
                            fps: Some(30),
                            estimated_size: None,
                            supports_resume: true,
                            audio_only: false,
                        },
                        MediaFormat {
                            format_id: "audio".into(),
                            resolution: "Audio Only (MP3)".into(),
                            extension: "mp3".into(),
                            bitrate: Some(192),
                            fps: None,
                            estimated_size: None,
                            supports_resume: true,
                            audio_only: true,
                        },
                    ];

                    return Ok(MediaMetadata {
                        source_url: source_url.to_string(),
                        provider_id: self.provider_id().to_string(),
                        title,
                        description: None,
                        thumbnail_url,
                        duration_seconds,
                        author,
                        formats,
                    });
                }
            }
        }

        // Fallback if dump-json failed
        Ok(MediaMetadata {
            source_url: source_url.to_string(),
            provider_id: self.provider_id().to_string(),
            title: format!("{} Media", platform),
            description: None,
            thumbnail_url: None,
            duration_seconds: None,
            author: Some(platform.to_string()),
            formats: vec![MediaFormat {
                format_id: "best".into(),
                resolution: "Best Available".into(),
                extension: "mp4".into(),
                bitrate: None,
                fps: None,
                estimated_size: None,
                supports_resume: true,
                audio_only: false,
            }],
        })
    }

    async fn resolve_formats(&self, source_url: &str) -> Result<Vec<MediaFormat>, AppError> {
        let meta = self.resolve_metadata(source_url).await?;
        Ok(meta.formats)
    }
}

use std::path::Path;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::domain::DownloadStatus;
use crate::errors::AppError;
use crate::persistence::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySummary {
    pub tasks_reconciled: usize,
    pub completed_files_found: usize,
    pub partial_files_recovered: usize,
    pub stale_tasks_reset: usize,
    pub integrity_ok: bool,
    pub message: String,
}

pub struct RecoveryManager {
    db: Arc<Database>,
}

impl RecoveryManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn run_startup_recovery(&self) -> Result<RecoverySummary, AppError> {
        let integrity_ok = self.db.check_integrity().unwrap_or(false);

        // Find tasks in transitional or active states
        let all_tasks = self.db.list_downloads(None, None, None, None, None)?;

        let mut completed_files_found = 0;
        let mut partial_files_recovered = 0;
        let mut stale_tasks_reset = 0;

        for task in all_tasks {
            if task.status.is_active() || task.status == DownloadStatus::Pausing {
                let out_dir = Path::new(&task.output_directory);
                let ext = task.selected_extension.as_deref().unwrap_or("mp4");
                let raw_filename = task
                    .output_filename
                    .clone()
                    .unwrap_or_else(|| format!("{}.{}", task.id, ext));

                let target_file_path = out_dir.join(&raw_filename);
                let partial_file_path = out_dir.join(format!("{}.bvd-partial", raw_filename));

                // 1. Check if target file already exists and is non-empty
                if target_file_path.exists() {
                    let file_size = std::fs::metadata(&target_file_path)
                        .map(|m| m.len())
                        .unwrap_or(0);

                    if file_size > 0 {
                        let _ = self.db.update_download_final_file(
                            &task.id,
                            &raw_filename,
                            &target_file_path.to_string_lossy(),
                            file_size,
                        );
                        completed_files_found += 1;
                        continue;
                    }
                }

                // 2. Check if .bvd-partial or yt-dlp .part file exists
                let stem = Path::new(&raw_filename)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&task.id);

                let mut partial_size = 0u64;
                if partial_file_path.exists() {
                    partial_size = std::fs::metadata(&partial_file_path).map(|m| m.len()).unwrap_or(0);
                } else {
                    let ytdlp_part = out_dir.join(format!("{}.part", raw_filename));
                    if ytdlp_part.exists() {
                        partial_size = std::fs::metadata(&ytdlp_part).map(|m| m.len()).unwrap_or(0);
                    } else if let Ok(entries) = std::fs::read_dir(out_dir) {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            if let Some(file_name) = p.file_name().and_then(|f| f.to_str()) {
                                if file_name.starts_with(stem) && (file_name.ends_with(".part") || file_name.ends_with(".ytdl")) {
                                    if let Ok(meta) = std::fs::metadata(&p) {
                                        partial_size = partial_size.max(meta.len());
                                    }
                                }
                            }
                        }
                    }
                }

                if partial_size > 0 {
                    let progress = if let Some(tot) = task.total_bytes {
                        if tot > 0 {
                            (partial_size as f64 / tot as f64).min(0.99)
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };

                    let _ = self.db.update_download_progress(
                        &task.id,
                        partial_size,
                        task.total_bytes,
                        progress,
                        None,
                        None,
                    );
                    // Reset to QUEUED with preserved partial progress so it seamlessly resumes on start
                    let _ = self.db.update_download_status(
                        &task.id,
                        DownloadStatus::Queued,
                        None,
                        None,
                    );
                    partial_files_recovered += 1;
                    continue;
                }

                // 3. Otherwise reset to QUEUED
                let _ = self.db.update_download_status(
                    &task.id,
                    DownloadStatus::Queued,
                    None,
                    None,
                );
                stale_tasks_reset += 1;
            }
        }

        let total_reconciled = completed_files_found + partial_files_recovered + stale_tasks_reset;
        let message = if total_reconciled > 0 {
            format!(
                "Startup recovery completed: {} finished files discovered, {} partial downloads preserved, {} tasks returned to queue.",
                completed_files_found, partial_files_recovered, stale_tasks_reset
            )
        } else {
            "Startup recovery completed: queue state is clean and synchronized.".to_string()
        };

        Ok(RecoverySummary {
            tasks_reconciled: total_reconciled,
            completed_files_found,
            partial_files_recovered,
            stale_tasks_reset,
            integrity_ok,
            message,
        })
    }
}

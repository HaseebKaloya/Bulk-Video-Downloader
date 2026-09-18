use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use crate::domain::{BatchStatus, CompletionPolicy, DownloadStatus, DownloadTask};
use crate::downloader::{DownloadProgress, DownloadWorker};
use crate::errors::AppError;
use crate::events::AppEventEmitter;
use crate::persistence::Database;

/// RAII Guard that guarantees worker count decrement and token cleanup even on panics or early exits.
struct WorkerSlotGuard {
    active_workers_count: Arc<Mutex<usize>>,
    notify: Arc<Notify>,
    task_id: String,
    active_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl Drop for WorkerSlotGuard {
    fn drop(&mut self) {
        self.active_tokens.lock().unwrap().remove(&self.task_id);
        let mut count = self.active_workers_count.lock().unwrap();
        *count = count.saturating_sub(1);
        self.notify.notify_waiters();
    }
}

pub struct Scheduler {
    db: Arc<Database>,
    events: AppEventEmitter,
    worker: Arc<DownloadWorker>,
    is_running: Arc<Mutex<bool>>,
    active_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    active_workers_count: Arc<Mutex<usize>>,
    notify: Arc<Notify>,
}

impl Scheduler {
    pub fn new(db: Arc<Database>, events: AppEventEmitter) -> Self {
        Self {
            db,
            events,
            worker: Arc::new(DownloadWorker::new()),
            is_running: Arc::new(Mutex::new(false)),
            active_tokens: Arc::new(Mutex::new(HashMap::new())),
            active_workers_count: Arc::new(Mutex::new(0)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn start(&self) {
        let mut running = self.is_running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;
        drop(running);

        self.events.emit("scheduler.state_changed", "system", serde_json::json!({ "is_running": true }));

        let db = self.db.clone();
        let events = self.events.clone();
        let worker = self.worker.clone();
        let is_running = self.is_running.clone();
        let active_tokens = self.active_tokens.clone();
        let active_workers_count = self.active_workers_count.clone();
        let notify = self.notify.clone();

        tauri::async_runtime::spawn(async move {
            while *is_running.lock().unwrap() {
                let batches = db.list_batches().unwrap_or_default();
                let settings = db.get_settings().unwrap_or_default();
                let global_concurrency = (settings.default_concurrency as usize).max(1);

                let mut any_batch_has_queued = false;

                // Evaluate each batch fairly across the system
                for batch in batches {
                    if batch.status == BatchStatus::Stopping {
                        continue;
                    }

                    // Check batch tasks
                    let tasks = db
                        .list_downloads(Some(&batch.id), None, None, None, None)
                        .unwrap_or_default();

                    // 1. Auto-retire empty batches
                    if tasks.is_empty() {
                        if batch.status == BatchStatus::Active || batch.status == BatchStatus::Ready {
                            let _ = db.update_batch_status(&batch.id, BatchStatus::Completed);
                            events.emit("batch.completed", &batch.id, serde_json::json!({ "batch_id": batch.id }));
                        }
                        continue;
                    }

                    // 2. Check if all tasks in batch reached a terminal status
                    let all_terminal = tasks.iter().all(|t| t.status.is_terminal());
                    let any_failed = tasks.iter().any(|t| t.status == DownloadStatus::Failed);

                    if all_terminal {
                        if batch.status == BatchStatus::Active {
                            let target_status = match batch.completion_policy {
                                CompletionPolicy::PauseOnFailure if any_failed => BatchStatus::Paused,
                                _ => BatchStatus::Completed,
                            };
                            let _ = db.update_batch_status(&batch.id, target_status);
                            let event_name = if target_status == BatchStatus::Paused { "batch.paused" } else { "batch.completed" };
                            events.emit(event_name, &batch.id, serde_json::json!({ "batch_id": batch.id }));
                        }
                        continue;
                    }

                    // 3. Auto-promote Ready batch if appropriate
                    if batch.status == BatchStatus::Ready {
                        let _ = db.update_batch_status(&batch.id, BatchStatus::Active);
                        events.emit("batch.started", &batch.id, serde_json::json!({ "batch_id": batch.id }));
                    }

                    if batch.status != BatchStatus::Active && batch.status != BatchStatus::Ready {
                        continue;
                    }

                    // 4. Calculate eligible tasks and available concurrency
                    let now = chrono::Utc::now().to_rfc3339();
                    let queued_count = tasks.iter().filter(|t| {
                        t.status == DownloadStatus::Queued
                            || (t.status == DownloadStatus::RetryWait
                                && t.next_retry_at.as_deref().unwrap_or("") <= now.as_str())
                    }).count();

                    if queued_count > 0 {
                        any_batch_has_queued = true;
                    }

                    let batch_concurrency = (batch.concurrency as usize).max(1);
                    let batch_size_limit = (batch.batch_size as usize).max(1);
                    let active_tasks_in_batch = tasks.iter().filter(|t| t.status.is_active()).count();

                    let current_workers = *active_workers_count.lock().unwrap();
                    let global_available = global_concurrency.saturating_sub(current_workers);
                    let batch_available = batch_concurrency.saturating_sub(active_tasks_in_batch)
                        .min(batch_size_limit.saturating_sub(active_tasks_in_batch));

                    let available_slots = global_available.min(batch_available);

                    if available_slots > 0 {
                        let eligible: Vec<DownloadTask> = tasks
                            .into_iter()
                            .filter(|t| {
                                t.status == DownloadStatus::Queued
                                    || (t.status == DownloadStatus::RetryWait
                                        && t.next_retry_at.as_deref().unwrap_or("") <= now.as_str())
                            })
                            .take(available_slots)
                            .collect();

                        for task in eligible {
                            let task_id = task.id.clone();
                            let cancel_token = CancellationToken::new();

                            active_tokens.lock().unwrap().insert(task_id.clone(), cancel_token.clone());
                            *active_workers_count.lock().unwrap() += 1;

                            let _ = db.update_download_status(&task_id, DownloadStatus::Downloading, None, None);
                            events.emit("download.preparing", &task_id, serde_json::json!({ "task_id": task_id }));

                            let task_worker = worker.clone();
                            let task_db = db.clone();
                            let task_events = events.clone();
                            let task_active_tokens = active_tokens.clone();
                            let task_workers_count = active_workers_count.clone();
                            let task_notify = notify.clone();

                            tauri::async_runtime::spawn(async move {
                                let _guard = WorkerSlotGuard {
                                    active_workers_count: task_workers_count,
                                    notify: task_notify,
                                    task_id: task_id.clone(),
                                    active_tokens: task_active_tokens,
                                };

                                let progress_events = task_events.clone();
                                let progress_db = task_db.clone();

                                let callback = move |p: DownloadProgress| {
                                    let _ = progress_db.update_download_progress(
                                        &p.task_id,
                                        p.bytes_downloaded,
                                        p.total_bytes,
                                        p.progress,
                                        Some(p.speed_bytes_per_sec),
                                        p.eta_seconds,
                                    );
                                    progress_events.emit(
                                        "download.progress",
                                        &p.task_id,
                                        serde_json::json!({
                                            "task_id": p.task_id,
                                            "bytes_downloaded": p.bytes_downloaded,
                                            "total_bytes": p.total_bytes,
                                            "progress": p.progress,
                                            "speed_bytes_per_second": p.speed_bytes_per_sec,
                                            "eta_seconds": p.eta_seconds,
                                        }),
                                    );
                                };

                                let result = task_worker
                                    .execute_download(&task, cancel_token, callback)
                                    .await;

                                match result {
                                    Ok(final_path) => {
                                        let filename = final_path
                                            .file_name()
                                            .map(|f| f.to_string_lossy().to_string())
                                            .unwrap_or_default();
                                        let path_str = final_path.to_string_lossy().to_string();
                                        let final_size = std::fs::metadata(&final_path).map(|m| m.len()).unwrap_or(0);

                                        let _ = task_db.update_download_final_file(
                                            &task.id,
                                            &filename,
                                            &path_str,
                                            final_size,
                                        );
                                        task_events.emit(
                                            "download.completed",
                                            &task.id,
                                            serde_json::json!({
                                                "task_id": task.id,
                                                "output_path": path_str,
                                                "output_filename": filename,
                                            }),
                                        );
                                    }
                                    Err(AppError::CancellationError { .. }) => {
                                        let _ = task_db.update_download_status(
                                            &task.id,
                                            DownloadStatus::Paused,
                                            None,
                                            None,
                                        );
                                        task_events.emit(
                                            "download.paused",
                                            &task.id,
                                            serde_json::json!({ "task_id": task.id }),
                                        );
                                    }
                                    Err(err) => {
                                        let err_code = match &err {
                                            AppError::NetworkError { code, .. } => code.clone(),
                                            AppError::FilesystemError { code, .. } => code.clone(),
                                            _ => "DOWNLOAD_FAILED".into(),
                                        };
                                        let err_msg = err.to_string();

                                        let is_retryable = match &err {
                                            AppError::NetworkError { retryable, .. } => *retryable,
                                            _ => false,
                                        };

                                        if is_retryable && task.retry_count < task.max_retries {
                                            let delay_secs = (2u64.pow(task.retry_count) * 2).min(60);
                                            let next_retry = chrono::Utc::now() + chrono::Duration::seconds(delay_secs as i64);

                                            let _ = task_db.increment_retry_count(
                                                &task.id,
                                                Some(&next_retry.to_rfc3339()),
                                                Some(&err_code),
                                                Some(&err_msg),
                                            );
                                            task_events.emit(
                                                "download.retry_scheduled",
                                                &task.id,
                                                serde_json::json!({
                                                    "task_id": task.id,
                                                    "retry_count": task.retry_count + 1,
                                                    "next_retry_at": next_retry.to_rfc3339(),
                                                    "error": err_msg,
                                                }),
                                            );
                                        } else {
                                            let _ = task_db.update_download_status(
                                                &task.id,
                                                DownloadStatus::Failed,
                                                Some(&err_code),
                                                Some(&err_msg),
                                            );
                                            task_events.emit(
                                                "download.failed",
                                                &task.id,
                                                serde_json::json!({
                                                    "task_id": task.id,
                                                    "error_code": err_code,
                                                    "error_message": err_msg,
                                                }),
                                            );
                                        }
                                    }
                                }
                            });
                        }
                    }
                }

                let current_workers = *active_workers_count.lock().unwrap();
                if !any_batch_has_queued && current_workers == 0 {
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_millis(500)) => {},
                        _ = notify.notified() => {},
                    }
                } else {
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_millis(250)) => {},
                        _ = notify.notified() => {},
                    }
                }
            }
        });
    }

    pub fn pause(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = false;

        // Cancel all active workers safely
        let tokens: Vec<CancellationToken> = self
            .active_tokens
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect();

        for token in tokens {
            token.cancel();
        }

        self.events.emit("scheduler.state_changed", "system", serde_json::json!({ "is_running": false }));
        self.notify.notify_waiters();
    }

    pub fn pause_batch(&self, batch_id: &str) {
        if let Ok(tasks) = self.db.list_downloads(Some(batch_id), None, None, None, None) {
            let tokens = self.active_tokens.lock().unwrap();
            for task in tasks {
                if let Some(token) = tokens.get(&task.id) {
                    token.cancel();
                } else if task.status == DownloadStatus::Queued || task.status == DownloadStatus::RetryWait {
                    let _ = self.db.update_download_status(&task.id, DownloadStatus::Paused, None, None);
                    self.events.emit("download.paused", &task.id, serde_json::json!({ "task_id": task.id }));
                }
            }
        }
        self.notify.notify_waiters();
    }

    pub fn pause_task(&self, task_id: &str) {
        if let Some(token) = self.active_tokens.lock().unwrap().get(task_id) {
            token.cancel();
        } else {
            let _ = self.db.update_download_status(task_id, DownloadStatus::Paused, None, None);
            self.events.emit("download.paused", task_id, serde_json::json!({ "task_id": task_id }));
        }
        self.notify.notify_waiters();
    }

    pub fn cancel_task(&self, task_id: &str, delete_partial: bool) {
        if let Some(token) = self.active_tokens.lock().unwrap().get(task_id) {
            token.cancel();
        }

        if delete_partial {
            if let Ok(Some(task)) = self.db.get_download(task_id) {
                let out_dir = std::path::Path::new(&task.output_directory);
                let ext = task.selected_extension.as_deref().unwrap_or("mp4");
                let raw_filename = task.output_filename.unwrap_or_else(|| format!("{}.{}", task.id, ext));
                let partial_path = out_dir.join(format!("{}.bvd-partial", raw_filename));
                let _ = std::fs::remove_file(partial_path);
            }
        }

        let _ = self.db.update_download_status(task_id, DownloadStatus::Cancelled, None, None);
        self.events.emit("download.cancelled", task_id, serde_json::json!({ "task_id": task_id }));
        self.notify.notify_waiters();
    }

    pub fn trigger(&self) {
        self.notify.notify_waiters();
    }
}

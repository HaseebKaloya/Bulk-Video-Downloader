use std::collections::HashSet;
use std::sync::Arc;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::domain::{
    render_filename, AppSettings, Batch, BatchStatus, CompletionPolicy, DownloadProfile,
    DownloadStatus, DownloadTask,
};
use crate::errors::ErrorResponse;
use crate::events::AppEventEmitter;
use crate::media::{FFmpegStatus, MediaProcessor};
use crate::persistence::Database;
use crate::providers::{
    DirectHttpProvider, MediaProvider, ProviderRegistry, YtDlpProvider, YtDlpStatus,
};
use crate::recovery::{RecoveryManager, RecoverySummary};
use crate::scheduler::Scheduler;

pub struct AppState {
    pub db: Arc<Database>,
    pub scheduler: Arc<Scheduler>,
    pub provider_registry: Arc<ProviderRegistry>,
    pub recovery_manager: Arc<RecoveryManager>,
    pub events: AppEventEmitter,
}

// --- IMPORT MODELS ---

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportItem {
    pub raw_url: String,
    pub normalized_url: String,
    pub is_valid: bool,
    pub is_duplicate_intra: bool,
    pub is_duplicate_queue: bool,
    pub title: String,
    pub platform: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportPreviewResult {
    pub total_rows: usize,
    pub valid_count: usize,
    pub invalid_count: usize,
    pub duplicate_intra_count: usize,
    pub duplicate_queue_count: usize,
    pub items: Vec<ImportItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitImportPayload {
    pub batch_id: Option<String>,
    pub batch_name: Option<String>,
    pub batch_size: Option<u32>,
    pub concurrency: Option<u32>,
    pub completion_policy: Option<String>,
    pub profile_id: Option<String>,
    pub selected_quality: Option<String>,
    pub selected_format: Option<String>,
    pub media_type: Option<String>,
    pub destination_directory: Option<String>,
    pub naming_template: Option<String>,
    pub duplicate_policy: String, // "SKIP", "KEEP_SEPARATE", "REPLACE"
    pub urls: Vec<String>,
    pub start_now: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitImportResult {
    pub batch_id: String,
    pub imported_count: usize,
    pub skipped_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBatchPayload {
    pub name: String,
    pub batch_size: u32,
    pub concurrency: u32,
    pub completion_policy: String,
    pub profile_id: Option<String>,
    pub destination_directory: String,
    pub naming_template: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBatchPayload {
    pub name: Option<String>,
    pub batch_size: Option<u32>,
    pub concurrency: Option<u32>,
    pub completion_policy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageStatus {
    pub available_bytes: u64,
    pub total_bytes: u64,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub total_tasks: usize,
    pub queued_tasks: usize,
    pub active_tasks: usize,
    pub paused_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub total_bytes_downloaded: u64,
    pub is_scheduler_running: bool,
    pub ffmpeg_available: bool,
    pub ytdlp_available: bool,
}

fn split_concatenated_urls(token: &str) -> Vec<String> {
    let lower = token.to_lowercase();
    let mut indices: Vec<usize> = Vec::new();
    let mut cursor = 0;
    while cursor < lower.len() {
        if let Some(pos) = lower[cursor..].find("http://").or_else(|| lower[cursor..].find("https://")) {
            let actual = cursor + pos;
            if actual > 0 {
                indices.push(actual);
            }
            cursor = actual + 7;
        } else {
            break;
        }
    }

    if indices.is_empty() {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            vec![trimmed.to_string()]
        } else {
            vec![]
        }
    } else {
        let mut result = Vec::new();
        let mut prev = 0;
        for idx in indices {
            let chunk = token[prev..idx].trim();
            if !chunk.is_empty() {
                result.push(chunk.to_string());
            }
            prev = idx;
        }
        let last = token[prev..].trim();
        if !last.is_empty() {
            result.push(last.to_string());
        }
        result
    }
}

// --- IMPORT COMMANDS ---

#[tauri::command]
pub async fn preview_import(
    raw_content: String,
    state: State<'_, AppState>,
) -> Result<ImportPreviewResult, ErrorResponse> {
    let mut extracted_urls = Vec::new();

    // Check if JSON
    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&raw_content) {
        if let Some(arr) = json_val.as_array() {
            for item in arr {
                if let Some(s) = item.as_str() {
                    extracted_urls.push(s.trim().to_string());
                } else if let Some(obj) = item.as_object() {
                    if let Some(url_val) = obj.get("url").or_else(|| obj.get("link")) {
                        if let Some(s) = url_val.as_str() {
                            extracted_urls.push(s.trim().to_string());
                        }
                    }
                }
            }
        }
    } else {
        // Plain text or CSV parsing
        for line in raw_content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.contains(',') {
                // CSV line
                for part in trimmed.split(',') {
                    let cleaned = part.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
                    for sub_url in split_concatenated_urls(cleaned) {
                        if !sub_url.is_empty() {
                            extracted_urls.push(sub_url);
                        }
                    }
                }
            } else {
                for token in trimmed.split_whitespace() {
                    for sub_url in split_concatenated_urls(token) {
                        if !sub_url.is_empty() {
                            extracted_urls.push(sub_url);
                        }
                    }
                }
            }
        }
    }

    let mut seen_intra = HashSet::new();
    let mut items = Vec::new();
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut duplicate_intra_count = 0;
    let mut duplicate_queue_count = 0;

    let provider = DirectHttpProvider::new();

    for raw in extracted_urls {
        let validation = provider.validate_source(&raw).await.map_err(ErrorResponse::from)?;
        let is_dup_intra = if validation.is_valid {
            !seen_intra.insert(validation.normalized_url.clone())
        } else {
            false
        };

        let is_dup_queue = if validation.is_valid {
            state.db.check_existing_url(&validation.normalized_url).unwrap_or(false)
        } else {
            false
        };

        if !validation.is_valid {
            invalid_count += 1;
        } else {
            valid_count += 1;
        }

        if is_dup_intra {
            duplicate_intra_count += 1;
        }
        if is_dup_queue {
            duplicate_queue_count += 1;
        }

        let parsed_title = if validation.is_valid {
            url::Url::parse(&validation.normalized_url)
                .ok()
                .and_then(|u| {
                    u.path_segments()
                        .and_then(|mut s| s.next_back())
                        .map(|f| f.to_string())
                })
                .unwrap_or_else(|| "video_download".into())
        } else {
            "invalid_link".into()
        };

        let platform = if validation.is_valid {
            Some(YtDlpProvider::identify_platform(&validation.normalized_url).to_string())
        } else {
            None
        };

        items.push(ImportItem {
            raw_url: raw,
            normalized_url: validation.normalized_url,
            is_valid: validation.is_valid,
            is_duplicate_intra: is_dup_intra,
            is_duplicate_queue: is_dup_queue,
            title: parsed_title,
            platform,
            error: validation.error_message,
        });
    }

    Ok(ImportPreviewResult {
        total_rows: items.len(),
        valid_count,
        invalid_count,
        duplicate_intra_count,
        duplicate_queue_count,
        items,
    })
}

#[tauri::command]
pub async fn commit_import(
    payload: CommitImportPayload,
    state: State<'_, AppState>,
) -> Result<CommitImportResult, ErrorResponse> {
    let settings = state.db.get_settings().unwrap_or_default();
    let now = Utc::now().to_rfc3339();

    // 1. Resolve Profile & Settings
    let requested_profile_id = payload
        .profile_id
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .or_else(|| settings.default_profile_id.clone());

    let profile = if let Some(ref pid) = requested_profile_id {
        state.db.get_profile(pid).ok().flatten()
    } else {
        None
    };

    let effective_dest_dir = payload
        .destination_directory
        .filter(|s| !s.trim().is_empty())
        .or_else(|| profile.as_ref().map(|p| p.destination_directory.clone()))
        .unwrap_or_else(|| settings.default_download_directory.clone());

    let effective_naming_tpl = payload
        .naming_template
        .filter(|s| !s.trim().is_empty())
        .or_else(|| profile.as_ref().map(|p| p.naming_template.clone()))
        .unwrap_or_else(|| settings.default_naming_template.clone());

    let is_audio_mode = payload.media_type.as_deref() == Some("audio_only")
        || payload.selected_quality.as_deref() == Some("audio")
        || profile.as_ref().map_or(false, |p| p.quality_policy == "audio" || p.format_policy == "mp3");

    let is_video_only = payload.media_type.as_deref() == Some("video_only");

    let raw_quality = payload
        .selected_quality
        .filter(|s| !s.trim().is_empty())
        .or_else(|| profile.as_ref().map(|p| p.quality_policy.clone()))
        .unwrap_or_else(|| "highest".into());

    let effective_quality = if is_audio_mode {
        "audio".to_string()
    } else if is_video_only {
        format!("{}_video_only", raw_quality.trim_end_matches("_video_only"))
    } else {
        raw_quality
    };

    let raw_format = payload
        .selected_format
        .filter(|s| !s.trim().is_empty())
        .or_else(|| profile.as_ref().map(|p| p.format_policy.clone()))
        .unwrap_or_else(|| if is_audio_mode { "mp3".into() } else { "mp4".into() });

    let effective_format = if is_audio_mode && (raw_format == "mp4" || raw_format == "mkv" || raw_format == "webm") {
        "mp3".to_string()
    } else {
        raw_format
    };

    // 2. Determine or create batch
    let batch_id = match payload.batch_id {
        Some(id) if !id.trim().is_empty() => id,
        _ => {
            let new_id = uuid::Uuid::new_v4().to_string();
            let batch_name = payload.batch_name.unwrap_or_else(|| {
                format!("Batch {}", Utc::now().format("%Y-%m-%d %H:%M"))
            });
            let batch_size = payload.batch_size.unwrap_or(settings.default_batch_size);
            let concurrency = payload.concurrency.unwrap_or(settings.default_concurrency);
            let completion_policy = payload.completion_policy.unwrap_or(settings.default_completion_policy);

            let new_batch = Batch {
                id: new_id.clone(),
                name: batch_name,
                status: BatchStatus::Ready,
                batch_size,
                concurrency,
                completion_policy: CompletionPolicy::from_str(&completion_policy),
                failure_policy: "PAUSE_BATCH".into(),
                profile_id: requested_profile_id.clone(),
                destination_directory: effective_dest_dir.clone(),
                naming_template: effective_naming_tpl.clone(),
                created_at: now.clone(),
                started_at: None,
                completed_at: None,
                updated_at: now.clone(),
            };

            state.db.insert_batch(&new_batch).map_err(ErrorResponse::from)?;
            new_id
        }
    };

    let batch = state
        .db
        .get_batch(&batch_id)
        .map_err(ErrorResponse::from)?
        .ok_or_else(|| ErrorResponse {
            title: "Batch Not Found".into(),
            message: format!("Batch with ID {} was not found", batch_id),
            action: "Create a new batch.".into(),
            code: "BATCH_NOT_FOUND".into(),
            retryable: false,
        })?;

    let mut tasks = Vec::new();
    let mut seen = HashSet::new();
    let mut skipped_count = 0;

    let provider = DirectHttpProvider::new();

    for (index, raw_url) in payload.urls.into_iter().enumerate() {
        let validation = match provider.validate_source(&raw_url).await {
            Ok(v) if v.is_valid => v,
            _ => {
                skipped_count += 1;
                continue;
            }
        };

        if payload.duplicate_policy == "SKIP" {
            if !seen.insert(validation.normalized_url.clone()) {
                skipped_count += 1;
                continue;
            }
            if state.db.check_existing_url(&validation.normalized_url).unwrap_or(false) {
                skipped_count += 1;
                continue;
            }
        }

        let task_id = uuid::Uuid::new_v4().to_string();
        let fallback_title = format!("video_{}", &task_id[..8]);
        let is_social = YtDlpProvider::is_supported_social_url(&validation.normalized_url);
        let provider_id = if is_social { "yt_dlp" } else { "direct_http" };
        let engine_label = if is_social { "SocialMedia" } else { "DirectHTTP" };

        let filename = render_filename(
            &batch.naming_template,
            &fallback_title,
            &task_id,
            engine_label,
            &effective_quality,
            &effective_format,
            &batch.id,
            index + 1,
        );

        let task = DownloadTask {
            id: task_id,
            batch_id: batch.id.clone(),
            source_url: validation.normalized_url,
            provider_id: Some(provider_id.into()),
            source_id: None,
            title: Some(fallback_title),
            thumbnail_url: None,
            status: DownloadStatus::Queued,
            priority: 0,
            queue_position: index as i32 + 1,
            selected_format_id: Some(effective_format.clone()),
            selected_quality: Some(effective_quality.clone()),
            selected_extension: Some(effective_format.clone()),
            output_directory: batch.destination_directory.clone(),
            output_filename: Some(filename),
            output_path: None,
            partial_path: None,
            bytes_downloaded: 0,
            total_bytes: None,
            progress: 0.0,
            speed_bytes_per_second: None,
            eta_seconds: None,
            retry_count: 0,
            max_retries: settings.default_max_retries,
            next_retry_at: None,
            last_error_code: None,
            last_error_message: None,
            created_at: now.clone(),
            started_at: None,
            completed_at: None,
            updated_at: now.clone(),
        };

        tasks.push(task);
    }

    let imported_count = tasks.len();
    if !tasks.is_empty() {
        state.db.insert_downloads(&tasks).map_err(ErrorResponse::from)?;
    }

    if payload.start_now {
        let _ = state.db.update_batch_status(&batch.id, BatchStatus::Active);
        state.scheduler.start();
        state.scheduler.trigger();
    }

    Ok(CommitImportResult {
        batch_id: batch.id,
        imported_count,
        skipped_count,
    })
}

// --- BATCH COMMANDS ---

#[tauri::command]
pub async fn create_batch(
    payload: CreateBatchPayload,
    state: State<'_, AppState>,
) -> Result<Batch, ErrorResponse> {
    let now = Utc::now().to_rfc3339();
    let batch = Batch {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.name,
        status: BatchStatus::Ready,
        batch_size: payload.batch_size,
        concurrency: payload.concurrency,
        completion_policy: CompletionPolicy::from_str(&payload.completion_policy),
        failure_policy: "PAUSE_BATCH".into(),
        profile_id: payload.profile_id,
        destination_directory: payload.destination_directory,
        naming_template: payload.naming_template,
        created_at: now.clone(),
        started_at: None,
        completed_at: None,
        updated_at: now,
    };

    state.db.insert_batch(&batch).map_err(ErrorResponse::from)?;
    Ok(batch)
}

#[tauri::command]
pub async fn list_batches(state: State<'_, AppState>) -> Result<Vec<Batch>, ErrorResponse> {
    state.db.list_batches().map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn get_batch(id: String, state: State<'_, AppState>) -> Result<Option<Batch>, ErrorResponse> {
    state.db.get_batch(&id).map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn update_batch(
    id: String,
    payload: UpdateBatchPayload,
    state: State<'_, AppState>,
) -> Result<Option<Batch>, ErrorResponse> {
    let existing = state.db.get_batch(&id).map_err(ErrorResponse::from)?;
    if let Some(mut batch) = existing {
        if let Some(bs) = payload.batch_size {
            batch.batch_size = bs;
        }
        if let Some(c) = payload.concurrency {
            batch.concurrency = c;
        }
        if let Some(cp) = payload.completion_policy {
            batch.completion_policy = CompletionPolicy::from_str(&cp);
        }
        state
            .db
            .update_batch_config(&id, batch.batch_size, batch.concurrency, batch.completion_policy)
            .map_err(ErrorResponse::from)?;
        state.scheduler.trigger();
        return Ok(Some(batch));
    }
    Ok(None)
}

#[tauri::command]
pub async fn start_batch(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state
        .db
        .update_batch_status(&id, BatchStatus::Active)
        .map_err(ErrorResponse::from)?;
    state.events.emit("batch.started", &id, serde_json::json!({ "batch_id": id }));
    state.scheduler.start();
    state.scheduler.trigger();
    Ok(())
}

#[tauri::command]
pub async fn pause_batch(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state
        .db
        .update_batch_status(&id, BatchStatus::Paused)
        .map_err(ErrorResponse::from)?;
    state.events.emit("batch.paused", &id, serde_json::json!({ "batch_id": id }));
    state.scheduler.pause_batch(&id);
    Ok(())
}

#[tauri::command]
pub async fn stop_batch(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state
        .db
        .update_batch_status(&id, BatchStatus::Stopping)
        .map_err(ErrorResponse::from)?;
    state.events.emit("batch.paused", &id, serde_json::json!({ "batch_id": id }));
    state.scheduler.pause_batch(&id);
    Ok(())
}

#[tauri::command]
pub async fn delete_batch(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state.db.delete_batch(&id).map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn delete_batches(ids: Vec<String>, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state.db.delete_batches(&ids).map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn clear_all_batches(state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state.db.clear_all_batches().map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn start_queue(state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    let batches = state.db.list_batches().map_err(ErrorResponse::from)?;
    let mut any_activated = false;
    for batch in &batches {
        if batch.status == BatchStatus::Stopping {
            continue;
        }
        let tasks = state
            .db
            .list_downloads(Some(&batch.id), None, None, None, None)
            .unwrap_or_default();
        let has_pending = tasks.iter().any(|t| {
            t.status == DownloadStatus::Queued
                || t.status == DownloadStatus::RetryWait
                || t.status == DownloadStatus::Paused
        });

        if has_pending || (batch.status == BatchStatus::Ready && !tasks.is_empty()) {
            let _ = state.db.update_batch_status(&batch.id, BatchStatus::Active);
            state.events.emit("batch.started", &batch.id, serde_json::json!({ "batch_id": batch.id }));
            any_activated = true;

            // Unpause all paused tasks in this batch so they resume immediately
            for task in &tasks {
                if task.status == DownloadStatus::Paused {
                    let _ = state.db.update_download_status(&task.id, DownloadStatus::Queued, None, None);
                    state.events.emit("download.queued", &task.id, serde_json::json!({ "task_id": task.id }));
                }
            }
        }
    }

    if !any_activated {
        if let Some(first_pending_batch) = batches.iter().find(|b| {
            let tasks = state.db.list_downloads(Some(&b.id), None, None, None, None).unwrap_or_default();
            tasks.iter().any(|t| {
                t.status == DownloadStatus::Queued
                    || t.status == DownloadStatus::RetryWait
                    || t.status == DownloadStatus::Paused
            })
        }) {
            let _ = state.db.update_batch_status(&first_pending_batch.id, BatchStatus::Active);
            state.events.emit("batch.started", &first_pending_batch.id, serde_json::json!({ "batch_id": first_pending_batch.id }));

            let tasks = state.db.list_downloads(Some(&first_pending_batch.id), None, None, None, None).unwrap_or_default();
            for task in &tasks {
                if task.status == DownloadStatus::Paused {
                    let _ = state.db.update_download_status(&task.id, DownloadStatus::Queued, None, None);
                    state.events.emit("download.queued", &task.id, serde_json::json!({ "task_id": task.id }));
                }
            }
        }
    }

    state.scheduler.start();
    state.scheduler.trigger();
    Ok(())
}

#[tauri::command]
pub async fn pause_queue(state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    let batches = state.db.list_batches().map_err(ErrorResponse::from)?;
    for batch in &batches {
        if batch.status == BatchStatus::Active {
            let _ = state.db.update_batch_status(&batch.id, BatchStatus::Paused);
            state.events.emit("batch.paused", &batch.id, serde_json::json!({ "batch_id": batch.id }));
        }
    }
    state.scheduler.pause();
    Ok(())
}

// --- DOWNLOAD COMMANDS ---

#[tauri::command]
pub async fn list_downloads(
    batch_id: Option<String>,
    status: Option<String>,
    search: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<DownloadTask>, ErrorResponse> {
    state
        .db
        .list_downloads(
            batch_id.as_deref(),
            status.as_deref(),
            search.as_deref(),
            limit,
            offset,
        )
        .map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn get_download(id: String, state: State<'_, AppState>) -> Result<Option<DownloadTask>, ErrorResponse> {
    state.db.get_download(&id).map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn pause_download(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state.scheduler.pause_task(&id);
    Ok(())
}

#[tauri::command]
pub async fn resume_download(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    if let Ok(Some(task)) = state.db.get_download(&id) {
        let _ = state.db.update_batch_status(&task.batch_id, BatchStatus::Active);
        state.events.emit("batch.started", &task.batch_id, serde_json::json!({ "batch_id": task.batch_id }));
    }
    state
        .db
        .update_download_status(&id, DownloadStatus::Queued, None, None)
        .map_err(ErrorResponse::from)?;
    state.events.emit("download.queued", &id, serde_json::json!({ "task_id": id }));
    state.scheduler.start();
    state.scheduler.trigger();
    Ok(())
}

#[tauri::command]
pub async fn cancel_download(
    id: String,
    delete_partial: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), ErrorResponse> {
    let del = delete_partial.unwrap_or(false);
    state.scheduler.cancel_task(&id, del);
    Ok(())
}

#[tauri::command]
pub async fn retry_download(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    if let Ok(Some(task)) = state.db.get_download(&id) {
        let _ = state.db.update_batch_status(&task.batch_id, BatchStatus::Active);
        state.events.emit("batch.started", &task.batch_id, serde_json::json!({ "batch_id": task.batch_id }));
    }
    state
        .db
        .update_download_status(&id, DownloadStatus::Queued, None, None)
        .map_err(ErrorResponse::from)?;
    state.events.emit("download.queued", &id, serde_json::json!({ "task_id": id }));
    state.scheduler.start();
    state.scheduler.trigger();
    Ok(())
}

#[tauri::command]
pub async fn retry_failed_downloads(
    batch_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<usize, ErrorResponse> {
    let failed = state
        .db
        .list_downloads(batch_id.as_deref(), Some("FAILED"), None, None, None)
        .map_err(ErrorResponse::from)?;

    let count = failed.len();
    let mut affected_batches = std::collections::HashSet::new();

    for task in failed {
        affected_batches.insert(task.batch_id.clone());
        let _ = state
            .db
            .update_download_status(&task.id, DownloadStatus::Queued, None, None);
        state.events.emit("download.queued", &task.id, serde_json::json!({ "task_id": task.id }));
    }

    for b_id in affected_batches {
        let _ = state.db.update_batch_status(&b_id, BatchStatus::Active);
        state.events.emit("batch.started", &b_id, serde_json::json!({ "batch_id": b_id }));
    }

    if count > 0 {
        state.scheduler.start();
        state.scheduler.trigger();
    }

    Ok(count)
}

#[tauri::command]
pub async fn skip_download(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state
        .db
        .update_download_status(&id, DownloadStatus::Skipped, None, None)
        .map_err(ErrorResponse::from)?;
    state.scheduler.trigger();
    Ok(())
}

#[tauri::command]
pub async fn remove_download(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state.scheduler.cancel_task(&id, true);
    state.db.delete_download(&id).map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn change_download_priority(
    id: String,
    priority: i32,
    state: State<'_, AppState>,
) -> Result<(), ErrorResponse> {
    state
        .db
        .change_priority(&id, priority)
        .map_err(ErrorResponse::from)?;
    state.scheduler.trigger();
    Ok(())
}

#[tauri::command]
pub async fn open_download_folder(output_directory: String) -> Result<(), ErrorResponse> {
    let path = std::path::Path::new(&output_directory);
    if path.exists() {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer").arg(path).spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg(path).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open").arg(path).spawn();
        }
    }
    Ok(())
}

// --- SETTINGS & PROFILES COMMANDS ---

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, ErrorResponse> {
    state.db.get_settings().map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), ErrorResponse> {
    state.db.save_settings(&settings).map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn list_profiles(state: State<'_, AppState>) -> Result<Vec<DownloadProfile>, ErrorResponse> {
    state.db.list_profiles().map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn create_profile(
    profile: DownloadProfile,
    state: State<'_, AppState>,
) -> Result<(), ErrorResponse> {
    state.db.save_profile(&profile).map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn update_profile(
    profile: DownloadProfile,
    state: State<'_, AppState>,
) -> Result<(), ErrorResponse> {
    state.db.save_profile(&profile).map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn delete_profile(id: String, state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state.db.delete_profile(&id).map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn set_default_profile(
    profile_id: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorResponse> {
    let mut settings = state.db.get_settings().map_err(ErrorResponse::from)?;
    settings.default_profile_id = Some(profile_id);
    state.db.save_settings(&settings).map_err(ErrorResponse::from)?;
    Ok(())
}

// --- DIAGNOSTICS & SYSTEM STATUS ---

#[tauri::command]
pub async fn get_recovery_summary(state: State<'_, AppState>) -> Result<RecoverySummary, ErrorResponse> {
    state
        .recovery_manager
        .run_startup_recovery()
        .map_err(ErrorResponse::from)
}

#[tauri::command]
pub async fn check_ffmpeg() -> Result<FFmpegStatus, ErrorResponse> {
    Ok(MediaProcessor::check_ffmpeg())
}

#[tauri::command]
pub async fn install_ffmpeg() -> Result<FFmpegStatus, ErrorResponse> {
    if cfg!(target_os = "windows") {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let mut cmd = std::process::Command::new("winget");
            cmd.args(["install", "Gyan.FFmpeg.Essentials", "--silent", "--accept-source-agreements", "--accept-package-agreements"]);
            cmd.creation_flags(0x08000000);
            let _ = cmd.output();
        }
    }
    let status = MediaProcessor::check_ffmpeg();
    if status.is_available {
        Ok(status)
    } else {
        Err(ErrorResponse {
            title: "FFmpeg Installation Failed".into(),
            code: "FFMPEG_INSTALL_FAILED".into(),
            message: "FFmpeg could not be installed automatically. Please install it via 'winget install Gyan.FFmpeg.Essentials' or place ffmpeg.exe and ffprobe.exe in the 'bin/' folder.".into(),
            action: "Install Gyan.FFmpeg.Essentials or copy ffmpeg.exe to bin/".into(),
            retryable: true,
        })
    }
}

#[tauri::command]
pub async fn check_ytdlp() -> Result<YtDlpStatus, ErrorResponse> {
    Ok(YtDlpProvider::check_status())
}

#[tauri::command]
pub async fn install_ytdlp() -> Result<YtDlpStatus, ErrorResponse> {
    let install_dir = if let Ok(cwd) = std::env::current_dir() {
        cwd.join("bin")
    } else if let Ok(exe) = std::env::current_exe() {
        exe.parent().map(|p| p.join("bin")).unwrap_or_else(|| std::path::PathBuf::from("bin"))
    } else {
        std::path::PathBuf::from("bin")
    };

    let _ = std::fs::create_dir_all(&install_dir);
    let exe_name = if cfg!(target_os = "windows") { "yt-dlp.exe" } else { "yt-dlp" };
    let dest_path = install_dir.join(exe_name);
    let download_url = if cfg!(target_os = "windows") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    };

    let mut downloaded = false;

    if let Ok(client) = reqwest::Client::builder()
        .user_agent("BulkVideoDownloader/1.0")
        .timeout(std::time::Duration::from_secs(180))
        .build()
    {
        if let Ok(response) = client.get(download_url).send().await {
            if response.status().is_success() {
                if let Ok(bytes) = response.bytes().await {
                    let temp_dest = dest_path.with_extension("tmp");
                    if std::fs::write(&temp_dest, bytes).is_ok() {
                        let _ = std::fs::rename(&temp_dest, &dest_path);
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let _ = std::fs::set_permissions(&dest_path, std::fs::Permissions::from_mode(0o755));
                        }
                        downloaded = true;
                    }
                }
            }
        }
    }

    if !downloaded && cfg!(target_os = "windows") {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let mut cmd = std::process::Command::new("winget");
            cmd.args(["install", "yt-dlp.yt-dlp", "--silent", "--accept-source-agreements", "--accept-package-agreements"]);
            cmd.creation_flags(0x08000000);
            let _ = cmd.output();
        }
    }

    let status = YtDlpProvider::check_status();
    if status.is_available {
        Ok(status)
    } else {
        Err(ErrorResponse {
            title: "Installation Failed".into(),
            code: "YTDLP_INSTALL_FAILED".into(),
            message: "Unable to download or install yt-dlp automatically. You can install it via 'winget install yt-dlp.yt-dlp' or place yt-dlp.exe in the 'bin' folder.".into(),
            action: "Install yt-dlp via winget or place yt-dlp.exe into bin/".into(),
            retryable: true,
        })
    }
}

#[tauri::command]
pub async fn check_storage(path: String) -> Result<StorageStatus, ErrorResponse> {
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();
    let target_path = std::path::Path::new(&path);

    for disk in &disks {
        if target_path.starts_with(disk.mount_point()) {
            return Ok(StorageStatus {
                available_bytes: disk.available_space(),
                total_bytes: disk.total_space(),
                path,
            });
        }
    }

    Ok(StorageStatus {
        available_bytes: 100 * 1024 * 1024 * 1024, // 100GB fallback
        total_bytes: 500 * 1024 * 1024 * 1024,
        path,
    })
}

#[tauri::command]
pub async fn get_system_status(state: State<'_, AppState>) -> Result<SystemStatus, ErrorResponse> {
    let all = state
        .db
        .list_downloads(None, None, None, None, None)
        .map_err(ErrorResponse::from)?;

    let queued = all.iter().filter(|t| t.status == DownloadStatus::Queued).count();
    let active = all.iter().filter(|t| t.status.is_active()).count();
    let paused = all.iter().filter(|t| t.status == DownloadStatus::Paused).count();
    let completed = all.iter().filter(|t| t.status == DownloadStatus::Completed).count();
    let failed = all.iter().filter(|t| t.status == DownloadStatus::Failed).count();
    let total_bytes: u64 = all.iter().map(|t| t.bytes_downloaded).sum();

    let ffmpeg_status = MediaProcessor::check_ffmpeg();
    let ytdlp_status = YtDlpProvider::check_status();

    Ok(SystemStatus {
        total_tasks: all.len(),
        queued_tasks: queued,
        active_tasks: active,
        paused_tasks: paused,
        completed_tasks: completed,
        failed_tasks: failed,
        total_bytes_downloaded: total_bytes,
        is_scheduler_running: false,
        ffmpeg_available: ffmpeg_status.is_available,
        ytdlp_available: ytdlp_status.is_available,
    })
}

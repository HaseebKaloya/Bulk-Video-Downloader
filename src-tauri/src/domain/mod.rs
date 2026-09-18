use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DownloadStatus {
    Queued,
    Preparing,
    Downloading,
    Pausing,
    Paused,
    RetryWait,
    Finalizing,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadStatus::Queued => "QUEUED",
            DownloadStatus::Preparing => "PREPARING",
            DownloadStatus::Downloading => "DOWNLOADING",
            DownloadStatus::Pausing => "PAUSING",
            DownloadStatus::Paused => "PAUSED",
            DownloadStatus::RetryWait => "RETRY_WAIT",
            DownloadStatus::Finalizing => "FINALIZING",
            DownloadStatus::Completed => "COMPLETED",
            DownloadStatus::Failed => "FAILED",
            DownloadStatus::Cancelled => "CANCELLED",
            DownloadStatus::Skipped => "SKIPPED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "QUEUED" => DownloadStatus::Queued,
            "PREPARING" => DownloadStatus::Preparing,
            "DOWNLOADING" => DownloadStatus::Downloading,
            "PAUSING" => DownloadStatus::Pausing,
            "PAUSED" => DownloadStatus::Paused,
            "RETRY_WAIT" => DownloadStatus::RetryWait,
            "FINALIZING" => DownloadStatus::Finalizing,
            "COMPLETED" => DownloadStatus::Completed,
            "FAILED" => DownloadStatus::Failed,
            "CANCELLED" => DownloadStatus::Cancelled,
            "SKIPPED" => DownloadStatus::Skipped,
            _ => DownloadStatus::Queued,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            DownloadStatus::Completed
                | DownloadStatus::Failed
                | DownloadStatus::Cancelled
                | DownloadStatus::Skipped
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self,
            DownloadStatus::Preparing | DownloadStatus::Downloading | DownloadStatus::Finalizing
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BatchStatus {
    Draft,
    Ready,
    Active,
    Paused,
    Stopping,
    WaitingForCompletion,
    Completed,
    Failed,
}

impl BatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BatchStatus::Draft => "DRAFT",
            BatchStatus::Ready => "READY",
            BatchStatus::Active => "ACTIVE",
            BatchStatus::Paused => "PAUSED",
            BatchStatus::Stopping => "STOPPING",
            BatchStatus::WaitingForCompletion => "WAITING_FOR_COMPLETION",
            BatchStatus::Completed => "COMPLETED",
            BatchStatus::Failed => "FAILED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "DRAFT" => BatchStatus::Draft,
            "READY" => BatchStatus::Ready,
            "ACTIVE" => BatchStatus::Active,
            "PAUSED" => BatchStatus::Paused,
            "STOPPING" => BatchStatus::Stopping,
            "WAITING_FOR_COMPLETION" => BatchStatus::WaitingForCompletion,
            "COMPLETED" => BatchStatus::Completed,
            "FAILED" => BatchStatus::Failed,
            _ => BatchStatus::Draft,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompletionPolicy {
    AllSuccessful,
    TerminalCompletion,
    PauseOnFailure,
    ContinueWithRetry,
}

impl CompletionPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompletionPolicy::AllSuccessful => "ALL_SUCCESSFUL",
            CompletionPolicy::TerminalCompletion => "TERMINAL_COMPLETION",
            CompletionPolicy::PauseOnFailure => "PAUSE_ON_FAILURE",
            CompletionPolicy::ContinueWithRetry => "CONTINUE_WITH_RETRY",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ALL_SUCCESSFUL" => CompletionPolicy::AllSuccessful,
            "TERMINAL_COMPLETION" => CompletionPolicy::TerminalCompletion,
            "PAUSE_ON_FAILURE" => CompletionPolicy::PauseOnFailure,
            "CONTINUE_WITH_RETRY" => CompletionPolicy::ContinueWithRetry,
            _ => CompletionPolicy::TerminalCompletion,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub batch_id: String,
    pub source_url: String,
    pub provider_id: Option<String>,
    pub source_id: Option<String>,
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub status: DownloadStatus,
    pub priority: i32,
    pub queue_position: i32,
    pub selected_format_id: Option<String>,
    pub selected_quality: Option<String>,
    pub selected_extension: Option<String>,
    pub output_directory: String,
    pub output_filename: Option<String>,
    pub output_path: Option<String>,
    pub partial_path: Option<String>,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub progress: f64,
    pub speed_bytes_per_second: Option<f64>,
    pub eta_seconds: Option<u64>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub next_retry_at: Option<String>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub id: String,
    pub name: String,
    pub status: BatchStatus,
    pub batch_size: u32,
    pub concurrency: u32,
    pub completion_policy: CompletionPolicy,
    pub failure_policy: String,
    pub profile_id: Option<String>,
    pub destination_directory: String,
    pub naming_template: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProfile {
    pub id: String,
    pub name: String,
    pub quality_policy: String,
    pub format_policy: String,
    pub destination_directory: String,
    pub naming_template: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEvent {
    pub id: String,
    pub download_id: String,
    pub event_type: String,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub message: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_download_directory: String,
    pub default_naming_template: String,
    pub default_batch_size: u32,
    pub default_concurrency: u32,
    pub default_completion_policy: String,
    pub default_max_retries: u32,
    pub default_profile_id: Option<String>,
    pub global_bandwidth_limit_kb: u64,
    pub auto_start_next_batch: bool,
    pub auto_open_folder: bool,
    pub preserve_partial_on_cancel: bool,
    pub notify_on_completion: bool,
    pub notify_on_failure: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| ".".into()))
            .to_string_lossy()
            .to_string();

        Self {
            default_download_directory: download_dir,
            default_naming_template: "{title}_{quality}.{format}".into(),
            default_batch_size: 10,
            default_concurrency: 3,
            default_completion_policy: "TERMINAL_COMPLETION".into(),
            default_max_retries: 3,
            default_profile_id: Some("highest-available".into()),
            global_bandwidth_limit_kb: 0,
            auto_start_next_batch: true,
            auto_open_folder: false,
            preserve_partial_on_cancel: true,
            notify_on_completion: true,
            notify_on_failure: true,
        }
    }
}

pub fn sanitize_filename(input: &str) -> String {
    let re = Regex::new(r#"[<>:"/\\|?*\x00-\x1F]"#).unwrap();
    let sanitized = re.replace_all(input, "_").trim().to_string();
    if sanitized.is_empty() {
        "video_download".to_string()
    } else {
        sanitized.chars().take(200).collect()
    }
}

pub fn render_filename(
    template: &str,
    title: &str,
    source_id: &str,
    provider: &str,
    quality: &str,
    format: &str,
    batch_id: &str,
    index: usize,
) -> String {
    let date_str = Utc::now().format("%Y-%m-%d").to_string();
    let clean_title = sanitize_filename(title);
    let clean_source_id = sanitize_filename(source_id);
    let clean_provider = sanitize_filename(provider);
    let clean_quality = sanitize_filename(quality);
    let clean_format = sanitize_filename(format);

    let mut result = template.to_string();
    result = result.replace("{title}", &clean_title);
    result = result.replace("{source_id}", &clean_source_id);
    result = result.replace("{provider}", &clean_provider);
    result = result.replace("{quality}", &clean_quality);
    result = result.replace("{format}", &clean_format);
    result = result.replace("{date}", &date_str);
    result = result.replace("{batch_id}", batch_id);
    result = result.replace("{index}", &index.to_string());

    sanitize_filename(&result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_sanitization() {
        let dirty = "A <Crazy> / Video: Title? *with* | bad chars";
        let clean = sanitize_filename(dirty);
        assert!(!clean.contains('<'));
        assert!(!clean.contains('>'));
        assert!(!clean.contains(':'));
        assert!(!clean.contains('/'));
        assert!(!clean.contains('\\'));
        assert!(!clean.contains('|'));
        assert!(!clean.contains('?'));
        assert!(!clean.contains('*'));
    }

    #[test]
    fn test_render_filename() {
        let template = "{title}_{quality}.{format}";
        let rendered = render_filename(
            template,
            "My Test Video",
            "12345",
            "DirectHTTP",
            "1080p",
            "mp4",
            "batch-1",
            1,
        );
        assert_eq!(rendered, "My Test Video_1080p.mp4");
    }
}

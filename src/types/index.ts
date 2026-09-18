export type DownloadStatus =
  | 'QUEUED'
  | 'PREPARING'
  | 'DOWNLOADING'
  | 'PAUSING'
  | 'PAUSED'
  | 'RETRY_WAIT'
  | 'FINALIZING'
  | 'COMPLETED'
  | 'FAILED'
  | 'CANCELLED'
  | 'SKIPPED';

export type BatchStatus =
  | 'DRAFT'
  | 'READY'
  | 'ACTIVE'
  | 'PAUSED'
  | 'STOPPING'
  | 'WAITING_FOR_COMPLETION'
  | 'COMPLETED'
  | 'FAILED';

export type CompletionPolicy =
  | 'ALL_SUCCESSFUL'
  | 'TERMINAL_COMPLETION'
  | 'PAUSE_ON_FAILURE'
  | 'CONTINUE_WITH_RETRY';

export interface DownloadTask {
  id: string;
  batch_id: string;
  source_url: string;
  provider_id?: string | null;
  source_id?: string | null;
  title?: string | null;
  thumbnail_url?: string | null;
  status: DownloadStatus;
  priority: number;
  queue_position: number;
  selected_format_id?: string | null;
  selected_quality?: string | null;
  selected_extension?: string | null;
  output_directory: string;
  output_filename?: string | null;
  output_path?: string | null;
  partial_path?: string | null;
  bytes_downloaded: number;
  total_bytes?: number | null;
  progress: number;
  speed_bytes_per_second?: number | null;
  eta_seconds?: number | null;
  retry_count: number;
  max_retries: number;
  next_retry_at?: string | null;
  last_error_code?: string | null;
  last_error_message?: string | null;
  created_at: string;
  started_at?: string | null;
  completed_at?: string | null;
  updated_at: string;
}

export interface Batch {
  id: string;
  name: string;
  status: BatchStatus;
  batch_size: number;
  concurrency: number;
  completion_policy: CompletionPolicy;
  failure_policy: string;
  profile_id?: string | null;
  destination_directory: string;
  naming_template: string;
  created_at: string;
  started_at?: string | null;
  completed_at?: string | null;
  updated_at: string;
}

export interface DownloadProfile {
  id: string;
  name: string;
  quality_policy: string;
  format_policy: string;
  destination_directory: string;
  naming_template: string;
  created_at: string;
  updated_at: string;
}

export interface AppSettings {
  default_download_directory: string;
  default_naming_template: string;
  default_batch_size: number;
  default_concurrency: number;
  default_completion_policy: string;
  default_max_retries: number;
  default_profile_id?: string | null;
  global_bandwidth_limit_kb: number;
  auto_start_next_batch: boolean;
  auto_open_folder: boolean;
  preserve_partial_on_cancel: boolean;
  notify_on_completion: boolean;
  notify_on_failure: boolean;
}

export interface RecoverySummary {
  tasks_reconciled: number;
  completed_files_found: number;
  partial_files_recovered: number;
  stale_tasks_reset: number;
  integrity_ok: boolean;
  message: string;
}

export interface FFmpegStatus {
  is_available: boolean;
  ffmpeg_version?: string | null;
  ffprobe_available: boolean;
  binary_path?: string | null;
  message: string;
}

export interface YtDlpStatus {
  is_available: boolean;
  version?: string | null;
  binary_path?: string | null;
  message: string;
}

export interface SystemStatus {
  total_tasks: number;
  queued_tasks: number;
  active_tasks: number;
  paused_tasks: number;
  completed_tasks: number;
  failed_tasks: number;
  total_bytes_downloaded: number;
  is_scheduler_running: boolean;
  ffmpeg_available: boolean;
  ytdlp_available: boolean;
}

export interface StorageStatus {
  available_bytes: number;
  total_bytes: number;
  path: string;
}

export interface ImportItem {
  raw_url: string;
  normalized_url: string;
  is_valid: boolean;
  is_duplicate_intra: boolean;
  is_duplicate_queue: boolean;
  title: string;
  platform?: string | null;
  error?: string | null;
}

export interface ImportPreviewResult {
  total_rows: number;
  valid_count: number;
  invalid_count: number;
  duplicate_intra_count: number;
  duplicate_queue_count: number;
  items: ImportItem[];
}

export interface CommitImportPayload {
  batch_id?: string | null;
  batch_name?: string | null;
  batch_size?: number | null;
  concurrency?: number | null;
  completion_policy?: string | null;
  profile_id?: string | null;
  selected_quality?: string | null;
  selected_format?: string | null;
  media_type?: 'video_audio' | 'audio_only' | 'video_only' | string | null;
  destination_directory?: string | null;
  naming_template?: string | null;
  duplicate_policy: 'SKIP' | 'KEEP_SEPARATE' | 'REPLACE';
  urls: string[];
  start_now: boolean;
}

export interface CommitImportResult {
  batch_id: string;
  imported_count: number;
  skipped_count: number;
}

export interface CreateBatchPayload {
  name: string;
  batch_size: number;
  concurrency: number;
  completion_policy: string;
  profile_id?: string | null;
  destination_directory: string;
  naming_template: string;
}

export interface UpdateBatchPayload {
  name?: string;
  batch_size?: number;
  concurrency?: number;
  completion_policy?: string;
}

export interface ErrorResponse {
  title: string;
  message: string;
  action: string;
  code: string;
  retryable: boolean;
}

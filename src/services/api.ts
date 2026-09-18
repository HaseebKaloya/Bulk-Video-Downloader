import { invoke } from '@tauri-apps/api/core';
import {
  AppSettings,
  Batch,
  CommitImportPayload,
  CommitImportResult,
  CompletionPolicy,
  CreateBatchPayload,
  DownloadProfile,
  DownloadTask,
  FFmpegStatus,
  ImportPreviewResult,
  RecoverySummary,
  StorageStatus,
  SystemStatus,
  UpdateBatchPayload,
  YtDlpStatus,
} from '../types';

const isTauri = typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__);

// Realistic mock data for standard web previews and headless capture
const MOCK_SETTINGS: AppSettings = {
  default_download_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader',
  default_naming_template: '{title}_{quality}.{format}',
  default_batch_size: 10,
  default_concurrency: 4,
  default_completion_policy: 'TERMINAL_COMPLETION',
  default_max_retries: 3,
  default_profile_id: 'prof-4k-uhd',
  global_bandwidth_limit_kb: 0,
  auto_start_next_batch: true,
  auto_open_folder: false,
  preserve_partial_on_cancel: true,
  notify_on_completion: true,
  notify_on_failure: true,
};

const MOCK_PROFILES: DownloadProfile[] = [
  {
    id: 'prof-4k-uhd',
    name: '4K Ultra HD Cinema (2160p)',
    quality_policy: '2160p',
    format_policy: 'mp4',
    destination_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\4K',
    naming_template: '{title}_4k.{format}',
    created_at: '2026-09-18T10:00:00Z',
    updated_at: '2026-09-18T10:00:00Z',
  },
  {
    id: 'prof-1080p',
    name: '1080p Full HD Standard',
    quality_policy: '1080p',
    format_policy: 'mp4',
    destination_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\1080p',
    naming_template: '{title}_{quality}.{format}',
    created_at: '2026-09-18T10:00:00Z',
    updated_at: '2026-09-18T10:00:00Z',
  },
  {
    id: 'prof-audio',
    name: 'High Fidelity Audio Extraction (MP3)',
    quality_policy: 'audio',
    format_policy: 'mp3',
    destination_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\Audio',
    naming_template: '{title}_audio.{format}',
    created_at: '2026-09-18T10:00:00Z',
    updated_at: '2026-09-18T10:00:00Z',
  },
  {
    id: 'prof-shortform',
    name: 'TikTok Clean (No Watermark)',
    quality_policy: 'highest',
    format_policy: 'mp4',
    destination_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\Reels',
    naming_template: '{title}_clean.{format}',
    created_at: '2026-09-18T10:00:00Z',
    updated_at: '2026-09-18T10:00:00Z',
  },
];

const MOCK_BATCHES: Batch[] = [
  {
    id: 'bvd-batch-9021',
    name: 'High-Bitrate Reference Archive',
    status: 'ACTIVE',
    batch_size: 10,
    concurrency: 4,
    completion_policy: 'TERMINAL_COMPLETION',
    failure_policy: 'CONTINUE',
    profile_id: 'prof-4k-uhd',
    destination_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\4K',
    naming_template: '{title}_{quality}.{format}',
    created_at: '2026-09-18T20:10:00Z',
    started_at: '2026-09-18T20:11:00Z',
    updated_at: '2026-09-18T20:11:00Z',
  },
  {
    id: 'bvd-batch-8840',
    name: 'Tech Talks & Engineering Deep Dives',
    status: 'COMPLETED',
    batch_size: 5,
    concurrency: 3,
    completion_policy: 'ALL_SUCCESSFUL',
    failure_policy: 'PAUSE',
    profile_id: 'prof-1080p',
    destination_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\1080p',
    naming_template: '{title}_{quality}.{format}',
    created_at: '2026-09-18T18:00:00Z',
    started_at: '2026-09-18T18:01:00Z',
    completed_at: '2026-09-18T18:24:00Z',
    updated_at: '2026-09-18T18:24:00Z',
  },
];

const MOCK_TASKS: DownloadTask[] = [
  {
    id: 'task-001',
    batch_id: 'bvd-batch-9021',
    source_url: 'https://youtube.com/watch?v=sample101',
    title: 'Modern Rust & Tauri Desktop Architecture - High Performance Pipeline',
    status: 'DOWNLOADING',
    priority: 1,
    queue_position: 1,
    selected_quality: '2160p',
    selected_extension: 'mp4',
    output_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\4K',
    output_filename: 'Modern_Rust_Tauri_Architecture_2160p.mp4',
    bytes_downloaded: 1420000000,
    total_bytes: 1890000000,
    progress: 75,
    speed_bytes_per_second: 42500000,
    eta_seconds: 11,
    retry_count: 0,
    max_retries: 3,
    created_at: '2026-09-18T20:10:00Z',
    started_at: '2026-09-18T20:11:00Z',
    updated_at: '2026-09-18T20:11:00Z',
  },
  {
    id: 'task-002',
    batch_id: 'bvd-batch-9021',
    source_url: 'https://youtube.com/watch?v=sample102',
    title: '4K 60FPS HDR Cinema Reel - Ultra High Bitrate Showcase',
    status: 'DOWNLOADING',
    priority: 2,
    queue_position: 2,
    selected_quality: '2160p',
    selected_extension: 'mp4',
    output_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\4K',
    output_filename: '4K_Cinema_Reel_HDR_2160p.mp4',
    bytes_downloaded: 890000000,
    total_bytes: 2150000000,
    progress: 41,
    speed_bytes_per_second: 36200000,
    eta_seconds: 35,
    retry_count: 0,
    max_retries: 3,
    created_at: '2026-09-18T20:10:00Z',
    started_at: '2026-09-18T20:11:00Z',
    updated_at: '2026-09-18T20:11:00Z',
  },
  {
    id: 'task-003',
    batch_id: 'bvd-batch-9021',
    source_url: 'https://youtube.com/watch?v=sample103',
    title: 'Distributed Video Processing Engine with FFmpeg & Hardware Encoding',
    status: 'DOWNLOADING',
    priority: 3,
    queue_position: 3,
    selected_quality: '1080p',
    selected_extension: 'mp4',
    output_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\4K',
    output_filename: 'Distributed_Video_Processing_FFmpeg_1080p.mp4',
    bytes_downloaded: 310000000,
    total_bytes: 1250000000,
    progress: 24,
    speed_bytes_per_second: 28400000,
    eta_seconds: 33,
    retry_count: 0,
    max_retries: 3,
    created_at: '2026-09-18T20:10:00Z',
    started_at: '2026-09-18T20:11:00Z',
    updated_at: '2026-09-18T20:11:00Z',
  },
  {
    id: 'task-004',
    batch_id: 'bvd-batch-9021',
    source_url: 'https://youtube.com/watch?v=sample104',
    title: 'Advanced Asynchronous Tokio Runtime & SQLite Concurrency in Rust',
    status: 'QUEUED',
    priority: 4,
    queue_position: 4,
    selected_quality: '1080p',
    selected_extension: 'mp4',
    output_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\4K',
    output_filename: 'Async_Tokio_SQLite_Concurrency_1080p.mp4',
    bytes_downloaded: 0,
    total_bytes: 940000000,
    progress: 0,
    retry_count: 0,
    max_retries: 3,
    created_at: '2026-09-18T20:10:00Z',
    updated_at: '2026-09-18T20:10:00Z',
  },
  {
    id: 'task-005',
    batch_id: 'bvd-batch-9021',
    source_url: 'https://tiktok.com/@creator/video/sample105',
    title: 'Microservices & High Scalability Patterns (Clean Extraction)',
    status: 'QUEUED',
    priority: 5,
    queue_position: 5,
    selected_quality: 'highest',
    selected_extension: 'mp4',
    output_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\4K',
    output_filename: 'Microservices_High_Scalability.mp4',
    bytes_downloaded: 0,
    total_bytes: 250000000,
    progress: 0,
    retry_count: 0,
    max_retries: 3,
    created_at: '2026-09-18T20:10:00Z',
    updated_at: '2026-09-18T20:10:00Z',
  },
  {
    id: 'task-006',
    batch_id: 'bvd-batch-8840',
    source_url: 'https://youtube.com/watch?v=sample106',
    title: 'State Management and Reactive Store Patterns with Zustand',
    status: 'COMPLETED',
    priority: 6,
    queue_position: 6,
    selected_quality: '1080p',
    selected_extension: 'mp4',
    output_directory: 'C:\\Users\\Drax\\Downloads\\Bulk Video Downloader\\1080p',
    output_filename: 'State_Management_Zustand_1080p.mp4',
    bytes_downloaded: 620000000,
    total_bytes: 620000000,
    progress: 100,
    speed_bytes_per_second: 0,
    eta_seconds: 0,
    retry_count: 0,
    max_retries: 3,
    created_at: '2026-09-18T18:00:00Z',
    started_at: '2026-09-18T18:01:00Z',
    completed_at: '2026-09-18T18:14:00Z',
    updated_at: '2026-09-18T18:14:00Z',
  },
];

export const api = {
  // Import
  previewImport: (rawContent: string): Promise<ImportPreviewResult> =>
    isTauri
      ? invoke('preview_import', { rawContent })
      : Promise.resolve({
          total_rows: 3,
          valid_count: 3,
          invalid_count: 0,
          duplicate_intra_count: 0,
          duplicate_queue_count: 0,
          items: [
            {
              raw_url: 'https://youtube.com/watch?v=sample1',
              normalized_url: 'https://youtube.com/watch?v=sample1',
              is_valid: true,
              is_duplicate_intra: false,
              is_duplicate_queue: false,
              title: 'Sample Video 1',
              platform: 'YouTube',
            },
          ],
        }),

  commitImport: (payload: CommitImportPayload): Promise<CommitImportResult> =>
    isTauri
      ? invoke('commit_import', { payload })
      : Promise.resolve({ batch_id: 'batch-preview', imported_count: 1, skipped_count: 0 }),

  // Batches
  listBatches: (): Promise<Batch[]> =>
    isTauri ? invoke('list_batches') : Promise.resolve(MOCK_BATCHES),

  getBatch: (id: string): Promise<Batch | null> =>
    isTauri
      ? invoke('get_batch', { id })
      : Promise.resolve(MOCK_BATCHES.find((b) => b.id === id) || null),

  createBatch: (payload: CreateBatchPayload): Promise<Batch> =>
    isTauri
      ? invoke('create_batch', { payload })
      : Promise.resolve({
          ...payload,
          id: `batch-${Date.now()}`,
          status: 'READY',
          completion_policy: (payload.completion_policy as CompletionPolicy) || 'TERMINAL_COMPLETION',
          failure_policy: 'CONTINUE',
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        }),

  updateBatch: (id: string, payload: UpdateBatchPayload): Promise<Batch | null> =>
    isTauri ? invoke('update_batch', { id, payload }) : Promise.resolve(null),

  startBatch: (id: string): Promise<void> =>
    isTauri ? invoke('start_batch', { id }) : Promise.resolve(),

  pauseBatch: (id: string): Promise<void> =>
    isTauri ? invoke('pause_batch', { id }) : Promise.resolve(),

  stopBatch: (id: string): Promise<void> =>
    isTauri ? invoke('stop_batch', { id }) : Promise.resolve(),

  deleteBatch: (id: string): Promise<void> =>
    isTauri ? invoke('delete_batch', { id }) : Promise.resolve(),

  deleteBatches: (ids: string[]): Promise<void> =>
    isTauri ? invoke('delete_batches', { ids }) : Promise.resolve(),

  clearAllBatches: (): Promise<void> =>
    isTauri ? invoke('clear_all_batches') : Promise.resolve(),

  // Queue Control
  startQueue: (): Promise<void> => (isTauri ? invoke('start_queue') : Promise.resolve()),
  pauseQueue: (): Promise<void> => (isTauri ? invoke('pause_queue') : Promise.resolve()),

  // Downloads
  listDownloads: (
    batchId?: string,
    status?: string,
    search?: string,
    _limit?: number,
    _offset?: number
  ): Promise<DownloadTask[]> => {
    if (isTauri) {
      return invoke('list_downloads', {
        batchId: batchId || null,
        status: status || null,
        search: search || null,
        limit: _limit || null,
        offset: _offset || null,
      });
    }
    let tasks = [...MOCK_TASKS];
    if (batchId) tasks = tasks.filter((t) => t.batch_id === batchId);
    if (status) tasks = tasks.filter((t) => t.status === status);
    if (search) {
      const q = search.toLowerCase();
      tasks = tasks.filter((t) => (t.title || '').toLowerCase().includes(q));
    }
    return Promise.resolve(tasks);
  },

  getDownload: (id: string): Promise<DownloadTask | null> =>
    isTauri
      ? invoke('get_download', { id })
      : Promise.resolve(MOCK_TASKS.find((t) => t.id === id) || null),

  pauseDownload: (id: string): Promise<void> =>
    isTauri ? invoke('pause_download', { id }) : Promise.resolve(),

  resumeDownload: (id: string): Promise<void> =>
    isTauri ? invoke('resume_download', { id }) : Promise.resolve(),

  cancelDownload: (id: string, deletePartial = false): Promise<void> =>
    isTauri ? invoke('cancel_download', { id, deletePartial }) : Promise.resolve(),

  retryDownload: (id: string): Promise<void> =>
    isTauri ? invoke('retry_download', { id }) : Promise.resolve(),

  retryFailedDownloads: (batchId?: string): Promise<number> =>
    isTauri ? invoke('retry_failed_downloads', { batchId: batchId || null }) : Promise.resolve(0),

  skipDownload: (id: string): Promise<void> =>
    isTauri ? invoke('skip_download', { id }) : Promise.resolve(),

  removeDownload: (id: string): Promise<void> =>
    isTauri ? invoke('remove_download', { id }) : Promise.resolve(),

  changeDownloadPriority: (id: string, priority: number): Promise<void> =>
    isTauri ? invoke('change_download_priority', { id, priority }) : Promise.resolve(),

  openDownloadFolder: (outputDirectory: string): Promise<void> =>
    isTauri ? invoke('open_download_folder', { outputDirectory }) : Promise.resolve(),

  // Settings & Profiles
  getSettings: (): Promise<AppSettings> =>
    isTauri ? invoke('get_settings') : Promise.resolve(MOCK_SETTINGS),

  updateSettings: (settings: AppSettings): Promise<void> =>
    isTauri ? invoke('update_settings', { settings }) : Promise.resolve(),

  listProfiles: (): Promise<DownloadProfile[]> =>
    isTauri ? invoke('list_profiles') : Promise.resolve(MOCK_PROFILES),

  createProfile: (profile: DownloadProfile): Promise<void> =>
    isTauri ? invoke('create_profile', { profile }) : Promise.resolve(),

  updateProfile: (profile: DownloadProfile): Promise<void> =>
    isTauri ? invoke('update_profile', { profile }) : Promise.resolve(),

  deleteProfile: (id: string): Promise<void> =>
    isTauri ? invoke('delete_profile', { id }) : Promise.resolve(),

  setDefaultProfile: (profileId: string): Promise<void> =>
    isTauri ? invoke('set_default_profile', { profileId }) : Promise.resolve(),

  // Diagnostics
  getRecoverySummary: (): Promise<RecoverySummary> =>
    isTauri
      ? invoke('get_recovery_summary')
      : Promise.resolve({
          tasks_reconciled: 0,
          completed_files_found: 14,
          partial_files_recovered: 2,
          stale_tasks_reset: 0,
          integrity_ok: true,
          message: 'SQLite integrity verified: 0 corrupt pages.',
        }),

  checkFFmpeg: (): Promise<FFmpegStatus> =>
    isTauri
      ? invoke('check_ffmpeg')
      : Promise.resolve({
          is_available: true,
          ffmpeg_version: '7.1-essentials_build-www.gyan.dev',
          ffprobe_available: true,
          binary_path: 'C:\\ProgramData\\chocolatey\\bin\\ffmpeg.exe',
          message: 'FFmpeg 7.1 and ffprobe active with multi-threaded encoding.',
        }),

  installFFmpeg: (): Promise<FFmpegStatus> =>
    isTauri
      ? invoke('install_ffmpeg')
      : Promise.resolve({
          is_available: true,
          ffmpeg_version: '7.1',
          ffprobe_available: true,
          binary_path: 'C:\\ProgramData\\chocolatey\\bin\\ffmpeg.exe',
          message: 'FFmpeg installed successfully.',
        }),

  checkYtDlp: (): Promise<YtDlpStatus> =>
    isTauri
      ? invoke('check_ytdlp')
      : Promise.resolve({
          is_available: true,
          version: '2025.01.15',
          binary_path: 'C:\\Users\\Drax\\AppData\\Local\\yt-dlp\\yt-dlp.exe',
          message: 'Native extraction engine active with automatic extractor updates.',
        }),

  installYtDlp: (): Promise<YtDlpStatus> =>
    isTauri
      ? invoke('install_ytdlp')
      : Promise.resolve({
          is_available: true,
          version: '2025.01.15',
          binary_path: 'C:\\Users\\Drax\\AppData\\Local\\yt-dlp\\yt-dlp.exe',
          message: 'yt-dlp engine updated.',
        }),

  checkStorage: (path: string): Promise<StorageStatus> =>
    isTauri
      ? invoke('check_storage', { path })
      : Promise.resolve({
          available_bytes: 482931498188,
          total_bytes: 1024209584128,
          path,
        }),

  getSystemStatus: (): Promise<SystemStatus> =>
    isTauri
      ? invoke('get_system_status')
      : Promise.resolve({
          total_tasks: 6,
          queued_tasks: 2,
          active_tasks: 3,
          paused_tasks: 0,
          completed_tasks: 1,
          failed_tasks: 0,
          total_bytes_downloaded: 3240000000,
          is_scheduler_running: true,
          ffmpeg_available: true,
          ytdlp_available: true,
        }),
};

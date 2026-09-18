import { invoke } from '@tauri-apps/api/core';
import {
  AppSettings,
  Batch,
  CommitImportPayload,
  CommitImportResult,
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

export const api = {
  // Import
  previewImport: (rawContent: string): Promise<ImportPreviewResult> =>
    invoke('preview_import', { rawContent }),

  commitImport: (payload: CommitImportPayload): Promise<CommitImportResult> =>
    invoke('commit_import', { payload }),

  // Batches
  listBatches: (): Promise<Batch[]> => invoke('list_batches'),

  getBatch: (id: string): Promise<Batch | null> => invoke('get_batch', { id }),

  createBatch: (payload: CreateBatchPayload): Promise<Batch> =>
    invoke('create_batch', { payload }),

  updateBatch: (id: string, payload: UpdateBatchPayload): Promise<Batch | null> =>
    invoke('update_batch', { id, payload }),

  startBatch: (id: string): Promise<void> => invoke('start_batch', { id }),

  pauseBatch: (id: string): Promise<void> => invoke('pause_batch', { id }),

  stopBatch: (id: string): Promise<void> => invoke('stop_batch', { id }),

  deleteBatch: (id: string): Promise<void> => invoke('delete_batch', { id }),
  deleteBatches: (ids: string[]): Promise<void> => invoke('delete_batches', { ids }),
  clearAllBatches: (): Promise<void> => invoke('clear_all_batches'),

  // Queue Control
  startQueue: (): Promise<void> => invoke('start_queue'),
  pauseQueue: (): Promise<void> => invoke('pause_queue'),

  // Downloads
  listDownloads: (
    batchId?: string,
    status?: string,
    search?: string,
    limit?: number,
    offset?: number
  ): Promise<DownloadTask[]> =>
    invoke('list_downloads', {
      batchId: batchId || null,
      status: status || null,
      search: search || null,
      limit: limit || null,
      offset: offset || null,
    }),

  getDownload: (id: string): Promise<DownloadTask | null> =>
    invoke('get_download', { id }),

  pauseDownload: (id: string): Promise<void> => invoke('pause_download', { id }),

  resumeDownload: (id: string): Promise<void> => invoke('resume_download', { id }),

  cancelDownload: (id: string, deletePartial = false): Promise<void> =>
    invoke('cancel_download', { id, deletePartial }),

  retryDownload: (id: string): Promise<void> => invoke('retry_download', { id }),

  retryFailedDownloads: (batchId?: string): Promise<number> =>
    invoke('retry_failed_downloads', { batchId: batchId || null }),

  skipDownload: (id: string): Promise<void> => invoke('skip_download', { id }),

  removeDownload: (id: string): Promise<void> => invoke('remove_download', { id }),

  changeDownloadPriority: (id: string, priority: number): Promise<void> =>
    invoke('change_download_priority', { id, priority }),

  openDownloadFolder: (outputDirectory: string): Promise<void> =>
    invoke('open_download_folder', { outputDirectory }),

  // Settings & Profiles
  getSettings: (): Promise<AppSettings> => invoke('get_settings'),

  updateSettings: (settings: AppSettings): Promise<void> =>
    invoke('update_settings', { settings }),

  listProfiles: (): Promise<DownloadProfile[]> => invoke('list_profiles'),

  createProfile: (profile: DownloadProfile): Promise<void> =>
    invoke('create_profile', { profile }),

  updateProfile: (profile: DownloadProfile): Promise<void> =>
    invoke('update_profile', { profile }),

  deleteProfile: (id: string): Promise<void> => invoke('delete_profile', { id }),

  setDefaultProfile: (profileId: string): Promise<void> =>
    invoke('set_default_profile', { profileId }),

  // Diagnostics
  getRecoverySummary: (): Promise<RecoverySummary> =>
    invoke('get_recovery_summary'),

  checkFFmpeg: (): Promise<FFmpegStatus> => invoke('check_ffmpeg'),
  installFFmpeg: (): Promise<FFmpegStatus> => invoke('install_ffmpeg'),

  checkYtDlp: (): Promise<YtDlpStatus> => invoke('check_ytdlp'),
  installYtDlp: (): Promise<YtDlpStatus> => invoke('install_ytdlp'),

  checkStorage: (path: string): Promise<StorageStatus> =>
    invoke('check_storage', { path }),

  getSystemStatus: (): Promise<SystemStatus> => invoke('get_system_status'),
};

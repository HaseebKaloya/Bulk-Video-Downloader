import { create } from 'zustand';
import { api } from '../services/api';
import {
  AppSettings,
  Batch,
  DownloadProfile,
  DownloadTask,
  ImportPreviewResult,
  RecoverySummary,
  SystemStatus,
} from '../types';

export type NavigationTab =
  | 'dashboard'
  | 'downloads'
  | 'batches'
  | 'completed'
  | 'failed'
  | 'profiles'
  | 'settings'
  | 'about';

export interface ToastItem {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  message: string;
}

export interface ConfirmDialogConfig {
  title: string;
  message: string;
  confirmLabel?: string;
  isDestructive?: boolean;
  onConfirm: () => void;
}

interface AppStoreState {
  // Navigation
  activeTab: NavigationTab;
  setActiveTab: (tab: NavigationTab) => void;

  // Data
  tasks: DownloadTask[];
  batches: Batch[];
  profiles: DownloadProfile[];
  settings: AppSettings | null;
  systemStatus: SystemStatus | null;
  recoverySummary: RecoverySummary | null;

  // Queue Filters
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  statusFilter: string | null;
  setStatusFilter: (status: string | null) => void;
  batchFilter: string | null;
  setBatchFilter: (batchId: string | null) => void;

  // Import Dialog
  isImportModalOpen: boolean;
  openImportModal: () => void;
  closeImportModal: () => void;
  importPreview: ImportPreviewResult | null;
  setImportPreview: (preview: ImportPreviewResult | null) => void;

  // Confirm Dialog
  confirmDialog: ConfirmDialogConfig | null;
  showConfirmDialog: (config: ConfirmDialogConfig) => void;
  hideConfirmDialog: () => void;

  // Toasts
  toasts: ToastItem[];
  addToast: (toast: Omit<ToastItem, 'id'>) => void;
  removeToast: (id: string) => void;

  // Actions
  refreshTasks: () => Promise<void>;
  refreshBatches: () => Promise<void>;
  refreshSettings: () => Promise<void>;
  refreshProfiles: () => Promise<void>;
  refreshSystemStatus: () => Promise<void>;
  runStartupRecovery: () => Promise<void>;

  // Real-time Event Handlers
  updateTaskProgress: (payload: {
    task_id: string;
    bytes_downloaded: number;
    total_bytes?: number | null;
    progress: number;
    speed_bytes_per_second?: number | null;
    eta_seconds?: number | null;
  }) => void;
  updateTaskStatus: (taskId: string, status: DownloadTask['status']) => void;
}

export const useAppStore = create<AppStoreState>((set, get) => ({
  activeTab: 'dashboard',
  setActiveTab: (tab) => set({ activeTab: tab }),

  tasks: [],
  batches: [],
  profiles: [],
  settings: null,
  systemStatus: null,
  recoverySummary: null,

  searchQuery: '',
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  statusFilter: null,
  setStatusFilter: (statusFilter) => set({ statusFilter }),
  batchFilter: null,
  setBatchFilter: (batchFilter) => set({ batchFilter }),

  isImportModalOpen: false,
  openImportModal: () => set({ isImportModalOpen: true }),
  closeImportModal: () => set({ isImportModalOpen: false, importPreview: null }),
  importPreview: null,
  setImportPreview: (importPreview) => set({ importPreview }),

  confirmDialog: null,
  showConfirmDialog: (confirmDialog) => set({ confirmDialog }),
  hideConfirmDialog: () => set({ confirmDialog: null }),

  toasts: [],
  addToast: (toast) => {
    const id = Math.random().toString(36).substring(2, 9);
    set((state) => ({ toasts: [...state.toasts, { ...toast, id }] }));
    setTimeout(() => {
      get().removeToast(id);
    }, 5000);
  },
  removeToast: (id) => {
    set((state) => ({ toasts: state.toasts.filter((t) => t.id !== id) }));
  },

  refreshTasks: async () => {
    try {
      const incomingTasks = await api.listDownloads();
      set((state) => {
        const mergedTasks = incomingTasks.map((newTask) => {
          const currentTask = state.tasks.find((t) => t.id === newTask.id);
          if (
            currentTask &&
            (currentTask.status === 'DOWNLOADING' || currentTask.status === 'PREPARING') &&
            (newTask.status === 'DOWNLOADING' || newTask.status === 'PREPARING')
          ) {
            return {
              ...newTask,
              bytes_downloaded: Math.max(currentTask.bytes_downloaded, newTask.bytes_downloaded),
              total_bytes: currentTask.total_bytes || newTask.total_bytes,
              progress: Math.max(currentTask.progress, newTask.progress),
              speed_bytes_per_second: currentTask.speed_bytes_per_second ?? newTask.speed_bytes_per_second,
              eta_seconds: currentTask.eta_seconds ?? newTask.eta_seconds,
            };
          }
          return newTask;
        });
        return { tasks: mergedTasks };
      });
    } catch (err) {
      console.error('Failed to load downloads:', err);
    }
  },

  refreshBatches: async () => {
    try {
      const batches = await api.listBatches();
      set({ batches });
    } catch (err) {
      console.error('Failed to load batches:', err);
    }
  },

  refreshSettings: async () => {
    try {
      const settings = await api.getSettings();
      set({ settings });
    } catch (err) {
      console.error('Failed to load settings:', err);
    }
  },

  refreshProfiles: async () => {
    try {
      const profiles = await api.listProfiles();
      set({ profiles });
    } catch (err) {
      console.error('Failed to load profiles:', err);
    }
  },

  refreshSystemStatus: async () => {
    try {
      const systemStatus = await api.getSystemStatus();
      set({ systemStatus });
    } catch (err) {
      console.error('Failed to load system status:', err);
    }
  },

  runStartupRecovery: async () => {
    try {
      const summary = await api.getRecoverySummary();
      set({ recoverySummary: summary });
      if (summary.tasks_reconciled > 0) {
        get().addToast({
          type: 'info',
          title: 'Startup Recovery',
          message: summary.message,
        });
      }
    } catch (err) {
      console.error('Recovery failed:', err);
    }
  },

  updateTaskProgress: (payload) => {
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === payload.task_id
          ? {
              ...t,
              status:
                t.status === 'COMPLETED' || t.status === 'FAILED'
                  ? t.status
                  : 'DOWNLOADING',
              bytes_downloaded: payload.bytes_downloaded,
              total_bytes: payload.total_bytes ?? t.total_bytes,
              progress: payload.progress,
              speed_bytes_per_second: payload.speed_bytes_per_second,
              eta_seconds: payload.eta_seconds,
            }
          : t
      ),
    }));
  },

  updateTaskStatus: (taskId, status) => {
    const now = new Date().toISOString();
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === taskId
          ? {
              ...t,
              status,
              ...(status === 'COMPLETED'
                ? { completed_at: now, progress: 100, eta_seconds: 0 }
                : {}),
              ...(status === 'COMPLETED' || status === 'FAILED' || status === 'PAUSED'
                ? { speed_bytes_per_second: 0 }
                : {}),
            }
          : t
      ),
    }));
    get().refreshSystemStatus();
  },
}));

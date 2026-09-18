import React, { useEffect } from 'react';
import { Sidebar } from './Sidebar';
import { Header } from './Header';
import { ToastContainer } from '../ui/ToastContainer';
import { ConfirmDialog } from '../ui/ConfirmDialog';
import { useAppStore } from '../../stores/useAppStore';
import { subscribeToEvent } from '../../services/events';
import styles from './Layout.module.css';

interface LayoutProps {
  children: React.ReactNode;
}

export const Layout: React.FC<LayoutProps> = ({ children }) => {
  const {
    refreshTasks,
    refreshBatches,
    refreshSettings,
    refreshProfiles,
    refreshSystemStatus,
    runStartupRecovery,
    updateTaskProgress,
    updateTaskStatus,
    addToast,
  } = useAppStore();

  useEffect(() => {
    // Initial bootstrap
    refreshSettings();
    refreshProfiles();
    refreshBatches();
    refreshTasks();
    refreshSystemStatus();
    runStartupRecovery();

    // Event listeners
    let unlistenPreparing: (() => void) | undefined;
    let unlistenProgress: (() => void) | undefined;
    let unlistenCompleted: (() => void) | undefined;
    let unlistenFailed: (() => void) | undefined;
    let unlistenPaused: (() => void) | undefined;
    let unlistenQueued: (() => void) | undefined;
    let unlistenRetryScheduled: (() => void) | undefined;
    let unlistenCancelled: (() => void) | undefined;
    let unlistenBatchStarted: (() => void) | undefined;
    let unlistenBatchPaused: (() => void) | undefined;
    let unlistenBatchCompleted: (() => void) | undefined;
    let unlistenSchedulerState: (() => void) | undefined;

    const setupListeners = async () => {
      unlistenPreparing = await subscribeToEvent<{ task_id: string }>(
        'download.preparing',
        (payload) => {
          updateTaskStatus(payload.data.task_id, 'PREPARING');
          refreshTasks();
        }
      );

      unlistenProgress = await subscribeToEvent<{
        task_id: string;
        bytes_downloaded: number;
        total_bytes?: number | null;
        progress: number;
        speed_bytes_per_second?: number | null;
        eta_seconds?: number | null;
      }>('download.progress', (payload) => {
        updateTaskProgress(payload.data);
      });

      unlistenCompleted = await subscribeToEvent<{
        task_id: string;
        output_path: string;
        output_filename: string;
      }>('download.completed', (payload) => {
        updateTaskStatus(payload.data.task_id, 'COMPLETED');
        addToast({
          type: 'success',
          title: 'Download Complete',
          message: payload.data.output_filename,
        });
        refreshTasks();
        refreshBatches();
      });

      unlistenFailed = await subscribeToEvent<{
        task_id: string;
        error_code: string;
        error_message: string;
      }>('download.failed', (payload) => {
        updateTaskStatus(payload.data.task_id, 'FAILED');
        addToast({
          type: 'error',
          title: 'Download Failed',
          message: payload.data.error_message,
        });
        refreshTasks();
      });

      unlistenPaused = await subscribeToEvent<{ task_id: string }>(
        'download.paused',
        (payload) => {
          updateTaskStatus(payload.data.task_id, 'PAUSED');
          refreshTasks();
          refreshBatches();
        }
      );

      unlistenQueued = await subscribeToEvent<{ task_id: string }>(
        'download.queued',
        (payload) => {
          updateTaskStatus(payload.data.task_id, 'QUEUED');
          refreshTasks();
          refreshBatches();
        }
      );

      unlistenRetryScheduled = await subscribeToEvent<{ task_id: string }>(
        'download.retry_scheduled',
        () => {
          refreshTasks();
        }
      );

      unlistenCancelled = await subscribeToEvent<{ task_id: string }>(
        'download.cancelled',
        (payload) => {
          updateTaskStatus(payload.data.task_id, 'CANCELLED');
          refreshTasks();
          refreshBatches();
        }
      );

      unlistenBatchStarted = await subscribeToEvent<{ batch_id: string }>(
        'batch.started',
        () => {
          refreshBatches();
          refreshTasks();
        }
      );

      unlistenBatchPaused = await subscribeToEvent<{ batch_id: string }>(
        'batch.paused',
        () => {
          refreshBatches();
          refreshTasks();
        }
      );

      unlistenBatchCompleted = await subscribeToEvent<{ batch_id: string }>(
        'batch.completed',
        (payload) => {
          addToast({
            type: 'success',
            title: 'Batch Completed',
            message: `Batch ${payload.data.batch_id.slice(0, 8)} finished successfully.`,
          });
          refreshBatches();
          refreshTasks();
        }
      );

      unlistenSchedulerState = await subscribeToEvent<{ is_running: boolean }>(
        'scheduler.state_changed',
        () => {
          refreshSystemStatus();
          refreshTasks();
          refreshBatches();
        }
      );
    };

    setupListeners();

    // Periodic status refresh (fallback for real-time events)
    const interval = setInterval(() => {
      refreshTasks();
      refreshBatches();
      refreshSystemStatus();
    }, 2500);

    return () => {
      clearInterval(interval);
      unlistenPreparing?.();
      unlistenProgress?.();
      unlistenCompleted?.();
      unlistenFailed?.();
      unlistenPaused?.();
      unlistenQueued?.();
      unlistenRetryScheduled?.();
      unlistenCancelled?.();
      unlistenBatchStarted?.();
      unlistenBatchPaused?.();
      unlistenBatchCompleted?.();
      unlistenSchedulerState?.();
    };
  }, []);

  return (
    <div className={styles.appContainer}>
      <Sidebar />
      <div className={styles.mainContent}>
        <Header />
        <main className={styles.pageBody}>{children}</main>
      </div>
      <ToastContainer />
      <ConfirmDialog />
    </div>
  );
};

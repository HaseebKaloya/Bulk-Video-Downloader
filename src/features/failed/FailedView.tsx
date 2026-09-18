import React from 'react';
import {
  AlertCircle,
  RotateCcw,
  Copy,
  Trash2,
  CheckCircle2,
} from 'lucide-react';
import { api } from '../../services/api';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from '../../components/ui/Button';
import styles from './Failed.module.css';

export const FailedView: React.FC = () => {
  const { tasks, addToast, refreshTasks, showConfirmDialog } = useAppStore();

  const failedTasks = tasks.filter((t) => t.status === 'FAILED');

  const handleRetryAll = async () => {
    try {
      const count = await api.retryFailedDownloads();
      addToast({ type: 'info', message: `Queued ${count} failed tasks for retry.` });
      await refreshTasks();
    } catch {
      addToast({ type: 'error', message: 'Failed to retry tasks.' });
    }
  };

  const handleCopyDiagnostics = (task: (typeof failedTasks)[0]) => {
    const diag = JSON.stringify(
      {
        id: task.id,
        url: task.source_url,
        errorCode: task.last_error_code,
        errorMessage: task.last_error_message,
        retries: task.retry_count,
        maxRetries: task.max_retries,
      },
      null,
      2
    );
    navigator.clipboard.writeText(diag);
    addToast({ type: 'success', message: 'Diagnostic information copied to clipboard.' });
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>Failed Tasks &amp; Diagnostics</h1>
          <p className={styles.subtitle}>
            Inspect failure diagnostics, view error categories, and trigger policy-based retries.
          </p>
        </div>

        {failedTasks.length > 0 && (
          <Button
            variant="primary"
            size="sm"
            icon={<RotateCcw size={15} />}
            onClick={handleRetryAll}
          >
            Retry All Failed ({failedTasks.length})
          </Button>
        )}
      </div>

      {failedTasks.length === 0 ? (
        <div className={styles.emptyCard}>
          <CheckCircle2 size={44} className={styles.emptyIconSuccess} />
          <h3 className={styles.emptyTitle}>Zero failed downloads</h3>
          <p className={styles.emptySubtitle}>
            All active, queued, and completed downloads are operating reliably without errors.
          </p>
        </div>
      ) : (
        <div className={styles.failedList}>
          {failedTasks.map((task) => (
            <div key={task.id} className={styles.failedCard}>
              <div className={styles.cardHeader}>
                <div className={styles.errorBanner}>
                  <AlertCircle size={18} className={styles.errorIcon} />
                  <div>
                    <h3 className={styles.taskTitle}>
                      {task.title || task.output_filename || task.id}
                    </h3>
                    <span className={styles.taskUrl}>{task.source_url}</span>
                  </div>
                </div>

                <div className={styles.headerActions}>
                  <Button
                    variant="outline"
                    size="sm"
                    icon={<RotateCcw size={14} />}
                    onClick={() => api.retryDownload(task.id).then(refreshTasks)}
                  >
                    Retry
                  </Button>

                  <Button
                    variant="ghost"
                    size="sm"
                    icon={<Copy size={14} />}
                    onClick={() => handleCopyDiagnostics(task)}
                    title="Copy diagnostics to clipboard"
                  />

                  <Button
                    variant="ghost"
                    size="sm"
                    icon={<Trash2 size={14} />}
                    onClick={() =>
                      showConfirmDialog({
                        title: 'Discard Failed Task',
                        message: 'Discard this failed task from the queue?',
                        confirmLabel: 'Discard',
                        isDestructive: true,
                        onConfirm: () => api.removeDownload(task.id).then(refreshTasks),
                      })
                    }
                    title="Remove task"
                  />
                </div>
              </div>

              <div className={styles.diagnosticsBox}>
                <div className={styles.diagItem}>
                  <span className={styles.diagLabel}>Error Code:</span>
                  <span className={styles.errorCodeTag}>
                    {task.last_error_code || 'ERR_GENERIC'}
                  </span>
                </div>

                <div className={styles.diagItem}>
                  <span className={styles.diagLabel}>Attempts:</span>
                  <span className={styles.diagValue}>
                    {task.retry_count} of {task.max_retries} max retries
                  </span>
                </div>

                <div className={styles.diagMessage}>
                  <span className={styles.diagLabel}>Message:</span>
                  <p className={styles.errorMessageText}>
                    {task.last_error_message || 'An unclassified transfer failure occurred.'}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

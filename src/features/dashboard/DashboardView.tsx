import React, { useState, useEffect, useMemo } from 'react';
import {
  DownloadCloud,
  Layers,
  CheckCircle2,
  Pause,
  Play,
  RotateCcw,
  FolderOpen,
  Plus,
  ArrowRight,
  Zap,
} from 'lucide-react';
import { api } from '../../services/api';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from '../../components/ui/Button';
import { Badge } from '../../components/ui/Badge';
import { ProgressBar } from '../../components/ui/ProgressBar';
import styles from './Dashboard.module.css';

export const DashboardView: React.FC = () => {
  const {
    tasks,
    batches,
    setActiveTab,
    openImportModal,
    settings,
    addToast,
    refreshTasks,
    refreshBatches,
  } = useAppStore();

  const [filterMode, setFilterMode] = useState<'all' | 'downloading' | 'queued' | 'completed'>('all');

  // Heartbeat to ensure real-time updates and active stream tracking
  useEffect(() => {
    const timer = setInterval(() => {
      refreshTasks();
    }, 1500);
    return () => clearInterval(timer);
  }, [refreshTasks]);

  // Categorized tasks for real-time tracking
  const downloadingTasks = useMemo(() => {
    return tasks.filter(
      (t) =>
        t.status === 'DOWNLOADING' ||
        t.status === 'PREPARING' ||
        t.status === 'FINALIZING'
    );
  }, [tasks]);

  const queuedList = useMemo(() => {
    return tasks
      .filter((t) => t.status === 'QUEUED' || t.status === 'RETRY_WAIT')
      .sort((a, b) => {
        // Step-by-step newly added videos at the top
        const timeA = a.created_at || '';
        const timeB = b.created_at || '';
        return timeB.localeCompare(timeA);
      });
  }, [tasks]);

  const completedList = useMemo(() => {
    return tasks
      .filter((t) => t.status === 'COMPLETED')
      .sort((a, b) => {
        // Video downloaded at the last shows at the top (LIFO)
        const timeA = a.completed_at || a.updated_at || a.created_at || '';
        const timeB = b.completed_at || b.updated_at || b.created_at || '';
        return timeB.localeCompare(timeA);
      });
  }, [tasks]);

  const dashboardTasks = useMemo(() => {
    if (filterMode === 'downloading') return downloadingTasks;
    if (filterMode === 'queued') return queuedList;
    if (filterMode === 'completed') return completedList;

    // 'all' Activity:
    // 1. Shows videos which are currently downloading at the top!
    // 2. Shows step by step newly added videos next!
    // 3. Shows recently completed videos (last downloaded at the top) next!
    const otherTasks = tasks.filter(
      (t) =>
        t.status !== 'DOWNLOADING' &&
        t.status !== 'PREPARING' &&
        t.status !== 'FINALIZING' &&
        t.status !== 'QUEUED' &&
        t.status !== 'RETRY_WAIT' &&
        t.status !== 'COMPLETED'
    );
    const combined = [...downloadingTasks, ...queuedList, ...completedList, ...otherTasks];
    return combined.slice(0, 8);
  }, [filterMode, downloadingTasks, queuedList, completedList, tasks]);

  const queuedTasks = tasks.filter((t) => t.status === 'QUEUED');
  const activeTasks = downloadingTasks;
  const pausedTasks = tasks.filter((t) => t.status === 'PAUSED');
  const completedTasks = tasks.filter((t) => t.status === 'COMPLETED');
  const failedTasks = tasks.filter((t) => t.status === 'FAILED');

  const totalBytes = tasks.reduce((acc, t) => acc + t.bytes_downloaded, 0);
  const totalSpeed = activeTasks.reduce(
    (acc, t) => acc + (t.speed_bytes_per_second || 0),
    0
  );

  const activeBatch = batches.find((b) => b.status === 'ACTIVE') || batches[0];

  const formatBytes = (bytes: number) => {
    if (!bytes || bytes <= 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const clampedIndex = Math.min(Math.max(i, 0), sizes.length - 1);
    return parseFloat((bytes / Math.pow(k, clampedIndex)).toFixed(1)) + ' ' + sizes[clampedIndex];
  };

  const formatSpeed = (bytesPerSec: number) => {
    if (!bytesPerSec || bytesPerSec <= 0) return '0 B/s';
    const k = 1024;
    const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    const i = Math.floor(Math.log(bytesPerSec) / Math.log(k));
    const clampedIndex = Math.min(Math.max(i, 0), sizes.length - 1);
    return parseFloat((bytesPerSec / Math.pow(k, clampedIndex)).toFixed(1)) + ' ' + sizes[clampedIndex];
  };

  const handleStartQueue = async () => {
    try {
      const hasPending = tasks.some(
        (t) => t.status === 'QUEUED' || t.status === 'RETRY_WAIT' || t.status === 'PAUSED'
      );
      if (hasPending || batches.length > 0) {
        await api.startQueue();
        addToast({ type: 'success', message: 'Started download queue.' });
      } else {
        openImportModal();
      }
      await refreshBatches();
      await refreshTasks();
    } catch {
      addToast({ type: 'error', message: 'Failed to start download queue.' });
    }
  };

  const handlePauseQueue = async () => {
    try {
      await api.pauseQueue();
      addToast({ type: 'info', message: 'Paused download queue.' });
      await refreshBatches();
      await refreshTasks();
    } catch {
      addToast({ type: 'error', message: 'Failed to pause download queue.' });
    }
  };

  const handleRetryFailed = async () => {
    try {
      const count = await api.retryFailedDownloads();
      addToast({
        type: 'info',
        message: `Queued ${count} failed tasks for retry.`,
      });
      await refreshTasks();
    } catch {
      addToast({ type: 'error', message: 'Failed to retry failed tasks.' });
    }
  };

  const handleOpenFolder = async () => {
    const dir = settings?.default_download_directory || '.';
    try {
      await api.openDownloadFolder(dir);
    } catch {
      addToast({ type: 'error', message: 'Failed to open directory.' });
    }
  };

  return (
    <div className={styles.dashboard}>
      {/* Welcome & Quick Actions Bar */}
      <div className={styles.welcomeSection}>
        <div>
          <h1 className={styles.pageTitle}>Dashboard</h1>
          <p className={styles.pageSubtitle}>
            Download orchestration, real-time worker pool and active batches.
          </p>
        </div>

        <div className={styles.quickActions}>
          <Button
            variant="outline"
            size="sm"
            icon={<FolderOpen size={16} />}
            onClick={handleOpenFolder}
          >
            Open Folder
          </Button>

          {failedTasks.length > 0 && (
            <Button
              variant="outline"
              size="sm"
              icon={<RotateCcw size={15} />}
              onClick={handleRetryFailed}
            >
              Retry Failed ({failedTasks.length})
            </Button>
          )}

          {activeTasks.length > 0 ? (
            <Button
              variant="secondary"
              size="sm"
              icon={<Pause size={15} />}
              onClick={handlePauseQueue}
            >
              Pause
            </Button>
          ) : (
            <Button
              variant="secondary"
              size="sm"
              icon={<Play size={15} />}
              onClick={handleStartQueue}
            >
              Start
            </Button>
          )}

          <Button
            variant="primary"
            size="sm"
            icon={<Plus size={16} />}
            onClick={openImportModal}
          >
            Import URLs
          </Button>
        </div>
      </div>

      {/* Primary Metrics Grid */}
      <div className={styles.metricsGrid}>
        <div className={styles.metricCard}>
          <div className={styles.cardHeader}>
            <span className={styles.cardTitle}>Total Tasks</span>
            <DownloadCloud size={18} className={styles.cardIcon} />
          </div>
          <div className={styles.cardValue}>{tasks.length}</div>
          <div className={styles.cardSubtext}>
            {completedTasks.length} completed &bull; {failedTasks.length} failed
          </div>
        </div>

        <div className={styles.metricCard}>
          <div className={styles.cardHeader}>
            <span className={styles.cardTitle}>Active Workers</span>
            <Zap size={18} className={styles.cardIconAccent} />
          </div>
          <div className={styles.cardValue}>{activeTasks.length}</div>
          <div className={styles.cardSubtext}>
            Speed: {formatSpeed(totalSpeed)}
          </div>
        </div>

        <div className={styles.metricCard}>
          <div className={styles.cardHeader}>
            <span className={styles.cardTitle}>Queued &amp; Paused</span>
            <Layers size={18} className={styles.cardIcon} />
          </div>
          <div className={styles.cardValue}>
            {queuedTasks.length + pausedTasks.length}
          </div>
          <div className={styles.cardSubtext}>
            {queuedTasks.length} ready &bull; {pausedTasks.length} paused
          </div>
        </div>

        <div className={styles.metricCard}>
          <div className={styles.cardHeader}>
            <span className={styles.cardTitle}>Downloaded</span>
            <CheckCircle2 size={18} className={styles.cardIconSuccess} />
          </div>
          <div className={styles.cardValue}>{formatBytes(totalBytes)}</div>
          <div className={styles.cardSubtext}>Across all completed files</div>
        </div>
      </div>

      {/* Active Batch Summary Card */}
      {activeBatch && (
        <div className={styles.activeBatchCard}>
          <div className={styles.batchCardHeader}>
            <div>
              <span className={styles.batchMetaLabel}>Current Batch</span>
              <h3 className={styles.batchName}>{activeBatch.name}</h3>
            </div>
            <div className={styles.batchCardActions}>
              <Badge status={activeBatch.status} />
              <Button
                variant="outline"
                size="sm"
                onClick={() => setActiveTab('batches')}
              >
                Configure Batch
              </Button>
            </div>
          </div>

          <div className={styles.batchDetailsRow}>
            <div className={styles.batchDetailItem}>
              <span className={styles.detailLabel}>Batch Size:</span>
              <span className={styles.detailValue}>{activeBatch.batch_size}</span>
            </div>
            <div className={styles.batchDetailItem}>
              <span className={styles.detailLabel}>Concurrency:</span>
              <span className={styles.detailValue}>{activeBatch.concurrency}</span>
            </div>
            <div className={styles.batchDetailItem}>
              <span className={styles.detailLabel}>Policy:</span>
              <span className={styles.detailValue}>
                {activeBatch.completion_policy.replace(/_/g, ' ')}
              </span>
            </div>
            <div className={styles.batchDetailItem}>
              <span className={styles.detailLabel}>Destination:</span>
              <span
                className={styles.detailValuePath}
                title={activeBatch.destination_directory}
              >
                {activeBatch.destination_directory}
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Recent Activity Table */}
      <div className={styles.recentSection}>
        <div className={styles.sectionHeader}>
          <div className={styles.headerTitleGroup}>
            <h3 className={styles.sectionTitle}>Recent Tasks</h3>
            <span className={styles.liveIndicator}>
              <span className={styles.pulseDot} /> Live Updates
            </span>
          </div>

          <div className={styles.headerRightGroup}>
            <div className={styles.dashFilterTabs}>
              <button
                className={`${styles.dashTabBtn} ${filterMode === 'all' ? styles.activeDashTab : ''}`}
                onClick={() => setFilterMode('all')}
              >
                All
              </button>
              <button
                className={`${styles.dashTabBtn} ${filterMode === 'downloading' ? styles.activeDashTab : ''} ${
                  downloadingTasks.length > 0 ? styles.hasActive : ''
                }`}
                onClick={() => setFilterMode('downloading')}
              >
                Downloading
                {downloadingTasks.length > 0 && (
                  <span className={styles.activeCountBadge}>{downloadingTasks.length}</span>
                )}
              </button>
              <button
                className={`${styles.dashTabBtn} ${filterMode === 'queued' ? styles.activeDashTab : ''}`}
                onClick={() => setFilterMode('queued')}
              >
                Newly Added ({queuedList.length})
              </button>
              <button
                className={`${styles.dashTabBtn} ${filterMode === 'completed' ? styles.activeDashTab : ''}`}
                onClick={() => setFilterMode('completed')}
              >
                Completed ({completedList.length})
              </button>
            </div>

            <Button
              variant="ghost"
              size="sm"
              onClick={() => setActiveTab('downloads')}
              icon={<ArrowRight size={15} />}
            >
              View All in Queue
            </Button>
          </div>
        </div>

        {dashboardTasks.length === 0 ? (
          <div className={styles.emptyCard}>
            <DownloadCloud size={40} className={styles.emptyIcon} />
            <h4 className={styles.emptyTitle}>No matching downloads</h4>
            <p className={styles.emptySubtitle}>
              {filterMode === 'downloading'
                ? 'No downloads are actively streaming right now.'
                : filterMode === 'queued'
                ? 'No queued tasks waiting to download.'
                : filterMode === 'completed'
                ? 'No downloads completed yet.'
                : 'Import video links from plain text, CSV, or JSON to begin downloading.'}
            </p>
            {filterMode === 'all' && (
              <Button
                variant="primary"
                size="sm"
                icon={<Plus size={15} />}
                onClick={openImportModal}
              >
                Import Links Now
              </Button>
            )}
          </div>
        ) : (
          <div className={styles.recentList}>
            {dashboardTasks.map((task) => {
              const isDownloading =
                task.status === 'DOWNLOADING' ||
                task.status === 'PREPARING' ||
                task.status === 'FINALIZING';

              return (
                <div
                  key={task.id}
                  className={`${styles.taskRow} ${isDownloading ? styles.activeDownloadingRow : ''}`}
                >
                  <div className={styles.taskInfo}>
                    <div className={styles.taskTitleRow}>
                      <span className={styles.taskTitle}>
                        {task.output_filename || task.title || task.id}
                      </span>
                      <Badge status={task.status} size="sm" />
                      {isDownloading && (task.speed_bytes_per_second || 0) > 0 && (
                        <span className={styles.liveSpeedBadge}>
                          ⚡ {formatSpeed(task.speed_bytes_per_second || 0)}
                        </span>
                      )}
                    </div>
                    <span className={styles.taskUrl} title={task.source_url}>
                      {task.source_url}
                    </span>
                  </div>

                  <div className={styles.taskProgressWrapper}>
                    <ProgressBar
                      progress={task.progress}
                      height={6}
                      showLabel
                      bytesDownloaded={task.bytes_downloaded}
                      totalBytes={task.total_bytes}
                      speed={task.speed_bytes_per_second}
                      eta={task.eta_seconds}
                    />
                  </div>

                  <div className={styles.taskActions}>
                    {task.status === 'DOWNLOADING' && (
                      <Button
                        variant="ghost"
                        size="sm"
                        icon={<Pause size={14} />}
                        onClick={() => api.pauseDownload(task.id)}
                        title="Pause"
                      />
                    )}
                    {task.status === 'PAUSED' && (
                      <Button
                        variant="ghost"
                        size="sm"
                        icon={<Play size={14} />}
                        onClick={() => api.resumeDownload(task.id)}
                        title="Resume"
                      />
                    )}
                    {task.status === 'FAILED' && (
                      <Button
                        variant="ghost"
                        size="sm"
                        icon={<RotateCcw size={14} />}
                        onClick={() => api.retryDownload(task.id)}
                        title="Retry"
                      />
                    )}
                    {task.status === 'COMPLETED' && (
                      <Button
                        variant="ghost"
                        size="sm"
                        icon={<FolderOpen size={14} />}
                        onClick={() => api.openDownloadFolder(task.output_directory)}
                        title="Open in folder"
                      />
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

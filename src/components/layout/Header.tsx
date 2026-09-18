import React from 'react';
import {
  FolderOpen,
  Plus,
  Play,
  Pause,
  Search,
  Activity,
  X,
} from 'lucide-react';
import { api } from '../../services/api';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from '../ui/Button';
import styles from './Header.module.css';

export const Header: React.FC = () => {
  const {
    tasks,
    batches,
    searchQuery,
    setSearchQuery,
    openImportModal,
    settings,
    addToast,
    refreshBatches,
    refreshTasks,
  } = useAppStore();

  const activeTasks = tasks.filter(
    (t) =>
      (t.status === 'DOWNLOADING' ||
        t.status === 'PREPARING' ||
        (t.speed_bytes_per_second || 0) > 0) &&
      t.status !== 'COMPLETED' &&
      t.status !== 'FAILED' &&
      t.status !== 'PAUSED'
  );

  const totalSpeed = activeTasks.reduce(
    (acc, t) => acc + (t.speed_bytes_per_second || 0),
    0
  );

  const hasActiveQueue = activeTasks.length > 0;

  const formatSpeed = (bytesPerSec: number) => {
    if (!bytesPerSec || bytesPerSec <= 0) return '0 B/s';
    const k = 1024;
    const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    const i = Math.floor(Math.log(bytesPerSec) / Math.log(k));
    const clampedIndex = Math.min(Math.max(i, 0), sizes.length - 1);
    return parseFloat((bytesPerSec / Math.pow(k, clampedIndex)).toFixed(1)) + ' ' + sizes[clampedIndex];
  };

  const handleToggleQueue = async () => {
    try {
      if (hasActiveQueue) {
        await api.pauseQueue();
        addToast({ type: 'info', message: 'Download queue paused.' });
      } else {
        const hasPending = tasks.some(
          (t) => t.status === 'QUEUED' || t.status === 'RETRY_WAIT' || t.status === 'PAUSED'
        );
        if (hasPending || batches.length > 0) {
          await api.startQueue();
          addToast({ type: 'success', message: 'Download queue started.' });
        } else {
          openImportModal();
        }
      }
      await refreshBatches();
      await refreshTasks();
    } catch (err) {
      addToast({ type: 'error', message: 'Failed to toggle queue execution.' });
    }
  };

  const handleOpenFolder = async () => {
    const dir = settings?.default_download_directory || '.';
    try {
      await api.openDownloadFolder(dir);
    } catch {
      addToast({ type: 'error', message: 'Failed to open download folder.' });
    }
  };

  return (
    <header className={styles.header}>
      <div className={styles.searchWrapper}>
        <Search size={16} className={styles.searchIcon} />
        <input
          type="text"
          className={styles.searchInput}
          placeholder="Search tasks by title, URL or filename..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
        {searchQuery && (
          <button
            type="button"
            className={styles.clearSearch}
            onClick={() => setSearchQuery('')}
            aria-label="Clear search"
          >
            <X size={14} />
          </button>
        )}
      </div>

      <div className={styles.statusMetrics}>
        <div className={styles.metricItem}>
          <Activity size={15} className={styles.metricIcon} />
          <span className={styles.metricLabel}>Speed:</span>
          <span className={styles.metricValue}>{formatSpeed(totalSpeed)}</span>
        </div>

        <div className={styles.metricDivider} />

        <div className={styles.metricItem}>
          <span className={styles.metricLabel}>Active:</span>
          <span className={styles.metricValue}>
            {activeTasks.length} {activeTasks.length === 1 ? 'stream' : 'streams'}
          </span>
        </div>
      </div>

      <div className={styles.actions}>
        <Button
          variant="outline"
          size="sm"
          icon={<FolderOpen size={16} />}
          onClick={handleOpenFolder}
          title="Open downloads directory"
        >
          Folder
        </Button>

        <Button
          variant={hasActiveQueue ? 'secondary' : 'primary'}
          size="sm"
          icon={hasActiveQueue ? <Pause size={15} /> : <Play size={15} />}
          onClick={handleToggleQueue}
        >
          {hasActiveQueue ? 'Pause Queue' : 'Start Queue'}
        </Button>

        <Button
          variant="primary"
          size="sm"
          icon={<Plus size={16} />}
          onClick={openImportModal}
        >
          Import
        </Button>
      </div>
    </header>
  );
};

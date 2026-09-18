import { useState, useRef, useMemo } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  DownloadCloud,
  Pause,
  Play,
  RotateCcw,
  Trash2,
  FolderOpen,
  ArrowUp,
  ArrowDown,
  ArrowUpDown,
  XCircle,
  Filter,
} from 'lucide-react';
import { api } from '../../services/api';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from '../../components/ui/Button';
import { Badge } from '../../components/ui/Badge';
import { ProgressBar } from '../../components/ui/ProgressBar';
import { DownloadStatus } from '../../types';
import styles from './Downloads.module.css';

export const DownloadsView: React.FC = () => {
  const {
    tasks,
    batches,
    searchQuery,
    statusFilter,
    setStatusFilter,
    batchFilter,
    setBatchFilter,
    addToast,
    refreshTasks,
    refreshBatches,
    showConfirmDialog,
  } = useAppStore();

  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [sortOrder, setSortOrder] = useState<'latest_downloaded' | 'newest_added' | 'queue_order'>('latest_downloaded');
  const parentRef = useRef<HTMLDivElement>(null);

  const getPlatformInfo = (url: string) => {
    const u = url.toLowerCase();
    if (u.includes('youtube.com') || u.includes('youtu.be')) {
      return { label: 'YouTube', cls: styles.platformYoutube };
    }
    if (u.includes('tiktok.com')) {
      return { label: 'TikTok (No WM)', cls: styles.platformTiktok };
    }
    if (u.includes('instagram.com')) {
      return { label: 'Instagram', cls: styles.platformInstagram };
    }
    if (u.includes('facebook.com') || u.includes('fb.watch') || u.includes('fb.com')) {
      return { label: 'Facebook', cls: styles.platformFacebook };
    }
    if (u.includes('twitter.com') || u.includes('x.com')) {
      return { label: 'X/Twitter', cls: styles.platformTwitter };
    }
    if (u.includes('reddit.com') || u.includes('redd.it')) {
      return { label: 'Reddit', cls: styles.platformReddit };
    }
    if (u.includes('vimeo.com')) {
      return { label: 'Vimeo', cls: styles.platformVimeo };
    }
    return null;
  };

  // Filter tasks based on filters and search, with LIFO / queue sorting
  const filteredTasks = useMemo(() => {
    const list = tasks.filter((task) => {
      if (statusFilter) {
        if (statusFilter === 'DOWNLOADING') {
          if (
            task.status !== 'DOWNLOADING' &&
            task.status !== 'PREPARING' &&
            task.status !== 'FINALIZING'
          ) {
            return false;
          }
        } else if (statusFilter === 'PAUSED') {
          if (task.status !== 'PAUSED' && task.status !== 'PAUSING') {
            return false;
          }
        } else if (statusFilter === 'CANCELLED') {
          if (task.status !== 'CANCELLED' && task.status !== 'SKIPPED') {
            return false;
          }
        } else if (task.status !== statusFilter) {
          return false;
        }
      }
      if (batchFilter && task.batch_id !== batchFilter) {
        return false;
      }
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        const matchesTitle = task.title?.toLowerCase().includes(q);
        const matchesUrl = task.source_url.toLowerCase().includes(q);
        const matchesFilename = task.output_filename?.toLowerCase().includes(q);
        if (!matchesTitle && !matchesUrl && !matchesFilename) {
          return false;
        }
      }
      return true;
    });

    return list.sort((a, b) => {
      if (sortOrder === 'newest_added') {
        const timeA = a.created_at || '';
        const timeB = b.created_at || '';
        return timeB.localeCompare(timeA);
      }

      if (sortOrder === 'queue_order') {
        if (b.priority !== a.priority) return b.priority - a.priority;
        return a.queue_position - b.queue_position;
      }

      // Default: 'latest_downloaded'
      // 1. If in Completed tab, sort so video downloaded last is at the very top
      if (statusFilter === 'COMPLETED') {
        const timeA = a.completed_at || a.updated_at || a.created_at || '';
        const timeB = b.completed_at || b.updated_at || b.created_at || '';
        return timeB.localeCompare(timeA);
      }

      // 2. If in Queued tab, keep priority descending then queue position ascending
      if (statusFilter === 'QUEUED') {
        if (b.priority !== a.priority) return b.priority - a.priority;
        return a.queue_position - b.queue_position;
      }

      // 3. If in Downloading tab, highest speed / progress first
      if (statusFilter === 'DOWNLOADING') {
        return (b.speed_bytes_per_second || 0) - (a.speed_bytes_per_second || 0);
      }

      // 4. In "ALL" or multi-status views:
      // Active downloads (DOWNLOADING, PREPARING, FINALIZING) always at the top!
      const isActiveA = a.status === 'DOWNLOADING' || a.status === 'PREPARING' || a.status === 'FINALIZING';
      const isActiveB = b.status === 'DOWNLOADING' || b.status === 'PREPARING' || b.status === 'FINALIZING';
      if (isActiveA && !isActiveB) return -1;
      if (!isActiveA && isActiveB) return 1;

      // Queued tasks next
      const isQueuedA = a.status === 'QUEUED' || a.status === 'RETRY_WAIT';
      const isQueuedB = b.status === 'QUEUED' || b.status === 'RETRY_WAIT';
      if (isQueuedA && !isQueuedB) return -1;
      if (!isQueuedA && isQueuedB) return 1;
      if (isQueuedA && isQueuedB) {
        if (b.priority !== a.priority) return b.priority - a.priority;
        return a.queue_position - b.queue_position;
      }

      // Paused next
      const isPausedA = a.status === 'PAUSED';
      const isPausedB = b.status === 'PAUSED';
      if (isPausedA && !isPausedB) return -1;
      if (!isPausedA && isPausedB) return 1;

      // Completed next: newest completed at top (last downloaded shows at top)
      const isCompA = a.status === 'COMPLETED';
      const isCompB = b.status === 'COMPLETED';
      if (isCompA && !isCompB) return -1;
      if (!isCompA && isCompB) return 1;
      if (isCompA && isCompB) {
        const timeA = a.completed_at || a.updated_at || a.created_at || '';
        const timeB = b.completed_at || b.updated_at || b.created_at || '';
        return timeB.localeCompare(timeA);
      }

      // Remaining tasks (Failed / Cancelled), newest updated at top
      const timeA = a.updated_at || a.created_at || '';
      const timeB = b.updated_at || b.created_at || '';
      return timeB.localeCompare(timeA);
    });
  }, [tasks, statusFilter, batchFilter, searchQuery, sortOrder]);

  // Virtualizer for smooth rendering of thousands of items
  const rowVirtualizer = useVirtualizer({
    count: filteredTasks.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 76,
    overscan: 10,
  });

  const toggleSelect = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (selectedIds.size === filteredTasks.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(filteredTasks.map((t) => t.id)));
    }
  };

  // Bulk actions
  const handleBulkPause = async () => {
    for (const id of selectedIds) {
      await api.pauseDownload(id);
    }
    addToast({ type: 'info', message: `Paused ${selectedIds.size} downloads.` });
    await refreshTasks();
    await refreshBatches();
  };

  const handleBulkResume = async () => {
    for (const id of selectedIds) {
      await api.resumeDownload(id);
    }
    addToast({ type: 'success', message: `Resumed ${selectedIds.size} downloads.` });
    await refreshTasks();
    await refreshBatches();
  };

  const handleBulkRetry = async () => {
    for (const id of selectedIds) {
      await api.retryDownload(id);
    }
    addToast({ type: 'info', message: `Queued ${selectedIds.size} downloads for retry.` });
    await refreshTasks();
    await refreshBatches();
  };

  const handleBulkCancel = () => {
    showConfirmDialog({
      title: 'Cancel Selected Downloads',
      message: `Are you sure you want to cancel ${selectedIds.size} downloads?`,
      confirmLabel: 'Cancel Tasks',
      isDestructive: true,
      onConfirm: async () => {
        for (const id of selectedIds) {
          await api.cancelDownload(id, false);
        }
        addToast({ type: 'info', message: `Cancelled ${selectedIds.size} downloads.` });
        setSelectedIds(new Set());
        await refreshTasks();
        await refreshBatches();
      },
    });
  };

  const handleBulkRemove = () => {
    showConfirmDialog({
      title: 'Remove Selected Downloads',
      message: `Are you sure you want to remove ${selectedIds.size} downloads from queue? Partial files will be purged.`,
      confirmLabel: 'Remove Tasks',
      isDestructive: true,
      onConfirm: async () => {
        for (const id of selectedIds) {
          await api.removeDownload(id);
        }
        addToast({ type: 'info', message: `Removed ${selectedIds.size} downloads.` });
        setSelectedIds(new Set());
        await refreshTasks();
        await refreshBatches();
      },
    });
  };

  const statusTabs: Array<{ id: DownloadStatus | 'ALL'; label: string }> = [
    { id: 'ALL', label: 'All' },
    { id: 'DOWNLOADING', label: 'Downloading' },
    { id: 'QUEUED', label: 'Queued' },
    { id: 'PAUSED', label: 'Paused' },
    { id: 'RETRY_WAIT', label: 'Retrying' },
    { id: 'COMPLETED', label: 'Completed' },
    { id: 'FAILED', label: 'Failed' },
    { id: 'CANCELLED', label: 'Cancelled' },
  ];

  return (
    <div className={styles.container}>
      {/* Header & Filter Controls */}
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>Download Queue</h1>
          <p className={styles.subtitle}>
            Showing {filteredTasks.length} of {tasks.length} total tasks
          </p>
        </div>

        {/* Batch filter selector */}
        <div className={styles.filterControls}>
          {filteredTasks.length > 0 && (
            <Button
              variant="outline"
              size="sm"
              onClick={toggleSelectAll}
            >
              {selectedIds.size === filteredTasks.length ? 'Deselect All' : 'Select All'}
            </Button>
          )}
          <div className={styles.batchFilterWrapper}>
            <ArrowUpDown size={15} className={styles.filterIcon} />
            <select
              className={styles.batchSelect}
              value={sortOrder}
              onChange={(e) =>
                setSortOrder(e.target.value as 'latest_downloaded' | 'newest_added' | 'queue_order')
              }
              title="Sort Order"
            >
              <option value="latest_downloaded">Last Downloaded First</option>
              <option value="newest_added">Newest Added First</option>
              <option value="queue_order">Queue / Priority Order</option>
            </select>
          </div>
          <div className={styles.batchFilterWrapper}>
            <Filter size={15} className={styles.filterIcon} />
            <select
              className={styles.batchSelect}
              value={batchFilter || ''}
              onChange={(e) => setBatchFilter(e.target.value || null)}
            >
              <option value="">All Batches</option>
              {batches.map((b) => (
                <option key={b.id} value={b.id}>
                  {b.name}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Status Filter Tabs */}
      <div className={styles.statusTabs}>
        {statusTabs.map((tab) => {
          const isActive = tab.id === 'ALL' ? !statusFilter : statusFilter === tab.id;
          let count = 0;
          if (tab.id === 'ALL') {
            count = tasks.length;
          } else if (tab.id === 'DOWNLOADING') {
            count = tasks.filter(
              (t) =>
                t.status === 'DOWNLOADING' ||
                t.status === 'PREPARING' ||
                t.status === 'FINALIZING'
            ).length;
          } else if (tab.id === 'PAUSED') {
            count = tasks.filter(
              (t) => t.status === 'PAUSED' || t.status === 'PAUSING'
            ).length;
          } else if (tab.id === 'CANCELLED') {
            count = tasks.filter(
              (t) => t.status === 'CANCELLED' || t.status === 'SKIPPED'
            ).length;
          } else {
            count = tasks.filter((t) => t.status === tab.id).length;
          }

          return (
            <button
              key={tab.id}
              className={`${styles.tabBtn} ${isActive ? styles.activeTab : ''}`}
              onClick={() => setStatusFilter(tab.id === 'ALL' ? null : tab.id)}
            >
              {tab.label}
              <span className={styles.tabCount}>{count}</span>
            </button>
          );
        })}
      </div>

      {/* Bulk Action Bar (Visible when items selected) */}
      {selectedIds.size > 0 && (
        <div className={styles.bulkActionBar}>
          <span className={styles.bulkCount}>{selectedIds.size} selected</span>
          <div className={styles.bulkButtons}>
            <Button
              variant="outline"
              size="sm"
              icon={<Play size={14} />}
              onClick={handleBulkResume}
            >
              Resume
            </Button>
            <Button
              variant="outline"
              size="sm"
              icon={<Pause size={14} />}
              onClick={handleBulkPause}
            >
              Pause
            </Button>
            <Button
              variant="outline"
              size="sm"
              icon={<RotateCcw size={14} />}
              onClick={handleBulkRetry}
            >
              Retry
            </Button>
            <Button
              variant="outline"
              size="sm"
              icon={<XCircle size={14} />}
              onClick={handleBulkCancel}
            >
              Cancel
            </Button>
            <Button
              variant="danger"
              size="sm"
              icon={<Trash2 size={14} />}
              onClick={handleBulkRemove}
            >
              Remove
            </Button>
          </div>
        </div>
      )}

      {/* Queue List Virtualized Container */}
      {filteredTasks.length === 0 ? (
        <div className={styles.emptyState}>
          <DownloadCloud size={44} className={styles.emptyIcon} />
          <h3 className={styles.emptyTitle}>No matching downloads found</h3>
          <p className={styles.emptySubtitle}>
            Try changing the status tab, selecting another batch, or clearing the search bar.
          </p>
        </div>
      ) : (
        <div className={styles.virtualListContainer} ref={parentRef}>
          <div
            className={styles.virtualListInner}
            style={{ height: `${rowVirtualizer.getTotalSize()}px` }}
          >
            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
              const task = filteredTasks[virtualRow.index];
              const isSelected = selectedIds.has(task.id);

              return (
                <div
                  key={task.id}
                  className={`${styles.taskRow} ${isSelected ? styles.selectedRow : ''}`}
                  style={{
                    transform: `translateY(${virtualRow.start}px)`,
                    height: `${virtualRow.size}px`,
                  }}
                >
                  <div className={styles.selectCol}>
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => toggleSelect(task.id)}
                    />
                  </div>

                  <div className={styles.mainInfoCol}>
                    <div className={styles.titleRow}>
                      {(() => {
                        const plat = getPlatformInfo(task.source_url);
                        return plat ? (
                          <span className={`${styles.platformTag} ${plat.cls}`}>
                            {plat.label}
                          </span>
                        ) : null;
                      })()}
                      <span className={styles.taskTitle}>
                        {task.output_filename || task.title || task.id}
                      </span>
                      <Badge status={task.status} size="sm" />
                      {task.selected_quality && (
                        <span className={styles.qualityTag}>
                          {task.selected_quality === 'audio'
                            ? '🎵 Audio'
                            : task.selected_quality.includes('video_only')
                            ? `🔇 ${task.selected_quality.replace('_video_only', '')}`
                            : task.selected_quality}
                          {task.selected_extension ? ` • ${task.selected_extension.toUpperCase()}` : ''}
                        </span>
                      )}
                    </div>
                    <span className={styles.taskUrl} title={task.source_url}>
                      {task.source_url}
                    </span>
                  </div>

                  <div className={styles.progressCol}>
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

                  <div className={styles.priorityCol}>
                    <button
                      className={styles.priorityBtn}
                      onClick={() =>
                        api.changeDownloadPriority(task.id, task.priority + 1).then(refreshTasks)
                      }
                      title="Increase priority"
                    >
                      <ArrowUp size={13} />
                    </button>
                    <span className={styles.priorityNum}>{task.priority}</span>
                    <button
                      className={styles.priorityBtn}
                      onClick={() =>
                        api.changeDownloadPriority(task.id, task.priority - 1).then(refreshTasks)
                      }
                      title="Decrease priority"
                    >
                      <ArrowDown size={13} />
                    </button>
                  </div>

                  <div className={styles.actionsCol}>
                    {(task.status === 'DOWNLOADING' ||
                      task.status === 'PREPARING' ||
                      task.status === 'FINALIZING') && (
                      <button
                        className={styles.actionBtn}
                        onClick={() =>
                          api.pauseDownload(task.id).then(() => {
                            refreshTasks();
                            refreshBatches();
                          })
                        }
                        title="Pause"
                      >
                        <Pause size={15} />
                      </button>
                    )}
                    {task.status === 'QUEUED' && (
                      <>
                        <button
                          className={styles.actionBtn}
                          onClick={() =>
                            api.resumeDownload(task.id).then(() => {
                              refreshTasks();
                              refreshBatches();
                            })
                          }
                          title="Start download now"
                        >
                          <Play size={15} />
                        </button>
                        <button
                          className={styles.actionBtn}
                          onClick={() =>
                            api.pauseDownload(task.id).then(() => {
                              refreshTasks();
                              refreshBatches();
                            })
                          }
                          title="Pause"
                        >
                          <Pause size={15} />
                        </button>
                      </>
                    )}
                    {(task.status === 'PAUSED' || task.status === 'PAUSING') && (
                      <button
                        className={styles.actionBtn}
                        onClick={() =>
                          api.resumeDownload(task.id).then(() => {
                            refreshTasks();
                            refreshBatches();
                          })
                        }
                        title="Resume"
                      >
                        <Play size={15} />
                      </button>
                    )}
                    {(task.status === 'FAILED' || task.status === 'RETRY_WAIT') && (
                      <button
                        className={styles.actionBtn}
                        onClick={() =>
                          api.retryDownload(task.id).then(() => {
                            refreshTasks();
                            refreshBatches();
                          })
                        }
                        title="Retry"
                      >
                        <RotateCcw size={15} />
                      </button>
                    )}
                    {task.status === 'COMPLETED' && (
                      <button
                        className={styles.actionBtn}
                        onClick={() => api.openDownloadFolder(task.output_directory)}
                        title="Open folder"
                      >
                        <FolderOpen size={15} />
                      </button>
                    )}
                    <button
                      className={styles.actionBtnDanger}
                      onClick={() =>
                        showConfirmDialog({
                          title: 'Remove Task',
                          message: `Remove '${task.title || task.id}' from the queue?`,
                          confirmLabel: 'Remove',
                          isDestructive: true,
                          onConfirm: () =>
                            api.removeDownload(task.id).then(() => {
                              refreshTasks();
                              refreshBatches();
                            }),
                        })
                      }
                      title="Remove"
                    >
                      <Trash2 size={15} />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};

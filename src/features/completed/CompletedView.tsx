import React, { useMemo } from 'react';
import {
  CheckCircle2,
  FolderOpen,
  FileVideo,
  Download,
} from 'lucide-react';
import { api } from '../../services/api';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from '../../components/ui/Button';
import styles from './Completed.module.css';

export const CompletedView: React.FC = () => {
  const { tasks, searchQuery, addToast } = useAppStore();

  const completedTasks = useMemo(() => {
    const list = tasks.filter((t) => {
      if (t.status !== 'COMPLETED') return false;
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        const matchName = t.output_filename?.toLowerCase().includes(q);
        const matchTitle = t.title?.toLowerCase().includes(q);
        const matchUrl = t.source_url.toLowerCase().includes(q);
        return matchName || matchTitle || matchUrl;
      }
      return true;
    });

    // Newest / last downloaded video always displayed at the top
    return list.sort((a, b) => {
      const timeA = a.completed_at || a.updated_at || a.created_at || '';
      const timeB = b.completed_at || b.updated_at || b.created_at || '';
      return timeB.localeCompare(timeA);
    });
  }, [tasks, searchQuery]);

  const totalBytes = completedTasks.reduce((acc, t) => acc + t.bytes_downloaded, 0);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  const handleExportCsv = () => {
    const headers = 'ID,Filename,Bytes,CompletedAt,SourceURL\n';
    const rows = completedTasks
      .map(
        (t) =>
          `"${t.id}","${t.output_filename || ''}",${t.bytes_downloaded},"${t.completed_at || ''}","${t.source_url}"`
      )
      .join('\n');

    const blob = new Blob([headers + rows], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `completed_downloads_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
    addToast({ type: 'success', message: 'Completed downloads exported to CSV.' });
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>Completed Downloads</h1>
          <p className={styles.subtitle}>
            {completedTasks.length} media files finalized ({formatBytes(totalBytes)})
          </p>
        </div>

        {completedTasks.length > 0 && (
          <Button
            variant="outline"
            size="sm"
            icon={<Download size={15} />}
            onClick={handleExportCsv}
          >
            Export Log (CSV)
          </Button>
        )}
      </div>

      {completedTasks.length === 0 ? (
        <div className={styles.emptyCard}>
          <CheckCircle2 size={44} className={styles.emptyIcon} />
          <h3 className={styles.emptyTitle}>No completed downloads yet</h3>
          <p className={styles.emptySubtitle}>
            Downloaded files will appear here with instant folder access once finalized.
          </p>
        </div>
      ) : (
        <div className={styles.tableWrapper}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th>File Name</th>
                <th>Format</th>
                <th>Size</th>
                <th>Completed Date</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {completedTasks.map((task) => (
                <tr key={task.id}>
                  <td>
                    <div className={styles.fileCell}>
                      <FileVideo size={18} className={styles.videoIcon} />
                      <div className={styles.fileInfo}>
                        <span className={styles.fileName}>
                          {task.output_filename || task.title || task.id}
                        </span>
                        <span className={styles.fileSource} title={task.source_url}>
                          {task.source_url}
                        </span>
                      </div>
                    </div>
                  </td>
                  <td>
                    <span className={styles.formatTag}>
                      {task.selected_extension?.toUpperCase() || 'MP4'}
                    </span>
                  </td>
                  <td className={styles.sizeCell}>
                    {formatBytes(task.bytes_downloaded)}
                  </td>
                  <td className={styles.dateCell}>
                    {task.completed_at
                      ? new Date(task.completed_at).toLocaleString()
                      : 'Just now'}
                  </td>
                  <td>
                    <Button
                      variant="ghost"
                      size="sm"
                      icon={<FolderOpen size={14} />}
                      onClick={() => api.openDownloadFolder(task.output_directory)}
                      title="Open enclosing folder"
                    >
                      Locate
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

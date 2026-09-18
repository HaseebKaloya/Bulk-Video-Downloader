import React from 'react';
import styles from './ProgressBar.module.css';

interface ProgressBarProps {
  progress: number; // 0.0 to 1.0
  height?: number;
  showLabel?: boolean;
  status?: string;
  bytesDownloaded?: number;
  totalBytes?: number | null;
  speed?: number | null;
  eta?: number | null;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  progress,
  height = 6,
  showLabel = false,
  bytesDownloaded,
  totalBytes,
  speed,
  eta,
}) => {
  const percentage = Math.min(Math.max(Math.round(progress * 100), 0), 100);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  const formatSpeed = (bytesPerSec?: number | null) => {
    if (!bytesPerSec || bytesPerSec <= 0) return '';
    return `${formatBytes(bytesPerSec)}/s`;
  };

  const formatEta = (seconds?: number | null) => {
    if (seconds === undefined || seconds === null || seconds <= 0) return '';
    if (seconds < 60) return `${seconds}s left`;
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    if (mins < 60) return `${mins}m ${secs}s left`;
    const hrs = Math.floor(mins / 60);
    return `${hrs}h ${mins % 60}m left`;
  };

  return (
    <div className={styles.wrapper}>
      <div className={styles.track} style={{ height: `${height}px` }}>
        <div
          className={styles.fill}
          style={{ width: `${percentage}%` }}
          role="progressbar"
          aria-valuenow={percentage}
          aria-valuemin={0}
          aria-valuemax={100}
        />
      </div>
      {showLabel && (
        <div className={styles.meta}>
          <span className={styles.percentage}>{percentage}%</span>
          <div className={styles.details}>
            {bytesDownloaded !== undefined && (
              <span>
                {formatBytes(bytesDownloaded)}
                {totalBytes ? ` / ${formatBytes(totalBytes)}` : ''}
              </span>
            )}
            {speed && <span className={styles.speed}>{formatSpeed(speed)}</span>}
            {eta && <span className={styles.eta}>{formatEta(eta)}</span>}
          </div>
        </div>
      )}
    </div>
  );
};

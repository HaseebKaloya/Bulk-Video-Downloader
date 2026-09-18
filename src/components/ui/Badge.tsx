import React from 'react';
import { DownloadStatus, BatchStatus } from '../../types';
import styles from './Badge.module.css';

interface BadgeProps {
  status: DownloadStatus | BatchStatus | string;
  size?: 'sm' | 'md';
}

export const Badge: React.FC<BadgeProps> = ({ status, size = 'md' }) => {
  const normalized = status.toLowerCase();

  return (
    <span className={`${styles.badge} ${styles[normalized] || styles.default} ${styles[size]}`}>
      <span className={styles.dot} />
      {status.replace(/_/g, ' ')}
    </span>
  );
};

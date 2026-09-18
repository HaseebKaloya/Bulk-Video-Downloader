import React from 'react';
import {
  LayoutDashboard,
  DownloadCloud,
  Layers,
  CheckCircle2,
  AlertCircle,
  Sliders,
  Settings,
  Info,
} from 'lucide-react';
import { NavigationTab, useAppStore } from '../../stores/useAppStore';
import styles from './Sidebar.module.css';

export const Sidebar: React.FC = () => {
  const { activeTab, setActiveTab, tasks } = useAppStore();

  const queuedCount = tasks.filter(
    (t) => t.status === 'QUEUED' || t.status === 'RETRY_WAIT'
  ).length;
  const activeCount = tasks.filter(
    (t) => t.status === 'DOWNLOADING' || t.status === 'PREPARING' || t.status === 'FINALIZING'
  ).length;
  const completedCount = tasks.filter((t) => t.status === 'COMPLETED').length;
  const failedCount = tasks.filter((t) => t.status === 'FAILED').length;
  const pendingCount = activeCount + queuedCount;

  const navItems: Array<{
    id: NavigationTab;
    label: string;
    icon: React.ReactNode;
    badge?: number;
    badgeVariant?: 'default' | 'accent' | 'danger';
  }> = [
    { id: 'dashboard', label: 'Dashboard', icon: <LayoutDashboard size={18} /> },
    {
      id: 'downloads',
      label: 'Downloads',
      icon: <DownloadCloud size={18} />,
      badge: pendingCount > 0 ? pendingCount : undefined,
      badgeVariant: activeCount > 0 ? 'accent' : 'default',
    },
    { id: 'batches', label: 'Batches', icon: <Layers size={18} /> },
    {
      id: 'completed',
      label: 'Completed',
      icon: <CheckCircle2 size={18} />,
      badge: completedCount > 0 ? completedCount : undefined,
    },
    {
      id: 'failed',
      label: 'Failed',
      icon: <AlertCircle size={18} />,
      badge: failedCount > 0 ? failedCount : undefined,
      badgeVariant: 'danger',
    },
    { id: 'profiles', label: 'Profiles', icon: <Sliders size={18} /> },
    { id: 'settings', label: 'Settings', icon: <Settings size={18} /> },
    { id: 'about', label: 'About', icon: <Info size={18} /> },
  ];

  return (
    <aside className={styles.sidebar}>
      <div className={styles.logoSection}>
        <div className={styles.logoIcon}>
          <img src="/app-logo.png" alt="Logo" className={styles.brandLogoImg} />
        </div>
        <div className={styles.logoText}>
          <span className={styles.brandTitle}>Bulk Video</span>
          <span className={styles.brandSubtitle}>Downloader</span>
        </div>
      </div>

      <nav className={styles.nav}>
        {navItems.map((item) => {
          const isActive = activeTab === item.id;
          return (
            <button
              key={item.id}
              className={`${styles.navItem} ${isActive ? styles.active : ''}`}
              onClick={() => setActiveTab(item.id)}
            >
              <span className={styles.itemIcon}>{item.icon}</span>
              <span className={styles.itemLabel}>{item.label}</span>
              {item.badge !== undefined && item.badge > 0 && (
                <span
                  className={`${styles.badge} ${
                    item.badgeVariant ? styles[item.badgeVariant] : ''
                  }`}
                >
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      <div className={styles.footer}>
        <div
          className={styles.creatorCard}
          onClick={() => setActiveTab('about')}
          title="Designed & Developed by Haseeb Kaloya"
        >
          <div className={styles.creatorInfo}>
            <span className={styles.creatorLead}>Created by</span>
            <span className={styles.creatorName}>Haseeb Kaloya</span>
          </div>
          <span className={styles.versionLabel}>v1.0.0</span>
        </div>
      </div>
    </aside>
  );
};

import React from 'react';
import { AlertCircle, CheckCircle, Info, X } from 'lucide-react';
import { useAppStore } from '../../stores/useAppStore';
import styles from './Toast.module.css';

export const ToastContainer: React.FC = () => {
  const { toasts, removeToast } = useAppStore();

  if (toasts.length === 0) return null;

  return (
    <div className={styles.container}>
      {toasts.map((toast) => {
        const Icon =
          toast.type === 'success'
            ? CheckCircle
            : toast.type === 'warning' || toast.type === 'error'
            ? AlertCircle
            : Info;

        return (
          <div
            key={toast.id}
            className={`${styles.toast} ${styles[toast.type]}`}
            role="alert"
          >
            <Icon size={18} className={styles.icon} />
            <div className={styles.content}>
              {toast.title && <div className={styles.title}>{toast.title}</div>}
              <div className={styles.message}>{toast.message}</div>
            </div>
            <button
              className={styles.closeBtn}
              onClick={() => removeToast(toast.id)}
              aria-label="Dismiss toast"
            >
              <X size={14} />
            </button>
          </div>
        );
      })}
    </div>
  );
};

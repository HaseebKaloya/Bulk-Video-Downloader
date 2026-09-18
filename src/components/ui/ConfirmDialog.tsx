import React from 'react';
import { AlertTriangle } from 'lucide-react';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from './Button';
import styles from './ConfirmDialog.module.css';

export const ConfirmDialog: React.FC = () => {
  const { confirmDialog, hideConfirmDialog } = useAppStore();

  if (!confirmDialog) return null;

  return (
    <div className={styles.overlay} onClick={hideConfirmDialog}>
      <div className={styles.dialog} onClick={(e) => e.stopPropagation()}>
        <div className={styles.iconWrapper}>
          <AlertTriangle
            size={24}
            className={confirmDialog.isDestructive ? styles.dangerIcon : styles.warningIcon}
          />
        </div>
        <div className={styles.content}>
          <h4 className={styles.title}>{confirmDialog.title}</h4>
          <p className={styles.message}>{confirmDialog.message}</p>
        </div>
        <div className={styles.actions}>
          <Button variant="outline" size="sm" onClick={hideConfirmDialog}>
            Cancel
          </Button>
          <Button
            variant={confirmDialog.isDestructive ? 'danger' : 'primary'}
            size="sm"
            onClick={() => {
              confirmDialog.onConfirm();
              hideConfirmDialog();
            }}
          >
            {confirmDialog.confirmLabel || 'Confirm'}
          </Button>
        </div>
      </div>
    </div>
  );
};

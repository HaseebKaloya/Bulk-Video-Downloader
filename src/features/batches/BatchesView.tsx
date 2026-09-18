import { useState } from 'react';
import {
  Layers,
  Plus,
  Play,
  Pause,
  Trash2,
  Sliders,
} from 'lucide-react';
import { api } from '../../services/api';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from '../../components/ui/Button';
import { Badge } from '../../components/ui/Badge';
import { Modal } from '../../components/ui/Modal';
import { Input, Select } from '../../components/ui/FormControls';
import { ProgressBar } from '../../components/ui/ProgressBar';
import { Batch } from '../../types';
import styles from './Batches.module.css';

export const BatchesView: React.FC = () => {
  const {
    batches,
    tasks,
    settings,
    addToast,
    refreshBatches,
    refreshTasks,
    showConfirmDialog,
  } = useAppStore();

  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [newBatchName, setNewBatchName] = useState('');
  const [newBatchSize, setNewBatchSize] = useState('10');
  const [newConcurrency, setNewConcurrency] = useState('3');
  const [newPolicy, setNewPolicy] = useState('TERMINAL_COMPLETION');

  const [editingBatch, setEditingBatch] = useState<Batch | null>(null);
  const [editBatchSize, setEditBatchSize] = useState('10');
  const [editConcurrency, setEditConcurrency] = useState('3');
  const [selectedBatchIds, setSelectedBatchIds] = useState<Set<string>>(new Set());

  const toggleSelectBatch = (id: string) => {
    setSelectedBatchIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (selectedBatchIds.size === batches.length) {
      setSelectedBatchIds(new Set());
    } else {
      setSelectedBatchIds(new Set(batches.map((b) => b.id)));
    }
  };

  const handleDeleteSelected = () => {
    if (selectedBatchIds.size === 0) return;
    const count = selectedBatchIds.size;
    showConfirmDialog({
      title: 'Delete Selected Batches',
      message: `Are you sure you want to delete ${count} selected batch${count > 1 ? 'es' : ''} and all associated tasks?`,
      confirmLabel: `Delete ${count} Batch${count > 1 ? 'es' : ''}`,
      isDestructive: true,
      onConfirm: async () => {
        try {
          await api.deleteBatches(Array.from(selectedBatchIds));
          addToast({
            type: 'info',
            message: `Deleted ${count} batch${count > 1 ? 'es' : ''}.`,
          });
          setSelectedBatchIds(new Set());
          await refreshBatches();
          await refreshTasks();
        } catch {
          addToast({ type: 'error', message: 'Failed to delete selected batches.' });
        }
      },
    });
  };

  const handleClearAllBatches = () => {
    if (batches.length === 0) return;
    showConfirmDialog({
      title: 'Clear All Batches',
      message: `Are you sure you want to remove all ${batches.length} batches and their tasks? This action cannot be undone.`,
      confirmLabel: 'Clear All Batches',
      isDestructive: true,
      onConfirm: async () => {
        try {
          await api.clearAllBatches();
          addToast({
            type: 'info',
            message: 'All batches have been cleared.',
          });
          setSelectedBatchIds(new Set());
          await refreshBatches();
          await refreshTasks();
        } catch {
          addToast({ type: 'error', message: 'Failed to clear batches.' });
        }
      },
    });
  };

  const handleCreateBatch = async () => {
    if (!newBatchName.trim()) {
      addToast({ type: 'error', message: 'Batch name is required.' });
      return;
    }

    try {
      await api.createBatch({
        name: newBatchName.trim(),
        batch_size: parseInt(newBatchSize) || 10,
        concurrency: parseInt(newConcurrency) || 3,
        completion_policy: newPolicy,
        destination_directory: settings?.default_download_directory || '.',
        naming_template: settings?.default_naming_template || '{title}_{quality}.{format}',
      });

      addToast({ type: 'success', message: 'Batch created successfully.' });
      setIsCreateOpen(false);
      setNewBatchName('');
      await refreshBatches();
    } catch {
      addToast({ type: 'error', message: 'Failed to create batch.' });
    }
  };

  const handleSaveBatchLimits = async () => {
    if (!editingBatch) return;
    try {
      await api.updateBatch(editingBatch.id, {
        batch_size: parseInt(editBatchSize) || 10,
        concurrency: parseInt(editConcurrency) || 3,
      });
      addToast({ type: 'success', message: 'Batch execution limits updated.' });
      setEditingBatch(null);
      await refreshBatches();
    } catch {
      addToast({ type: 'error', message: 'Failed to update batch limits.' });
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>Batch Management</h1>
          <p className={styles.subtitle}>
            Organize queues into discrete scheduling units with concurrency boundaries and policies.
          </p>
        </div>

        <div className={styles.headerActions}>
          {batches.length > 0 && (
            <>
              <Button
                variant="outline"
                size="sm"
                onClick={toggleSelectAll}
              >
                {selectedBatchIds.size === batches.length ? 'Deselect All' : 'Select All'}
              </Button>

              {selectedBatchIds.size > 0 && (
                <Button
                  variant="danger"
                  size="sm"
                  icon={<Trash2 size={15} />}
                  onClick={handleDeleteSelected}
                >
                  Delete Selected ({selectedBatchIds.size})
                </Button>
              )}

              <Button
                variant="outline"
                size="sm"
                icon={<Trash2 size={15} />}
                onClick={handleClearAllBatches}
                title="Clear all batches"
              >
                Clear All Batches
              </Button>
            </>
          )}

          <Button
            variant="primary"
            size="sm"
            icon={<Plus size={16} />}
            onClick={() => setIsCreateOpen(true)}
          >
            Create Batch
          </Button>
        </div>
      </div>

      {batches.length === 0 ? (
        <div className={styles.emptyCard}>
          <Layers size={44} className={styles.emptyIcon} />
          <h3 className={styles.emptyTitle}>No batches created</h3>
          <p className={styles.emptySubtitle}>
            Create your first batch or import links to automatically generate one.
          </p>
          <Button
            variant="primary"
            size="sm"
            icon={<Plus size={15} />}
            onClick={() => setIsCreateOpen(true)}
          >
            Create New Batch
          </Button>
        </div>
      ) : (
        <div className={styles.batchGrid}>
          {batches.map((batch) => {
            const batchTasks = tasks.filter((t) => t.batch_id === batch.id);
            const completedCount = batchTasks.filter((t) => t.status === 'COMPLETED').length;
            const failedCount = batchTasks.filter((t) => t.status === 'FAILED').length;
            const progress =
              batchTasks.length > 0 ? completedCount / batchTasks.length : 0;
            const isSelected = selectedBatchIds.has(batch.id);

            return (
              <div
                key={batch.id}
                className={`${styles.batchCard} ${isSelected ? styles.selectedCard : ''}`}
              >
                <div className={styles.cardTop}>
                  <div className={styles.cardTopLeft}>
                    <input
                      type="checkbox"
                      className={styles.batchCheckbox}
                      checked={isSelected}
                      onChange={() => toggleSelectBatch(batch.id)}
                      title="Select batch"
                    />
                    <div className={styles.cardHeaderInfo}>
                      <h3 className={styles.batchName}>{batch.name}</h3>
                      <Badge status={batch.status} size="sm" />
                    </div>
                  </div>

                  <div className={styles.topActions}>
                    {batch.status === 'ACTIVE' ? (
                      <Button
                        variant="secondary"
                        size="sm"
                        icon={<Pause size={14} />}
                        onClick={() => api.pauseBatch(batch.id).then(refreshBatches)}
                      >
                        Pause
                      </Button>
                    ) : (
                      <Button
                        variant="primary"
                        size="sm"
                        icon={<Play size={14} />}
                        onClick={() => api.startBatch(batch.id).then(refreshBatches)}
                      >
                        Start
                      </Button>
                    )}

                    <Button
                      variant="outline"
                      size="sm"
                      icon={<Sliders size={14} />}
                      onClick={() => {
                        setEditingBatch(batch);
                        setEditBatchSize(batch.batch_size.toString());
                        setEditConcurrency(batch.concurrency.toString());
                      }}
                      title="Adjust execution limits"
                    />

                    <Button
                      variant="ghost"
                      size="sm"
                      icon={<Trash2 size={14} />}
                      onClick={() =>
                        showConfirmDialog({
                          title: 'Delete Batch',
                          message: `Are you sure you want to delete batch '${batch.name}' and all its tasks?`,
                          confirmLabel: 'Delete Batch',
                          isDestructive: true,
                          onConfirm: () => api.deleteBatch(batch.id).then(() => {
                            refreshBatches();
                            refreshTasks();
                          }),
                        })
                      }
                      title="Delete batch"
                    />
                  </div>
                </div>

                <div className={styles.progressBarWrapper}>
                  <ProgressBar progress={progress} height={6} showLabel={false} />
                  <div className={styles.progressStats}>
                    <span>
                      {completedCount} / {batchTasks.length} Completed
                    </span>
                    {failedCount > 0 && (
                      <span className={styles.failedTag}>{failedCount} Failed</span>
                    )}
                    <span>{Math.round(progress * 100)}%</span>
                  </div>
                </div>

                <div className={styles.batchMetaGrid}>
                  <div className={styles.metaItem}>
                    <span className={styles.metaLabel}>Batch Size:</span>
                    <span className={styles.metaValue}>{batch.batch_size} tasks</span>
                  </div>
                  <div className={styles.metaItem}>
                    <span className={styles.metaLabel}>Concurrency:</span>
                    <span className={styles.metaValue}>{batch.concurrency} streams</span>
                  </div>
                  <div className={styles.metaItem}>
                    <span className={styles.metaLabel}>Completion Policy:</span>
                    <span className={styles.metaValue}>
                      {batch.completion_policy.replace(/_/g, ' ')}
                    </span>
                  </div>
                  <div className={styles.metaItem}>
                    <span className={styles.metaLabel}>Destination:</span>
                    <span className={styles.metaPath} title={batch.destination_directory}>
                      {batch.destination_directory}
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Create Batch Modal */}
      <Modal
        isOpen={isCreateOpen}
        onClose={() => setIsCreateOpen(false)}
        title="Create New Batch"
        footer={
          <>
            <Button variant="outline" size="sm" onClick={() => setIsCreateOpen(false)}>
              Cancel
            </Button>
            <Button variant="primary" size="sm" onClick={handleCreateBatch}>
              Create Batch
            </Button>
          </>
        }
      >
        <div className={styles.formStack}>
          <Input
            label="Batch Name"
            placeholder="e.g. Documentaries Collection"
            value={newBatchName}
            onChange={(e) => setNewBatchName(e.target.value)}
          />

          <div className={styles.formRow}>
            <Input
              label="Batch Sizing (Tasks per batch)"
              type="number"
              min="1"
              max="500"
              value={newBatchSize}
              onChange={(e) => setNewBatchSize(e.target.value)}
            />

            <Input
              label="Max Concurrency (Simultaneous workers)"
              type="number"
              min="1"
              max="16"
              value={newConcurrency}
              onChange={(e) => setNewConcurrency(e.target.value)}
            />
          </div>

          <Select
            label="Batch Completion Policy"
            value={newPolicy}
            onChange={(e) => setNewPolicy(e.target.value)}
            options={[
              {
                value: 'TERMINAL_COMPLETION',
                label: 'Policy B: Terminal Completion (Advance when all items finish or fail)',
              },
              {
                value: 'ALL_SUCCESSFUL',
                label: 'Policy A: All Successful (Advance only if 100% succeed)',
              },
              {
                value: 'PAUSE_ON_FAILURE',
                label: 'Policy C: Pause on Failure (Halt batch immediately on error)',
              },
              {
                value: 'CONTINUE_WITH_RETRY',
                label: 'Policy D: Continue with Retry (Auto-retry before advancing)',
              },
            ]}
          />
        </div>
      </Modal>

      {/* Edit Limits Modal */}
      <Modal
        isOpen={!!editingBatch}
        onClose={() => setEditingBatch(null)}
        title={`Adjust Limits — ${editingBatch?.name}`}
        footer={
          <>
            <Button variant="outline" size="sm" onClick={() => setEditingBatch(null)}>
              Cancel
            </Button>
            <Button variant="primary" size="sm" onClick={handleSaveBatchLimits}>
              Save Limits
            </Button>
          </>
        }
      >
        <div className={styles.formStack}>
          <p className={styles.editNote}>
            Adjusting limits takes effect immediately on the live scheduler without interrupting
            currently active transfers.
          </p>
          <div className={styles.formRow}>
            <Input
              label="Batch Size"
              type="number"
              min="1"
              max="500"
              value={editBatchSize}
              onChange={(e) => setEditBatchSize(e.target.value)}
            />

            <Input
              label="Concurrency Limit"
              type="number"
              min="1"
              max="16"
              value={editConcurrency}
              onChange={(e) => setEditConcurrency(e.target.value)}
            />
          </div>
        </div>
      </Modal>
    </div>
  );
};

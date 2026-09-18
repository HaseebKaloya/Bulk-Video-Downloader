import { useState, useEffect } from 'react';
import {
  Settings,
  FolderOpen,
  RefreshCw,
  HardDrive,
  ShieldCheck,
  DownloadCloud,
  User,
} from 'lucide-react';
import { api } from '../../services/api';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from '../../components/ui/Button';
import { Input, Select } from '../../components/ui/FormControls';
import { AppSettings, FFmpegStatus, YtDlpStatus } from '../../types';
import styles from './Settings.module.css';

export const SettingsView: React.FC = () => {
  const { settings, addToast, refreshSettings } = useAppStore();

  const [form, setForm] = useState<AppSettings | null>(settings);
  const [ffmpegStatus, setFfmpegStatus] = useState<FFmpegStatus | null>(null);
  const [ytdlpStatus, setYtdlpStatus] = useState<YtDlpStatus | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [isInstallingYtDlp, setIsInstallingYtDlp] = useState(false);
  const [isInstallingFFmpeg, setIsInstallingFFmpeg] = useState(false);

  useEffect(() => {
    if (settings) {
      setForm(settings);
    }
  }, [settings]);

  useEffect(() => {
    api.checkFFmpeg().then(setFfmpegStatus);
    api.checkYtDlp().then(setYtdlpStatus);
  }, []);

  const handleSave = async () => {
    if (!form) return;
    setIsSaving(true);
    try {
      await api.updateSettings(form);
      addToast({ type: 'success', message: 'Settings saved successfully.' });
      await refreshSettings();
    } catch {
      addToast({ type: 'error', message: 'Failed to save settings.' });
    } finally {
      setIsSaving(false);
    }
  };

  const handleIntegrityCheck = async () => {
    try {
      const summary = await api.getRecoverySummary();
      if (summary.integrity_ok) {
        addToast({
          type: 'success',
          title: 'Database Integrity OK',
          message: 'SQLite integrity check passed. 0 corrupt pages detected.',
        });
      } else {
        addToast({
          type: 'error',
          title: 'Database Warning',
          message: 'SQLite database reported integrity warnings.',
        });
      }
    } catch {
      addToast({ type: 'error', message: 'Failed to verify database integrity.' });
    }
  };

  const handleInstallYtDlp = async () => {
    setIsInstallingYtDlp(true);
    try {
      const status = await api.installYtDlp();
      setYtdlpStatus(status);
      addToast({
        type: 'success',
        title: 'yt-dlp Engine Ready',
        message: `yt-dlp version ${status.version || 'latest'} is active. YouTube, TikTok (without watermark), and Instagram extraction are ready.`,
      });
    } catch (err: any) {
      addToast({
        type: 'error',
        title: 'Installation Notice',
        message: err?.message || 'Could not install yt-dlp automatically. You can install it via "winget install yt-dlp.yt-dlp".',
      });
    } finally {
      setIsInstallingYtDlp(false);
    }
  };

  const handleInstallFFmpeg = async () => {
    setIsInstallingFFmpeg(true);
    try {
      const status = await api.installFFmpeg();
      setFfmpegStatus(status);
      addToast({
        type: 'success',
        title: 'FFmpeg Ready',
        message: 'FFmpeg is active and ready for media merging and transcode operations.',
      });
    } catch (err: any) {
      addToast({
        type: 'error',
        title: 'FFmpeg Notice',
        message: err?.message || 'Could not install FFmpeg automatically. Run "winget install Gyan.FFmpeg.Essentials".',
      });
    } finally {
      setIsInstallingFFmpeg(false);
    }
  };

  if (!form) return null;

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>Application Settings</h1>
          <p className={styles.subtitle}>
            Configure global download behavior, scheduler defaults, retry logic, and system paths.
          </p>
        </div>

        <Button
          variant="primary"
          size="sm"
          loading={isSaving}
          onClick={handleSave}
        >
          Save Changes
        </Button>
      </div>

      <div className={styles.settingsGrid}>
        {/* Storage & Directories */}
        <div className={styles.card}>
          <div className={styles.cardHeader}>
            <FolderOpen size={18} className={styles.cardIcon} />
            <h3 className={styles.cardTitle}>Downloads &amp; Storage</h3>
          </div>

          <div className={styles.cardBody}>
            <Input
              label="Default Output Directory"
              value={form.default_download_directory}
              onChange={(e) =>
                setForm({ ...form, default_download_directory: e.target.value })
              }
              helperText="Destination directory for newly imported batches"
            />

            <Input
              label="Default File Naming Template"
              value={form.default_naming_template}
              onChange={(e) =>
                setForm({ ...form, default_naming_template: e.target.value })
              }
              helperText="Variables: {title}, {quality}, {format}, {date}, {batch_id}, {index}"
            />

            <label className={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={form.preserve_partial_on_cancel}
                onChange={(e) =>
                  setForm({ ...form, preserve_partial_on_cancel: e.target.checked })
                }
              />
              Preserve .bvd-partial files on task cancellation (resumable)
            </label>

            <label className={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={form.auto_open_folder}
                onChange={(e) =>
                  setForm({ ...form, auto_open_folder: e.target.checked })
                }
              />
              Automatically open folder when a batch finishes
            </label>
          </div>
        </div>

        {/* Batching & Concurrency */}
        <div className={styles.card}>
          <div className={styles.cardHeader}>
            <Settings size={18} className={styles.cardIcon} />
            <h3 className={styles.cardTitle}>Batch Scheduler Defaults</h3>
          </div>

          <div className={styles.cardBody}>
            <div className={styles.twoCol}>
              <Input
                label="Default Batch Size"
                type="number"
                min="1"
                max="500"
                value={form.default_batch_size.toString()}
                onChange={(e) =>
                  setForm({
                    ...form,
                    default_batch_size: parseInt(e.target.value) || 10,
                  })
                }
              />

              <Input
                label="Default Concurrency Limit"
                type="number"
                min="1"
                max="16"
                value={form.default_concurrency.toString()}
                onChange={(e) =>
                  setForm({
                    ...form,
                    default_concurrency: parseInt(e.target.value) || 3,
                  })
                }
              />
            </div>

            <Select
              label="Default Completion Policy"
              value={form.default_completion_policy}
              onChange={(e) =>
                setForm({ ...form, default_completion_policy: e.target.value })
              }
              options={[
                {
                  value: 'TERMINAL_COMPLETION',
                  label: 'Policy B: Terminal Completion (Advance on finish or fail)',
                },
                {
                  value: 'ALL_SUCCESSFUL',
                  label: 'Policy A: All Successful (Advance only if 100% succeed)',
                },
                {
                  value: 'PAUSE_ON_FAILURE',
                  label: 'Policy C: Pause on Failure (Halt batch on first error)',
                },
                {
                  value: 'CONTINUE_WITH_RETRY',
                  label: 'Policy D: Continue with Retry (Auto-retry before advancing)',
                },
              ]}
            />

            <label className={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={form.auto_start_next_batch}
                onChange={(e) =>
                  setForm({ ...form, auto_start_next_batch: e.target.checked })
                }
              />
              Automatically activate next batch when current completes
            </label>
          </div>
        </div>

        {/* Retry & Fault Tolerance */}
        <div className={styles.card}>
          <div className={styles.cardHeader}>
            <RefreshCw size={18} className={styles.cardIcon} />
            <h3 className={styles.cardTitle}>Retry Policy &amp; Fault Tolerance</h3>
          </div>

          <div className={styles.cardBody}>
            <Input
              label="Max Automatic Retries"
              type="number"
              min="0"
              max="10"
              value={form.default_max_retries.toString()}
              onChange={(e) =>
                setForm({
                  ...form,
                  default_max_retries: parseInt(e.target.value) || 0,
                })
              }
              helperText="Exponential backoff with randomized jitter is automatically applied to transient network drops."
            />

            <label className={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={form.notify_on_failure}
                onChange={(e) =>
                  setForm({ ...form, notify_on_failure: e.target.checked })
                }
              />
              Emit toast notification when a task exhausts retries and fails
            </label>

            <label className={styles.checkboxLabel}>
              <input
                type="checkbox"
                checked={form.notify_on_completion}
                onChange={(e) =>
                  setForm({ ...form, notify_on_completion: e.target.checked })
                }
              />
              Emit toast notification when a batch or task completes
            </label>
          </div>
        </div>

        {/* Social Media & Streaming Engine (yt-dlp) */}
        <div className={styles.card}>
          <div className={styles.cardHeader}>
            <DownloadCloud size={18} className={styles.cardIcon} />
            <h3 className={styles.cardTitle}>Social Media &amp; Streaming Engine (yt-dlp)</h3>
          </div>

          <div className={styles.cardBody}>
            <div className={styles.ffmpegBox}>
              <div className={styles.ffmpegHeader}>
                <span className={styles.ffmpegLabel}>yt-dlp Status:</span>
                <span
                  className={`${styles.ffmpegBadge} ${
                    ytdlpStatus?.is_available ? styles.badgeSuccess : styles.badgeWarning
                  }`}
                >
                  {ytdlpStatus?.is_available ? `Active (v${ytdlpStatus.version || 'installed'})` : 'Not Detected'}
                </span>
              </div>
              <p className={styles.ffmpegDesc}>
                {ytdlpStatus?.message ||
                  'Enables native downloading from YouTube, TikTok (clean without moving watermark), Instagram Reels/Stories, X/Twitter, and 1,000+ streaming sites.'}
              </p>
              {ytdlpStatus?.binary_path && (
                <p style={{ fontSize: '11px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)', wordBreak: 'break-all' }}>
                  Path: {ytdlpStatus.binary_path}
                </p>
              )}
              <div className={styles.btnGroup}>
                <Button
                  variant="primary"
                  size="sm"
                  loading={isInstallingYtDlp}
                  onClick={handleInstallYtDlp}
                >
                  {ytdlpStatus?.is_available ? 'Update yt-dlp' : 'Install yt-dlp'}
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => api.checkYtDlp().then(setYtdlpStatus)}
                >
                  Re-check Engine
                </Button>
              </div>
            </div>
          </div>
        </div>

        {/* Media & System Tools */}
        <div className={styles.card}>
          <div className={styles.cardHeader}>
            <HardDrive size={18} className={styles.cardIcon} />
            <h3 className={styles.cardTitle}>Media Processing &amp; Diagnostics</h3>
          </div>

          <div className={styles.cardBody}>
            <div className={styles.ffmpegBox}>
              <div className={styles.ffmpegHeader}>
                <span className={styles.ffmpegLabel}>FFmpeg Subprocess Status:</span>
                <span
                  className={`${styles.ffmpegBadge} ${
                    ffmpegStatus?.is_available ? styles.badgeSuccess : styles.badgeWarning
                  }`}
                >
                  {ffmpegStatus?.is_available ? 'Available' : 'Not Detected'}
                </span>
              </div>
              <p className={styles.ffmpegDesc}>{ffmpegStatus?.message}</p>
              {ffmpegStatus?.binary_path && (
                <p style={{ fontSize: '11px', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)', wordBreak: 'break-all' }}>
                  Path: {ffmpegStatus.binary_path}
                </p>
              )}
              <div className={styles.btnGroup}>
                {!ffmpegStatus?.is_available && (
                  <Button
                    variant="primary"
                    size="sm"
                    loading={isInstallingFFmpeg}
                    onClick={handleInstallFFmpeg}
                  >
                    Install FFmpeg
                  </Button>
                )}
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => api.checkFFmpeg().then(setFfmpegStatus)}
                >
                  Re-check FFmpeg
                </Button>
              </div>
            </div>

            <div className={styles.actionsDivider} />

            <div className={styles.maintenanceRow}>
              <div>
                <span className={styles.maintenanceTitle}>Database Integrity Check</span>
                <span className={styles.maintenanceSubtitle}>
                  Verify SQLite WAL pages, B-tree balance, and foreign key relations.
                </span>
              </div>
              <Button
                variant="outline"
                size="sm"
                icon={<ShieldCheck size={15} />}
                onClick={handleIntegrityCheck}
              >
                Run Check
              </Button>
            </div>
          </div>
        </div>

        {/* Engineering & Developer Attribution */}
        <div className={styles.card}>
          <div className={styles.cardHeader}>
            <User size={18} className={styles.cardIcon} />
            <h3 className={styles.cardTitle}>Software Engineering &amp; Credits</h3>
          </div>

          <div className={styles.cardBody}>
            <div className={styles.ffmpegBox}>
              <div className={styles.ffmpegHeader}>
                <div>
                  <span style={{ fontWeight: 700, fontSize: '14px', color: 'var(--text-primary)', display: 'block' }}>
                    Haseeb Kaloya
                  </span>
                  <span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                    Product Owner &amp; Lead Engineer
                  </span>
                </div>
                <span className={styles.badgeSuccess}>
                  Lead Developer
                </span>
              </div>
              <p className={styles.ffmpegDesc}>
                Architected and engineered for high-performance concurrent media capture, zero-drop queue scheduling, and robust local persistence.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

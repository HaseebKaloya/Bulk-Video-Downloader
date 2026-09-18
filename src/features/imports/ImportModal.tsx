import React, { useState, useRef, useEffect } from 'react';
import { Upload, FileText, CheckCircle2, AlertTriangle, Play, Sparkles, Sliders } from 'lucide-react';
import { api } from '../../services/api';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from '../../components/ui/Button';
import { Modal } from '../../components/ui/Modal';
import { Input, Select, Textarea } from '../../components/ui/FormControls';
import styles from './ImportModal.module.css';

export const ImportModal: React.FC = () => {
  const {
    isImportModalOpen,
    closeImportModal,
    importPreview,
    setImportPreview,
    profiles,
    settings,
    addToast,
    refreshTasks,
    refreshBatches,
  } = useAppStore();

  const [inputMode, setInputMode] = useState<'paste' | 'file'>('paste');
  const [pastedText, setPastedText] = useState('');
  const [duplicatePolicy, setDuplicatePolicy] = useState<'SKIP' | 'KEEP_SEPARATE' | 'REPLACE'>('SKIP');
  const [batchName, setBatchName] = useState('');
  const [startNow, setStartNow] = useState(true);
  const [isParsing, setIsParsing] = useState(false);
  const [isCommitting, setIsCommitting] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Profile & Format Configuration
  const [selectedProfileId, setSelectedProfileId] = useState('');
  const [mediaType, setMediaType] = useState<'video_audio' | 'audio_only' | 'video_only'>('video_audio');
  const [qualityOverride, setQualityOverride] = useState('highest');
  const [formatOverride, setFormatOverride] = useState('mp4');
  const [destinationDirectory, setDestinationDirectory] = useState('');

  // Sync with default profile upon modal open or profiles change
  useEffect(() => {
    const defaultId = settings?.default_profile_id;
    const target = profiles.find((p) => p.id === defaultId) || profiles[0];
    if (target) {
      setSelectedProfileId(target.id);
      applyProfile(target);
    } else {
      setDestinationDirectory(settings?.default_download_directory || '.');
    }
  }, [isImportModalOpen, settings?.default_profile_id, profiles]);

  const applyProfile = (p: typeof profiles[0]) => {
    const isAudio = p.quality_policy === 'audio' || p.format_policy === 'mp3' || p.format_policy === 'm4a';
    const isVideoOnly = p.quality_policy.includes('video_only');
    setMediaType(isAudio ? 'audio_only' : isVideoOnly ? 'video_only' : 'video_audio');
    setQualityOverride(p.quality_policy);
    setFormatOverride(p.format_policy);
    setDestinationDirectory(p.destination_directory || settings?.default_download_directory || '.');
  };

  const handleProfileChange = (profileId: string) => {
    setSelectedProfileId(profileId);
    const target = profiles.find((p) => p.id === profileId);
    if (target) {
      applyProfile(target);
    }
  };

  const handleMediaTypeChange = (type: 'video_audio' | 'audio_only' | 'video_only') => {
    setMediaType(type);
    if (type === 'audio_only') {
      setQualityOverride('audio');
      setFormatOverride('mp3');
    } else if (type === 'video_only') {
      setQualityOverride('1080p');
      setFormatOverride('mp4');
    } else {
      setQualityOverride('highest');
      setFormatOverride('mp4');
    }
  };

  const getPlatformBadgeClass = (platform?: string | null) => {
    if (!platform) return styles.platformGeneric;
    const p = platform.toLowerCase();
    if (p.includes('youtube')) return styles.platformYoutube;
    if (p.includes('tiktok')) return styles.platformTiktok;
    if (p.includes('instagram')) return styles.platformInstagram;
    if (p.includes('twitter') || p.includes('x')) return styles.platformTwitter;
    return styles.platformGeneric;
  };

  const handlePreview = async (content: string) => {
    if (!content.trim()) return;
    setIsParsing(true);
    try {
      const preview = await api.previewImport(content);
      setImportPreview(preview);
    } catch (err) {
      addToast({ type: 'error', message: 'Failed to parse import content.' });
    } finally {
      setIsParsing(false);
    }
  };

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      const text = event.target?.result as string;
      if (text) {
        setPastedText(text);
        handlePreview(text);
      }
    };
    reader.readAsText(file);
  };

  const handleCommit = async () => {
    if (!importPreview || importPreview.valid_count === 0) return;
    setIsCommitting(true);

    const validUrls = importPreview.items
      .filter((item) => item.is_valid)
      .map((item) => item.normalized_url);

    try {
      const result = await api.commitImport({
        batch_name: batchName.trim() || undefined,
        duplicate_policy: duplicatePolicy,
        urls: validUrls,
        start_now: startNow,
        profile_id: selectedProfileId || undefined,
        selected_quality: qualityOverride,
        selected_format: formatOverride,
        media_type: mediaType,
        destination_directory: destinationDirectory || settings?.default_download_directory,
      });

      addToast({
        type: 'success',
        title: 'Import Successful',
        message: `Imported ${result.imported_count} tasks into batch (Skipped: ${result.skipped_count}).`,
      });

      await refreshTasks();
      await refreshBatches();
      closeImportModal();
      setPastedText('');
    } catch (err) {
      addToast({ type: 'error', message: 'Failed to commit import tasks.' });
    } finally {
      setIsCommitting(false);
    }
  };

  return (
    <Modal
      isOpen={isImportModalOpen}
      onClose={closeImportModal}
      title="Import Video URLs"
      maxWidth="lg"
      footer={
        <div className={styles.modalFooter}>
          <Button variant="ghost" size="sm" onClick={closeImportModal}>
            Cancel
          </Button>

          {importPreview ? (
            <Button
              variant="primary"
              size="sm"
              icon={<Play size={15} />}
              loading={isCommitting}
              disabled={importPreview.valid_count === 0}
              onClick={handleCommit}
            >
              Start Import ({importPreview.valid_count} Tasks)
            </Button>
          ) : (
            <Button
              variant="primary"
              size="sm"
              loading={isParsing}
              disabled={!pastedText.trim()}
              onClick={() => handlePreview(pastedText)}
            >
              Preview Import
            </Button>
          )}
        </div>
      }
    >
      <div className={styles.container}>
        {!importPreview ? (
          <>
            <div className={styles.platformBanner}>
              <Sparkles size={15} style={{ color: 'var(--accent-primary)', flexShrink: 0 }} />
              <span>
                <strong>Smart Video Engine:</strong> Native support for YouTube, TikTok (clean without moving watermark), Instagram Reels/Stories, X/Twitter, and direct media links.
              </span>
            </div>

            <div className={styles.tabBar}>
              <button
                className={`${styles.tabBtn} ${inputMode === 'paste' ? styles.activeTab : ''}`}
                onClick={() => setInputMode('paste')}
              >
                <FileText size={16} />
                Paste Text / URLs
              </button>
              <button
                className={`${styles.tabBtn} ${inputMode === 'file' ? styles.activeTab : ''}`}
                onClick={() => setInputMode('file')}
              >
                <Upload size={16} />
                Upload File (TXT, CSV, JSON)
              </button>
            </div>

            {inputMode === 'paste' ? (
              <div className={styles.pasteSection}>
                <Textarea
                  placeholder="Paste links here (one per line, comma-separated, or JSON array)...&#10;https://www.youtube.com/watch?v=...&#10;https://www.tiktok.com/@creator/video/...&#10;https://www.instagram.com/reel/...&#10;https://example.com/video.mp4"
                  value={pastedText}
                  onChange={(e) => setPastedText(e.target.value)}
                  className={styles.largeTextarea}
                />
              </div>
            ) : (
              <div
                className={styles.dropzone}
                onClick={() => fileInputRef.current?.click()}
                onDragOver={(e) => e.preventDefault()}
                onDrop={(e) => {
                  e.preventDefault();
                  const file = e.dataTransfer.files[0];
                  if (file) {
                    const reader = new FileReader();
                    reader.onload = (event) => {
                      const text = event.target?.result as string;
                      if (text) {
                        setPastedText(text);
                        handlePreview(text);
                      }
                    };
                    reader.readAsText(file);
                  }
                }}
              >
                <Upload size={32} className={styles.dropzoneIcon} />
                <p className={styles.dropzoneTitle}>Click to upload or drag &amp; drop</p>
                <p className={styles.dropzoneSubtitle}>Supported: .txt, .csv, .json (up to thousands of URLs)</p>
                <input
                  type="file"
                  ref={fileInputRef}
                  style={{ display: 'none' }}
                  accept=".txt,.csv,.json"
                  onChange={handleFileUpload}
                />
              </div>
            )}
          </>
        ) : (
          <div className={styles.previewContainer}>
            <div className={styles.statsGrid}>
              <div className={styles.statCard}>
                <span className={styles.statNumber}>{importPreview.total_rows}</span>
                <span className={styles.statLabel}>Total Detected</span>
              </div>
              <div className={`${styles.statCard} ${styles.statSuccess}`}>
                <span className={styles.statNumber}>{importPreview.valid_count}</span>
                <span className={styles.statLabel}>Valid URLs</span>
              </div>
              <div className={`${styles.statCard} ${styles.statWarning}`}>
                <span className={styles.statNumber}>
                  {importPreview.duplicate_intra_count + importPreview.duplicate_queue_count}
                </span>
                <span className={styles.statLabel}>Duplicates</span>
              </div>
              <div className={`${styles.statCard} ${styles.statDanger}`}>
                <span className={styles.statNumber}>{importPreview.invalid_count}</span>
                <span className={styles.statLabel}>Invalid Rows</span>
              </div>
            </div>

            <div className={styles.configRow}>
              <Input
                label="Batch Name (Optional)"
                placeholder="e.g. Vacation Videos 2026"
                value={batchName}
                onChange={(e) => setBatchName(e.target.value)}
              />

              <Select
                label="Duplicate Handling"
                value={duplicatePolicy}
                onChange={(e) =>
                  setDuplicatePolicy(e.target.value as 'SKIP' | 'KEEP_SEPARATE' | 'REPLACE')
                }
                options={[
                  { value: 'SKIP', label: 'Skip duplicates (Recommended)' },
                  { value: 'KEEP_SEPARATE', label: 'Keep as separate tasks' },
                  { value: 'REPLACE', label: 'Replace existing queued tasks' },
                ]}
              />
            </div>

            <div className={styles.profileSection}>
              <div className={styles.profileHeaderRow}>
                <Sliders size={16} className={styles.sectionIcon} />
                <span className={styles.sectionTitle}>Download Profile &amp; Format Options</span>
              </div>

              <div className={styles.profileSelectGrid}>
                <Select
                  label="Download Profile"
                  value={selectedProfileId}
                  onChange={(e) => handleProfileChange(e.target.value)}
                  options={profiles.map((p) => ({
                    value: p.id,
                    label: `${p.name}${p.id === settings?.default_profile_id ? ' ⭐ (Default)' : ''} [${p.quality_policy.toUpperCase().replace('_VIDEO_ONLY', ' MUTE')} • ${p.format_policy.toUpperCase()}]`,
                  }))}
                />

                <Select
                  label="Download Mode"
                  value={mediaType}
                  onChange={(e) =>
                    handleMediaTypeChange(e.target.value as 'video_audio' | 'audio_only' | 'video_only')
                  }
                  options={[
                    { value: 'video_audio', label: '🎬 Video + Audio (Full Media)' },
                    { value: 'audio_only', label: '🎵 Audio Only (MP3 / M4A Extraction)' },
                    { value: 'video_only', label: '🔇 Video Only (No Audio / Mute)' },
                  ]}
                />
              </div>

              <div className={styles.overridesGrid}>
                <Select
                  label="Resolution / Quality"
                  value={qualityOverride.replace('_video_only', '')}
                  onChange={(e) => setQualityOverride(e.target.value)}
                  disabled={mediaType === 'audio_only'}
                  options={
                    mediaType === 'audio_only'
                      ? [{ value: 'audio', label: 'Best Audio Quality' }]
                      : [
                          { value: 'highest', label: 'Highest Available (Best)' },
                          { value: '2160p', label: '4K Ultra HD (2160p)' },
                          { value: '1440p', label: '2K Quad HD (1440p)' },
                          { value: '1080p', label: '1080p Full HD' },
                          { value: '720p', label: '720p HD' },
                          { value: '480p', label: '480p Standard' },
                          { value: '360p', label: '360p Low' },
                        ]
                  }
                />

                <Select
                  label="Container Format"
                  value={formatOverride}
                  onChange={(e) => setFormatOverride(e.target.value)}
                  options={
                    mediaType === 'audio_only'
                      ? [
                          { value: 'mp3', label: 'MP3 (Standard Audio)' },
                          { value: 'm4a', label: 'M4A / AAC (Apple Audio)' },
                          { value: 'wav', label: 'WAV (Uncompressed)' },
                          { value: 'flac', label: 'FLAC (Lossless)' },
                        ]
                      : [
                          { value: 'mp4', label: 'MP4 (Universal Video)' },
                          { value: 'mkv', label: 'MKV (Matroska)' },
                          { value: 'webm', label: 'WebM (Web Video)' },
                        ]
                  }
                />
              </div>
            </div>

            <div className={styles.checkboxRow}>
              <label className={styles.checkboxLabel}>
                <input
                  type="checkbox"
                  checked={startNow}
                  onChange={(e) => setStartNow(e.target.checked)}
                />
                Start downloading immediately after import
              </label>
              <button
                type="button"
                className={styles.resetBtn}
                onClick={() => setImportPreview(null)}
              >
                Change Input
              </button>
            </div>

            <div className={styles.previewTableWrapper}>
              <table className={styles.previewTable}>
                <thead>
                  <tr>
                    <th>Status</th>
                    <th>Source Engine</th>
                    <th>URL</th>
                    <th>Extracted Title</th>
                    <th>Notes</th>
                  </tr>
                </thead>
                <tbody>
                  {importPreview.items.slice(0, 100).map((item, idx) => (
                    <tr key={idx}>
                      <td>
                        {item.is_valid ? (
                          <CheckCircle2 size={16} className={styles.validIcon} />
                        ) : (
                          <AlertTriangle size={16} className={styles.invalidIcon} />
                        )}
                      </td>
                      <td>
                        <span
                          className={`${styles.platformBadge} ${getPlatformBadgeClass(
                            item.platform
                          )}`}
                        >
                          {item.platform || 'Direct HTTP'}
                        </span>
                      </td>
                      <td className={styles.urlCell} title={item.normalized_url}>
                        {item.normalized_url}
                      </td>
                      <td>{item.title}</td>
                      <td className={styles.notesCell}>
                        {item.is_duplicate_intra && 'Duplicate in list '}
                        {item.is_duplicate_queue && 'Already in queue '}
                        {item.error && item.error}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {importPreview.items.length > 100 && (
                <div className={styles.tableFooterNote}>
                  Showing first 100 of {importPreview.items.length} items.
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </Modal>
  );
};

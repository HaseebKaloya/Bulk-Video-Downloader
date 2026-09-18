import React, { useState } from 'react';
import { Sliders, Plus, Copy, Trash2, Edit2, Sparkles, Check } from 'lucide-react';
import { api } from '../../services/api';
import { useAppStore } from '../../stores/useAppStore';
import { Button } from '../../components/ui/Button';
import { Modal } from '../../components/ui/Modal';
import { Input, Select } from '../../components/ui/FormControls';
import { DownloadProfile } from '../../types';
import styles from './Profiles.module.css';

const uuid = (): string => {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return Math.random().toString(36).substring(2, 9);
};

export const ProfilesView: React.FC = () => {
  const { profiles, settings, addToast, refreshProfiles, refreshSettings, showConfirmDialog } = useAppStore();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingProfile, setEditingProfile] = useState<DownloadProfile | null>(null);

  const [name, setName] = useState('');
  const [mediaType, setMediaType] = useState<'video_audio' | 'audio_only' | 'video_only'>('video_audio');
  const [qualityPolicy, setQualityPolicy] = useState('1080p');
  const [formatPolicy, setFormatPolicy] = useState('mp4');
  const [namingTemplate, setNamingTemplate] = useState('{title}_{quality}.{format}');
  const [destinationDirectory, setDestinationDirectory] = useState('');
  const [setAsDefault, setSetAsDefault] = useState(false);

  const openCreateModal = () => {
    setEditingProfile(null);
    setName('');
    setMediaType('video_audio');
    setQualityPolicy('highest');
    setFormatPolicy('mp4');
    setNamingTemplate('{title}_{quality}.{format}');
    setDestinationDirectory(settings?.default_download_directory || '.');
    setSetAsDefault(false);
    setIsModalOpen(true);
  };

  const openEditModal = (p: DownloadProfile) => {
    setEditingProfile(p);
    setName(p.name);
    const isAudio = p.quality_policy === 'audio' || p.format_policy === 'mp3' || p.format_policy === 'm4a';
    const isVideoOnly = p.quality_policy.includes('video_only');
    setMediaType(isAudio ? 'audio_only' : isVideoOnly ? 'video_only' : 'video_audio');
    setQualityPolicy(p.quality_policy);
    setFormatPolicy(p.format_policy);
    setNamingTemplate(p.naming_template);
    setDestinationDirectory(p.destination_directory);
    setSetAsDefault(settings?.default_profile_id === p.id);
    setIsModalOpen(true);
  };

  const handleSetDefault = async (profileId: string) => {
    try {
      await api.setDefaultProfile(profileId);
      await refreshSettings();
      const p = profiles.find((item) => item.id === profileId);
      addToast({
        type: 'success',
        message: `Set '${p?.name || profileId}' as the default download profile.`,
      });
    } catch {
      addToast({ type: 'error', message: 'Failed to update default profile.' });
    }
  };

  const handleDuplicate = async (p: DownloadProfile) => {
    const dupId = uuid();
    const dup: DownloadProfile = {
      ...p,
      id: dupId,
      name: `${p.name} (Copy)`,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    try {
      await api.createProfile(dup);
      addToast({ type: 'success', message: `Duplicated profile '${p.name}'.` });
      await refreshProfiles();
    } catch {
      addToast({ type: 'error', message: 'Failed to duplicate profile.' });
    }
  };

  const handleMediaTypeChange = (type: 'video_audio' | 'audio_only' | 'video_only') => {
    setMediaType(type);
    if (type === 'audio_only') {
      setQualityPolicy('audio');
      setFormatPolicy('mp3');
      setNamingTemplate('{title}_audio.{format}');
    } else if (type === 'video_only') {
      setQualityPolicy('1080p');
      setFormatPolicy('mp4');
      setNamingTemplate('{title}_video.{format}');
    } else {
      setQualityPolicy('highest');
      setFormatPolicy('mp4');
      setNamingTemplate('{title}_{quality}.{format}');
    }
  };

  const handleSave = async () => {
    if (!name.trim()) {
      addToast({ type: 'error', message: 'Profile name is required.' });
      return;
    }

    const targetId = editingProfile?.id || uuid();
    const finalQuality = mediaType === 'video_only' && !qualityPolicy.includes('video_only')
      ? `${qualityPolicy}_video_only`
      : qualityPolicy;

    const payload: DownloadProfile = {
      id: targetId,
      name: name.trim(),
      quality_policy: finalQuality,
      format_policy: formatPolicy,
      naming_template: namingTemplate.trim() || '{title}_{quality}.{format}',
      destination_directory: destinationDirectory.trim() || '.',
      created_at: editingProfile?.created_at || new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    try {
      if (editingProfile) {
        await api.updateProfile(payload);
        addToast({ type: 'success', message: 'Profile updated.' });
      } else {
        await api.createProfile(payload);
        addToast({ type: 'success', message: 'Profile created.' });
      }

      if (setAsDefault) {
        await api.setDefaultProfile(targetId);
        await refreshSettings();
      }

      setIsModalOpen(false);
      await refreshProfiles();
    } catch {
      addToast({ type: 'error', message: 'Failed to save profile.' });
    }
  };


  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>Download Profiles</h1>
          <p className={styles.subtitle}>
            Create reusable presets for resolution preferences, container formats, and directory targets.
          </p>
        </div>

        <Button
          variant="primary"
          size="sm"
          icon={<Plus size={16} />}
          onClick={openCreateModal}
        >
          Create Profile
        </Button>
      </div>

      <div className={styles.profileGrid}>
        {profiles.map((profile) => {
          const isDefault = settings?.default_profile_id === profile.id;

          return (
            <div
              key={profile.id}
              className={`${styles.profileCard} ${isDefault ? styles.defaultCard : ''}`}
            >
              <div className={styles.cardTop}>
                <div className={styles.profileHeader}>
                  <Sliders size={20} className={styles.profileIcon} />
                  <div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <h3 className={styles.profileName}>{profile.name}</h3>
                      {isDefault && (
                        <span className={styles.defaultBadge}>
                          <Sparkles size={10} /> DEFAULT
                        </span>
                      )}
                    </div>
                    <span className={styles.policySubtext}>
                      {profile.quality_policy.toUpperCase().replace('_VIDEO_ONLY', ' (MUTE)')} &bull; {profile.format_policy.toUpperCase()}
                    </span>
                  </div>
                </div>

                <div className={styles.cardActions}>
                  {!isDefault && (
                    <button
                      type="button"
                      className={styles.setDefaultBtn}
                      onClick={() => handleSetDefault(profile.id)}
                      title="Set as Default Profile"
                    >
                      <Check size={12} /> Set Default
                    </button>
                  )}
                  <Button
                    variant="ghost"
                    size="sm"
                    icon={<Copy size={14} />}
                    onClick={() => handleDuplicate(profile)}
                    title="Duplicate profile"
                  />
                  <Button
                    variant="ghost"
                    size="sm"
                    icon={<Edit2 size={14} />}
                    onClick={() => openEditModal(profile)}
                    title="Edit profile"
                  />
                  <Button
                    variant="ghost"
                    size="sm"
                    icon={<Trash2 size={14} />}
                    onClick={() =>
                      showConfirmDialog({
                        title: 'Delete Profile',
                        message: `Are you sure you want to delete profile '${profile.name}'?`,
                        confirmLabel: 'Delete',
                        isDestructive: true,
                        onConfirm: () => api.deleteProfile(profile.id).then(refreshProfiles),
                      })
                    }
                    title="Delete profile"
                  />
                </div>
              </div>

              <div className={styles.configDetails}>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Naming Template:</span>
                  <code className={styles.codeSnippet}>{profile.naming_template}</code>
                </div>
                <div className={styles.detailItem}>
                  <span className={styles.detailLabel}>Output Directory:</span>
                  <span className={styles.pathText} title={profile.destination_directory}>
                    {profile.destination_directory}
                  </span>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {/* Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={editingProfile ? 'Edit Download Profile' : 'New Download Profile'}
        footer={
          <>
            <Button variant="outline" size="sm" onClick={() => setIsModalOpen(false)}>
              Cancel
            </Button>
            <Button variant="primary" size="sm" onClick={handleSave}>
              Save Profile
            </Button>
          </>
        }
      >
        <div className={styles.formStack}>
          <Input
            label="Profile Name"
            placeholder="e.g. 4K High Fidelity, Audio MP3, etc."
            value={name}
            onChange={(e) => setName(e.target.value)}
          />

          <Select
            label="Media Type"
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

          <div className={styles.formRow}>
            <Select
              label="Quality / Resolution"
              value={qualityPolicy.replace('_video_only', '')}
              onChange={(e) => setQualityPolicy(e.target.value)}
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
              label="Format / Container"
              value={formatPolicy}
              onChange={(e) => setFormatPolicy(e.target.value)}
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

          <Input
            label="Naming Template"
            value={namingTemplate}
            onChange={(e) => setNamingTemplate(e.target.value)}
            helperText="Tokens: {title}, {quality}, {format}, {date}, {batch_id}, {index}"
          />

          <Input
            label="Destination Directory"
            value={destinationDirectory}
            onChange={(e) => setDestinationDirectory(e.target.value)}
          />

          <label className={styles.checkboxRow}>
            <input
              type="checkbox"
              checked={setAsDefault}
              onChange={(e) => setSetAsDefault(e.target.checked)}
            />
            <span>Set this profile as default for all new downloads</span>
          </label>
        </div>
      </Modal>
    </div>
  );
};

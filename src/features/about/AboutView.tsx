import {
  Mail,
  Shield,
  Layers,
  Code2,
  ExternalLink,
} from 'lucide-react';
import styles from './About.module.css';

export const AboutView: React.FC = () => {
  return (
    <div className={styles.container}>
      <div className={styles.heroCard}>
        <div className={styles.heroIcon}>
          <img src="/app-logo.png" alt="Logo" className={styles.heroLogoImg} />
        </div>
        <h1 className={styles.heroTitle}>Bulk Video Downloader</h1>
        <p className={styles.heroSubtitle}>
          High-performance desktop media downloader engineered for pure original quality without watermarks. Download from TikTok, YouTube, Instagram Reels, X/Twitter, and direct streams with multi-threaded resume.
        </p>
        <div className={styles.heroMetaRow}>
          <div className={styles.versionBadge}>Version 1.0.0 (Build 2026.09)</div>
          <a
            href="https://github.com/HaseebKaloya"
            target="_blank"
            rel="noreferrer"
            className={styles.developerBadgeTag}
            style={{ textDecoration: 'none', color: 'inherit' }}
          >
            Developed by <strong>Haseeb Kaloya</strong> (GitHub)
          </a>
        </div>
      </div>

      <div className={styles.grid}>
        {/* Creator & Contact Info */}
        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Product Ownership &amp; Design</h3>
          <div className={styles.authorSection}>
            <div className={styles.authorAvatarWrapper}>
              <img
                src="/haseeb-kaloya.jpg"
                alt="Haseeb Kaloya"
                className={styles.authorPhoto}
              />
              <span className={styles.authorStatusBadge} title="Active Developer & Designer" />
            </div>
            <div className={styles.authorDetails}>
              <div className={styles.authorNameRow}>
                <span className={styles.authorName}>Haseeb Kaloya</span>
                <span className={styles.designerBadge}>Creator</span>
              </div>
              <span className={styles.authorRole}>Product Owner &amp; Lead Designer</span>
            </div>
          </div>

          <div className={styles.linkList}>
            <a
              href="mailto:contact.haseebkaloya@gmail.com"
              className={styles.contactLink}
            >
              <Mail size={16} />
              contact.haseebkaloya@gmail.com
            </a>

            <a
              href="https://github.com/HaseebKaloya"
              target="_blank"
              rel="noreferrer"
              className={styles.contactLink}
            >
              <Code2 size={16} />
              https://github.com/HaseebKaloya
              <ExternalLink size={12} className={styles.extIcon} />
            </a>
          </div>
        </div>

        {/* Architectural Guarantees */}
        <div className={styles.card}>
          <h3 className={styles.cardTitle}>Architectural Principles</h3>
          <ul className={styles.principlesList}>
            <li>
              <Code2 size={16} className={styles.bulletIcon} />
              <span>
                <strong>Tauri 2 + Rust Engine:</strong> Native binary runtime with Tokio async I/O and zero arbitrary shell execution.
              </span>
            </li>
            <li>
              <Layers size={16} className={styles.bulletIcon} />
              <span>
                <strong>SQLite WAL Persistence:</strong> Transactional task state changes with integrity verification on launch.
              </span>
            </li>
            <li>
              <Shield size={16} className={styles.bulletIcon} />
              <span>
                <strong>Safe Atomic Finalization:</strong> Range resumption streaming directly to isolated <code>.bvd-partial</code> files.
              </span>
            </li>
          </ul>
        </div>
      </div>

      {/* Compliance & Legal Notice */}
      <div className={styles.complianceCard}>
        <div className={styles.complianceHeader}>
          <Shield size={18} className={styles.complianceIcon} />
          <h4 className={styles.complianceTitle}>Authorized Media &amp; Compliance Policy</h4>
        </div>
        <p className={styles.complianceText}>
          Bulk Video Downloader is engineered for authorized media workflows, backups of user-owned content, educational material, and open media. This software does not bypass DRM, technical restrictions, paywalls, or authentication access controls. Users are responsible for ensuring compliance with copyright laws and host platform terms of service.
        </p>
      </div>

      <div className={styles.aboutFooter}>
        <span>Bulk Video Downloader Desktop Edition • Designed &amp; Developed by <strong>Haseeb Kaloya</strong></span>
      </div>
    </div>
  );
};

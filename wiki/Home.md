# Bulk Video Downloader Documentation

Welcome to the official technical documentation and user reference for **Bulk Video Downloader**, an enterprise-grade desktop media acquisition suite engineered by [Haseeb Kaloya](https://github.com/HaseebKaloya).

Bulk Video Downloader is architected using **Tauri 2**, a compiled **Rust** backend, and a reactive **TypeScript / React 19** frontend. It is designed specifically for content creators, researchers, digital archivists, and video professionals who require deterministic, high-throughput media extraction across major streaming and social video platforms without platform-imposed compression artifacts or moving watermarks.

---

## Documentation Sitemap

| Section | Description | Target Audience |
| :--- | :--- | :--- |
| **[Installation Guide](Installation-Guide)** | Walkthrough of all 4 official distribution formats (NSIS Setup, MSI, Net Installer, Portable) and silent deployment commands. | General Users & Sysadmins |
| **[Batch Download Operations](Batch-Download-Operations)** | Instructions for batch ingestion, multi-threaded worker pools, completion policies, and rate throttling. | Power Users & Content Teams |
| **[Download Profiles & Formats](Download-Profiles-&-Formats)** | Pre-configuring resolution constraints (4K/1080p), audio-only extraction (MP3/FLAC), and dynamic filename templates. | All Users |
| **[Supported Platforms & Extraction](Supported-Platforms-&-Extraction)** | Deep-dive into extractor mechanics: clean TikTok streams without moving watermark, YouTube 4K/8K 60FPS, and Instagram Reels. | General Users |
| **[Architecture & Data Persistence](Architecture-&-Data-Persistence)** | Detailed breakdown of the Tauri 2 IPC layer, Tokio asynchronous runtime, SQLite WAL state machine, and crash recovery reconciliation. | Developers & Contributors |
| **[Troubleshooting & Diagnostics](Troubleshooting-&-Diagnostics)** | Resolving network drops, rate limits (HTTP 429), FFmpeg hardware acceleration, and SQLite page repair. | All Users |

---

## Core System Architecture

Bulk Video Downloader operates on four core engineering principles:

1. **Zero Shell Execution**: All external processing tasks (yt-dlp and FFmpeg) are executed strictly via sanitized vector argument arrays (`std::process::Command`), preventing shell injection vulnerabilities and command prompt popups on Windows.
2. **Transactional SQLite WAL Persistence**: Every download task, batch definition, and chunk offset is recorded in a local SQLite database configured with Write-Ahead Logging (`PRAGMA journal_mode = WAL`). Task state transitions are ACID-compliant.
3. **Resilient Range-Header Streaming**: Direct video downloads are segmented into chunked HTTP range requests streaming directly into isolated `.bvd-partial` files. If a network interruption occurs, transfer resumes from the exact byte offset.
4. **Clean Stream Prioritization**: For platforms known to burn moving visual watermarks into shared clips (such as TikTok), the extraction engine specifically targets upstream API CDN endpoints to retrieve the raw, unadulterated source video file.

---

## Author & Engineering Attribution

- **Architect & Product Owner**: Haseeb Kaloya
- **GitHub Profile**: [https://github.com/HaseebKaloya](https://github.com/HaseebKaloya)
- **Direct Contact**: [contact.haseebkaloya@gmail.com](mailto:contact.haseebkaloya@gmail.com)
- **Source Repository**: [https://github.com/HaseebKaloya/Bulk-Video-Downloader](https://github.com/HaseebKaloya/Bulk-Video-Downloader)
- **Official License**: MIT License

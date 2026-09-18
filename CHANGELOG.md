# Changelog

All notable changes to **Bulk Video Downloader** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-09-18
### Added
- **Official Production Release v1.0.0** by **Haseeb Kaloya**.
- **100% Watermark-Free Engine:** Full resolution media extraction (4K UHD, 1080p, 60fps, HDR) from YouTube, TikTok, Instagram Reels, Facebook, Twitter/X, and 1,000+ sources without branding.
- **Enterprise Windows NSIS Installer:**
  - Standard per-machine installation into `C:\Program Files\Bulk Video Downloader` with UAC elevation.
  - Authentic 24-bit DIB Bitmap sidebar (`installer-sidebar.bmp`) featuring product owner Haseeb Kaloya with headphones, 3D social badges, and watermark-free seal.
  - Desktop shortcut, Start Menu folder, and clean Windows Startup registration.
- **Chrome-Style Foreground WebView2 Bootstrapper:** Interactive foreground downloader for Microsoft Edge WebView2 with live progress animations, preventing silent stalls on fresh Windows installations.
- **Silent Subprocess Execution:** Implemented Windows `CREATE_NO_WINDOW (0x08000000)` across Rust engines (`ytdlp.rs` and `commands/mod.rs`) ensuring zero flashing black CMD prompt windows.
- **High-Performance Architecture:**
  - Rust backend with Tokio async engine and bounded worker pool concurrency.
  - SQLite transactional persistence with Write-Ahead Logging (WAL) and crash reconciliation.
  - Streaming HTTP Range download engine with isolated `.bvd-partial` buffers and atomic finalization.
  - React 19 + TypeScript frontend with pure CSS Modules (zero Tailwind CSS overhead).
  - Virtualized queue rendering capable of tracking 10,000+ concurrent links.
  - Portable standalone edition (`.zip`) packaged with bundled yt-dlp and ffmpeg runtime binaries.

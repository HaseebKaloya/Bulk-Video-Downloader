# Bulk Video Downloader — Troubleshooting

## Common Scenarios & Solutions

### 1. Interrupted Downloads & Crash Recovery
- **Symptoms:** The app was closed or crashed during an active download.
- **Resolution:** On subsequent launch, the built-in startup recovery scanner automatically scans partial `.bvd-partial` files, reconciles database records, and recovers tasks without duplicate downloads. A recovery notification appears summarizing restored tasks.

### 2. Network Timeouts & HTTP 429 (Rate Limits)
- **Symptoms:** Tasks transition to `RetryWait` status.
- **Resolution:** The application utilizes exponential backoff with jitter and respects `Retry-After` HTTP headers. You can adjust the retry limits in **Settings > Retry**.

### 3. Embedded FFmpeg & yt-dlp Binaries
- **Status:** FFmpeg (`ffmpeg.exe`, `ffprobe.exe`) and `yt-dlp.exe` are embedded directly inside the application's `bin/` directory and bundled automatically via `bundle.resources` in `tauri.conf.json`.
- **Runtime Discovery:** The application's `find_binary` prober automatically resolves binaries from the app bundle/resource directory, project `bin/`, and adjacent directories before falling back to system PATH.
- **Cross-Platform Compilation:** For Linux or macOS builds, placing the corresponding platform binaries in `src-tauri/bin/` ensures that `tauri build` packages them into the final `.deb`, `.AppImage`, or `.dmg` with zero runtime download requirements.

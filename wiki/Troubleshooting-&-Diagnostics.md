# Troubleshooting & Diagnostics

This guide provides technical resolutions for network anomalies, platform rate limits, FFmpeg multiplexing warnings, and database maintenance.

---

## 1. Diagnostic Matrix

| Symptom / Error | Root Cause | Recommended Action |
| :--- | :--- | :--- |
| **HTTP 429: Too Many Requests** | Target streaming platform has temporarily throttled your IP due to rapid consecutive requests. | Navigate to **Settings > Batch Scheduler**, reduce **Default Concurrency Limit** to `2` or `3`, and enable **Exponential Backoff** in Retry Policy. |
| **FFmpeg Not Detected** | FFmpeg binary is missing from PATH or local application directory. | Open **Settings > Media Processing & Diagnostics** and click **Install FFmpeg** (or run `winget install Gyan.FFmpeg.Essentials` in terminal). |
| **yt-dlp Extraction Error** | Platform updated upstream API tokens or manifest structures. | Open **Settings > Social Media Engine** and click **Update yt-dlp** to fetch the latest upstream extractor rules. |
| **Download Stalled at 99%** | Stream multiplexing in progress; FFmpeg is merging separate 4K video and audio tracks. | Allow 15–30 seconds for disk I/O to finalize the file. Check disk write activity in Task Manager. |
| **Windows SmartScreen Alert** | Fresh binary release without accumulated historical reputation. | Click **More Info** followed by **Run anyway**. All official binaries match signatures published in `checksums.txt`. |
| **Storage Warning: Insufficient Space** | Destination drive has fewer than 2 GB remaining space. | Navigate to **Settings > Storage** and reassign **Default Output Directory** to a volume with ample storage. |

---

## 2. Running Database Diagnostics

Bulk Video Downloader includes a built-in diagnostic tool to inspect SQLite database health:

1. Open **Settings** from the sidebar.
2. Scroll to the **Media Processing & Diagnostics** card.
3. Locate **Database Integrity Check** and click **Run Check**.
4. The system executes `PRAGMA integrity_check` across all B-tree pages:
   - **Passed**: Confirms 0 corrupt pages and clean index relations.
   - **Warning**: Displays recommendations for rebuilding orphaned indices.

---

## 3. Log File Locations

When reporting an issue on GitHub, attaching your local execution logs helps diagnose the root cause quickly:

- **Application Event Logs**:
  `%LOCALAPPDATA%\com.haseebkaloya.bulk-video-downloader\logs\app.log`
- **Subprocess Extraction Output**:
  `%LOCALAPPDATA%\com.haseebkaloya.bulk-video-downloader\logs\extractor.log`
- **SQLite Database**:
  `%LOCALAPPDATA%\com.haseebkaloya.bulk-video-downloader\app_state.db`

---

## 4. Submitting Technical Issues

If you encounter an unhandled exception or platform extraction regression:

1. Visit the official GitHub issue tracker:
   [https://github.com/HaseebKaloya/Bulk-Video-Downloader/issues](https://github.com/HaseebKaloya/Bulk-Video-Downloader/issues)
2. Include:
   - Target Platform (YouTube, TikTok, Instagram, etc.)
   - Target Resolution (1080p, 4K, Audio)
   - Application Version (e.g., `v1.0.0`)
   - Relevant error snippet from `app.log`.

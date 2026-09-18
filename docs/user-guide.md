# Bulk Video Downloader — User Guide

## 1. Quick Start

### 1.1 Importing Links
1. Click the **Import** button in the global header or dashboard quick actions.
2. Choose your preferred input mode:
   - **Paste Text:** Paste one or more video URLs directly.
   - **File Upload / Drag & Drop:** Select or drop `.txt`, `.csv`, or `.json` files.
3. Review the **Import Preview**:
   - Inspect valid links, invalid entries, and duplicate counts.
   - Select your duplicate handling policy (**Skip**, **Keep Separate**, or **Replace Queued**).
4. Configure batch sizing and destination directory, then click **Commit Import**.

### 1.2 Managing Batches & Concurrency
- Navigate to the **Batches** tab.
- Set **Batch Size** (how many tasks advance together) and **Concurrency** (how many downloads stream simultaneously).
- Select your **Completion Policy**:
  - *All Successful:* Advance only when all tasks finish cleanly.
  - *Terminal Completion:* Advance as soon as every task reaches a final state (completed, failed, cancelled, or skipped).
  - *Pause on Failure:* Halt the batch immediately if any item encounters a permanent failure.
  - *Continue with Retry:* Automatically retry eligible items before advancing.

### 1.3 Download Controls
- Use **Start Queue** or **Pause Queue** to control queue processing.
- Control individual tasks directly: **Pause**, **Resume**, **Cancel**, **Retry**, or **Open Folder**.
- Reorder items in the queue by changing priority.

## 2. Social Media Video Downloads (YouTube, TikTok, Instagram, X/Twitter)

Bulk Video Downloader comes equipped with an intelligent media provider powered by **yt-dlp**:
- **YouTube:** High-definition video, audio-only extraction, and automatic stream multiplexing.
- **TikTok:** Extracts direct video CDN streams **cleanly without moving watermarks**.
- **Instagram:** Full support for Reels, Stories, and post videos.
- **X / Twitter & Streaming Platforms:** Automatic format negotiation for 1,000+ online video services.

### 2.1 One-Click Setup
1. Go to the **Settings** view in the application.
2. In the **Social Media & Streaming Engine (yt-dlp)** card, click **Install yt-dlp**.
3. The engine automatically acquires the binary and validates readiness.
4. Paste any social media URL directly in the Import modal—the application will automatically recognize the platform with a dedicated badge and route execution through the social engine.

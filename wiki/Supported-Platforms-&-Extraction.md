# Supported Platforms & Extraction Mechanics

Bulk Video Downloader leverages a hybrid extraction architecture combining direct HTTP range streaming with an embedded, isolated **yt-dlp** subprocess engine. This enables high-speed acquisition across 1,000+ video hosting services while prioritizing clean source streams without watermarks.

---

## 1. TikTok Clean Extraction (Zero Watermark)

Standard TikTok web and mobile downloaders download the final transcoded file containing the platform's animated moving watermark (which displays the TikTok logo and creator handle bouncing across the screen).

### The Clean Stream Bypass Mechanism

```text
[Submitted TikTok Link] 
         │
         ▼
[Extractor Resolution Phase] ──► Queries TikTok mobile Web API endpoints
         │
         ▼
[Upstream CDN Inspection]   ──► Extracts direct URL to raw source MP4 prior to overlay injection
         │
         ▼
[Direct Chunked Download]   ──► Streams raw, 100% clean video at original bitrate
```

- **Clean MP4**: Video output contains no corner watermarks, no end-screen animations, and no audio re-compression.
- **Batch TikTok Ingestion**: Paste 50+ TikTok video links at once; the engine processes them concurrently according to your batch limit.

---

## 2. YouTube (Up to 4K & 8K 60FPS HDR)

YouTube serves high-definition video (1080p, 1440p, 4K, and 8K) as separate DASH streams: one independent stream for video frames (often VP9 or AV1 codecs) and another independent stream for audio (Opus or AAC).

### Multi-Stream Multiplexing Pipeline

1. **Parallel Stream Ingestion**: The video track and audio track are downloaded simultaneously across dedicated worker connections.
2. **Lossless Multiplexing**: Once both tracks reach 100%, **FFmpeg** merges them into a standardized `.mp4` or `.mkv` container.
3. **Zero Transcode Overhead**: Because frames are merged directly without decoding/re-encoding, the merge process finishes in seconds with 0% quality loss.

---

## 3. Instagram (Reels, Posts, & Stories)

- **Reels & Video Posts**: Extracts full-quality H.264 MP4 streams directly from Instagram CDN nodes.
- **Carousel Sets**: When an Instagram link contains multiple video slides, the parser detects all constituent media and imports each item as an individual queued task.

---

## 4. X / Twitter

- **Dynamic Bitrate Selection**: X serves multiple video bitrate variants. Bulk Video Downloader automatically identifies and selects the highest available bitrate variant (typically 1080p or 720p).
- **Animated GIFs**: Automatically converts MP4-loop video clips served by X back into true `.gif` or keeps them as lightweight `.mp4` depending on your profile preferences.

---

## 5. Direct Streaming Links & HLS (m3u8)

- **HTTP/HTTPS Direct Video Links**: Direct `.mp4`, `.webm`, or `.mkv` URLs are downloaded using the native Rust multi-chunk engine, bypassing yt-dlp entirely for maximum raw bandwidth utilization.
- **HTTP Live Streaming (HLS / m3u8)**: Automatically parses m3u8 playlists, downloads constituent TS segments concurrently, and unifies them into a single clean container.

---

## Platform Support Summary Table

| Platform | Max Supported Resolution | Watermark Removal | Audio Extraction | Batch Lists |
| :--- | :--- | :--- | :--- | :--- |
| **TikTok** | Original Camera Quality | **Yes (100% Clean)** | Yes (MP3/M4A) | Yes |
| **YouTube** | Up to 8K 60FPS HDR | Not Applicable | Yes (High-Bitrate) | Yes |
| **Instagram** | 1080p HD | **Yes (Clean Stream)** | Yes | Yes |
| **X / Twitter** | 1080p Highest Bitrate | Not Applicable | Yes | Yes |
| **Vimeo** | Up to 4K Pro | Not Applicable | Yes | Yes |
| **Direct URLs** | Source Bitrate | Not Applicable | Yes | Yes |

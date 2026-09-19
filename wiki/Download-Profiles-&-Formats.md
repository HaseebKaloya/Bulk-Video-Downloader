# Download Profiles & Formats

Download Profiles allow you to define reusable presets governing media type, resolution constraints, container formats, post-processing rules, and file naming templates.

---

## 1. Quality & Resolution Policies

When configuring a profile, you select a target resolution policy:

| Policy Token | Target Resolution | Fallback Behavior |
| :--- | :--- | :--- |
| `highest` | **Highest Available Source** | Automatically queries the upstream stream manifests and extracts the single highest resolution available (including 8K and 4K 60FPS). |
| `2160p` | **4K Ultra HD (3840×2160)** | Selects 4K if present; gracefully steps down to 1440p or 1080p if 4K is not hosted. |
| `1440p` | **2K Quad HD (2560×1440)** | Ideal for high-density monitors and balanced storage footprints. |
| `1080p` | **Full HD (1920×1080)** | Recommended standard for broad compatibility across television displays and mobile devices. |
| `720p` | **HD (1280×720)** | Compact file sizes suitable for previews, archival reference, and metered storage. |
| `audio` | **Audio Extraction Only** | Bypasses video stream acquisition; downloads the highest-bitrate audio track (typically 128–320 kbps Opus/AAC) and transcodes to target format. |

---

## 2. Media Types & Output Containers

Bulk Video Downloader categorizes downloads into three operational media types:

### A. Video + Audio (Full Media)
- **Container Options**: `mp4`, `mkv`, `webm`
- **Processing**: Video and audio streams are downloaded concurrently in separate threads to maximize throughput, then multiplexed losslessly using FFmpeg into the designated container without re-encoding video frames.

### B. Audio Only (Music & Podcast Extraction)
- **Container Options**: `mp3`, `m4a`, `wav`, `flac`
- **Processing**: Converts upstream Opus/Vorbis/AAC audio streams into standard ID3-tagged MP3 (constant 320 kbps) or lossless FLAC/WAV using FFmpeg libmp3lame / flac encoders.

### C. Video Only (Muted B-Roll)
- **Container Options**: `mp4`, `mkv`
- **Processing**: Strips all audio tracks during demuxing, outputting clean, silent video files for video editors, VFX compositing, and background visual reels.

---

## 3. Dynamic Naming Templates

Customize output filenames automatically using dynamic variable tokens:

```text
{title}_{quality}_{date}.{format}
```

### Supported Tokens

| Token | Replacement Value | Example Output |
| :--- | :--- | :--- |
| `{title}` | Sanitized video title (illegal OS characters like `/ \ : * ? " < > \|` removed). | `Rust_Concurrency_Guide` |
| `{quality}` | Numerical resolution or audio indicator. | `1080p`, `2160p`, `audio` |
| `{format}` | Final container extension. | `mp4`, `mkv`, `mp3` |
| `{date}` | Download execution timestamp (`YYYY-MM-DD`). | `2026-09-19` |
| `{batch_id}` | Short 8-character unique batch identifier. | `bvd-8840` |
| `{index}` | Sequential numeric index within the current batch. | `001`, `002`, `003` |

---

## 4. Default Profiles Bundled in v1.0.0

| Profile Name | Media Type | Resolution | Container | Template |
| :--- | :--- | :--- | :--- | :--- |
| **4K Ultra HD Cinema** | Video + Audio | 2160p | `.mp4` | `{title}_4k.{format}` |
| **1080p Standard Full HD** | Video + Audio | 1080p | `.mp4` | `{title}_{quality}.{format}` |
| **High-Fidelity Audio** | Audio Only | audio | `.mp3` | `{title}_audio.{format}` |
| **TikTok Clean No-Watermark** | Video + Audio | highest | `.mp4` | `{title}_clean.{format}` |

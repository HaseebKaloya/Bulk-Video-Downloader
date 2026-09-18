# 🚀 Bulk Video Downloader v1.0.0 — Production Release (100% Watermark-Free)

[![Release](https://img.shields.io/badge/Release-v1.0.0-blue.svg?style=for-the-badge&logo=github)](https://github.com/HaseebKaloya)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20(x64)-0078D6.svg?style=for-the-badge&logo=windows)](https://github.com/HaseebKaloya)
[![Architecture](https://img.shields.io/badge/Engine-Tauri%202%20%2B%20Rust-DEA584.svg?style=for-the-badge&logo=rust)](https://github.com/HaseebKaloya)
[![Quality](https://img.shields.io/badge/Quality-100%25%20Watermark--Free-00C853.svg?style=for-the-badge&logo=checkmarx)](https://github.com/HaseebKaloya)
[![License](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](https://github.com/HaseebKaloya)
[![Developer](https://img.shields.io/badge/Architect-Haseeb%20Kaloya-purple.svg?style=for-the-badge&logo=github)](https://github.com/HaseebKaloya)

---

## 🌟 Executive Overview

We are thrilled to announce the official **v1.0.0 Production Release** of **Bulk Video Downloader**, engineered and designed by **Haseeb Kaloya**!

Bulk Video Downloader is an enterprise-grade desktop media application crafted from the ground up for content creators, digital archivists, educators, and media professionals. It empowers users to queue, organize, schedule, and download thousands of high-definition video assets simultaneously with **100% original quality and absolutely ZERO watermarks**.

Built on a blazing-fast native **Rust (Tauri 2)** backend paired with a high-performance **React 19** frontend, Bulk Video Downloader delivers instant response times, zero console popups, atomic file safety, and automatic resume capabilities.

---

## ⚡ Key Highlights & Features

### 💎 100% Pure Original Quality & Watermark-Free
- **Zero Watermarks:** Download pristine original media from TikTok, YouTube, Instagram Reels, Facebook, Twitter/X, and 1,000+ platforms without platform branding or overlays.
- **Max Resolution Available:** Automatic extraction for 4K UHD, 2K, 1080p, 60 FPS, and HDR video streams.
- **Pure Audio Stems:** High-bitrate audio ripping directly to MP3, FLAC, AAC, or WAV with embedded metadata.

### 🛡️ Enterprise Windows Integration & Foreground WebView2 Downloader
- **Chrome-Style Foreground Installer Downloader:** Missing Microsoft Edge WebView2 runtime? The installer automatically launches Microsoft's official foreground downloader with live progress animation (no silent hanging or confusion).
- **Native UAC Installation:** Installs into `C:\Program Files\Bulk Video Downloader` with complete uninstaller registration in Windows Control Panel & Settings.
- **Desktop & Startup Registration:** Automatically sets up Desktop shortcuts, Start Menu launcher, and clean Windows Startup registration.
- **Zero Command Prompt Popups:** Fully sandboxed subprocess execution using Windows `CREATE_NO_WINDOW (0x08000000)`—no flashing black CMD boxes.

### 🚀 Industrial-Strength Queue & Batch Engine
- **Massive Queues:** Effortlessly import and track 10,000+ media links with zero interface lag via virtualized rendering.
- **Multi-Format Ingestion:** Paste URLs directly, or import TXT, CSV, and JSON batches with duplicate detection.
- **Atomic File Finalization:** Downloads stream directly to isolated `.bvd-partial` buffers and atomically rename upon checksum verification, preventing corrupted output files.
- **Crash Resilience:** SQLite WAL transactional state ensures instant recovery and auto-reconciliation even across sudden power outages or system restarts.
- **Intelligent Backoff:** Configurable retry system with exponential backoff and randomized jitter to handle transient network drops gracefully.

---

## 📦 Download Assets

| Distribution Package | Filename | Size | Target Architecture | SHA-256 Checksum |
| :--- | :--- | :--- | :--- | :--- |
| 💿 **Windows Setup Installer** *(Recommended)* | `Bulk-Video-Downloader_1.0.0_x64-setup.exe` | **74.3 MB** | Windows 10 / 11 (64-bit) | `99B91976DB801D7269B07907B3EFDB43CA2D4248A0AC72038DCE32336CBF6BA2` |
| 💼 **Portable Standalone Edition** | `Bulk-Video-Downloader_1.0.0_x64_Portable.zip` | **99.9 MB** | Windows 10 / 11 (64-bit) | `C5707E8AC4218428C80B3987C2D91E9AE3C2944BE3E1311934BD1DB09AEE4726` |
| 🔒 **Integrity Manifest** | `checksums.txt` | **359 B** | Universal | Official Cryptographic Hashes |

---

## 🚀 Installation & Quick Start

### Option 1: Standard Setup Installer (Recommended)
1. Download **`Bulk-Video-Downloader_1.0.0_x64-setup.exe`** from the assets below.
2. Double-click the installer. When prompted by Windows UAC, click **Yes** to grant installation privileges.
3. Follow the installation wizard. If your PC does not already have Microsoft Edge WebView2, the official Microsoft setup downloader will automatically run in the foreground with a live progress bar.
4. Launch **Bulk Video Downloader** from your Desktop shortcut or Start Menu.

### Option 2: Portable Edition (No Installation Required)
1. Download **`Bulk-Video-Downloader_1.0.0_x64_Portable.zip`**.
2. Extract the archive to any folder (e.g. `D:\Bulk-Video-Downloader` or a USB drive).
3. Run **`Bulk Video Downloader.exe`** directly.

---

## 🔒 Cryptographic Verification

You can verify the cryptographic integrity and authenticity of downloaded release binaries using Windows PowerShell:

```powershell
# Verify the Setup Installer
Get-FileHash -Algorithm SHA256 "Bulk-Video-Downloader_1.0.0_x64-setup.exe"

# Expected Output:
# 99B91976DB801D7269B07907B3EFDB43CA2D4248A0AC72038DCE32336CBF6BA2

# Verify the Portable Archive
Get-FileHash -Algorithm SHA256 "Bulk-Video-Downloader_1.0.0_x64_Portable.zip"

# Expected Output:
# C5707E8AC4218428C80B3987C2D91E9AE3C2944BE3E1311934BD1DB09AEE4726
```

---

## 💻 System Requirements

| Specification | Minimum Requirement | Recommended |
| :--- | :--- | :--- |
| **Operating System** | Windows 10 (Version 19041+) 64-bit | Windows 11 (64-bit) |
| **Processor** | Intel Core i3 / AMD Ryzen 3 or equivalent | Intel Core i5 / AMD Ryzen 5 or higher |
| **Memory (RAM)** | 4 GB RAM | 8 GB+ RAM (for simultaneous 4K streams) |
| **Storage** | 300 MB free disk space | High-speed SSD for storage |
| **Network** | Broadband internet connection | High-speed fiber connection |

---

## 👤 Author & Architectural Leadership

- **Lead Architect & Product Owner:** **Haseeb Kaloya**
- **GitHub Profile:** [https://github.com/HaseebKaloya](https://github.com/HaseebKaloya)
- **Official Inquiries & Support:** [contact.haseebkaloya@gmail.com](mailto:contact.haseebkaloya@gmail.com)

---

## ⚖️ Legal & Authorized Use Policy

*Bulk Video Downloader is engineered exclusively for authorized media workflows, educational preservation, personal backups of user-owned content, and open media. This software does not circumvent DRM, paywalls, or platform authentication mechanisms. Users remain fully responsible for complying with the intellectual property rights and terms of service of respective media hosts.*

---

© 2026 **Haseeb Kaloya**. Released under the [MIT License](LICENSE.txt).

# Installation Guide

Bulk Video Downloader is packaged in four official distribution formats tailored for individual workstations, enterprise networks, automated deployment pipelines, and portable flash storage.

---

## Package Comparison Matrix

| Package Name | Extension | Size | Engine Delivery | Target Environment |
| :--- | :--- | :--- | :--- | :--- |
| **Full Setup Installer** | `.exe` | ~77.9 MB | **Offline Bundled** (FFmpeg + yt-dlp included) | Standard workstation setups, offline systems |
| **Enterprise Windows Installer** | `.msi` | ~99.2 MB | **Offline Bundled** (Pre-compiled WiX payload) | Active Directory, GPO, SCCM, Intune |
| **Net Installer** | `.exe` | ~5.3 MB | **Online Streaming** (Downloaded via CDN on launch) | Fast initial download, bandwidth-constrained links |
| **Portable Standalone** | `.zip` | ~99.0 MB | **Offline Pre-extracted** (`bin/` directory included) | USB flash drives, restricted permission environments |

---

## Minimum System Requirements

- **Operating System**: Microsoft Windows 10 (64-bit, Version 2004 / Build 19041 or later) or Windows 11.
- **Architecture**: x86_64 (Intel / AMD 64-bit).
- **RAM**: 4 GB minimum (8 GB+ recommended for concurrent 4K multiplexing).
- **Disk Space**: 500 MB free space on target volume (additional storage required for downloaded media).
- **Runtime**: Microsoft Edge WebView2 Evergreen Runtime (pre-installed on Windows 10/11; bootstrapper bundled if missing).

---

## Installation Walkthroughs

### 1. Standard Desktop Setup (`.exe`)

The full setup installer is built using NSIS with solid LZMA compression:
1. Download `Bulk-Video-Downloader_1.0.0_x64-setup.exe` from the [Official Releases](https://github.com/HaseebKaloya/Bulk-Video-Downloader/releases).
2. Execute the installer. If Windows SmartScreen displays a verification notice, click **More Info** followed by **Run anyway**.
3. Choose the destination directory (default is `C:\Program Files\Bulk Video Downloader`).
4. The installer automatically registers start menu entries, generates desktop shortcuts, and registers uninstall hooks in Windows Settings.

### 2. Enterprise MSI Deployment (`.msi`)

System administrators can perform unattended, silent rollouts across domain workstations without user interaction:

#### Silent Install Command:
```cmd
msiexec /i "Bulk-Video-Downloader_1.0.0_x64.msi" /qn /norestart ALLUSERS=1
```

#### Custom Target Directory:
```cmd
msiexec /i "Bulk-Video-Downloader_1.0.0_x64.msi" /qn INSTALLDIR="D:\CustomApps\BulkVideoDownloader"
```

#### Silent Uninstallation:
```cmd
msiexec /x "Bulk-Video-Downloader_1.0.0_x64.msi" /qn /norestart
```

### 3. Lightweight Net Installer (`.exe`)

For users on metered connections or rapid provisioning environments:
1. Download `Bulk-Video-Downloader_1.0.0_x64_NetInstaller.exe` (~5.3 MB).
2. The setup installs the native core binary immediately.
3. On first execution, the application checks for the existence of `ffmpeg.exe` and `yt-dlp.exe` in `%LOCALAPPDATA%\com.haseebkaloya.bulk-video-downloader\bin` and automatically streams the latest updates over high-speed HTTPS.

### 4. Portable Standalone Archive (`.zip`)

For running directly from external storage or secondary hard drives without modifying the Windows registry:
1. Download `Bulk-Video-Downloader_1.0.0_x64_Portable.zip`.
2. Extract the archive contents into any local folder (e.g., `D:\Tools\BulkVideoDownloader`).
3. Ensure the extracted `bin/` directory remains adjacent to `Bulk Video Downloader.exe`.
4. Launch `Bulk Video Downloader.exe` directly.

---

## Verifying Checksums

Before launching any binary in sensitive corporate networks, verify package integrity against the official SHA-256 signatures:

```powershell
Get-FileHash -Algorithm SHA256 .\Bulk-Video-Downloader_1.0.0_x64-setup.exe
```

Compare the resulting hash with the official hash recorded in `checksums.txt`:

```text
b014e1cfdc175a420581b42d2eb3e4f06b66b2f8cb31dc92d431856c0a470b18  Bulk-Video-Downloader_1.0.0_x64-setup.exe
05c61493da5f6a40f66aac7b1d761d72506939fbeef22f7890f85130c14de9e7  Bulk-Video-Downloader_1.0.0_x64.msi
b53179ff47c1b29a0b0ba56b7e16bf66fcb343830dcf9ffa567149eee599a26e  Bulk-Video-Downloader_1.0.0_x64_NetInstaller.exe
1ace483750807d1f400127d6c293b724cc47854f007824c9b5723e214b1bdd68  Bulk-Video-Downloader_1.0.0_x64_Portable.zip
```

# Architecture & Data Persistence

Bulk Video Downloader is engineered for rock-solid stability, zero memory leaks, and complete resilience against application crashes, system reboots, and network drops.

---

## 1. System Component Stack

```text
┌───────────────────────────────────────────────────────────┐
│               Frontend Interface (Webview2)               │
│      React 19 • TypeScript • Zustand Store • Vanilla CSS  │
└─────────────────────────────┬─────────────────────────────┘
                              │
                    Tauri 2 IPC Channel
              (Type-Safe Asynchronous Commands)
                              │
┌─────────────────────────────▼─────────────────────────────┐
│                 Rust Core Runtime (Native)                 │
│                                                           │
│  ┌────────────────────────┐    ┌────────────────────────┐ │
│  │     Tokio Runtime      │    │  Isolated Subprocesses │ │
│  │ (Multi-Threaded Pool)  │    │  (yt-dlp & FFmpeg 7.1) │ │
│  └───────────┬────────────┘    └────────────────────────┘ │
│              │                                            │
│  ┌───────────▼────────────┐    ┌────────────────────────┐ │
│  │  Streaming Range Core  │    │   SQLite WAL Storage   │ │
│  │ (.bvd-partial Buffers) │    │  (ACID Task Machine)   │ │
│  └────────────────────────┘    └────────────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

### Component Roles

- **Frontend (TypeScript & React 19)**: Delivers 60 FPS UI rendering, real-time download velocity calculation, reactive queue sorting, and modal orchestration. Does not execute business logic; all heavy operations are delegated to the Rust backend.
- **Tauri 2 IPC Layer**: Replaces legacy Electron multi-process overhead. Memory footprint is typically under 80 MB RAM at idle.
- **Rust Backend**: Manages thread concurrency pools, stream network sockets, process supervision, and atomic file operations.

---

## 2. SQLite Write-Ahead Logging (WAL) Persistence

Task state, batch definitions, and system settings are stored locally in an embedded SQLite database located at:
`%LOCALAPPDATA%\com.haseebkaloya.bulk-video-downloader\app_state.db`

### Database Engine Configuration

During startup initialization, the database connection executes the following pragmas:

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA temp_store = MEMORY;
```

### Benefits of the WAL Engine
- **Concurrent Readers & Writers**: The UI can query the database thousands of times per second to refresh progress metrics while worker threads simultaneously commit progress updates without database locking.
- **Zero Corruption on Sudden Power Loss**: Writes append sequentially to the `.db-wal` file. In the event of a sudden power outage, uncommitted frames are automatically rolled back cleanly upon restart.

---

## 3. Safe Atomic Finalization (`.bvd-partial`)

To prevent corrupt or truncated video files from appearing in your destination folders:

1. **Isolation Phase**: When a download begins, data streams into an isolated temporary file named:
   `[filename].[task_id].bvd-partial`
2. **Resumption Tracking**: The file's byte length on disk is continuously reconciled with the server's `Content-Range` headers.
3. **Atomic Rename**: Only when 100% of bytes are transferred, verified, and (if applicable) multiplexed by FFmpeg, the `.bvd-partial` file is atomically renamed to its final output name (e.g., `video.mp4`).
4. **Resumable Cancellation**: If a user cancels a download, the application preserves the `.bvd-partial` file (configurable in Settings), allowing the user to resume later without losing downloaded data.

---

## 4. Startup Crash Recovery Machine

If Windows shuts down or the process is terminated unexpectedly during an active batch:

1. **Boot Reconciliation**: On startup, the `run_startup_recovery()` routine scans the database for tasks left in `DOWNLOADING` or `PREPARING` states.
2. **Filesystem Cross-Check**:
   - If the completed output file exists on disk and matches target size, the task status is promoted to `COMPLETED`.
   - If a valid `.bvd-partial` exists, the task state is reset to `QUEUED` with its downloaded byte offset intact.
3. **Integrity Pass**: Runs `PRAGMA integrity_check` to ensure the SQLite B-tree structure is 100% healthy.

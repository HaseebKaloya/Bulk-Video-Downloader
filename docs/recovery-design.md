# Bulk Video Downloader — Recovery Design

## 1. Objectives

The recovery subsystem guarantees that network drops, application crashes, operating system reboots, and sudden power cuts do not corrupt the database or discard downloaded bytes where HTTP range resumption is supported.

## 2. Temporary File & Finalization Strategy

1. **Active Stream Path:**
   - Files download to `<destination>/<filename>.bvd-partial`.
   - File locks are acquired during streaming to prevent concurrent process modification.
2. **Safe Atomic Finalization:**
   ```text
   Download to .bvd-partial
     -> Flush OS disk buffers (flush / sync_all)
     -> Check file size against expected total_bytes (if known)
     -> Atomic Rename (.bvd-partial -> destination file)
     -> Persist status = COMPLETED transactionally
   ```
3. **Crash Between Rename and DB Write:**
   - If the app terminates after rename but before the database marks `COMPLETED`, startup recovery detects that the destination file exists with matching size and reconciles the task as `COMPLETED`.

## 3. Startup Recovery Algorithm

Upon startup:
1. Open SQLite database, apply migrations, run `PRAGMA integrity_check`.
2. Query all tasks with status in `['PREPARING', 'DOWNLOADING', 'PAUSING', 'FINALIZING']`.
3. For each interrupted task:
   - Check if the target destination file already exists and is complete. If so, update DB status to `COMPLETED`.
   - Check if `<filename>.bvd-partial` exists on disk.
     - If yes and source supports Range requests, record bytes downloaded and set status to `PAUSED`.
     - If corrupted or source does not support resume, reset downloaded bytes to 0 and set status to `QUEUED`.
4. Reclaim stale worker leases.
5. Reconstruct the active batch and generate a `RecoverySummary` payload for the UI.

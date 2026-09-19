# Batch Download Operations

Bulk Video Downloader is engineered around a deterministic batch execution model. Rather than treating each URL as an isolated download, URLs are organized into managed batches with dedicated concurrency limits, priority weightings, and failure containment policies.

---

## 1. Importing URLs & Lists

The application provides three ingestion channels via the **Import URLs** modal:

1. **Direct Paste**: Paste multi-line raw text directly into the modal text area.
2. **File Ingestion**: Upload plain text (`.txt`) or comma-separated (`.csv`) files containing lists of video links.
3. **Clipboard Auto-Detection**: If a supported streaming URL is detected in the system clipboard when opening the application, an auto-import prompt is presented.

### Pre-Import Validation Engine

Before committing any batch to the SQLite database, the backend runs an automated verification pass:

- **Syntax & Schema Normalization**: Strips tracking query parameters (e.g., `?si=`, `?utm_source=`, `&feature=shared`), normalizes shortened domains (e.g., `youtu.be` to `youtube.com/watch?v=`, `vm.tiktok.com` to canonical video IDs).
- **Intra-Batch Duplicate Scrubbing**: Flags duplicate URLs within the submitted import payload.
- **Queue Collision Verification**: Checks the active and completed SQLite task tables to detect whether a requested URL was already downloaded or is currently active in another queue.

---

## 2. Batch Scheduler Policies

When creating or modifying a batch, you can select one of four completion policies governing how the engine reacts when tasks finish or encounter errors:

| Policy Identifier | Policy Name | Operational Behavior | Ideal Use Case |
| :--- | :--- | :--- | :--- |
| `TERMINAL_COMPLETION` | **Policy B: Terminal Completion (Default)** | The batch advances continuously. When all tasks reach a terminal state (`COMPLETED` or `FAILED`), the batch completes and triggers the next queued batch. | Production archival, overnight unattended scraping. |
| `ALL_SUCCESSFUL` | **Policy A: All Successful** | Requires 100% of all batch items to finish with `COMPLETED` status. If any task exhausts retries and fails, batch transition halts for user review. | Mission-critical video asset ingestion where every file is mandatory. |
| `PAUSE_ON_FAILURE` | **Policy C: Pause on Failure** | Immediately halts the entire batch queue upon the first encountered error. Unfinished tasks remain paused. | Unstable network conditions or investigating platform rate limits. |
| `CONTINUE_WITH_RETRY` | **Policy D: Continue with Retry** | Automatically isolates failed tasks into an exponential retry queue while allowing healthy streams to continue unimpeded. | Unreliable Wi-Fi connections, cellular tethering. |

---

## 3. Concurrency Limits & Bandwidth Allocation

- **Per-Batch Concurrency**: Configurable between **1 and 16 parallel worker threads**.
- **Global Concurrency Ceiling**: Prevents thread contention when multiple batches run concurrently. Default is 4 threads; high-bandwidth gigabit connections can safely scale to 8–12 threads.
- **Dynamic Speed Throttling**: Configure global bandwidth limits in **Settings > Network** to prevent Bulk Video Downloader from saturating office or residential internet bandwidth. Set to `0` for unconstrained maximum throughput.

---

## 4. Priority Queue Controls

Each queued task possesses a numerical priority weighting:
- **Move to Top / Priority Increment**: Moves urgent downloads to position #1 in the queue, activating workers as soon as the current stream chunk finishes.
- **Selective Pausing**: Pause individual streams without stalling the parent batch.
- **Batch Export**: Export batch status, output file paths, and metadata to JSON or CSV for external automation scripts.

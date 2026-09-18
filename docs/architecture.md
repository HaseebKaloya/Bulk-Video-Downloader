# Bulk Video Downloader — Architecture

## 1. System Topology

```text
React 19 Frontend (TypeScript + CSS Modules)
  │
  │ Tauri 2 IPC Commands & Typed Event Bus
  ▼
Backend Application Layer (Rust / Tokio)
  ├── Command Dispatcher (commands/mod.rs)
  ├── Typed Event Stream (events/mod.rs)
  │
  ├── Batch Scheduler (scheduler/mod.rs)
  │     ├── Batch Invariant Enforcer
  │     ├── Completion Policies (A: AllSuccessful, B: Terminal, C: PauseOnFailure, D: ContinueWithRetry)
  │     ├── Dynamic Concurrency Limiter
  │     └── Worker Pool Manager
  │
  ├── Download Engine (downloader/mod.rs)
  │     ├── HTTP Streaming Engine (reqwest)
  │     ├── Range Resume Handler (bytes=X-)
  │     ├── Safe Atomic Finalization (.bvd-partial -> Target)
  │     └── Rate Limiter & Throttled Progress Emitter (4-10 updates/sec)
  │
  ├── Persistence Layer (persistence/mod.rs)
  │     ├── SQLite Connection with WAL Mode
  │     ├── Schema Migration Runner
  │     └── Asynchronous DB Worker Actor
  │
  ├── Recovery Manager (recovery/mod.rs)
  │     ├── Startup Integrity Verification
  │     ├── Stale Lease Reclamation
  │     └── Filesystem Reconciliation
  │
  ├── Media Layer (media/mod.rs)
  │     ├── FFmpeg / FFprobe Subprocess Wrapper
  │     └── Media Prober & Thumbnail Generator
  │
  └── Provider Abstraction (providers/mod.rs)
        ├── MediaProvider Trait
        └── DirectHttpProvider & Extensible Adapters
```

## 2. Core Principles & Guarantees

1. **Backend as Source of Truth:** Frontend is a pure projection of the Rust backend database state.
2. **Deterministic State Transitions:** All status changes (e.g. `Queued` -> `Preparing` -> `Downloading` -> `Finalizing` -> `Completed`) are verified by backend invariants and persisted in transactions.
3. **Safe File Operations:** Media streams are written into unique temporary files (`<filename>.bvd-partial`). Writes are flushed and validated before an atomic rename to destination.
4. **Crash Recovery:** If execution terminates mid-transfer, startup recovery inspects the partial file and database state, reclaiming leases and presenting a `RecoverySummary` to the user.

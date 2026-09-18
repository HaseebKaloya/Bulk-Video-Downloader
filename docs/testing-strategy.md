# Bulk Video Downloader — Testing Strategy

## 1. Testing Pyramid

1. **Unit Tests (Rust):**
   - URL normalization and validation.
   - Duplicate detection algorithms.
   - Filename sanitization and template variable substitution.
   - Batch scheduler invariant checks (active count <= batch_size, worker count <= concurrency).
   - Retry exponential backoff calculation with jitter.
   - Error code mapping and classifications.
2. **Persistence & Integration Tests:**
   - SQLite migrations from clean state to current version.
   - Transactional state updates and rollback safety.
   - File streamer range resumption handling.
   - Atomic finalization (.bvd-partial -> final file).
   - Startup recovery scanner simulating crash states.
3. **Frontend Tests & Build Verification:**
   - Strict TypeScript compilation (`tsc --noEmit`).
   - Vite production bundle creation (`vite build`).
   - Virtualized list rendering stability under 1,000+ task rows.
   - Keyboard accessibility & tab focus flow.

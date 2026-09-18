# Bulk Video Downloader — Security Policy & Guidelines

## 1. Threat Model & Principles

1. **Least-Privilege Tauri Permissions:**
   - Filesystem operations are strictly bound to user-selected download directories and temporary cache paths.
   - Arbitrary shell executions from the frontend are prohibited.
2. **Path Traversal Prevention:**
   - All destination file paths derived from naming templates or server titles are sanitized:
     - Removal of control characters (`\0`, `\r`, `\n`).
     - Removal of path separators (`/`, `\`) and directory traversal sequences (`..`).
     - Replacement of Windows reserved characters (`<`, `>`, `:`, `"`, `/`, `\`, `|`, `?`, `*`).
     - Filename length clamped to 240 bytes to ensure headroom on standard filesystems.
3. **Log Sanitization & Secret Redaction:**
   - Any query parameters or headers containing authorization tokens, cookies, or secrets are redacted prior to tracing/logging.
4. **Controlled Subprocess Execution:**
   - FFmpeg invocations use explicit, array-based command arguments (`std::process::Command`), preventing shell injection vulnerabilities.

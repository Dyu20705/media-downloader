# Security Architecture & Hardening — openDownloader

## 1. Threat Model & Security Principles

openDownloader adheres to a strict principle of least privilege, defense-in-depth, and zero-trust input handling.

---

## 2. Core Security Controls

### 2.1 No Shell String Interpolation
- **Command Spawning**: All process invocations (`yt-dlp`, `FFmpeg`, `ffprobe`, `MediaInfo`) use vector argument arrays (`std::process::Command` / `tokio::process::Command`) rather than shell string execution (`sh -c` or `cmd /c`).
- **No Wildcard Execution**: Shell operators (`|`, `&`, `;`, `$`, `>`, `<`) in URLs or filenames cannot trigger shell execution.

### 2.2 Strict Input & URL Validation
- **URL Sanitation**: Validated via `validate_media_url()`:
  - Allowed schemes: Strictly `http://` and `https://`. Unsupported schemes (`file://`, `ftp://`, `gopher://`, `javascript:`) are rejected.
  - Reject NUL bytes (`\0`), ASCII control characters (`0x00–0x1F`), and newlines.
  - Length ceiling: Max 2048 characters.
  - Domain structure verification.
- **Path Sanitation**: Validated via `validate_and_ensure_directory()` and `sanitize_file_name()`:
  - Strips OS-reserved characters (`\ / : * ? " < > |`).
  - Rejects NUL bytes and control codes.
  - Enforces length caps (180 chars) to prevent Windows `MAX_PATH` overflow.

### 2.3 Strict Production Content Security Policy (CSP)
Defined in `src-tauri/tauri.conf.json`:
```text
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
font-src 'self' data:;
img-src 'self' data: https: blob:;
connect-src 'self' ipc: http://localhost:3000;
frame-src 'none';
object-src 'none';
```

### 2.4 Minimal Tauri Capabilities
Frontend capabilities are scoped strictly to required operations:
- `core:default`: Standard Tauri IPC invocation.
- `dialog:allow-open`: Frontend folder selection dialog. No opener plugin permission is granted.
- Arbitrary filesystem read/write and wildcard shell plugins are **disabled**.

### 2.5 Safe Diagnostics & Log Redaction
The in-memory ring buffer sanitizes all diagnostic log entries before retention. URL values are normalized to `https://example.com/[REDACTED]`: userinfo, signed path components, query parameters, and fragments are removed. Authorization, Cookie, and X-Api-Key header values are redacted. URL-bearing inputs are not included in resolver/analyzer log messages.
- Redacts authorization tokens (`Bearer [REDACTED]`, `Authorization: [REDACTED]`).
- Redacts session cookies (`Cookie: [REDACTED]`).
- Redacts API keys (`api_key=[REDACTED]`).
- Redacts signature tokens (`sig=[REDACTED]`, `signature=[REDACTED]`, `token=[REDACTED]`).
- Strict memory boundary: 256-line ring buffer capped at <= 64 KiB retained text.

### 2.6 Safe OS Folder / File Opening
Commands `open_folder` and `open_file` validate that the path:
1. Is non-empty, contains no NUL bytes, and no control codes.
2. Exists on the local filesystem.
3. Invokes OS file managers directly (`explorer /select,<path>` on Windows, `open` on macOS, `xdg-open` on Linux) without passing through `cmd.exe` or shell interpreters.

## Persisted history privacy

Download job snapshots are serialized to SQLite only after recursive URL sanitization. URL usernames, passwords, fragments, and non-allowlisted query values are removed. Only public YouTube video/playlist/time identifiers are retained to support an explicit retry; source URLs requiring other query parameters are redacted, so retry asks the user to paste the source again. The database is stored in the per-user local application-data directory and is not encrypted by this application.

## Executable trust and network boundary

Release builds resolve tools from an explicitly configured executable path, the application-local managed tool directory, or system `PATH`. Managed downloads are checked against the catalog SHA-256 before archive extraction and installation; extraction selects only the declared executable. Debug builds retain project-relative discovery for development. Release builds do not search the process working directory or repository tree. Explicit custom paths are executable code selected by the user and are validated by running the tool's version command.

URL parsing rejects non-HTTP(S) schemes, localhost names, private, loopback, link-local, unspecified, multicast, and listed special-use IP ranges. Before external-tool delegation, the app resolves the supplied host and rejects it if any returned address is prohibited. For the in-process TikTok fallback, every redirect destination is parsed and DNS-validated, and its validated addresses are pinned in the HTTP client for the connection.

The preflight check cannot constrain yt-dlp's later independent DNS resolution, redirects, or connections. Therefore private-network protection is not end-to-end for traffic delegated to yt-dlp. A network sandbox or proxy enforcing destination policy for child processes would be needed to provide that guarantee.

The current production release target is Linux x86_64. Its Debian artifact is authenticated with a dedicated GPG release key held only in the protected `production-release` environment: the workflow signs the exact `.deb`, signs `SHA256SUMS`, verifies both signatures, re-checks the package hash, and publishes the corresponding public key. GitHub provenance attestation is also produced for the staged package. Windows Authenticode and Apple Developer ID/notarization helpers are retained for future platform enablement but are not invoked by the current production release workflow. No signing material is stored in the repository.

## v1.0.0 upstream advisory exceptions

PR/scheduled and release audits share `scripts/ci/audit-rust.sh`, retaining `--deny warnings` and yanked-package checks. Core has no exceptions. Host accepts exactly RUSTSEC-2024-0370 (unmaintained build-time proc-macro-error) and RUSTSEC-2024-0429 (actual GLib runtime unsoundness on the Linux GTK path) for v1.0.0. See [the security disposition](release-remediation.md) for dependency paths, risk evidence, and re-evaluation requirements. These exceptions do not mean the dependency graph is advisory-free.

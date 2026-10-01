# openDownloader — Current State

> Updated: 2026-10-01
>
> Integration branch: `dev`. The current implementation includes durable SQLite history and crash recovery.

## Product flow

The application implements the single-media path:

```text
Paste URL → analyze → configure acquisition → review backend plan → execute
→ inspect and verify output → persisted history / receipt
```

Supported operations are Entire Media, Audio Only, Clip, Chapter, Thumbnail (JPEG), and Subtitles (one WebVTT language per job). Profiles are Best Source, Universal, Editing, and Small. Whole-playlist execution is unavailable. When a URL includes a playlist, “This video” remains the safe default; the executor uses explicit no-playlist behavior.

Requests beyond the active job enter a durable FIFO queue. Only one process executes at a time; queued items can be cancelled before execution. On restart, queued items are marked interrupted and require explicit retry. The Rust planner is authoritative. Before execution, the backend recomputes the plan against current settings and rejects a stale plan ID. Processes are invoked with argument vectors, never through a shell. Only one download process may run at a time.

## Persistence and recovery

The desktop stores download snapshots in SQLite under the platform's local application-data directory at `openDownloader/downloads.sqlite3`. Schema migrations use SQLite `user_version`; foreign keys, WAL journaling, a busy timeout, and full synchronous commits are enabled. Each job snapshot and its attempt lifecycle update are written transactionally.

Startup changes any queued or in-flight job to `INTERRUPTED` and records a user-facing explanation. It does not resume a process or infer success. The user can explicitly retry failed, interrupted, or cancelled records; retry replans against current settings and uses a new job ID. A source URL is retained only after removing credentials, fragments, and unapproved query values. YouTube's public video/playlist/time identifiers are retained; if a source cannot be safely reconstructed, the retry action asks the user to paste and review it again.

History is read from SQLite on startup. Progress remains process-local and is saved at job creation and terminal state, so an application crash can lose the last displayed progress value but not the existence or interruption state of a started job.

## Security and diagnostics

Diagnostics use a bounded in-memory buffer and sanitize URLs, credentials, cookies, and authorization/API-key headers. SQLite snapshots recursively sanitize URL values before serialization. Media URLs are validated as HTTP(S), and public-address/DNS preflight, output path validation, archive validation, and process-tree cancellation are implemented. External providers may follow redirects internally; redirect revalidation is outside this application's control.

## Validation

The tracked Vite config pins the development server to port 3000 with strict port behavior and configures React and Tailwind. Contract CI runs on pushes to `dev` and `main` and on pull requests targeting either branch. Shared Rust core checks run on Ubuntu, Windows, and macOS. The version-tagged release workflow builds Windows, macOS, and Linux packages, requires Windows/macOS signing credentials, signs the release checksum manifest, and publishes only tags that match the app version and point to dev history. It has not been exercised with real signing credentials. Linux package startup is smoke-tested in CI; Windows/macOS installed-app smoke coverage remains unverified.

Useful local checks:

```bash
npm ci --no-audit --fund=false
npm run check:ci
npm run lint
npm test
npm run build
cargo test --locked --manifest-path src-tauri/crates/core/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/crates/core/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

See [architecture](docs/architecture.md), [security](docs/security.md), [CI gates](docs/ci-security-gates.md), and [packaging](docs/packaging.md) for details and current limits.

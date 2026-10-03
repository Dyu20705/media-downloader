# CI and security gates

These gates establish a production-oriented engineering baseline and reduce specific classes of regressions. Passing them does not prove that the application is secure, reliable at scale, or ready for a particular release.

## Pull-request blocking checks

`.github/workflows/contracts.yml` runs for pull requests targeting `dev` or `main`, and direct pushes to either integration branch. Repository permissions are read-only:

| Check | Risk or regression it detects | Expected cost |
| --- | --- | --- |
| Rust format, tests, and Clippy on Ubuntu, Windows, and macOS | Rust syntax/style drift, test regressions, and OS-specific compilation/lint failures in the core | 8–15 minutes per OS; runs in parallel |
| Frontend typecheck, ESLint, tests, production build, and `npm audit --omit=dev --audit-level=high` | UI/type regressions, lint findings, build breakage, and high/critical advisories affecting shipped npm dependencies; moderate advisories do not block | 3–7 minutes |
| `npm run check:ci` | Common frontend process/shell imports, direct fetch/XHR/WebSocket APIs, or raw-HTML sinks; Tauri/Vite port mismatch; missing Tailwind Vite integration; weakened script/frame/object CSP; broadened shell/filesystem capability | Usually under 1 minute |
| Tauri host `cargo check`, strict Clippy, and offline-capable integration tests on Ubuntu | Native host/plugin/config compilation, lint, and host-boundary regressions not exercised by core-only tests | 8–18 minutes, subject to Rust cache |
| `actionlint` and offline `zizmor` | Invalid workflow syntax/expressions, mutable action references, unsafe workflow patterns, and selected privilege/input hazards | 2–5 minutes including pinned scanner setup |
| Gitleaks | Secret-like material in changed commits; on a new branch without a usable base commit, scans the checked-out files. Output is redacted. | Included with workflow checks |

Third-party actions are referenced by immutable commit SHA. Dependabot proposes reviewed updates to those pins and the locked Cargo/npm dependencies. CI uses `npm ci`, Cargo `--locked`, and fixed Rust/Node/Python/Go tool versions. Media utilities installed for cross-platform fixtures come from the hosted runner's package repositories, so those OS package snapshots are not fully reproducible.

The npm install-script allowlist is intentionally narrow: only `esbuild@0.25.12` is approved, pinned to the exact lockfile version because its install hook selects and validates the platform binary needed by Vite. No blanket lifecycle-script approval is enabled.

## Scheduled/advisory checks

`.github/workflows/security.yml` runs weekly, on pushes to `dev` or `main`, and manually:

- CodeQL analyzes Rust and TypeScript and publishes code-scanning results. Findings require human triage; this is not a PR merge gate.
- `cargo-audit` checks both committed Rust lockfiles against the changing RustSec advisory database. A finding fails the security workflow on every dev/main push and scheduled run.

The contract workflow runs on direct pushes to `dev` and `main`, and pull requests targeting either branch; feature-branch pushes do not trigger a duplicate full suite.

The PR npm audit is deliberately limited to installed production dependencies and high/critical severity. It uses the live npm advisory database, so a fresh advisory can change the result without a source change. Lower-severity and development-only findings remain visible through dependency update/review work rather than making this deterministic gate noisy.

The current RustSec check also reports a transitive `glib 0.18.5` unsoundness advisory (`RUSTSEC-2024-0429`) and several unmaintained/yanked-crate warnings. The glib advisory concerns `VariantStrIter` methods; the application does not call those methods directly, and the Tauri GTK dependency graph currently constrains glib below the fixed `0.20.0` line. These warnings are visible but do not fail `cargo audit` by default. Revisit them when the Tauri GTK stack changes; do not add a blanket advisory ignore.

## Architecture and threat boundaries

The checked boundary is:

```text
React UI → Tauri IPC → Rust core planner/execution → yt-dlp / FFmpeg / FFprobe / MediaInfo → filesystem
```

The backend remains the authority for URL validation, canonical acquisition planning, process arguments, execution, and artifact verification. The TypeScript AST check blocks common direct process/shell imports and browser fetch/XHR/WebSocket APIs from bypassing that IPC path; it is an invariant check, not a proof about arbitrary third-party packages. Structural configuration checks keep the Tauri development URL, strict Vite port, Tailwind Vite plugin, CSP essentials, and narrow capabilities aligned.

The existing Rust regression suite supplies behavior-level checks for invalid/local URLs, archive traversal, checksums, plan freshness, cancellation/concurrent-job rejection, verification failures, and diagnostics redaction. CI runs those tests; it does not substitute static pattern bans for structured `Command::args` calls. The one real-YouTube smoke test is marked ignored by default so it cannot make PR CI depend on provider availability; run it deliberately with `cargo test --manifest-path src-tauri/Cargo.toml -- --ignored test_real_ytdlp_metadata_analysis_integration` when a live smoke check is useful.

Prompt injection is currently outside the repository's runtime threat model. The application has no LLM calls, prompts, AI agents, or model-generated actions.

## Reproduce locally

From the repository root:

```sh
cargo fmt --manifest-path src-tauri/crates/core/Cargo.toml --check
cargo test --locked --manifest-path src-tauri/crates/core/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/crates/core/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm ci --no-audit --fund=false
npm run check:ci
npm audit --omit=dev --audit-level=high
npm run lint
npm test
npm run build
cargo check --locked --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

Core tests require `ffmpeg`, `ffprobe`, and pinned `yt-dlp` `2026.8.19` on `PATH`. The Tauri host check requires the platform prerequisites from the [Tauri v2 setup guide](https://v2.tauri.app/start/prerequisites/). Workflow checks require `actionlint`, `zizmor`, and Gitleaks; CI pins their versions.

## What these checks do not prove

- In-process TikTok HTTP fallback requests validate each redirect, reject prohibited destinations, resolve every hop, and pin the validated addresses for its connection. URL preflight also rejects private and special-use IP addresses.
- yt-dlp receives a URL after preflight validation, then performs its own DNS lookups, redirects, and media requests outside the application network boundary. The application cannot guarantee an end-to-end private-network block for that delegated traffic; that limitation remains for human review.
- The gates do not test live provider availability, provider anti-bot changes, or media behavior against public sites; no live-provider test blocks a PR.
- The Linux package gate builds and launches a Debian artifact in a headless display. Windows/macOS CI packages are intentionally unsigned portability checks and are not production artifacts. Stable release tags must point into `main`; the release workflow builds one Linux x86_64 Debian package, launches that exact payload under Xvfb, attests its provenance, signs both the package and SHA256SUMS with the protected release GPG key, verifies both signatures, and publishes those same staged bytes. Manual `workflow_dispatch` is a fast GPG credential preflight only and never publishes.
- External media tools run outside the process boundary. Tool download checksums and archive-path tests reduce install risk, but CI does not prove upstream tool publishers or the checksum catalog are uncompromised.
- The current local RustSec audit reported a yanked `chacha20 0.10.1` pin in the core lockfile; unmaintained transitive `proc-macro-error 1.0.4` and `unic-* 0.9.0` crates; and unsound `glib 0.18.5` (`RUSTSEC-2024-0429`) in the Tauri host dependency graph. The GTK/glib path is host UI code; the iterator-specific reachability from app code is not established. The other findings are maintenance/build graph warnings rather than reported exploitable advisories. The release workflow denies all audit warnings, so the exact tag gate will fail until these are upgraded with regression evidence or explicitly dispositioned by an authorized human; no advisories are ignored.
- Process execution is limited to one active download; additional submissions enter a durable FIFO queue. Diagnostics retention is bounded. Subprocess output is drained continuously through a 128-message bounded queue; full queues drop diagnostic events and individual lines over 8 KiB are discarded.
- SQLite migration, privacy filtering, crash-state recovery, queue sequencing/cancellation, and explicit retry paths have deterministic core tests. Full UI-driven provider analysis/download and process-kill/relaunch scenarios still need broader end-to-end coverage.
- The production release workflow runs the frontend, binding, Rust tests, formatting, Clippy, dependency audits, Debian packaging, exact-package launch smoke, provenance attestation, checksum verification, and GPG verification against the exact Linux release tag SHA. Windows/macOS production signing/notarization and installed-app smoke coverage are deferred future work; the existing unsigned CI package jobs only protect portability.

No cargo-deny license allowlist is added here: the dependency metadata includes multiple alternative and file-level license expressions, and a durable third-party redistribution policy should be explicitly maintained rather than inferred from a scanner default. The repository's application license and actual locked dependency license expressions were inspected for this decision.

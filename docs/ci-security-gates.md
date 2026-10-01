# CI and security gates

These gates establish a production-oriented engineering baseline and reduce specific classes of regressions. Passing them does not prove that the application is secure, reliable at scale, or ready for a particular release.

## Pull-request blocking checks

`.github/workflows/contracts.yml` runs for pull requests targeting `dev` or `main`, and pushes to `main`, avoiding duplicate feature-branch push/PR runs. Repository permissions are read-only:

| Check | Risk or regression it detects | Expected cost |
| --- | --- | --- |
| Rust format, tests, and Clippy on Ubuntu, Windows, and macOS | Rust syntax/style drift, test regressions, and OS-specific compilation/lint failures in the core | 8–15 minutes per OS; runs in parallel |
| Frontend TypeScript, tests, production build, and `npm audit --omit=dev --audit-level=high` | UI/type regressions, build breakage, and high/critical advisories affecting shipped npm dependencies; moderate advisories do not block | 3–7 minutes |
| `npm run check:ci` | Common frontend process/shell imports, direct fetch/XHR/WebSocket APIs, or raw-HTML sinks; Tauri/Vite port mismatch; missing Tailwind Vite integration; weakened script/frame/object CSP; broadened shell/filesystem capability | Usually under 1 minute |
| Tauri host `cargo check`, strict Clippy, and offline-capable integration tests on Ubuntu | Native host/plugin/config compilation, lint, and host-boundary regressions not exercised by core-only tests | 8–18 minutes, subject to Rust cache |
| `actionlint` and offline `zizmor` | Invalid workflow syntax/expressions, mutable action references, unsafe workflow patterns, and selected privilege/input hazards | 2–5 minutes including pinned scanner setup |
| Gitleaks | Secret-like material in changed commits; on a new branch without a usable base commit, scans the checked-out files. Output is redacted. | Included with workflow checks |

Third-party actions are referenced by immutable commit SHA. Dependabot proposes reviewed updates to those pins and the locked Cargo/npm dependencies. CI uses `npm ci`, Cargo `--locked`, and fixed Rust/Node/Python/Go tool versions. Media utilities installed for cross-platform fixtures come from the hosted runner's package repositories, so those OS package snapshots are not fully reproducible.

The npm install-script allowlist is intentionally narrow: only `esbuild@0.25.12` is approved, pinned to the exact lockfile version because its install hook selects and validates the platform binary needed by Vite. No blanket lifecycle-script approval is enabled.

## Scheduled/advisory checks

`.github/workflows/security.yml` runs weekly, on pushes to `main`, and manually:

- CodeQL analyzes Rust and TypeScript and publishes code-scanning results. Findings require human triage; this is not a PR merge gate.
- `cargo-audit` checks both committed Rust lockfiles against the changing RustSec advisory database. A finding makes the scheduled run fail but does not automatically block unrelated PRs.

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

Core tests require `ffmpeg`, `ffprobe`, and pinned `yt-dlp` `2025.2.19` on `PATH`. The Tauri host check requires the platform prerequisites from the [Tauri v2 setup guide](https://v2.tauri.app/start/prerequisites/). Workflow checks require `actionlint`, `zizmor`, and Gitleaks; CI pins their versions.

## What these checks do not prove

- URL validation runs before external tools. Redirects/DNS lookups performed internally by yt-dlp cannot be revalidated by the application at every hop.
- The gates do not test live provider availability, provider anti-bot changes, or media behavior against public sites; no live-provider test blocks a PR.
- The current tests do not package installers or exercise installed WebViews/OS permissions end-to-end. Signing, notarization, SBOM/provenance, and release artifact inspection require a separately defined release/signing policy.
- External media tools run outside the process boundary. Tool download checksums and archive-path tests reduce install risk, but CI does not prove upstream tool publishers or the checksum catalog are uncompromised.
- Process admission is limited to one active download and diagnostics retention is bounded. The subprocess output path currently uses unbounded line channels; limiting noisy/oversized child output remains a production reliability improvement not enforced by this gate.
- The SQLite schema and recovery primitives are covered by deterministic core tests; live user-driven retry and restart behavior still need broader end-to-end coverage.
- Linux CI builds the Debian package, inspects its bundled executable/desktop entry, and launches it under a virtual display. Windows/macOS installer launch smoke coverage and signed release validation remain outstanding.

No cargo-deny license allowlist is added here: the dependency metadata includes multiple alternative and file-level license expressions, and a durable third-party redistribution policy should be explicitly maintained rather than inferred from a scanner default. The repository's application license and actual locked dependency license expressions were inspected for this decision.

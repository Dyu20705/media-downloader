# v1.0.0 final security disposition

The final engineering pass starts from `faee83087709800a5f71dbc84a41e4c4f9120edd` on `release/1.0.0-production-hardening`, confirmed against the fetched remote and PR #6. The owner explicitly accepted the two upstream advisory exceptions below for v1.0.0. Engineering validation is separate from final human acceptance, merge, tag creation, and signed release verification.

## RustSec disposition

PR/scheduled and release checks invoke the same repository-owned `scripts/ci/audit-rust.sh` with cargo-audit 0.22.2. It uses `set -euo pipefail`, retains `--deny warnings`, and leaves yanked-package checks enabled. Core has zero exceptions. Host ignores exactly RUSTSEC-2024-0370 and RUSTSEC-2024-0429; every other warning/advisory remains denied. No framework migration, vendored relabelling, lockfile hand-edit, or broad ignore is used.

**PASS with two documented v1.0.0 upstream advisory exceptions** is the accepted audit disposition. These exceptions do not mean the dependency graph is advisory-free.

### RUSTSEC-2024-0370 — proc-macro-error 1.0.4

This is an informational/unmaintained advisory with no patched version. The current inverse dependency tree leads through `tauri 2.11.5 → gtk 0.18.2 → gtk3-macros 0.18.2 → proc-macro-error`, and through `gtk → glib 0.18.5 → glib-macros 0.18.5 → proc-macro-error`. It is a transitive GTK/glib procedural-macro build-time dependency rather than application runtime functionality. Replacement requires upstream GTK/glib macro migration. Its maintenance risk is accepted for v1.0.0 only.

### RUSTSEC-2024-0429 — glib 0.18.5

This is an actual unsoundness advisory affecting `VariantStrIter` iteration, with undefined behavior and possible optimized-build null dereferences. The Linux-only application runtime dependency comes through the Tauri GTK3/WebKitGTK stack: `tauri 2.11.5 → gtk 0.18.2 → glib`, and `tauri → tauri-runtime-wry 2.11.4 → wry 0.55.1 → webkit2gtk 2.0.2 → glib`. Windows/macOS do not compile the Linux GTK runtime path.

The upstream fix starts at glib 0.20.0. Inspection of the current official Cargo registry metadata and published manifest confirms the latest stable compatible Tauri 2.12.1 still requires GTK 0.18 and WebKitGTK 2; GTK 0.18 requires glib 0.18. A compatible stable Tauri upgrade therefore does not eliminate either accepted advisory. A direct glib 0.20 dependency would coexist with the affected version rather than replace it.

Application/static source inspection found no direct use of `VariantStrIter` or `array_iter_str` in the application core/host or the inspected Tauri, GTK, GDK, GIO, Tao, and Wry sources. This is risk reduction evidence, NOT proof of complete unreachability. The Linux runtime still includes the affected library. This residual risk is accepted for v1.0.0 rather than introducing an unstable framework migration immediately before release. Any Linux smoke-test failure attributable to this GLib behavior invalidates the waiver and blocks release.

Any future openDownloader release must re-evaluate and remove these waivers if the upstream Tauri/GTK dependency graph permits it.

Sources: [GLib advisory and fixed versions](https://rustsec.org/advisories/RUSTSEC-2024-0429), [proc-macro-error advisory](https://rustsec.org/advisories/RUSTSEC-2024-0370), [official Tauri registry metadata](https://index.crates.io/ta/ur/tauri), [Tauri 2.12.1 dependencies](https://docs.rs/crate/tauri/2.12.1/source/Cargo.toml), [Tauri upstream GLib upgrade discussion](https://github.com/tauri-apps/tauri/issues/12564).

### Prior advisory remediation retained

Both lockfiles retain chacha20 0.10.2, replacing yanked 0.10.1 in the optional HTTP/3 graph. The compatible tauri-utils 2.10.1 / urlpattern 0.6.0 update removed the previous unic advisories RUSTSEC-2025-0081, RUSTSEC-2025-0075, RUSTSEC-2025-0080, RUSTSEC-2025-0100, and RUSTSEC-2025-0098. No additional advisory exception is accepted.

## Other security observations

The baseline's two CodeQL CSP findings point at substring-based Google Fonts exclusion checks. The preserved CSP commit replaces them with exact directive allowlists. Exact-candidate hosted CodeQL reanalysis is required; an older check result cannot validate the final candidate.

Production npm audit reports zero vulnerabilities. Full npm audit reports the moderate development-only `GHSA-82fw-gwwq-j7x9` in Vitest / @vitest/mocker 3.2.7. The workflow uses non-serving `vitest run`; these dependencies are absent from the production bundle. The published fix requires Vitest >=4.1.11, a separate major upgrade. No forced unrelated test-framework upgrade or audit suppression was applied.

MediaInfo hashes, official URLs, verified Windows layout, and reduced macOS managed support are recorded in [tool-management.md](tool-management.md). The Windows executable's version command has not been run on this Linux host.

## Local validation

Node 24.21.0 and Rust 1.96.0 match the workflow versions. Checks below record local validation for this final pass; exact-candidate hosted results must be reviewed separately.

| Gate | Result | Evidence |
|---|---|---|
| Clean dependency installation | PASS | `npm ci` |
| Frontend boundary check, TypeScript, ESLint | PASS | `check:ci`, `typecheck`, `lint` |
| Frontend and release-note tests | PASS | 15 Vitest tests; four release-note extraction test groups |
| Generated bindings | PASS | Regeneration leaves `src/generated/ipc.ts` unchanged |
| Production frontend build | PASS | Vite production bundle |
| Production npm audit | PASS | Zero vulnerabilities with `--omit=dev --audit-level=high` |
| Rust core tests | PASS | 70 unit, 3 execution contract, 7 integration, 11 regression, 9 tool-manager tests |
| Rust host tests | PASS | 2 unit, 5 integration; existing manual provider test remains ignored |
| Formatting | PASS | Both manifests, all crates |
| Clippy | PASS | Both manifests, all targets, `-D warnings`; core release profile also passes |
| Host check and metadata | PASS | Locked cargo check and no-dependency metadata |
| Strict core audit | PASS | No exceptions; warning denial and yanked-package checks retained |
| Host audit | PASS with two documented v1.0.0 upstream advisory exceptions | Only RUSTSEC-2024-0370 and RUSTSEC-2024-0429; all other findings denied |
| actionlint, workflow structure, ShellCheck | PASS | Pinned actionlint 1.7.12; exact-SHA dependency and package matrix checks; all repository shell scripts |
| zizmor | PASS | CI version 1.30.1, offline, default configuration; no new suppressions |
| Gitleaks | PASS | CI version 8.30.1: base-to-head commit scan plus all tracked working-tree files and new source files |
| MediaInfo Windows artifact pin | PASS | Fresh official download matches the catalog SHA-256; root executable inspection, archive and checksum regressions |
| Tool-manager and URL security | PASS | Tampering, traversal, executable planting, cancellation, bounded output, private destinations and redirect regressions |
| Windows signing command contract | PASS | PowerShell 7.5.0 with mocked native signing and signature inspection; literal special-character paths and rejection on signing/verification failure, invalid status, signer mismatch, invalid thumbprint, missing file |
| Artifact, checksum, and GPG contracts | PASS | Actual workflow shell rejects zero/two artifacts and accepts one; preserves special-character paths; rejects tampered/missing files. Actual manifest signing/verification passes with an ephemeral test key and rejects a tampered manifest. |
| Linux package build and inspection | PASS | Debian `open-downloader` 1.0.0 amd64; `Name=openDownloader`, executable `opendownloader`, desktop entry and three icons; GTK3/WebKitGTK dependencies; no development files or extra executables |
| Linux package launch | PASS | Extracted Debian executable stays alive for 15 seconds under temporary Xvfb; software-display DRI3 warnings only |
| Naming | PASS | Zero obsolete-brand matches; lowercase matches are machine identifiers |
| Hosted CI, CodeQL, Windows/macOS | REQUIRED after the single push | Older SHA results are not final-candidate evidence |

Source and base-to-head history scans and the extracted Debian payload scan are clean. No new Gitleaks exclusion or allowlist was added. Local build output is not part of the source scan or release package.

Earlier remediation of the development-only helper cfg and Unix test-only constructor is retained. Strict release Clippy and the optimized production-resolution regression pass again. Release quality retains FFmpeg installation so real transformation tests run; FFmpeg, ffprobe, and yt-dlp are also available for local tests.

Release semantics reviewed: version/tag/main-ancestry validation, exact-SHA quality dependencies, all three package targets, protected signing environment, Windows Authenticode/Apple signing and notarization verification, provenance attestation, signed checksum verification, exact release-note extraction, and tag-only publishing remain enabled. Owner-controlled signing and final human approval remain required. The documented v1.0.0 advisory exceptions do not waive any signing, quality, or platform gate.

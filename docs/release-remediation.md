# v1.0.0 remediation status

Release is blocked. No remediation commit or push is permitted while the strict host audit fails. No advisory ignores or platform exclusions were added to Rust auditing. This document describes a local working tree, not an accepted release candidate.

The starting commit was `9c82e6c77b550ecdf60c8ce8350816997ac54559` on `release/1.0.0-production-hardening`. During inspection, another actor committed `12b353a832b8fa3cb9773496848567923e9958b5` (the CSP allowlist fix). That commit is preserved as the current baseline. This remediation performed zero pushes, merges, tags, or publications.

## RustSec disposition

Audits use cargo-audit 0.22.2 and `--deny warnings`, with the refreshed official RustSec database (1279 advisories). Both lockfiles were audited after targeted Cargo updates.

| Finding | Baseline path | Local disposition |
|---|---|---|
| Yanked `chacha20 0.10.1` (no RustSec ID) | core → reqwest 0.12.28 → quinn 0.11.11 → quinn-proto 0.11.17 → rand 0.10.2 → chacha20 | Updated to 0.10.2 in both lockfiles. The inverse tree has no active path with the application's selected features, even with `--target all`; the path is present in the lockfile's optional HTTP/3 graph. |
| RUSTSEC-2025-0081, `unic-char-property` | tauri-utils 2.9.3 → urlpattern 0.3.0 → unic-ucd-ident 0.9.0 → unic-char-property 0.9.0 | Removed by compatible tauri-utils 2.10.1 / urlpattern 0.6.0 update. |
| RUSTSEC-2025-0075, `unic-char-range` | previous path → unic-char-range 0.9.0 | Removed. |
| RUSTSEC-2025-0080, `unic-common` | tauri-utils → urlpattern → unic-ucd-ident → unic-ucd-version → unic-common 0.9.0 | Removed. |
| RUSTSEC-2025-0100, `unic-ucd-ident` | tauri-utils → urlpattern → unic-ucd-ident 0.9.0 | Removed. |
| RUSTSEC-2025-0098, `unic-ucd-version` | tauri-utils → urlpattern → unic-ucd-ident → unic-ucd-version 0.9.0 | Removed. |
| RUSTSEC-2024-0370, `proc-macro-error 1.0.4` (unmaintained) | tauri 2.11.5 → gtk 0.18.2 → gtk3-macros 0.18.2 → proc-macro-error; also gtk → glib 0.18.5 → glib-macros 0.18.5 → proc-macro-error | **BLOCKER.** Build-time procedural macro dependency. No compatible replacement is offered by these GTK/glib macro versions. Maintenance risk remains even though the macro crate is not shipped as executable runtime code. |
| RUSTSEC-2024-0429, `glib 0.18.5` (unsound) | tauri 2.11.5 → gtk 0.18.2 → glib; also tauri → tauri-runtime-wry 2.11.4 → wry 0.55.1 → webkit2gtk 2.0.2 → glib | **BLOCKER.** Linux desktop runtime dependency. Upstream fix starts at glib 0.20.0, outside GTK 0.18's allowed range. |

The GLib advisory affects `VariantStrIter` iteration, producing undefined behavior and possible optimized-build null dereferences. A source search of application core/host, GTK, GDK, GIO, Tauri, Tao, and Wry found no direct call to `VariantStrIter` or `array_iter_str`. This is limited static evidence, not a proof of unreachability across the entire runtime. The affected library is linked on Linux; release remains blocked regardless of apparent reachability.

`cargo update -p glib --dry-run` selects no new compatible package. `glib 0.18.6` is absent from the registry. Inspection of the published Tauri 2.12.1 manifest confirms it still requires GTK 0.18 and WebKitGTK 2. A direct GLib 0.20 dependency would coexist with, rather than replace, the affected GLib 0.18. A safe resolution needs an upstream-compatible GTK/WebKit/Tauri dependency migration, with native regression validation. A framework alpha migration or relabelled vendored crate is not a release-hardening fix.

Sources: [GLib advisory and fixed versions](https://rustsec.org/advisories/RUSTSEC-2024-0429), [proc-macro-error advisory](https://rustsec.org/advisories/RUSTSEC-2024-0370), [Tauri upstream GLib upgrade discussion](https://github.com/tauri-apps/tauri/issues/12564).

Final local strict audit: core **PASS**; host **FAIL**, exactly the two blockers above. The scheduled/PR audit now denies warnings just as release does.

## Other security observations

The baseline's two CodeQL CSP findings point at substring-based Google Fonts exclusion checks. The preserved CSP commit replaces them with exact directive allowlists. They need hosted CodeQL reanalysis; an older check result cannot validate this local tree. No hosted analysis is claimed for these uncommitted changes.

Production npm audit reports zero vulnerabilities. Full npm audit reports the moderate development-only `GHSA-82fw-gwwq-j7x9` in Vitest / @vitest/mocker 3.2.7. The workflow uses non-serving `vitest run`; these dependencies are absent from the production bundle. The published fix requires Vitest >=4.1.11, a separate major upgrade. No forced unrelated test-framework upgrade or audit suppression was applied.

MediaInfo hashes, official URLs, verified Windows layout, and reduced macOS managed support are recorded in [tool-management.md](tool-management.md). The Windows executable's version command has not been run on this Linux host.

## Local validation

Node 24.21.0 and Rust 1.96.0 match the workflow versions. Checks below refer to the uncommitted remediation tree, not a pushed SHA.

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
| Strict core audit | PASS | No warnings |
| Strict host audit | FAIL | RUSTSEC-2024-0370 and RUSTSEC-2024-0429 |
| actionlint, workflow structure, ShellCheck | PASS | Pinned actionlint 1.7.12; exact-SHA dependency and package matrix checks; all repository shell scripts |
| zizmor | PASS | CI version 1.30.1, offline, default configuration; no new suppressions |
| Gitleaks | PASS | CI version 8.30.1: base-to-head commit scan plus all tracked working-tree files and new source files |
| MediaInfo Windows artifact pin | PASS | Official download, calculated SHA-256, root executable inspection, PE import inspection, archive and checksum regressions |
| Tool-manager and URL security | PASS | Tampering, traversal, executable planting, cancellation, bounded output, private destinations and redirect regressions |
| Windows signing command contract | PASS | PowerShell 7.5.0 with mocked native signing and signature inspection; literal special-character paths and rejection on signing/verification failure, invalid status, signer mismatch, invalid thumbprint, missing file |
| Artifact and checksum contracts | PASS | Executed actual workflow shell: zero/two artifacts rejected, one accepted; spaces and special characters preserved; valid manifest accepted, tampered/missing files rejected |
| Linux package build and inspection | PASS | Debian `open-downloader` 1.0.0 amd64; `Name=openDownloader`, executable `opendownloader`, desktop entry and three icons; GTK3/WebKitGTK dependencies; no development files or extra executables |
| Linux package launch | PASS | Extracted Debian executable stays alive for 15 seconds under temporary Xvfb; software-display DRI3 warnings only |
| Naming | PASS | Zero obsolete-brand matches; lowercase matches are machine identifiers |
| Hosted CI, CodeQL, Windows/macOS for this tree | NOT RUN | No remediation commit/push; older SHA results are not candidate evidence |

The full build-directory Gitleaks scan reports nine false positives: seven Rust metadata files embed Muda accelerator documentation (`shift+alt+KeyQ`), and two old AppImage GNOME schemas contain long setting key names. Source inspection confirms these are not credentials. No exclusion or allowlist was added. The Debian payload secret scan is clean.

Release-profile validation also exposed a development-only helper warning and a Unix test-only constructor unavailable in optimized tests. The helper now shares the development discovery cfg; the test constructor compiles only for Unix tests, with development fields initialized only in debug builds. Strict release Clippy and optimized production-resolution regression pass. Release quality now installs FFmpeg so real transformation tests are exercised rather than silently skipped for missing fixtures.

Release semantics reviewed: version/tag/main-ancestry validation, exact-SHA quality dependencies, all three package targets, protected signing environment, Windows Authenticode/Apple signing and notarization verification, provenance attestation, signed checksum verification, exact release-note extraction, and tag-only publishing remain enabled. Owner-controlled signing and final human approval remain separate from the unresolved engineering blockers.

# Media Downloader — Current State

> Snapshot: 2026-09-21
> Branch: `fix/runtime-hardening-and-current-state`
> Baseline commit: `c1cb7682f87590e92b23b1945f890bb51b4016d1` (`p0.0`, 2026-09-20)
> Source of truth for the target product: [`design-master.md`](design-master.md)

## 1. Executive summary

The repository is a build-green Tauri desktop application with a working single-media download flow:

```text
URL
  -> resolve/analyze with yt-dlp
  -> choose a legacy preset and quality
  -> start one in-memory download job
  -> poll progress and allow cancellation
  -> locate the final artifact
  -> inspect with ffprobe
  -> produce verification, fingerprint, recipe, and completion receipt
```

The current product is a reliable development baseline, not the completed product described by `design-master.md`. Slice 0 (Baseline Integrity) is complete. Parts of quality transparency and platform hardening exist, but the canonical `AcquisitionPlan`, persistent queue/history, structured recovery, playlist workflow, operation model, and finishing tools are not implemented end to end.

This branch contains runtime-hardening changes beyond the baseline commit. This document describes the branch state rather than an uncommitted local working tree.

## 2. What works now

### User flow

- Paste and analyze an HTTP/HTTPS media URL.
- Display source metadata, available resolutions/frame rates, formats, subtitles, chapters, and a recommendation when the extractor supplies them.
- Choose one of five preset-driven outputs:
  - MP4 Compatible;
  - Best Video;
  - Best Audio;
  - MP3;
  - FLAC.
- Choose a quality and output directory.
- Preview the generated yt-dlp command.
- Run one download at a time, show progress/speed/ETA, and cancel it.
- Open the completed file or its containing folder.
- Inspect actual container, codec, dimensions, frame rate, bitrate, streams, and verification details.
- View an in-session download history, diagnostics, application settings, and tool status.

### Backend capabilities

- Universal resolver using yt-dlp, with direct-media/TikTok fallback paths.
- Source metadata normalization into `MediaMetadata` and `SourceMediaGraph` domain types.
- Backend recommendation engine and processing-cost vocabulary.
- Preset compilation for yt-dlp/FFmpeg arguments.
- Async subprocess execution without shell command interpolation.
- Process-tree termination on cancellation.
- Single-active-job enforcement.
- Post-download inspection and verification.
- Fingerprint, explainable result, and reproducible recipe generation after download.
- App-local settings persistence and managed-tool manifest persistence.
- Managed/custom resolution for yt-dlp, FFmpeg, FFprobe, and MediaInfo.
- Bounded, redacted in-memory diagnostics.

### Current hardening in the working tree

- Final output discovery prefers a machine-readable `after_move` marker and parses yt-dlp destination output before generic progress lines.
- Successful completion requires deep verification. Missing ffprobe yields only `BASIC_INSPECTION`; an ffprobe execution/parsing failure is not reported as verified success.
- The UI no longer fabricates H.264/AAC/resolution values when actual inspection data is absent.
- Archive-derived managed executables are checked against their trusted manifest hash; tampered installed binaries are rejected.
- URL validation uses parsed URLs, rejects non-public literal IPs, and performs DNS preflight that rejects a host if any resolved address is private, loopback, link-local, multicast, or unspecified.
- Download admission is serialized so simultaneous starts cannot both reserve the single job slot.
- Cancellation reports `CANCELLING` until the worker has terminated the process tree, then reports `CANCELLED`.
- An existing regular file cannot be accepted as an output directory.
- Custom executable paths take precedence over cached resolution; saving settings clears the tool cache.
- yt-dlp date-style versions are compared as dates rather than by a year-prefix shortcut.

## 3. Actual architecture

```text
React 18 + TypeScript
  -> typed wrappers around Tauri invoke
Tauri command layer
  -> resolver / settings / tools / download manager
Rust core
  -> yt-dlp + FFmpeg/FFprobe + optional MediaInfo
Local filesystem
  -> output media + settings JSON + managed-tool manifest
```

Important runtime facts:

- The frontend polls `get_active_job` every 400 ms while a job is active. It does not currently use push events/SSE for progress.
- The backend holds a single `active_job` in memory and enforces maximum concurrency of one.
- Frontend history is the `allJobs` React state accumulated during the current session.
- Settings and the managed-tool manifest are persisted as local files. Jobs, history, diagnostics, plans, and recipes are not persisted in a database.
- `DownloadRecipe`, `SourceMediaGraph`, fingerprint, diff, workspace, deduplication, and archive domain modules exist. Only some are connected to the main runtime path; their presence must not be read as a complete user workflow.

## 4. State versus the canonical roadmap

| Slice | Status | Current evidence | Missing for acceptance |
| --- | --- | --- | --- |
| 0 — Baseline Integrity | **Complete** | Frontend and Rust checks/tests are green. Missing frontend support modules were restored before `c1cb768`. | Keep CI/build gates green. |
| 1 — Canonical Plan | **Not implemented** | Resolver, source graph, presets, recommendation, and recipe types provide reusable foundations. | No `AcquisitionRequest`, canonical `AcquisitionPlan`, `ProcessingClass`, or planner. Stream selection/output/processing are not determined through one authoritative backend plan. |
| 2 — Quality Transparency | **Partial** | Source metadata, processing cost/explanation, actual inspection, verification checklist, fingerprint, recipe, and receipt UI exist. | No authoritative Planned Output model, stable plan identity, or plan-versus-actual diff wired through UI/runtime. Pre-download behavior still centers on preset/quality and a raw command preview. |
| 3 — Acquisition Operations | **Partial foundation** | Entire-media behavior is implicit; audio outputs are available as presets; subtitle/chapter metadata can be analyzed. | No orthogonal operation model for Entire/Clip/Audio/Thumbnail/Chapter/Subtitle and no Best Source/Universal/Editing/Small profile model. Clip, thumbnail-only, chapter, and subtitle-only execution are absent. |
| 4 — Reliability Core | **Not implemented** | Single-job state machine, cancellation, verification, and in-session history exist. | No SQLite `JobStore`, persistent queue/history, typed error taxonomy, retry flow, per-job failure isolation, or interrupted-job recovery. |
| 5 — Platform Recovery | **Partial** | Tool health/install/repair, pinned hashes, custom paths, diagnostics, URL/path validation, and executable-cache invalidation exist. | No authentication assistant, disk-space preflight, user-facing duplicate policy, or complete deterministic recovery actions. Tool-update behavior is not yet the full workflow in the design. |
| 6 — Playlist | **Not implemented as a product flow** | Extracted metadata can contain playlist title/index/count. Analysis uses `--flat-playlist`. | No explicit single-item/playlist intent guard, browser, selection, batch queue, lazy detail analysis, or per-item override. |
| 7 — Media Controls | **Partial metadata support** | Subtitle/chapter/audio-language fields and related settings/types exist. | No track-selection workflow, metadata editor, filename preview, cover crop, or split-chapter execution. |
| 8 — Lightweight Finishing | **Not implemented** | Open file/open folder actions exist; isolated archive/workspace extension modules exist. | No Trim, Extract Audio, Merge Audio, Convert/Remux, Edit Metadata, Save/Crop Thumbnail, Split Chapters, or share workflow. |

## 5. Data and state ownership

| State | Current owner | Persistence |
| --- | --- | --- |
| Source analysis | React hook + resolver response | None |
| Active download | Rust `DownloadManager` | Memory only |
| Download history | React `allJobs` state | Memory only; lost on restart |
| Progress | Rust job, polled by React | Memory only |
| Verification/fingerprint/recipe | Attached to the active/completed `DownloadJob` | Memory only |
| Settings | Rust `SettingsManager` | Local JSON with staging/backup recovery |
| Tool inventory | `ToolManager` | Local manifest and app-local binaries |
| Diagnostics | Rust bounded buffer | Memory only |

There is no SQLite dependency in either Cargo manifest and no queue/job-store IPC surface. Persistence claims in the target design therefore remain future work.

## 6. Quality and verification semantics

The current working tree distinguishes:

- `VERIFIED`: ffprobe produced usable technical inspection and all required verification checks passed;
- `BASIC_INSPECTION`: only basic filesystem inspection was possible, so the job is not accepted as deeply verified;
- `UNVERIFIED`: no valid verification evidence.

A downloaded artifact is marked `COMPLETED` only when `VerificationResult.isValid` is true. If verification fails, the artifact is retained for inspection and the job is marked `FAILED` with an explanatory message.

This is stronger than the prior fallback behavior, but it also means a machine without a usable FFprobe cannot produce a successful verified completion even if yt-dlp wrote a playable file. The setup flow should therefore continue treating FFprobe as required.

## 7. Security and supply-chain posture

Implemented safeguards include:

- only HTTP/HTTPS input URLs;
- URL length/control-character/userinfo checks;
- public-IP and DNS preflight checks;
- no shell interpolation for media subprocesses;
- output-directory validation;
- process-tree cleanup;
- diagnostics redaction;
- staged managed-tool installation with a bounded 256 MiB download size;
- SHA-256 validation and manifest-bound checks for managed binaries;
- executable validation and ffprobe inspection are time-bounded so hung helper processes cannot stall these checks indefinitely;
- custom tool paths resolved from current settings.

Known boundary: DNS validation occurs before handing the URL to external tools. Redirects followed internally by yt-dlp/FFmpeg cannot be revalidated by the application at every hop. Complete redirect-time SSRF enforcement would require a controlled proxy/network sandbox or equivalent egress policy.

Pinned tool versions in source are yt-dlp `2025.02.19`, FFmpeg/FFprobe `7.1`, and MediaInfo `24.12`. This document does not assert that those are current upstream releases or that every embedded upstream checksum has independent provenance evidence in this repository. The Linux FFmpeg/FFprobe source is release-oriented rather than version-qualified, so immutable artifact pinning remains release work even though checksum mismatches fail closed.

## 8. Known gaps and risks

1. **No canonical plan.** The UI submits URL, metadata, preset, quality, and directory. The backend compiles the command, but no durable plan explains exact selected streams, output properties, and transformations before execution.
2. **No durable jobs.** Restarting the application loses active-job context, history, verification receipts, recipes, and failure details.
3. **Errors are mostly strings.** Resolver categories exist, but the IPC/download flow does not expose the structured error taxonomy and recovery actions required by the design.
4. **Playlist ambiguity remains.** Playlist metadata can be observed without an explicit user intent/scope guard or safe batch workflow.
5. **Feature-shaped types exceed runtime behavior.** Subtitle, chapter, workspace, archive, deduplication, and diff types/modules are foundations, not evidence of completed UI-to-backend features.
6. **Progress is polling-based.** `docs/architecture.md` currently claims zero polling/SSE, while the implementation polls at 400 ms. That documentation is stale.
7. **Quality documentation is stale.** `docs/quality-transparency.md` uses an older slice numbering/status and should be reconciled after the canonical planner lands.
8. **No frontend automated test suite.** Type checking and production bundling cover compilation, but there are no component/hook tests or end-to-end desktop tests in `package.json`.
9. **Cross-platform release validation is incomplete.** Rust unit/integration tests exercise core behavior locally; packaged Windows/macOS/Linux install, tool bootstrap, cancellation, and real-download matrices still need explicit release testing. In particular, the macOS FFmpeg/FFprobe pins currently point to `.7z` artifacts while the installer only has ZIP and Unix `tar` extraction paths, so managed installation on macOS is not release-ready.
10. **Managed-tool pins are not yet a release-grade provenance system.** Checksums are enforced, but pin provenance/refresh is manual and the Linux FFmpeg/FFprobe source URL is not immutable.
11. **Command preview is product-facing raw machinery.** `build_command` remains in IPC even though the target design calls for plan-first product APIs and removal of raw command construction from the product surface.

## 9. Verification evidence

The following local checks completed successfully after the branch hardening changes:

| Command | Result |
| --- | --- |
| `npm run lint` | Pass — TypeScript no-emit check |
| `npm run build` | Pass — production Vite bundle; benign dependency `use client` warnings only |
| `cargo fmt --all --check` in `src-tauri/crates/core` | Pass |
| `cargo clippy --all-targets -- -D warnings` in `src-tauri/crates/core` | Pass |
| `cargo test` in `src-tauri/crates/core` | Pass — 46 unit + 7 integration + 9 tool-manager tests |
| `cargo fmt --all --check` in `src-tauri` | Pass |
| `cargo clippy --all-targets -- -D warnings` in `src-tauri` | Pass |
| `cargo test` in `src-tauri` | Pass — 6 integration tests |
| `git diff --check` | Pass |

Total Rust tests executed across the two crates: 68. These results validate the branch development tree; they do not replace packaged desktop or live-provider testing.

## 10. Recommended next implementation order

Follow the dependency order in `design-master.md` rather than expanding the legacy preset surface:

1. Implement Slice 1 with `AcquisitionRequest`, canonical `AcquisitionPlan`, `ProcessingClass`, and a single backend planner.
2. Route analyze/start-download through a stable plan identity and remove frontend quality/processing inference.
3. Complete Slice 2 with explicit Source -> Planned Output -> Processing/Warnings -> Actual Output -> Diff presentation.
4. Introduce the orthogonal operation/profile model before adding clip, thumbnail, chapter, or subtitle execution.
5. Add SQLite-backed jobs and queue, structured errors, retry, and crash recovery before playlist batch execution.
6. Reconcile `README.md`, `docs/architecture.md`, and `docs/quality-transparency.md` with the implemented behavior as each slice is completed.

## 11. Current release assessment

**Ready for:** continued local development, canonical-plan implementation, and controlled manual testing with installed media tools.

**Not yet ready for:** claiming the full `design-master.md` product, durable production job management, safe playlist batch workflows, or broad cross-platform production release without packaging and live-provider validation.

# Media Downloader — Current State

> Snapshot: 2026-09-30
>
> Branch: `feat/production-readiness` (based on `dev` at `d33e3c8b685491cf85444390683dad381e82028b`)
>
> Product target: [design-master.md](design-master.md)

## Production-readiness increment (2026-09-30)

This branch selectively ports the two reviewed post-merge commits from the former canonical-plan branch. It adds cross-platform core checks, Tauri host validation, frontend capability/configuration contracts, scheduled CodeQL and RustSec scans, Dependabot updates, and secret/workflow checks. Contract PR checks now target both `dev` and `main`.

Diagnostics omit user-provided URLs at call sites and structurally normalize any URL found in text to its origin plus `/[REDACTED]`. URL userinfo, path, query and fragment are removed. Authorization, Cookie and X-Api-Key headers are redacted. The sanitizer tests cover those secret locations.

The UI identifies playlist context. When a video URL also names a playlist, “This video” is the only available choice and is the default. Entire-playlist execution remains unavailable. yt-dlp acquisition continues to compile explicit `--no-playlist`.

This increment does not add durable jobs, retry history, startup recovery, typed recovery errors, or packaged desktop smoke tests. Those remain release blockers; CI passing alone does not make this a production candidate.

## Implemented slice

The single-media flow supports Entire Media, Audio Only, Clip, Chapter, Thumbnail (JPEG), and Subtitles (one WebVTT language per job). Profiles are Best Source, Universal, Editing, and Small. Each job receives a fresh output folder.

The canonical loop is:

```text
Source metadata + request + semantic settings
  -> backend plan and plan ID
  -> user reviews plan
  -> Start supplies expectedPlanId
  -> backend replans and rejects drift
  -> required-tool preflight
  -> exact yt-dlp stream selection and explicit transforms
  -> inspect artifact
  -> backend verifies against plan
  -> completed receipt or failed verification with retained artifact
```

Plan version 3 contains embedding, subtitle and SponsorBlock policy. Settings changes invalidate frontend plans. Stale asynchronous planning responses cannot replace newer plans. Operational settings such as fragment concurrency do not choose output codecs.

The execution compiler lives in `core/src/execution.rs`. The legacy preset compiler and duplicate job preset/subtitle/SponsorBlock fields were removed. Persisted defaults and recommendations still use preset names as a UI migration adapter, not as an execution authority.

## Merge-review follow-up

- Separate video can only merge with a format explicitly identified as audio-only. Without one, the planner selects a combined A/V source and exposes its actual resolution; it never compiles video-only + combined-A/V.
- Saved subtitle preferences apply to Entire Media, Clip and Chapter. The backend resolves comma-separated patterns, all, and exclusions to concrete available tracks before hashing the plan. Missing matches produce a warning; invalid or unsupported patterns produce a planning error. The settings UI documents the supported regex subset and manual-track default.
- Best Source/Small audio use shared codec-family and extraction-format mappings. AAC variants such as mp4a.40.5 are supported; raw AAC is retained where the source reports it. Unsupported preservation codecs are rejected during planning with a conversion-profile alternative, rather than producing an unexecutable mka plan.
- Subtitle-only plans explicitly convert to WebVTT and preflight FFmpeg. Native VTT is preferred; other available subtitle formats can be converted.
- Trim transform milliseconds now generate TypeScript number fields, consistent with the clip and plan JSON contract.

## Verification contract

- Video/audio codec, container aliases, dimensions, known FPS and duration are compared by the backend.
- Embedded subtitle requests require at least the planned number of subtitle streams. This does not yet verify every embedded language tag or cue content.
- Full-media duration tolerance is max(1 second, 1%). Clip/Chapter fast cuts allow ±2 seconds.
- SponsorBlock removal makes duration content-dependent; the verifier reports that duration comparison is skipped.
- JPEG thumbnails require a decodable image codec and positive dimensions. WebVTT exports require a valid header and timed cues.
- Cover artwork is excluded from the main-video stream check.
- Unknown source FPS, language and codec values are not replaced by fabricated UI defaults.
- A valid/playable file alone is insufficient for completion. Backend plan mismatches fail the job and are shown in the receipt.
- Recipe transformations come from plan steps; the receipt uses the created recipe ID and inspected output container.

Explicit video/audio conversion is a separate FFmpeg step. Local tests generate H.264+Opus, VP9+AAC and VP9+Opus fixtures and verify H.264/AAC outputs, including embedded subtitles and cover artwork. MP4/MOV text subtitles are converted to mov_text. Diagnostic previews show argument vectors for both yt-dlp and FFmpeg, not runnable shell commands.

## Tool management

Artifacts declare OS, architecture, packaging, URL, checksum and executable path. Unsupported combinations fail before download and can use configured custom executables.

- Downloads stream to disk while hashing, with a 256 MiB limit.
- ZIP/tar extraction selects one declared executable and rejects ambiguous or unsafe member paths.
- Executables are validated before activation. Repair retains the previous binary until replacement validation; activation/manifest failures attempt rollback.
- Existing managed binaries are checked against their manifest hash.
- The Linux MediaInfo source archive is not offered as an installable executable.
- Managed 7z extraction is explicitly unsupported. macOS FFmpeg/FFprobe currently require custom tools; no silent tar fallback is attempted.

The existing pins have not been independently audited against upstream artifacts in this work. Linux FFmpeg/FFprobe still use a mutable release URL; hashes fail closed, but immutable URLs and recorded checksum provenance remain release work.

## State and boundaries

One backend job may run at a time. Progress is polled every 400 ms. Jobs, history, plans, recipes and diagnostics are in memory; settings and tool manifests are persisted.

Implemented safeguards include parsed HTTP(S) URL checks, public-IP/DNS preflight, output-directory checks, subprocess execution without shell interpolation, cancellation/process-tree cleanup, and bounded/redacted diagnostics. Redirects followed inside external tools are not revalidated at every hop.

The planner rejects custom profiles, metadata patches and duplicate policies other than Rename rather than silently ignoring them. Explicit unavailable audio/subtitle languages are rejected.

## Roadmap acceptance

| Slice | Current status | Remaining acceptance work |
| --- | --- | --- |
| 0 — Baseline integrity | Local checks pass; CI configured | Observe cross-OS CI and packaged builds |
| 1 — Canonical plan | Reviewed identity, semantic policy, exact transforms and preflight implemented | Extend invariants with future profiles |
| 2 — Quality transparency | Backend-owned plan/actual comparison and receipts implemented | More artifact-policy checks, including subtitle language tags and embedded metadata |
| 3 — Acquisition operations | Six single-media operations implemented | Packaged desktop and live-provider validation |
| 4 — Reliability core | Single in-memory job and cancellation | Durable queue/history, typed recovery, retry and crash recovery |
| 5 — Platform recovery | Managed/custom tools and safer installer | Audited immutable pins, ARM coverage, macOS bootstrap, disk-space/recovery UX |
| 6 — Playlist | Metadata only | Explicit scope guard and batch workflow |
| 7 — Media controls | Partial track/chapter support | Metadata editor, filename preview, crop and split-all-chapters |
| 8 — Finishing | Open file/folder | Standalone editing/conversion workflows |

Fast cuts are not frame-accurate. Multi-artifact subtitle jobs and split-all-chapters are not implemented. Final-transcode source intermediates are retained. Desktop end-to-end tests, live-provider tests and packaged cross-OS release tests remain necessary.

## Verification evidence

Local Linux validation is separate from cross-platform CI. The local results below were rerun on 2026-09-30 for `feat/production-readiness`; GitHub Actions is authoritative for remote validation.

- Core: 59 unit tests, 3 execution-contract tests, 7 integration tests, 11 merge-review regression tests, 9 tool-manager tests (89 total).
- Desktop: 5 integration tests pass; 1 live-provider smoke test is ignored by default.
- The offline SRT-only fixture runs real yt-dlp and FFmpeg and validates the final artifact.en.vtt file. Local yt-dlp: 2026.08.25.233329. CI installs the managed catalog version, 2025.02.19.
- Frontend: 11 tests covering stale plans, settings invalidation, planning failure, backend-owned mismatch rendering, numeric millisecond IPC types, and playlist intent detection.
- Rust/TypeScript IPC DTOs are generated from Rust; a test fails if checked-in bindings drift, ignoring CRLF/LF differences. Regression coverage checks that type changes still fail comparison. Regenerate with `npm run bindings`.
- TypeScript checking, production Vite build, Rust formatting, Clippy and desktop Cargo checks are run before handoff.
- CI runs Rust tests/Clippy on Linux, Windows and macOS, plus frontend tests/build. A green matrix for the candidate commit is required before merge. See [GitHub Actions](https://github.com/Dyu20705/media-downloader/actions) for current results. Unix shell-executable fixtures are Unix-only; archive and artifact-selection tests are portable.

These checks establish a development baseline, not a broad production-release claim.

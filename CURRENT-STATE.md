# Media Downloader — Current State

> Snapshot: 2026-09-29
>
> Branch: `feat/canonical-acquisition-plan`
>
> Product target: [design-master.md](design-master.md)

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

The plan contains embedding, subtitle and SponsorBlock policy. Settings changes invalidate frontend plans. Stale asynchronous planning responses cannot replace newer plans. Operational settings such as fragment concurrency do not choose output codecs.

The execution compiler lives in `core/src/execution.rs`. The legacy preset compiler and duplicate job preset/subtitle/SponsorBlock fields were removed. Persisted defaults and recommendations still use preset names as a UI migration adapter, not as an execution authority.

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

Local Linux checks:

- Core: 55 unit tests, 3 execution-contract tests, 7 integration tests, 9 tool-manager tests.
- Frontend: 5 hook/component tests covering stale plans, settings invalidation, planning failure and backend-owned mismatch rendering.
- Rust/TypeScript IPC DTOs are generated from Rust; a test fails if checked-in bindings drift. Regenerate with `npm run bindings`.
- TypeScript checking, production Vite build, Rust formatting, Clippy and desktop Cargo checks are run before handoff.
- CI is configured for Rust tests/Clippy on Linux, Windows and macOS, plus frontend tests/build. It has not been run remotely in this session. Unix shell-executable fixtures are Unix-only; archive and artifact-selection tests are portable.

These checks establish a development baseline, not a broad production-release claim.

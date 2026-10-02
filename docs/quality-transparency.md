# Quality Transparency & Download Plan Design

Status: **Slice 2 complete — canonical plan, pre-download presentation, verified actual output, and diff implemented**
Owner: openDownloader
Target branch: `feat/canonical-acquisition-plan`

## 1. Problem

The application analyzes media, accepts an operation and output profile, downloads through yt-dlp/FFmpeg, and verifies the resulting file. The quality-transparency layer makes the relationship between **source quality**, **requested behavior**, **processing required**, and **verified output** explicit.

This creates several avoidable UX problems:

- A preset name such as `MP3 320 kbps` can be interpreted as a quality upgrade even when the source is lower-bitrate lossy audio.
- A FLAC output can be interpreted as a lossless *source*, although transcoding AAC/Opus to FLAC cannot restore information already discarded upstream.
- Without the plan, the user cannot tell whether a requested profile preserves streams, only remuxes/merges them, or requires re-encoding.
- Requested quality and actual downloaded quality can diverge because of extractor availability, codec/container compatibility, or platform constraints.

The product should answer three questions clearly:

1. **What is the source?**
2. **What does this preset plan to produce, and what processing is required?**
3. **What file was actually produced after verification?**

## 2. Design goals

### Goals

- Make source-vs-output quality understandable before download.
- Reuse the resolver's existing cost model instead of inventing a second processing taxonomy.
- Prefer source preservation over transcoding when the user's intent allows it.
- Never claim that bitrate/container/codec conversion improves unavailable source information.
- Distinguish a **planned output** from a **verified actual output**.
- Degrade gracefully when yt-dlp does not provide a field such as bitrate, HDR metadata, filesize, or language.
- Keep metadata analysis lightweight; do not add heavyweight FFmpeg passes to the Analyze stage.

### Non-goals for v1

- Per-playlist-item format overrides.
- Time-range/chapter download controls.
- Audio metadata editing.
- Full post-processing editor.
- AI upscaling or frame interpolation.
- Predicting exact final file size when the extractor does not provide sufficient information.
- Claiming exact processing behavior from frontend heuristics alone.

## 3. Terminology

### Source

The media streams and metadata reported by the extractor before preset-specific processing.

### Planned output

The best current description of the output implied by the selected operation/profile and resolver decision. It is a plan, not a guarantee.

### Actual output

Properties read from the downloaded file by the existing verification pipeline (FFprobe/MediaInfo). This is the authoritative post-download representation.

### Processing cost

Use the same conceptual ordering already defined by the Universal Media Resolver:

```text
STREAM_COPY → REMUX → MERGE → TRANSCODE
```

Lower-cost processing should be preferred when it satisfies the requested intent.

## 4. UX model

The backend now provides an authoritative normalized `AcquisitionPlan`. The next UI slice should expose a `DownloadPlanCard` below the operation/profile controls. The frontend must not infer this plan itself.

### 4.1 Source section

Example:

```text
SOURCE
2160p60 · VP9 · HDR
Opus · ~160 kbps
WebM
```

Only render fields that are known. Unknown values must not be fabricated.

Useful fields, when available:

- video resolution
- frame rate
- HDR/SDR indicator
- video codec
- audio codec
- audio bitrate
- audio language
- container / extension
- exact or approximate size

### 4.2 Planned output section

Example preserving streams:

```text
PLANNED OUTPUT
2160p60 · VP9 · HDR
Opus · ~160 kbps
MKV

PROCESSING
✓ Video stream preserved
✓ Audio stream preserved
Merge only · No re-encoding
```

Example MP3 conversion:

```text
PLANNED OUTPUT
MP3 · compatibility-oriented transcode

PROCESSING
⚠ Audio transcode
Transcoding cannot restore detail missing from the source stream.
```

The pre-download UI must say **Planned output**, not **Actual output**.

### 4.3 Actual output section

After the existing verification stage succeeds, the UI may replace or supplement the plan with verified data:

```text
ACTUAL OUTPUT
1920×1080 · H.264 · 30 fps
AAC · 128 kbps
MP4 · 92.8 MB

✓ Verified
```

If planned and actual properties differ materially, the UI should surface the difference instead of silently hiding it.

## 5. Processing classification

The frontend should consume a normalized backend classification rather than infer processing from preset names.

Implemented UI-facing classification:

```ts
export type ProcessingClass =
  | 'SOURCE_PRESERVED'
  | 'MERGE_ONLY'
  | 'REMUX_ONLY'
  | 'AUDIO_TRANSCODE'
  | 'VIDEO_TRANSCODE'
  | 'FULL_TRANSCODE'
  | 'UNKNOWN';
```

Suggested mapping from resolver behavior:

| Resolver behavior | UI classification | User-facing meaning |
| --- | --- | --- |
| direct/unchanged stream | `SOURCE_PRESERVED` | No re-encoding |
| separate streams multiplexed | `MERGE_ONLY` | Streams preserved; container assembly only |
| container changed without codec change | `REMUX_ONLY` | No re-encoding |
| audio codec conversion | `AUDIO_TRANSCODE` | Audio quality can only stay equal or degrade relative to decoded source |
| video codec conversion | `VIDEO_TRANSCODE` | Video is re-encoded |
| both video and audio re-encoded | `FULL_TRANSCODE` | Both streams are re-encoded |
| backend cannot determine plan | `UNKNOWN` | Do not speculate |

The backend remains authoritative because actual command construction and container compatibility decisions live there.

## 6. Implemented backend contract

The `plan_acquisition` IPC command exposes the normalized `AcquisitionPlan` defined in `types.rs`. It contains stable identity, source summary, selected stream IDs, planned artifact, processing, estimated size, warnings, and requirements. Execution replans from the same request and compiles those exact selected IDs rather than independently resolving quality.

```ts
interface AcquisitionPlan {
  id: string;
  version: number;
  source: SourceSummary;
  scope: SourceScope;
  operation: AcquisitionOperation;
  outputProfile: OutputProfile;
  selectedStreams: SelectedStreams;
  output: PlannedArtifact;
  processing: { class: ProcessingClass; requiresFfmpeg: boolean; steps: string[] };
  estimatedSize?: { bytes: number; confidence: string } | null;
  warnings: Array<{ code: string; message: string }>;
  requirements: Array<{ code: string; message: string }>;
}
```

Use nullable/optional fields for extractor uncertainty. Do not convert unknown values into zeros or defaults that look authoritative.

## 7. Preset semantics

### Best Audio

Intent: preserve the best available source audio stream where practical.

User-facing copy should emphasize:

- source preservation
- no unnecessary lossy-to-lossy conversion
- output codec may remain Opus/AAC/etc. depending on source

### MP3

Intent: compatibility, not quality enhancement.

Required message:

- MP3 is a lossy transcode for most web-media sources.
- The encoder quality setting describes the output request; it is not evidence of source quality.
- Re-encoding cannot restore detail absent from the source.

### FLAC

Intent: produce FLAC output for a workflow that requires FLAC.

Required message:

- FLAC encoding itself is lossless relative to decoded PCM at the point of encoding.
- A lossy AAC/Opus/etc. source does not become an original lossless recording after conversion to FLAC.
- The UI must not use wording that implies source quality was recovered.

## 8. Integration points

### Frontend

- `src/components/AcquisitionControls.tsx`
  - independent operation and output-profile intent with concise quality guidance
- `src/components/MediaSummaryCard.tsx`
  - source identity remains here; avoid duplicating title/uploader information in the plan
- `src/components/DownloadPlanCard.tsx`
  - consume the normalized backend plan
  - selected source stream presentation keyed by the authoritative plan stream IDs
  - planned output, processing class/steps, estimates, requirements, and warnings
- `src/components/PlanActualComparison.tsx`
  - verified actual-output presentation
  - material plan-versus-actual comparison where both values are known
- `src/App.tsx`
  - owns analyzed metadata + selected operation/profile + normalized plan state

### Rust core

The concrete modules should follow the current core layout, but responsibilities are:

- resolver: determine selected streams, target container, and processing cost
- download manager: execute the resolved plan
- verifier: return actual output properties after download
- IPC types: expose normalized source/plan/verification data to React

## 9. State transitions

Quality transparency should align with the existing application state machine:

```text
IDLE
  ↓
ANALYZING
  ↓
READY        → show SOURCE + PLANNED OUTPUT
  ↓
DOWNLOADING
  ↓
POST_PROCESSING
  ↓
VERIFYING
  ↓
COMPLETED    → show VERIFIED ACTUAL OUTPUT
```

A failed verification must not be displayed as a verified actual output.

## 10. Error and uncertainty rules

- Unknown bitrate → omit bitrate or show `Unknown`; never estimate without a supported calculation.
- Unknown filesize → omit size or explicitly mark it estimated.
- Missing HDR metadata → do not assume SDR.
- Backend processing class unavailable → show `Processing details unavailable`; do not infer from extension alone.
- Extractor offers lower quality than user target → planned output must reflect the selected available stream, not the target label.
- Verification returns a different codec/resolution than planned → actual output wins and the mismatch should be visible.

## 11. Accessibility and copy rules

- Do not communicate quality state using color alone.
- Warnings require textual labels such as `Transcode` or `No re-encoding`.
- Keep operation/profile choices concise; longer explanations belong in contextual guidance or the plan card.
- Avoid marketing terms such as `lossless quality`, `studio quality`, or `high-quality 320 kbps` unless they describe verified source properties accurately.

## 12. Testing strategy

### Unit tests

Backend plan classification should cover at least:

1. progressive source → stream copy
2. separate video/audio → merge only
3. compatible codec + different container → remux only
4. Opus/AAC → MP3 → audio transcode
5. lossy audio → FLAC → audio transcode with source-quality warning
6. unknown metadata → no fabricated fields

Frontend pure rendering/mapping tests should cover:

- source-preserved copy
- transcode warning copy
- unknown fields
- planned vs actual terminology

### Integration/manual checks

Use representative URLs where legally accessible:

- progressive MP4
- YouTube DASH video+audio
- WebM/Opus audio source
- source where requested maximum resolution is unavailable

Confirm that the pre-download plan matches the resolver and that post-download actual properties match verifier output.

## 13. Incremental implementation plan

### Foundation — preset semantics (implemented)

- Clarify Best Audio source-preservation intent.
- Clarify MP3 as a compatibility-oriented lossy transcode.
- Clarify FLAC output vs source quality.
- Add contextual quality note to the selected audio preset.
- Update README wording.

### Slice 1 — canonical acquisition plan (implemented)

- Rust owns `AcquisitionRequest`, `AcquisitionPlan`, and `ProcessingClass`.
- One planner selects streams, expected output, processing, estimates, warnings, and requirements.
- Download execution compiles exact stream IDs from that plan.
- `plan_acquisition` exposes the normalized plan through typed Tauri IPC.

### Slice 2 — pre-download plan presentation (implemented)

- Render source, planned output, and processing.
- Refresh plan when operation/profile/quality selection changes.

### Slice 2 completion — verified actual output and diff (implemented)

- Map verifier output to `Actual output` UI.
- Surface material plan-vs-actual differences.

## 14. Acceptance criteria for the complete feature

- Users can distinguish source properties from requested/preset properties.
- Users can tell whether re-encoding is expected before download.
- MP3/FLAC copy does not imply quality restoration.
- The UI never labels a planned property as verified actual output.
- Actual output comes from the post-download verifier.
- Unknown extractor data remains explicitly unknown.
- Analyze remains lightweight and does not introduce heavy FFmpeg scanning.

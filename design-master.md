# Media Downloader — Canonical Product & Technical Design

**Repository:** `Dyu20705/media-downloader`
**Baseline:** `main @ 1f708d32cf8bab6257790762327918ae50dee325`
**Status:** Canonical design proposal — chưa phải implementation approval
**Product boundary:** **Local Media Acquisition + Lightweight Finishing**

---

# 1. Product definition

`media-downloader` không nên phát triển thành:

> GUI chứa càng nhiều option `yt-dlp` càng tốt.

Product nên giải quyết workflow hoàn chỉnh:

```text
URL / Clipboard
      ↓
Understand Source
      ↓
Choose WHAT to acquire
      ↓
Choose HOW output should behave
      ↓
Resolve authoritative plan
      ↓
Acquire
      ↓
Verify actual output
      ↓
Recover / Retry if necessary
      ↓
Lightweight finishing
      ↓
Open / Share / Use
```

## 1.1 Product promise

Người dùng phải luôn biết:

1. **Mình đang tải cái gì?**
2. **App sẽ lấy phần nào của media?**
3. **Nguồn thực tế có chất lượng gì?**
4. **Output dự kiến sẽ là gì?**
5. **Có re-encode hay không?**
6. **File thực tế sau download là gì?**
7. **Nếu thất bại, tại sao và phải làm gì?**

Điểm khác biệt của product vì vậy là:

> **Transparent, source-aware media acquisition with reliable recovery and lightweight finishing.**

Không phải:

> “4K/320K/FLAC downloader”.

---

# 2. Hard product boundaries

## Included

* single video/audio acquisition;
* playlist detection;
* playlist browser;
* selected playlist items;
* fast playlist analysis;
* source-quality inspector;
* planned-output inspector;
* actual-output verification;
* clip/time-range download;
* chapter download;
* split chapters;
* thumbnail-only;
* cover crop;
* subtitle selection;
* audio-language selection;
* source-preserving audio;
* intent-based output profiles;
* output size estimation;
* disk preflight;
* persistent queue/history;
* failure isolation;
* retries;
* interrupted download recovery;
* duplicate handling;
* authentication assistant;
* extractor/tool health;
* metadata editing;
* filename preview;
* lightweight post-processing;
* open/reveal/copy/share completed output.

## Explicitly out of scope

Không đưa vào core product:

* AI video upscale;
* frame interpolation;
* generative restoration;
* transcription;
* summarization;
* full NLE/video editor;
* multi-track timeline;
* transitions/effects/text overlays;
* Plex-like media library;
* cloud synchronization;
* account system;
* arbitrary automation platform;
* DRM bypass;
* CAPTCHA bypass;
* paywall bypass.

AI upscale đặc biệt không nên nằm trong roadmap core trước khi acquisition/recovery workflow ổn định.

---

# 3. Baseline integrity blocker — P0.0

Trước khi thêm feature mới phải đưa repository về **build-green baseline**.

Hiện `main` có `src/App.tsx` import:

```text
./types
./services/ipc
./hooks/useToolEngine
./hooks/useMediaAnalysis
./hooks/useRecommendation
./hooks/useDownloadEngine
./hooks/useDiagnostics
```

nhưng các path tương ứng hiện không tồn tại trên `main` khi đọc trực tiếp repository.

Vì vậy:

```text
P0.0
Restore repository integrity
        ↓
npm run lint
npm run build
cargo test
cargo check
        ↓
GREEN
        ↓
feature work
```

**Không nên thiết kế implementation mới dựa trên một frontend tree chưa self-consistent.**

---

# 4. Priority order

Đây là thứ tự tôi khuyến nghị triển khai, dựa trên **dependency + user impact + reliability**, không dựa trên độ hấp dẫn của feature.

| Order | Priority | Capability                                                    | Reason                                                            |
| ----: | -------- | ------------------------------------------------------------- | ----------------------------------------------------------------- |
|     0 | **P0**   | Restore build-green baseline                                  | Mọi feature khác phụ thuộc vào baseline đáng tin cậy              |
|     1 | **P0**   | Canonical `AcquisitionPlan`                                   | Foundation của quality, clip, playlist, audio, error transparency |
|     2 | **P0**   | Source → Plan → Actual transparency                           | Giải quyết mismatch quality và hidden transcoding                 |
|     3 | **P0**   | Single-video vs playlist intent guard                         | Tránh hành vi tải nhầm nghiêm trọng                               |
|     4 | **P0**   | Operation model: Entire / Clip / Audio / Thumbnail / Subtitle | Ngăn preset explosion                                             |
|     5 | **P0**   | Intent output profiles                                        | Basic UX không bắt người dùng hiểu codec                          |
|     6 | **P0**   | Structured error taxonomy + recovery actions                  | Failure hiện tại phải actionable                                  |
|     7 | **P0**   | Persistent queue + failure isolation                          | Reliability foundation                                            |
|     8 | **P0**   | Failed history + Retry                                        | Không mất failure context                                         |
|     9 | **P0**   | Engine health / yt-dlp recovery                               | Extractor breakage là recurring failure                           |
|    10 | **P0**   | Authentication assistant                                      | Giải quyết login/cookie friction                                  |
|    11 | **P0**   | Clip/time-range acquisition                                   | High-value workflow                                               |
|    12 | **P0**   | Thumbnail-only                                                | High-value, low architectural cost                                |
|    13 | **P0**   | Source-preserving audio                                       | Quality correctness                                               |
|    14 | **P0**   | Verification diff                                             | Planned vs actual mismatch phải visible                           |
|    15 | **P1**   | Fast playlist browser                                         | Batch workflow                                                    |
|    16 | **P1**   | Playlist item selection                                       | User control                                                      |
|    17 | **P1**   | Per-item override                                             | Advanced playlist workflow                                        |
|    18 | **P1**   | Chapter acquisition / split                                   | Podcast/course workflow                                           |
|    19 | **P1**   | Audio language selection                                      | Dub/original workflow                                             |
|    20 | **P1**   | Size estimate + disk-space preflight                          | Prevent predictable failures                                      |
|    21 | **P1**   | Resume interrupted jobs                                       | Reliability                                                       |
|    22 | **P1**   | Duplicate policy                                              | Prevent incorrect skip/overwrite behavior                         |
|    23 | **P1**   | Metadata editor + filename preview                            | Music workflow                                                    |
|    24 | **P1**   | Cover crop 1:1                                                | Music workflow                                                    |
|    25 | **P1**   | Quick Tools                                                   | Finish without leaving app                                        |
|    26 | **P1**   | Open / reveal / copy / share                                  | Completion workflow                                               |
|    27 | **P2**   | Advanced precise cuts                                         | Requires deliberate re-encode trade-off                           |
|    28 | **P2**   | Advanced batch rules/templates                                | Useful only after queue/playlist mature                           |
|    29 | **P2**   | Extended archive/dedup/fingerprint workflows                  | Existing architecture already contains extension points           |

---

# 5. Central architectural decision: Operation ≠ Output Profile

Đây là thay đổi quan trọng nhất.

Current presets đang trộn hai câu hỏi khác nhau:

```text
"What do I want?"
+
"What encoding/container do I want?"
```

Ví dụ:

```text
Best Audio
MP3
FLAC
Best Video
```

Điều này không scale khi thêm:

* clip;
* thumbnail;
* chapter;
* subtitles;
* playlist;
* cover art.

## Target model

### Dimension A — Acquisition Operation

```rust
enum AcquisitionOperation {
    EntireMedia,
    Clip {
        start_ms: u64,
        end_ms: u64,
    },
    AudioOnly,
    ThumbnailOnly,
    Chapter {
        chapter_index: u32,
    },
    SubtitlesOnly,
}
```

### Dimension B — Output Profile

```rust
enum OutputProfile {
    BestSource,
    Universal,
    Editing,
    Small,
    Custom,
}
```

### Dimension C — Track Selection

```rust
struct TrackSelection {
    audio_language: Option<String>,
    subtitle_languages: Vec<String>,
    include_auto_subtitles: bool,
}
```

Như vậy:

```text
Audio Only + Best Source
Video Clip + Universal
Chapter 4 + Best Source
Entire Video + Editing
Thumbnail Only + PNG
Playlist + Small
```

thay vì tạo hàng chục preset.

---

# 6. Canonical source of truth: `AcquisitionPlan`

Frontend **không được tự suy luận command, codec, transcode hoặc expected output**.

Rust backend phải là authority.

Flow:

```text
Analyze
   ↓
SourceMediaGraph
   ↓
AcquisitionRequest
   ↓
Planner
   ↓
AcquisitionPlan
   ↓
Executor
   ↓
Output artifact
   ↓
Verifier
   ↓
ActualOutput
```

## 6.1 Proposed contract

```rust
struct AcquisitionRequest {
    source_scope: SourceScope,
    operation: AcquisitionOperation,
    output_profile: OutputProfile,
    track_selection: TrackSelection,
    metadata_patch: Option<MetadataPatch>,
    duplicate_policy: DuplicatePolicy,
    output_directory: PathBuf,
}

struct AcquisitionPlan {
    id: String,
    version: u32,

    source: SourceSummary,
    scope: SourceScope,
    operation: AcquisitionOperation,

    selected_streams: SelectedStreams,
    output: PlannedArtifact,

    processing: ProcessingPlan,
    estimated_size: Option<SizeEstimate>,

    warnings: Vec<PlanWarning>,
    requirements: Vec<PlanRequirement>,
}
```

`ExecutionRecipe`/CLI arguments phải là backend-internal.

React chỉ nhận normalized plan.

---

# 7. Existing domain objects should be reused

Repository hiện đã có nhiều primitive tốt:

```text
MediaMetadata
SourceMediaGraph
VideoStreamSpec
AudioStreamSpec
SubtitleTrack
MediaChapter
MediaFormatSpec
ResolvedMediaSource
DownloadRecipe
MediaFingerprint
VerificationResult
OutputMediaArtifact
TranscodingCost
DownloadStrategy
```

Không nên tạo một second architecture song song.

## Correct evolution

```text
MediaMetadata
      ↓
SourceMediaGraph
      ↓
AcquisitionPlan       ← NEW authoritative planning layer
      ↓
DownloadManager
      ↓
VerificationResult
      ↓
DownloadRecipe        ← provenance/receipt after execution
```

### Important distinction

`AcquisitionPlan`

> What the application intends to do.

`DownloadRecipe`

> What was actually selected/executed.

`VerificationResult`

> What artifact actually exists.

Ba khái niệm này không được trộn.

---

# 8. Processing model

Existing:

```text
NoProcessing
StreamCopy
Remux
Merge
Transcode
```

nên được giữ.

UI-facing classification:

```text
SOURCE_PRESERVED
MERGE_ONLY
REMUX_ONLY
AUDIO_TRANSCODE
VIDEO_TRANSCODE
FULL_TRANSCODE
UNKNOWN
```

## Mandatory invariant

```text
Source
≠
Requested output
≠
Planned output
≠
Actual output
```

Ví dụ:

```text
SOURCE
1440p60 VP9
Opus ~150 kbps

REQUEST
Universal Video

PLANNED OUTPUT
1080p H.264
AAC
MP4

PROCESSING
Video transcode
Audio transcode

ACTUAL OUTPUT
1920×1080
H.264 High
AAC 128 kbps
93.4 MB

VERIFIED
Yes
```

Không được hiển thị target `"2160p"` như actual output trước verification.

---

# 9. Basic-mode output profiles

Top-level UI chỉ nên có 4 lựa chọn chính.

## Best Source

Goal:

> Preserve maximum available source information with minimum processing.

Policy:

```text
stream copy
   >
merge
   >
remux
   >
transcode
```

Video:

* max suitable source resolution;
* preserve HDR/FPS;
* preserve video codec;
* preserve audio codec;
* MKV allowed when necessary.

Audio:

* source Opus/AAC/FLAC/etc.;
* no unnecessary conversion.

---

## Universal

Goal:

> File dễ mở/chia sẻ trên hầu hết devices/apps.

Video target:

```text
MP4
H.264
AAC
```

Planner first searches compatible existing streams.

Only transcode when required.

Audio target:

```text
MP3 high-quality VBR
```

UI phải ghi rõ:

```text
Compatibility-oriented lossy transcode
```

không phải `"320 kbps quality upgrade"`.

---

## Editing

Goal:

> Produce media compatible with common editing software.

Video policy:

* MP4;
* H.264;
* AAC;
* preserve resolution/FPS unless explicitly constrained;
* warn strongly when AV1/VP9/HDR source requires expensive transcode.

Audio:

* FLAC can be used as editing intermediate;
* UI explicitly states that lossy source → FLAC **does not recover source quality**.

---

## Small

Goal:

> Reduce expected storage/bandwidth while avoiding unnecessary processing.

Prefer:

1. lower-resolution existing source;
2. efficient source codec;
3. only transcode when required to meet explicit size constraints.

Không nên tự động transcode 4K AV1 → low-bitrate H.264 nếu một source 720p đã tồn tại.

---

# 10. Advanced / Custom mode

Advanced panel được mở riêng.

Có thể expose:

* resolution;
* FPS;
* HDR preference;
* video codec;
* audio codec;
* container;
* audio language;
* subtitle selection;
* bitrate/size limit;
* SponsorBlock;
* metadata/chapter embedding.

Basic mode không expose raw `yt-dlp` flags.

Raw arbitrary CLI arguments **không nên được hỗ trợ trong normal application mode**.

---

# 11. Single video vs playlist guard

Khi URL dạng:

```text
youtube.com/watch?v=ABC&list=XYZ
```

application phải detect ambiguity.

UI:

```text
This URL belongs to a playlist.

What do you want?

● This video
○ Entire playlist

[Continue]
```

Default:

```text
This video
```

Không bao giờ silently choose playlist.

yt-dlp đã cung cấp semantic rõ:

* `--no-playlist`
* `--yes-playlist`

Backend phải explicit flag thay vì dựa vào default behavior.

---

# 12. Playlist architecture

Playlist phải có **2 analysis modes**.

## Fast Playlist — default

Purpose:

> nhanh chóng cho người dùng thấy playlist và chọn items.

Use shallow extraction:

```text
--flat-playlist
```

yt-dlp xác nhận mode này tránh full extraction từng entry nhưng một số metadata có thể thiếu.

Return:

```text
index
id
title
duration if known
thumbnail if known
uploader if known
availability
```

Không scan toàn bộ format của 100 videos.

---

## Customize Each Item

Chỉ khi người dùng yêu cầu.

Selected row có thể lazy-analyze:

```text
item → full metadata → format information
```

Không full-resolve tất cả items ngay khi playlist mở.

---

# 13. Playlist execution

Có hai paths.

## Homogeneous playlist

Nếu tất cả selected items dùng cùng:

```text
operation
output profile
subtitle policy
audio language policy
metadata policy
```

planner có thể tạo **one batch execution** với selected indices.

yt-dlp hỗ trợ explicit playlist index/ranges thông qua `--playlist-items`.

Lợi ích:

* ít process startup;
* extractor reused;
* efficient cho playlist lớn.

---

## Heterogeneous playlist

Khi item có per-item override:

```text
item 1 → Video / Best Source
item 2 → Audio / MP3
item 3 → Chapter 2
```

playlist được split thành independent child jobs.

```text
PlaylistJob
├── ItemJob #1
├── ItemJob #2
└── ItemJob #3
```

Mỗi child có retry/status riêng.

---

# 14. Queue model

Repository hiện enforce active download concurrency = 1.

**Giữ nguyên.**

Không cần tăng parallel media jobs ở giai đoạn này.

```text
Persistent Queue
        ↓
Worker concurrency = 1
        ↓
yt-dlp
```

`concurrent_fragments` vẫn có thể >1 trong **một download**, khác hoàn toàn concurrent jobs.

yt-dlp hỗ trợ `--concurrent-fragments`; default upstream hiện là 1.

Điều này cho:

* predictable CPU;
* predictable disk I/O;
* easier cancellation;
* easier recovery;
* simpler state machine;
* không làm laptop bị thrash.

---

# 15. Persistent job state

Analysis UI state và persisted job state nên tách riêng.

## Analysis states

```text
EMPTY
ANALYZING
ANALYZED
PLAN_READY
ANALYSIS_FAILED
```

## Job states

```text
QUEUED
   ↓
PREPARING
   ↓
DOWNLOADING
   ↓
POST_PROCESSING
   ↓
VERIFYING
   ↓
COMPLETED
```

Terminal/exception paths:

```text
FAILED
CANCELLED
INTERRUPTED
SKIPPED_DUPLICATE
```

Không cần `PAUSED` ở v1.

Resume không đồng nghĩa pause/resume controls.

---

# 16. Persistence design

Full feature set này đã vượt mức phù hợp cho `Vec<DownloadJob>` hoặc single JSON history.

Canonical design nên dùng:

> **SQLite local single-file database**

Chỉ metadata/state được lưu.

Media bytes vẫn ở filesystem.

## Minimal schema

```text
jobs
----
id
parent_id
source_url
source_key
status
operation
output_profile
plan_json
metadata_json
error_json
output_directory
created_at
updated_at
completed_at

playlist_items
--------------
parent_job_id
media_id
playlist_index
title
selected
override_json
status

artifacts
---------
id
job_id
role
path
verification_json
exists
created_at

schema_meta
-----------
schema_version
```

Không normalize tất cả metadata thành 50 columns.

Complex evolving structures giữ dạng JSON.

Relational columns chỉ dành cho:

* identity;
* state;
* filtering;
* relations;
* recovery.

## SQLite rationale

So với mutable JSON history:

* transaction;
* crash consistency;
* queue queries;
* playlist children;
* retries;
* schema migration;
* duplicate lookup;
* missing-file tracking;

đều đơn giản và đáng tin cậy hơn.

---

# 17. Crash/restart recovery

yt-dlp mặc định hỗ trợ `.part` và continuing partial downloads.

Application startup:

```text
load persisted jobs
      ↓
find PREPARING / DOWNLOADING / POST_PROCESSING / VERIFYING
      ↓
mark INTERRUPTED
      ↓
inspect artifact / .part
      ↓
show:
"Interrupted download found"
      ↓
[Resume] [Restart] [Discard]
```

Default không tự tạo network activity ngay khi app mở.

User chooses Resume.

---

# 18. Failure isolation

Một item fail **không được làm queue chết**.

Worker invariant:

```text
execute job
   ↓
success? → COMPLETED
failure? → FAILED
   ↓
persist state
   ↓
continue next QUEUED job
```

Queue chỉ dừng toàn cục khi:

* download directory inaccessible;
* disk full globally;
* required tool invalid/missing;
* database failure;
* explicit user stop.

yt-dlp cũng mặc định có behavior tiếp tục video kế tiếp khi một playlist item lỗi.

---

# 19. Error taxonomy

Không lưu mỗi:

```text
error_message: String
```

Target:

```rust
enum ErrorCode {
    InvalidInput,
    UnsupportedSource,
    MediaUnavailable,

    AuthenticationRequired,
    AccessDenied,
    GeoRestricted,
    DrmProtected,

    NetworkFailure,
    Timeout,
    RateLimited,

    ExtractorFailure,
    ToolOutdated,
    ToolMissing,

    FormatUnavailable,
    QualityUnavailable,

    DiskFull,
    PermissionDenied,
    OutputConflict,

    PostProcessingFailed,
    VerificationFailed,

    Cancelled,
    Unknown,
}
```

Error object:

```rust
struct AppError {
    code: ErrorCode,
    user_message: String,
    technical_summary: String,
    retriable: bool,
    suggested_actions: Vec<RecoveryAction>,
}
```

Example:

```text
Download failed

YouTube requires authentication for this media.

Recommended:
[Use browser session]

Other options:
[Try another account]
[View technical details]
```

thay vì:

```text
ERROR: Sign in to confirm you're not a bot
```

---

# 20. Engine health

`yt-dlp` extractors thay đổi thường xuyên.

Không nên blindly retry cùng command.

Flow:

```text
ExtractorFailure
      ↓
check installed yt-dlp health/version
      ↓
known newer approved version?
      ↓
yes
      ↓
Offer:
"Update media engine and retry"
```

Không silent update binary trong lúc download.

Existing cryptographic tool management nên tiếp tục là trust boundary.

Recommended:

```text
stable approved tool manifest
      ↓
download
      ↓
SHA-256 verify
      ↓
atomic install
      ↓
retry
```

Self-update không được bypass pinned/verified supply-chain model.

---

# 21. Authentication Assistant

yt-dlp hỗ trợ browser cookie extraction trực tiếp qua `--cookies-from-browser`, bao gồm Chrome, Chromium, Edge, Firefox, Brave và các browser khác.

UI:

```text
This media requires a signed-in session.

Use login from:

[ Chrome ▼ ]

Profile:
[ Default ▼ ]

Cookies remain local and are passed directly to the media engine.

[Test access]
```

## Rules

* no cookies by default;
* no cookie values in React;
* no cookie values in database;
* no cookie values in logs;
* browser/profile preference may be stored;
* cookie content is resolved only when executing;
* diagnostic sanitizer must redact auth information;
* test via lightweight/simulated extraction;
* no DRM bypass.

If cookie file import được hỗ trợ:

```text
path only
```

và không copy cookie content vào application DB.

---

# 22. Clip / selected time range

Model:

```rust
Clip {
    start_ms,
    end_ms,
}
```

Validation:

```text
0 <= start < end <= duration
```

UI:

```text
Clip

Start   01:13:20.000
End     01:16:45.000

Duration 03:25
```

Backend có thể sử dụng `--download-sections`, được yt-dlp hỗ trợ cho timestamp hoặc chapter và cần FFmpeg.

## Important processing distinction

Default:

### Fast cut

* preserve streams where possible;
* cut boundary may follow nearby keyframe;
* cheap.

Advanced:

### Exact cut

* force accurate boundary;
* may require video re-encode;
* expensive.

yt-dlp explicitly notes forcing keyframes around cuts is slower because it requires re-encoding.

UI must therefore say:

```text
Fast cut
No forced video re-encoding

or

Exact cut
⚠ Video will be re-encoded
```

Không được silently re-encode chỉ để đạt timestamp chính xác.

---

# 23. Chapter acquisition

Current metadata already contains `MediaChapter`.

UI:

```text
Chapters

○ Entire media
● 03 — Transformer Architecture        12:41 → 24:18
○ 04 — Training                       24:18 → 41:02

[Download chapter]
```

Two operations:

```text
Download one chapter
Split all chapters
```

yt-dlp hỗ trợ cả chapter selection/time-range và `--split-chapters`.

Naming:

```text
03 - Transformer Architecture.ext
04 - Training.ext
```

sanitized through current path validator.

---

# 24. Thumbnail-only acquisition

Thumbnail phải là first-class operation:

```text
ThumbnailOnly
```

không phải hack từ:

```text
embed_thumbnail = true
```

UI:

```text
Thumbnail

Source
● Best available
○ Select thumbnail

Format
● Original
○ JPG
○ PNG
○ WebP

[Save thumbnail]
```

yt-dlp hỗ trợ write thumbnail và thumbnail conversion sang JPG/PNG/WebP.

Artifacts phải được tracked:

```text
artifact.role = THUMBNAIL
```

---

# 25. Cover crop 1:1

Không thêm computer vision.

Simple deterministic crop:

```text
source thumbnail
      ↓
center square crop
      ↓
optional resize
      ↓
JPG / PNG / WebP
```

Use existing FFmpeg.

Modes:

```text
Keep original
Square center crop
```

Không face-detection, AI crop hoặc object tracking.

---

# 26. Audio-source preservation

`Best Source + Audio Only`:

```text
select best source audio
      ↓
extract
      ↓
no codec conversion where possible
```

Examples:

```text
Opus → Opus
AAC → M4A/AAC
FLAC → FLAC
```

Do not force:

```text
Opus → M4A
AAC → Opus
```

unless user selected a compatibility profile.

---

# 27. MP3 and FLAC semantics

## MP3

UI:

```text
MP3
Compatibility-oriented lossy conversion.

Output encoder quality does not imply the
source contained equivalent information.
```

## FLAC

UI:

```text
FLAC output

Lossless encoding from decoded audio.
If the source is Opus/AAC/etc., information
already lost in the source cannot be recovered.
```

This truthfulness must be maintained consistently in:

* preset card;
* plan;
* README;
* help;
* completed receipt.

---

# 28. Audio-language selection

Extend `AudioStreamSpec` rather than invent separate structures.

UI:

```text
Audio track

● Original / Default
○ Vietnamese
○ English
○ Japanese
```

But `"Original"` may only be shown if extractor metadata actually identifies a default/original track.

Otherwise:

```text
Unknown
Default
Vietnamese
English
```

Backend selection priority:

```text
explicit selected track
   ↓
selected language
   ↓
extractor default
   ↓
best audio
```

Never infer language from title alone unless explicitly marked heuristic.

---

# 29. Subtitle model

Current subtitle support becomes:

```text
SubtitleSelection {
    mode:
      None
      Embed
      Separate

    languages: [...]
    include_auto_generated: bool
}
```

Basic mode:

```text
Subtitles
Off / Original / Preferred language
```

Advanced:

* multiple languages;
* manual vs automatic;
* embed/separate;
* format conversion.

---

# 30. Size estimation

Estimated file size must carry confidence.

```rust
enum EstimateConfidence {
    Exact,
    Strong,
    Approximate,
    Unknown,
}
```

UI:

```text
Estimated output

≈ 245 MB
Estimate based on source stream sizes
```

or:

```text
Size unavailable
The source does not expose enough information.
```

Never show fake precision:

```text
245.12 MB
```

when calculation is approximate.

---

# 31. Disk preflight

Before enqueue/start:

```text
estimated bytes known?
       ↓
yes
       ↓
available disk space
       ↓
estimated + safety margin <= free?
```

If insufficient:

```text
Not enough disk space

Expected: ~8.2 GB
Available: 5.7 GB

[Choose another folder]
```

Unknown estimated size:

* do not block;
* allow download;
* show uncertainty.

---

# 32. Duplicate policy

Do **not** use simple:

```text
"source ID exists in history → skip"
```

because file may have been deleted.

Canonical duplicate key:

```text
extractor
+
media_id
+
operation scope
```

Examples:

```text
youtube:ABC:entire
youtube:ABC:chapter:3
youtube:ABC:clip:10000-30000
youtube:ABC:thumbnail
```

Before acquisition:

```text
history contains successful artifact?
        ↓
does artifact still exist?
```

Then policy:

```text
Skip
Replace
Save Copy
Redownload
```

Default when existing file is valid:

```text
Ask
```

When DB record exists but file is missing:

```text
Artifact missing
→ allow Redownload
```

Do not blindly rely on yt-dlp `--download-archive`, even though upstream supports it, because archive membership alone cannot tell whether the user's file still exists.

Existing `MediaFingerprint` can later strengthen duplicate detection.

---

# 33. Metadata editor

Metadata editing belongs **before execution plan finalization**.

For audio:

```text
Title
Artist
Album
Album Artist
Track
Disc
Date
Genre
```

For generic video:

```text
Title
Description
Comment/source URL
```

Show:

```text
SOURCE METADATA        EDITED OUTPUT
-------------------------------------
Title A                Title B
Uploader X             Artist X
```

Source remains immutable.

User edits are represented as:

```rust
MetadataPatch
```

rather than modifying `MediaMetadata`.

---

# 34. Filename preview

Filename is an independent concern.

Show pre-download:

```text
Output filename

03 - Example Artist - Example Track.opus
```

Changing tags must not accidentally alter path rules unless output template specifically references them.

Basic mode should expose safe templates:

```text
Title
Uploader - Title
Playlist Index - Title
Artist - Title
```

Advanced custom templates can come later.

All paths still pass current sanitization logic.

---

# 35. Quick Tools subsystem

Quick Tools should use **the same planning/execution/verification concepts** as acquisition.

Do not create a second ad-hoc FFmpeg runner in React.

```text
Existing file
    ↓
TransformRequest
    ↓
TransformPlan
    ↓
FFmpeg execution
    ↓
Verification
    ↓
New Artifact
```

Supported:

| Tool              | Default behavior                                   |
| ----------------- | -------------------------------------------------- |
| Trim              | stream-copy when practical                         |
| Extract Audio     | preserve source audio first                        |
| Merge Audio Clips | stream concat when compatible                      |
| Save Thumbnail    | extract/copy                                       |
| Crop Cover        | deterministic square crop                          |
| Edit Metadata     | copy streams                                       |
| Convert / Remux   | remux if compatible, transcode only when necessary |
| Split Chapters    | chapter-aware split                                |

---

# 36. Non-destructive Quick Tools rule

Default:

```text
input.ext
   ↓
transform
   ↓
input-trimmed.ext
```

Never overwrite original by default.

`Replace original` belongs in Advanced mode and requires explicit confirmation.

---

# 37. Trim behavior

Two modes:

```text
Fast Trim
- stream copy
- fast
- no generational loss
- cuts may align near keyframes
```

```text
Precise Trim
- re-encode where required
- slower
- exact boundary
```

This is exactly the same semantic distinction as clip acquisition.

---

# 38. Merge audio clips

Before merge:

```text
codec
sample rate
channel layout
container
```

compatible?

### Yes

```text
concat / stream-copy
```

### No

Planner proposes normalization/transcode:

```text
These clips cannot be joined without conversion.

Output:
AAC 48 kHz Stereo

⚠ Audio transcode
```

No silent conversion.

---

# 39. Convert vs Remux

UI should not expose only `"Convert"`.

Plan distinguishes:

```text
WEBM VP9 + Opus
       ↓
MKV
```

= remux.

versus:

```text
AV1 + Opus
    ↓
MP4 H.264 + AAC
```

= full transcode.

This distinction must be visible before execution.

---

# 40. Completion workflow

Completed receipt:

```text
✓ Verified

Example Video.mp4
1920×1080 · H.264 · AAC
93.4 MB

Source
1440p VP9 · Opus

Processing
Video transcoded
Audio transcoded

[Open]
[Show in Folder]
[Copy Path]
[Share]

Quick Tools
[Trim] [Extract Audio] [Convert]
```

`Share` should use platform-native share capability where supported; otherwise fallback to revealing/copying file path rather than implementing custom upload infrastructure.

---

# 41. Basic vs Advanced UX

Basic mode should answer user intent.

```text
What do you want?

● Entire Video
○ Clip
○ Audio
○ Thumbnail
○ Chapter
○ Subtitles
```

Then:

```text
Output

● Best Source
○ Universal
○ Editing
○ Small
```

Advanced mode contains technical controls.

This is preferable to putting:

```text
H264
H265
VP9
AV1
AAC
Opus
CBR
VBR
MKV
MP4
WEBM
...
```

trực tiếp vào main workflow.

---

# 42. Proposed primary screen

```text
┌──────────────────────────────────────────────────────┐
│ URL                                                  │
│ [ https://youtube.com/...                         ]  │
├──────────────────────────────────────────────────────┤
│ [ thumbnail ]  3 Hour Podcast                       │
│                Channel · 03:14:53                   │
│                YouTube                              │
├──────────────────────────────────────────────────────┤
│ WHAT DO YOU WANT?                                    │
│ ● Entire   ○ Clip   ○ Audio   ○ Thumbnail           │
│ ○ Chapter  ○ Subtitles                              │
├──────────────────────────────────────────────────────┤
│ OUTPUT                                               │
│ ● Best Source  ○ Universal  ○ Editing  ○ Small      │
├──────────────────────────────────────────────────────┤
│ PLAN                                                 │
│ Source                                               │
│ 1440p60 · VP9 · Opus                                │
│                                                      │
│ Planned output                                      │
│ 1440p60 · VP9 · Opus · MKV                         │
│                                                      │
│ ✓ Video preserved                                   │
│ ✓ Audio preserved                                   │
│ Merge only · No re-encoding                         │
│                                                      │
│ Estimated size ~1.8 GB                              │
├──────────────────────────────────────────────────────┤
│ [Advanced / Details]                                │
│                                                      │
│                [ Add to Queue ]                     │
└──────────────────────────────────────────────────────┘
```

---

# 43. Playlist screen

```text
Playlist
Machine Learning Course
68 items

Analysis mode:
● Fast
○ Customize each item

[x] 01  Introduction                 12:11
[x] 02  Linear Algebra               44:32
[ ] 03  Probability                  38:10
[x] 04  Neural Networks              51:29

Selected: 3 / 68

Output:
Best Source

[Add Selected to Queue]
```

Per-item override hidden until:

```text
Customize each item
```

---

# 44. Persistent history UX

History filters:

```text
All
Completed
Failed
Interrupted
Skipped
```

Each failed item displays:

```text
FAILED

Title
Extractor error

Recommended:
Update yt-dlp and retry

[Retry]
[Details]
```

Not merely a red icon.

---

# 45. Backend target architecture

Preserve existing core.

```text
React / TypeScript
│
│ typed IPC
▼
Tauri commands
│
▼
Rust Core
├── analyzer.rs
├── media_graph.rs
├── universal_resolver.rs
├── planner.rs              NEW
├── presets.rs              EVOLVE → output-profile policy
├── download_manager.rs
├── queue.rs                NEW
├── job_store.rs            NEW
├── error.rs                NEW
├── auth.rs                 NEW
├── quick_tools.rs          NEW
├── media_verifier.rs
├── fingerprint.rs
├── diagnostics.rs
├── tool_manager.rs
├── process_runner.rs
├── path_validator.rs
└── types.rs
```

Không cần tạo 30 service abstractions.

---

# 46. Module responsibilities

## `analyzer.rs`

Only:

* source extraction;
* source metadata normalization.

No user output decision.

---

## `media_graph.rs`

Canonical source representation:

```text
video streams
audio streams
subtitles
chapters
thumbnails
formats
```

---

## `planner.rs`

**Most important new module.**

Input:

```text
SourceMediaGraph
+
AcquisitionRequest
+
AppSettings
```

Output:

```text
AcquisitionPlan
```

Responsibilities:

* stream selection;
* profile policy;
* processing classification;
* size estimation;
* warnings;
* expected artifact;
* preflight requirements.

---

## `presets.rs`

Evolve from hardcoded download options into:

```text
OutputProfile policy compiler
```

Do not directly become UI semantics.

---

## `download_manager.rs`

Only execution lifecycle.

It must not independently make quality decisions that disagree with Planner.

---

## `media_verifier.rs`

Authoritative post-download representation.

Actual output always wins over planned output.

---

## `queue.rs`

Responsibilities:

```text
enqueue
next
retry
cancel
continue after failure
startup recovery
```

No persistence implementation details.

---

## `job_store.rs`

Responsibilities:

```text
SQLite transactions
schema migrations
job serialization
artifact records
playlist children
```

---

## `error.rs`

Responsibilities:

```text
stderr/exit → ErrorCode
ErrorCode → recovery options
```

---

## `auth.rs`

Responsibilities:

```text
browser/profile detection
auth option construction
test-access flow
credential redaction boundaries
```

---

## `quick_tools.rs`

Build typed FFmpeg transforms.

Never accept arbitrary shell strings from UI.

---

# 47. IPC surface

Recommended production IPC:

```text
analyze_source(request)
resolve_plan(request)
enqueue_plan(plan_id)
list_jobs(filter)
retry_job(job_id)
cancel_job(job_id)
remove_job(job_id)

list_playlist_items(source_id)
analyze_playlist_item(item_id)

test_auth(auth_request)

get_engine_health()
update_engine()

plan_transform(request)
enqueue_transform(plan_id)

open_artifact(artifact_id)
reveal_artifact(artifact_id)
copy_artifact_path(artifact_id)
share_artifact(artifact_id)
```

Events:

```text
analysis.updated
job.updated
queue.updated
engine.health_changed
```

---

# 48. Remove raw command construction from product IPC

Current code exposes `BuildCommandRequest`/`BuildCommandResponse`.

Long-term production flow should **not** depend on frontend calling:

```text
build command
```

because that:

* leaks backend implementation;
* duplicates domain logic;
* makes frontend/backend drift easier;
* weakens plan authority.

Raw command preview may remain as:

```text
Developer diagnostics
```

but not product workflow.

---

# 49. Plan identity

Every plan gets:

```text
plan_id
plan_version
```

When user changes:

* profile;
* quality;
* clip range;
* language;
* subtitle;
* metadata;
* output path;

old plan becomes stale.

```text
request change
     ↓
invalidate plan
     ↓
resolve new plan
```

Executor must not execute stale UI state accidentally.

---

# 50. Error recovery should be deterministic

Examples:

## AuthenticationRequired

Actions:

```text
Use browser cookies
Retry
Details
```

## ToolOutdated

```text
Update engine
Retry
Details
```

## DiskFull

```text
Choose another location
Retry
```

## FormatUnavailable

```text
Re-analyze
Use Best Source
Advanced format selection
```

## VerificationFailed

```text
Keep file anyway
Retry
View technical inspection
```

Do not show irrelevant actions.

---

# 51. Security invariants

Existing security direction should remain.

Hard requirements:

```text
No shell interpolation
No arbitrary shell command from UI
HTTP/HTTPS only
SSRF protections remain
Sanitized paths
Bounded diagnostics
Credential redaction
Managed verified binaries
No cookie persistence
No silent DRM bypass
No unsafe file:// support
```

Playlist/thumbnail/auth features must not weaken these guarantees.

---

# 52. Performance invariants

Keep:

```text
No media bytes in React
No media bytes buffered in Rust
Disk streaming
Push-based events
Bounded diagnostics
Single active media job
```

Playlist additionally:

```text
Fast mode must not fully resolve formats for every item.
```

Plan recalculation should operate on already extracted metadata when possible.

---

# 53. Testing architecture

Tests should be dominated by deterministic fixtures, not live YouTube calls.

## Core unit tests

Planner matrix:

```text
source VP9 + Opus + BestSource
→ merge / preserve

source H264 + AAC + Universal
→ preserve

source AV1 + Opus + Universal
→ full transcode

AAC source + Audio + BestSource
→ source preserve

Opus source + Audio + Universal
→ MP3 transcode

lossy source + FLAC
→ audio transcode + quality warning
```

---

## Operation tests

```text
Entire
Clip
Chapter
Thumbnail
Audio
Subtitle
```

---

## Error-classifier fixtures

Use captured sanitized stderr:

```text
auth required
rate limit
no matching formats
disk full
permission denied
extractor error
DRM
network timeout
```

Expected typed `ErrorCode`.

---

## Queue tests

```text
A complete → B starts
A fail → B starts
A cancel → B starts
restart during A → A becomes INTERRUPTED
retry A → new execution attempt
```

---

## Persistence tests

```text
schema create
migration
transaction rollback
corrupted optional JSON
missing artifact
playlist child recovery
```

---

## Frontend tests

Pure UI contracts:

* planned vs actual terminology;
* no unknown value fabrication;
* transcode warning;
* single-video/playlist chooser;
* failed job recovery buttons;
* source-quality display.

---

# 54. CI gate

Minimum required before merge:

```bash
npm run lint
npm run build

cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Do not use live network extraction as a mandatory deterministic CI test.

Real URL tests belong in:

```text
optional smoke/manual test
```

because platform/extractor changes make them inherently flaky.

---

# 55. Recommended implementation sequence

## Slice 0 — Baseline Integrity

Deliverable:

```text
main builds and tests cleanly
```

Nothing else.

---

## Slice 1 — Canonical Plan

Implement:

```text
AcquisitionRequest
AcquisitionPlan
ProcessingClass
planner
```

Reuse current resolver.

Acceptance:

```text
one backend path determines
selected streams + output + processing
```

No UI quality heuristic.

---

## Slice 2 — Quality Transparency

Implement:

```text
Source
Planned Output
Processing
Warnings
Actual Output
Plan-vs-Actual Diff
```

This completes the current quality-transparency work already started in the repo.

---

## Slice 3 — Acquisition Operations

Implement:

```text
Entire
Clip
Audio
Thumbnail
Chapter
Subtitle
```

Output profiles:

```text
Best Source
Universal
Editing
Small
```

---

## Slice 4 — Reliability Core

Implement:

```text
SQLite JobStore
Persistent Queue
Error taxonomy
Failed History
Retry
Failure isolation
Interrupted recovery
```

---

## Slice 5 — Platform Recovery

Implement:

```text
Engine Health
yt-dlp update workflow
Authentication Assistant
Disk preflight
Duplicate policy
```

---

## Slice 6 — Playlist

Implement:

```text
scope detection
Fast Playlist
selected items
batch execution
lazy detailed analysis
per-item override
```

---

## Slice 7 — Media Controls

Implement:

```text
audio language
metadata editor
filename preview
cover crop
split chapters
```

---

## Slice 8 — Lightweight Finishing

Implement:

```text
Trim
Extract Audio
Merge Audio
Convert / Remux
Edit Metadata
Save/Crop Thumbnail
Split Chapters
Share/Open actions
```

At this point product boundary is complete.

---

# 56. Features intentionally deferred after Slice 8

Only after usage data demonstrates a real need:

```text
higher job concurrency
advanced batch automation
complex download rules
AI upscale
transcription
media library
cloud sync
plugin marketplace
```

No implementation merely vì “có thể làm”.

---

# 57. Three architectural decision gates

Before implementation crosses from current architecture into this full design, three decisions need explicit owner acceptance.

## Gate A — Product boundary

**Recommended:**

```text
Downloader
+
Lightweight Finishing
```

not Downloader-only.

Reason:

The requested workflow already includes clip, thumbnail, metadata, crop, remux, trim, split chapters and completion actions. Treating them as bounded finishing operations gives them a clean architectural home without becoming an editor.

---

## Gate B — Persistent state

**Recommended:**

```text
SQLite
```

for jobs/history/playlist/artifacts.

Not arbitrary JSON files.

Reason:

Reliable recovery, playlist children, retries and duplicate tracking justify transactional persistence.

---

## Gate C — Preset model

**Recommended:**

```text
AcquisitionOperation
+
OutputProfile
```

replacing the conceptual role of the current format-only preset model.

Current `PresetType` can remain temporarily as compatibility mapping during migration.

---

# 58. Definition of Done for the full product design

The complete feature set is successful when the following user journey works reliably:

```text
paste URL
   ↓
correctly identify video vs playlist
   ↓
show thumbnail/title/source
   ↓
choose entire/clip/audio/chapter/thumbnail/subtitle
   ↓
choose Best Source/Universal/Editing/Small
   ↓
see exact planned streams/output
   ↓
see whether anything will be transcoded
   ↓
see approximate size where possible
   ↓
enqueue
   ↓
download survives unrelated queue failures
   ↓
app restart can recover interruption
   ↓
auth/extractor failures produce actionable recovery
   ↓
verification tells user what file actually exists
   ↓
history retains failed and successful jobs
   ↓
duplicate behavior is explicit
   ↓
user can open/share or apply lightweight finishing
```

A product passing that sequence has moved meaningfully beyond:

> “GUI wrapper around yt-dlp”.

It becomes:

> **a local, transparent, reliable media acquisition and finishing system.**

---

# 59. Final canonical architecture

```text
                         ┌───────────────────────┐
                         │      React UI         │
                         │ Basic + Advanced UX   │
                         └───────────┬───────────┘
                                     │
                                  Typed IPC
                                     │
                    ┌────────────────▼────────────────┐
                    │          Tauri Commands         │
                    └────────────────┬────────────────┘
                                     │
               ┌─────────────────────▼─────────────────────┐
               │                 Rust Core                 │
               │                                           │
URL ─────────► │ Analyzer ─────► SourceMediaGraph          │
               │                       │                   │
User intent ─► │ Planner ──────────────┘                   │
               │    │                                      │
               │    ▼                                      │
               │ AcquisitionPlan                           │
               │    │                                      │
               │    ├────► Queue / JobStore               │
               │    │          │                           │
               │    │          ▼                           │
               │    └────► DownloadManager                │
               │               │                           │
               │               ▼                           │
               │          yt-dlp / FFmpeg                  │
               │               │                           │
               │               ▼                           │
               │          MediaVerifier                    │
               │               │                           │
               │               ▼                           │
               │        Verified Artifact                 │
               │               │                           │
               │               └────► Quick Tools         │
               │                                           │
               │ ErrorClassifier ◄── process failures     │
               │ AuthAssistant                            │
               │ ToolManager / EngineHealth               │
               └───────────────────────────────────────────┘
```

## Architectural invariant

Nếu chỉ giữ một nguyên tắc cho toàn bộ implementation, hãy giữ nguyên tắc này:

> **Frontend expresses intent; backend resolves intent into one authoritative plan; executor executes that exact plan; verifier reports reality.**

Nếu invariant này không bị phá vỡ, phần lớn feature phía trên có thể được thêm dần mà không biến codebase thành tập hợp condition/flag rời rạc.

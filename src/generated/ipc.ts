// Generated from core/src/types.rs. Do not edit.
// Regenerate: UPDATE_BINDINGS=1 cargo test --manifest-path src-tauri/crates/core/Cargo.toml ipc_bindings_are_current

export type SourceScope = "SINGLE_MEDIA";

export type AcquisitionOperation = { "type": "ENTIRE_MEDIA" } | { "type": "CLIP", startMs: number, endMs: number, } | { "type": "AUDIO_ONLY" } | { "type": "THUMBNAIL_ONLY" } | { "type": "CHAPTER", chapterIndex: number, } | { "type": "SUBTITLES_ONLY" };

export type OutputProfile = "BEST_SOURCE" | "UNIVERSAL" | "EDITING" | "SMALL" | "CUSTOM";

export type TrackSelection = { audioLanguage?: string | null, subtitleLanguages: Array<string>, includeAutoSubtitles: boolean, };

export type DuplicatePolicy = "RENAME" | "SKIP" | "OVERWRITE";

export type MetadataPatch = { title?: string | null, artist?: string | null, album?: string | null, };

export type AcquisitionRequest = { sourceScope: SourceScope, operation: AcquisitionOperation, outputProfile: OutputProfile, trackSelection: TrackSelection, metadataPatch?: MetadataPatch | null, duplicatePolicy: DuplicatePolicy, outputDirectory: string, 
/**
 * Optional user constraint; `None` means the best suitable source height.
 */
maxVideoHeight?: number | null, };

export type SourceSummary = { url: string, title: string, extractor: string, mediaKind: MediaKind, durationSeconds?: number | null, };

export type SelectedStreams = { videoStreamId?: string | null, audioStreamId?: string | null, subtitleLanguages: Array<string>, };

export type PlannedArtifact = { container: string, videoCodec?: string | null, audioCodec?: string | null, width?: number | null, height?: number | null, fps?: number | null, audioOnly: boolean, };

export type ProcessingClass = "SOURCE_PRESERVED" | "MERGE_ONLY" | "REMUX_ONLY" | "AUDIO_TRANSCODE" | "VIDEO_TRANSCODE" | "FULL_TRANSCODE" | "UNKNOWN";

export type ProcessingPlan = { class: ProcessingClass, requiresFfmpeg: boolean, steps: Array<string>, };

export type SizeEstimate = { bytes: number, confidence: string, };

export type PlanWarning = { code: string, message: string, };

export type PlanRequirement = { code: string, message: string, };

export type AcquisitionPlan = { transforms: Array<PlannedTransform>, includeAutoSubtitles: boolean, postProcess: PostProcessPolicy, timeRangeMs?: [number, number] | null, id: string, version: number, source: SourceSummary, scope: SourceScope, operation: AcquisitionOperation, outputProfile: OutputProfile, selectedStreams: SelectedStreams, output: PlannedArtifact, processing: ProcessingPlan, estimatedSize?: SizeEstimate | null, warnings: Array<PlanWarning>, requirements: Array<PlanRequirement>, };

export type PlannedTransform = { "type": "MERGE" } | { "type": "REMUX", container: string, } | { "type": "EXTRACT_AUDIO", format: string, } | { "type": "TRANSCODE_VIDEO", codec: string, } | { "type": "TRANSCODE_AUDIO", codec: string, } | { "type": "TRIM", startMs: bigint, endMs: bigint, };

export type PostProcessPolicy = { embedMetadata: boolean, embedThumbnail: boolean, embedChapters: boolean, subtitleMode: SubtitleMode, sponsorBlockMode: SponsorBlockMode, };

export type MediaSourceType = "YT_DLP_EXTRACTOR" | "YT_DLP_GENERIC" | "DIRECT_FILE" | "HLS" | "DASH" | "UNSUPPORTED" | "INACCESSIBLE";

export type DownloadStrategy = "DIRECT_COPY" | "YT_DLP_DOWNLOAD" | "YT_DLP_MERGE" | "FFMPEG_REMUX" | "FFMPEG_TRANSCODE" | "HLS_DOWNLOAD" | "DASH_DOWNLOAD";

export type TranscodingCost = "NO_PROCESSING" | "STREAM_COPY" | "REMUX" | "MERGE" | "TRANSCODE";

export type ResolverErrorCategory = "NONE" | "INVALID_URL" | "UNSUPPORTED_PROTOCOL" | "NO_MEDIA_FOUND" | "REQUIRES_AUTHENTICATION" | "DRM_PROTECTED" | "NETWORK_UNREACHABLE" | "TIMEOUT" | "EXTRACTOR_FAILED";

export type ResolverErrorDetail = { category: ResolverErrorCategory, technicalMessage: string, userFriendlyMessage: string, httpStatus?: number | null, };

export type MediaCapabilities = { video: boolean, audio: boolean, subtitles: boolean, chapters: boolean, thumbnails: boolean, metadataEmbedding: boolean, containerSupport: Array<string>, transcodingRequired: boolean, };

export type PresetType = "mp4-compatible" | "best-video" | "best-audio" | "mp3" | "flac";

export type MediaKind = "video" | "audio" | "livestream";

export type SponsorBlockMode = "off" | "mark-chapters" | "remove-segments";

export type SubtitleMode = "none" | "embed" | "download-separate";

export type SubtitleTrack = { language: string, name?: string | null, ext?: string | null, isAuto?: boolean | null, };

export type MediaChapter = { title: string, startTime: number, endTime: number, };

export type MediaFormatSpec = { language?: string | null, formatId: string, ext: string, resolution?: string | null, width?: number | null, height?: number | null, fps?: number | null, vcodec?: string | null, acodec?: string | null, filesize?: number | null, filesizeApprox?: number | null, tbr?: number | null, vbr?: number | null, abr?: number | null, hdr?: boolean | null, dynamicRange?: string | null, audioSampleRate?: number | null, audioChannels?: number | null, };

export type UserIntent = "max-quality" | "smallest-size" | "best-compatibility" | "balanced";

export type RecommendationConstraints = { maxFilesizeBytes?: number | null, minHeight?: number | null, preferredFps?: number | null, preferHdr?: boolean | null, avoidTranscoding: boolean, };

export type VideoStreamSpec = { streamId: string, codec: string, profile?: string | null, width: number, height: number, fps?: number | null, bitrateKbps?: number | null, isHdr: boolean, dynamicRange?: string | null, aspectRatio?: string | null, filesizeApprox?: number | null, };

export type AudioStreamSpec = { streamId: string, codec: string, bitrateKbps?: number | null, sampleRateHz?: number | null, channels?: number | null, language?: string | null, isDefault: boolean, filesizeApprox?: number | null, };

export type ThumbnailSpec = { url: string, width?: number | null, height?: number | null, id?: string | null, };

export type SourceMediaGraph = { sourceUrl: string, extractor: string, sourceType: MediaSourceType, title: string, mediaKind: MediaKind, durationSeconds?: number | null, videoStreams: Array<VideoStreamSpec>, audioStreams: Array<AudioStreamSpec>, subtitleStreams: Array<SubtitleTrack>, chapters: Array<MediaChapter>, thumbnails: Array<ThumbnailSpec>, formats: Array<MediaFormatSpec>, };

export type OutputMediaArtifact = { artifactPath: string, fileName: string, container: string, fileSizeBytes: number, durationSeconds?: number | null, videoCodec?: string | null, audioCodec?: string | null, width?: number | null, height?: number | null, fps?: number | null, isVerified: boolean, };

export type FormatRecommendation = { preset: PresetType, label: string, targetQuality: string, reason: string, whyReasons: Array<string>, isTranscodeFree: boolean, transcodingCost: TranscodingCost, estimatedSizeBytes?: number | null, container: string, details?: string | null, };

export type MediaMetadata = { id: string, title: string, uploader?: string | null, uploaderAvatar?: string | null, channelId?: string | null, uploaderUrl?: string | null, duration?: number | null, thumbnail?: string | null, webpageUrl: string, mediaKind: MediaKind, uploadDate?: string | null, releaseTimestamp?: number | null, viewCount?: number | null, likeCount?: number | null, description?: string | null, categories?: Array<string> | null, tags?: Array<string> | null, language?: string | null, isLive?: boolean | null, wasLive?: boolean | null, extractor?: string | null, extractorKey?: string | null, playlistTitle?: string | null, playlistIndex?: number | null, playlistCount?: number | null, availableResolutions: Array<number>, availableFrameRates: Array<number>, hasVideo: boolean, hasAudio: boolean, isHdr?: boolean | null, subtitles?: Array<SubtitleTrack> | null, automaticCaptions?: Array<SubtitleTrack> | null, chapters?: Array<MediaChapter> | null, formats?: Array<MediaFormatSpec> | null, smartRecommendation?: FormatRecommendation | null, sourceType?: MediaSourceType | null, strategy?: DownloadStrategy | null, transcodingCost?: TranscodingCost | null, transcodingExplanation?: string | null, capabilities?: MediaCapabilities | null, };

export type ResolvedMediaSource = { sourceType: MediaSourceType, extractor?: string | null, extractorKey?: string | null, webpageUrl: string, title: string, mediaKind: MediaKind, capabilities: MediaCapabilities, candidates: Array<MediaFormatSpec>, strategy: DownloadStrategy, transcodingCost: TranscodingCost, transcodingExplanation: string, metadata?: MediaMetadata | null, errorDetail?: ResolverErrorDetail | null, isResolved: boolean, };

export type DownloadStatus = "IDLE" | "ANALYZING" | "READY" | "DOWNLOADING" | "POST_PROCESSING" | "VERIFYING" | "COMPLETED" | "FAILED" | "CANCELLING" | "CANCELLED";

export type DownloadProgress = { percentage: number, downloadedBytes: number, totalBytes: number, speedBytesPerSec: number, etaSeconds?: number | null, currentSpeed: string, rawStatusLine: string, };

export type MediaInspection = { verificationLevel: VerificationLevel, containerFormat: string, videoCodec?: string | null, videoProfile?: string | null, audioCodec?: string | null, width?: number | null, height?: number | null, fps?: number | null, bitDepth?: number | null, colorSpace?: string | null, isHdr?: boolean | null, bitrateKbps?: number | null, audioChannels?: number | null, audioSampleRateHz?: number | null, audioBitrateKbps?: number | null, audioLanguage?: string | null, fileSizeBytes: number, durationSeconds?: number | null, isLossyTranscodeWarning: boolean, streamCount?: number | null, chaptersCount?: number | null, };

export type VerificationLevel = "VERIFIED" | "BASIC_INSPECTION" | "UNVERIFIED";

export type SourceFingerprint = { extractor: string, sourceUrl: string, sourceId: string, title: string, };

export type StreamFingerprint = { streamType: string, codec: string, profile?: string | null, dimensionsOrChannels?: string | null, rate: string | null, };

export type MediaFingerprint = { canonicalId: string, source: SourceFingerprint, durationSeconds?: number | null, videoCodec?: string | null, audioCodec?: string | null, maxResolution?: string | null, streamCount: number, streams: Array<StreamFingerprint>, fileHash: string | null, createdAt: string, };

export type VerificationChecklist = { fileExists: boolean, fileSizeValid: boolean, durationValid: boolean, videoStreamValid: boolean, audioStreamValid: boolean, containerValid: boolean, verifiedAt: string, notes: Array<string>, };

export type VerificationResult = { planVerification: PlanVerification, isValid: boolean, verificationLevel: VerificationLevel, checklist: VerificationChecklist, outputArtifact?: OutputMediaArtifact | null, fingerprint?: MediaFingerprint | null, };

export type PlanVerification = { conforms: boolean, mismatches: Array<PlanMismatch>, warnings: Array<string>, };

export type PlanMismatch = { field: string, planned: string, actual?: string | null, };

export type DownloadRecipe = { id: string, sourceUrl: string, resolverType: MediaSourceType, extractor: string, selectedCandidates: Array<string>, strategy: DownloadStrategy, outputContainer: string, transformations: Array<string>, intent?: UserIntent | null, verification?: VerificationChecklist | null, timestamp: string, resultingArtifactPath?: string | null, };

export type ExplainableResult = { title: string, specsLabel: string, whyReasons: Array<string>, processingSummary: string, transcodingCost: TranscodingCost, verificationChecklist: VerificationChecklist, recipeId?: string | null, };

export type SubtitleOptions = { mode: SubtitleMode, selectedLanguage?: string | null, };

export type DownloadJob = { id: string, url: string, outputDirectory: string, status: DownloadStatus, progress: DownloadProgress, metadata: MediaMetadata, finalFileName?: string | null, finalFilePath?: string | null, inspection?: MediaInspection | null, errorMessage?: string | null, createdAt: string, completedAt?: string | null, subtitleOptions?: SubtitleOptions | null, sponsorBlockMode?: SponsorBlockMode | null, intent?: UserIntent | null, recipe?: DownloadRecipe | null, fingerprint?: MediaFingerprint | null, explainableResult?: ExplainableResult | null, verification?: VerificationResult | null, acquisitionPlan: AcquisitionPlan, };

export type AppSettings = { downloadDirectory: string, lastPreset: PresetType, defaultQuality: string, openFolderAfterDownload: boolean, autoAnalyzeOnPaste: boolean, embedMetadata: boolean, embedThumbnail: boolean, embedChapters: boolean, concurrentFragments: number, trimFilenames: number, sponsorBlockMode: SponsorBlockMode, subtitleMode: SubtitleMode, preferredSubtitleLanguage: string, customYtdlpPath?: string | null, customFfmpegPath?: string | null, customFfprobePath?: string | null, customMediainfoPath?: string | null, };

export type ToolStatus = "READY" | "MISSING" | "INSTALLING" | "INVALID" | "OUTDATED" | "ERROR";

export type ToolStatusInfo = { name: string, status: ToolStatus, version?: string | null, pinnedVersion: string, path?: string | null, managed: boolean, sourceUrl?: string | null, sha256?: string | null, errorMessage?: string | null, license: string, licenseUrl: string, isRequired: boolean, };

export type ToolHealth = { name: string, available: boolean, path?: string | null, version?: string | null, repairMessage?: string | null, };

export type ToolManifestEntry = { name: string, version: string, path: string, sha256: string, installedAt: string, verified: boolean, };

export type ToolsManifest = { schemaVersion: number, tools: { [key in string]?: ToolManifestEntry }, lastUpdated: string, };

export type DiagnosticLog = { id: string, timestamp: string, level: string, source: string, message: string, };

export type StartDownloadRequest = { expectedPlanId: string, metadata: MediaMetadata, acquisition: AcquisitionRequest, };

export type BuildCommandRequest = { metadata: MediaMetadata, acquisition: AcquisitionRequest, settings?: AppSettings | null, };

export type BuildCommandResponse = { command: string, arguments: Array<string>, };

export type WorkspaceItem = { id: string, url: string, title: string, addedAt: string, status: string, recipeId?: string | null, fingerprintId?: string | null, };

export type MediaWorkspace = { id: string, name: string, items: Array<WorkspaceItem>, createdAt: string, updatedAt: string, };

export type DuplicateCandidate = { sourceId: string, candidateId: string, matchConfidence: number, reason: string, };

export type MediaDiff = { originalFingerprint: MediaFingerprint, downloadedFingerprint: MediaFingerprint, durationDiffSeconds: number, resolutionChanged: boolean, codecChanged: boolean, containerChanged: boolean, isExactMatch: boolean, };

export type ArchivePolicy = { autoDeduplicate: boolean, keepHighestQuality: boolean, exportNfoMetadata: boolean, embedProvenanceRecipe: boolean, };

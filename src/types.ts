export type MediaSourceType =
  | 'YT_DLP_EXTRACTOR'
  | 'YT_DLP_GENERIC'
  | 'DIRECT_FILE'
  | 'HLS'
  | 'DASH'
  | 'UNSUPPORTED'
  | 'INACCESSIBLE';

export type DownloadStrategy =
  | 'DIRECT_COPY'
  | 'YT_DLP_DOWNLOAD'
  | 'YT_DLP_MERGE'
  | 'FFMPEG_REMUX'
  | 'FFMPEG_TRANSCODE'
  | 'HLS_DOWNLOAD'
  | 'DASH_DOWNLOAD';

export type TranscodingCost =
  | 'NO_PROCESSING'
  | 'STREAM_COPY'
  | 'REMUX'
  | 'MERGE'
  | 'TRANSCODE';

export type PresetType =
  | 'mp4-compatible'
  | 'best-video'
  | 'best-audio'
  | 'mp3'
  | 'flac';

export type MediaKind = 'video' | 'audio' | 'livestream';
export type UserIntent = 'max-quality' | 'smallest-size' | 'best-compatibility' | 'balanced';
export type SponsorBlockMode = 'off' | 'mark-chapters' | 'remove-segments';
export type SubtitleMode = 'none' | 'embed' | 'download-separate';

export interface SubtitleTrack {
  language: string;
  name?: string | null;
  ext?: string | null;
  isAuto?: boolean | null;
}

export interface MediaChapter {
  title: string;
  startTime: number;
  endTime: number;
}

export interface MediaFormatSpec {
  formatId: string;
  ext: string;
  resolution?: string | null;
  width?: number | null;
  height?: number | null;
  fps?: number | null;
  vcodec?: string | null;
  acodec?: string | null;
  filesize?: number | null;
  filesizeApprox?: number | null;
  tbr?: number | null;
  vbr?: number | null;
  abr?: number | null;
  hdr?: boolean | null;
  dynamicRange?: string | null;
  audioSampleRate?: number | null;
  audioChannels?: number | null;
}

export interface MediaCapabilities {
  video: boolean;
  audio: boolean;
  subtitles: boolean;
  chapters: boolean;
  thumbnails: boolean;
  metadataEmbedding: boolean;
  containerSupport: string[];
  transcodingRequired: boolean;
}

export interface FormatRecommendation {
  preset: PresetType;
  label: string;
  targetQuality: string;
  reason: string;
  whyReasons: string[];
  isTranscodeFree: boolean;
  transcodingCost: TranscodingCost;
  estimatedSizeBytes?: number | null;
  container: string;
  details?: string | null;
}

export interface MediaMetadata {
  id: string;
  title: string;
  uploader?: string | null;
  uploaderAvatar?: string | null;
  channelId?: string | null;
  uploaderUrl?: string | null;
  duration?: number | null;
  thumbnail?: string | null;
  webpageUrl: string;
  mediaKind: MediaKind;
  uploadDate?: string | null;
  releaseTimestamp?: number | null;
  viewCount?: number | null;
  likeCount?: number | null;
  description?: string | null;
  categories?: string[] | null;
  tags?: string[] | null;
  language?: string | null;
  isLive?: boolean | null;
  wasLive?: boolean | null;
  extractor?: string | null;
  extractorKey?: string | null;
  playlistTitle?: string | null;
  playlistIndex?: number | null;
  playlistCount?: number | null;
  availableResolutions: number[];
  availableFrameRates: number[];
  hasVideo: boolean;
  hasAudio: boolean;
  isHdr?: boolean | null;
  subtitles?: SubtitleTrack[] | null;
  automaticCaptions?: SubtitleTrack[] | null;
  chapters?: MediaChapter[] | null;
  formats?: MediaFormatSpec[] | null;
  smartRecommendation?: FormatRecommendation | null;
  sourceType?: MediaSourceType | null;
  strategy?: DownloadStrategy | null;
  transcodingCost?: TranscodingCost | null;
  transcodingExplanation?: string | null;
  capabilities?: MediaCapabilities | null;
}

export type DownloadStatus =
  | 'IDLE'
  | 'ANALYZING'
  | 'READY'
  | 'DOWNLOADING'
  | 'POST_PROCESSING'
  | 'VERIFYING'
  | 'COMPLETED'
  | 'FAILED'
  | 'CANCELLING'
  | 'CANCELLED';

export interface DownloadProgress {
  percentage: number;
  downloadedBytes: number;
  totalBytes: number;
  speedBytesPerSec: number;
  etaSeconds?: number | null;
  currentSpeed: string;
  rawStatusLine: string;
}

export interface MediaInspection {
  verificationLevel: VerificationLevel;
  containerFormat: string;
  videoCodec?: string | null;
  videoProfile?: string | null;
  audioCodec?: string | null;
  width?: number | null;
  height?: number | null;
  fps?: number | null;
  bitDepth?: number | null;
  colorSpace?: string | null;
  isHdr?: boolean | null;
  bitrateKbps?: number | null;
  audioChannels?: number | null;
  audioSampleRateHz?: number | null;
  audioBitrateKbps?: number | null;
  audioLanguage?: string | null;
  fileSizeBytes: number;
  durationSeconds?: number | null;
  isLossyTranscodeWarning: boolean;
  streamCount?: number | null;
  chaptersCount?: number | null;
}

export type VerificationLevel = 'VERIFIED' | 'BASIC_INSPECTION' | 'UNVERIFIED';

export interface SourceFingerprint {
  extractor: string;
  sourceUrl: string;
  sourceId: string;
  title: string;
}

export interface StreamFingerprint {
  streamType: string;
  codec: string;
  profile?: string | null;
  dimensionsOrChannels?: string | null;
  rate?: string | null;
}

export interface MediaFingerprint {
  canonicalId: string;
  source: SourceFingerprint;
  durationSeconds?: number | null;
  videoCodec?: string | null;
  audioCodec?: string | null;
  maxResolution?: string | null;
  streamCount: number;
  streams: StreamFingerprint[];
  fileHash?: string | null;
  createdAt: string;
}

export interface VerificationChecklist {
  fileExists: boolean;
  fileSizeValid: boolean;
  durationValid: boolean;
  videoStreamValid: boolean;
  audioStreamValid: boolean;
  containerValid: boolean;
  verifiedAt: string;
  notes: string[];
}

export interface OutputMediaArtifact {
  artifactPath: string;
  fileName: string;
  container: string;
  fileSizeBytes: number;
  durationSeconds?: number | null;
  videoCodec?: string | null;
  audioCodec?: string | null;
  width?: number | null;
  height?: number | null;
  fps?: number | null;
  isVerified: boolean;
}

export interface VerificationResult {
  isValid: boolean;
  verificationLevel: VerificationLevel;
  checklist: VerificationChecklist;
  outputArtifact?: OutputMediaArtifact | null;
  fingerprint?: MediaFingerprint | null;
}

export interface DownloadRecipe {
  id: string;
  sourceUrl: string;
  resolverType: MediaSourceType;
  extractor: string;
  selectedCandidates: string[];
  strategy: DownloadStrategy;
  outputContainer: string;
  transformations: string[];
  intent?: UserIntent | null;
  verification?: VerificationChecklist | null;
  timestamp: string;
  resultingArtifactPath?: string | null;
}

export interface ExplainableResult {
  title: string;
  specsLabel: string;
  whyReasons: string[];
  processingSummary: string;
  transcodingCost: TranscodingCost;
  verificationChecklist: VerificationChecklist;
  recipeId?: string | null;
}

export interface DownloadJob {
  id: string;
  url: string;
  preset: PresetType;
  quality: string;
  outputDirectory: string;
  status: DownloadStatus;
  progress: DownloadProgress;
  metadata: MediaMetadata;
  finalFileName?: string | null;
  finalFilePath?: string | null;
  inspection?: MediaInspection | null;
  errorMessage?: string | null;
  createdAt: string;
  completedAt?: string | null;
  recipe?: DownloadRecipe | null;
  fingerprint?: MediaFingerprint | null;
  explainableResult?: ExplainableResult | null;
  verification?: VerificationResult | null;
}

export interface AppSettings {
  downloadDirectory: string;
  lastPreset: PresetType;
  defaultQuality: string;
  openFolderAfterDownload: boolean;
  autoAnalyzeOnPaste: boolean;
  embedMetadata: boolean;
  embedThumbnail: boolean;
  embedChapters: boolean;
  concurrentFragments: number;
  trimFilenames: number;
  sponsorBlockMode: SponsorBlockMode;
  subtitleMode: SubtitleMode;
  preferredSubtitleLanguage: string;
  customYtdlpPath?: string | null;
  customFfmpegPath?: string | null;
  customFfprobePath?: string | null;
  customMediainfoPath?: string | null;
}

export type ToolStatus = 'READY' | 'MISSING' | 'INSTALLING' | 'INVALID' | 'OUTDATED' | 'ERROR';

export interface ToolStatusInfo {
  name: string;
  status: ToolStatus;
  version?: string | null;
  pinnedVersion: string;
  path?: string | null;
  managed: boolean;
  sourceUrl?: string | null;
  sha256?: string | null;
  errorMessage?: string | null;
  license: string;
  licenseUrl: string;
  isRequired: boolean;
}

export interface ToolHealth {
  name: string;
  available: boolean;
  path?: string | null;
  version?: string | null;
  repairMessage?: string | null;
}

export interface DiagnosticLog {
  id: string;
  timestamp: string;
  level: string;
  source: string;
  message: string;
}

export interface StartDownloadRequest {
  url: string;
  metadata: MediaMetadata;
  preset: PresetType;
  quality: string;
  outputDirectory: string;
}

export interface BuildCommandRequest {
  preset: PresetType;
  quality: string;
  outputDirectory: string;
  url: string;
  settings?: AppSettings | null;
}

export interface BuildCommandResponse {
  command: string;
  arguments: string[];
}

export interface AppError {
  userMessage: string;
  technicalDetails?: string;
}

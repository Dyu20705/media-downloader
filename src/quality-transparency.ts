export type ProcessingClass =
  | 'source-preserved'
  | 'merge-only'
  | 'remux-only'
  | 'audio-transcode'
  | 'video-transcode'
  | 'full-transcode'
  | 'unknown';

export type QualityPreset =
  | 'mp4-compatible'
  | 'best-video'
  | 'best-audio'
  | 'mp3'
  | 'flac';

export interface StreamSummary {
  codec?: string | null;
  bitrateKbps?: number | null;
  width?: number | null;
  height?: number | null;
  fps?: number | null;
  hdr?: boolean | null;
  language?: string | null;
}

export interface PlanMediaSummary {
  video?: StreamSummary | null;
  audio?: StreamSummary | null;
  container?: string | null;
  estimatedBytes?: number | null;
}

export interface DownloadPlan {
  source: PlanMediaSummary;
  output: PlanMediaSummary;
  processing: {
    class: ProcessingClass;
    videoReencoded?: boolean | null;
    audioReencoded?: boolean | null;
    explanation?: string | null;
  };
}

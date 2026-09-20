import React from 'react';
import { AlertTriangle, ArrowRight, CheckCircle2, Loader2 } from 'lucide-react';
import { DownloadPlan, PlanMediaSummary, ProcessingClass, StreamSummary } from '../quality-transparency';

interface DownloadPlanCardProps {
  plan: DownloadPlan | null;
  isLoading?: boolean;
  error?: string | null;
}

const compactCodec = (codec?: string | null) => codec ? codec.split('.')[0].toUpperCase() : null;

const streamParts = (stream: StreamSummary | null | undefined, kind: 'video' | 'audio') => {
  if (!stream) return [];
  const parts: string[] = [];
  if (kind === 'video' && stream.height) {
    parts.push(`${stream.height}p${stream.fps ? Math.round(stream.fps) : ''}`);
  }
  const codec = compactCodec(stream.codec);
  if (codec) parts.push(codec);
  if (stream.hdr) parts.push('HDR');
  if (stream.bitrateKbps) parts.push(`~${stream.bitrateKbps} kbps`);
  if (stream.language) parts.push(stream.language.toUpperCase());
  return parts;
};

const formatBytes = (bytes?: number | null) => {
  if (!bytes) return null;
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `~${value.toFixed(unit > 1 ? 1 : 0)} ${units[unit]}`;
};

const Summary: React.FC<{ title: string; summary: PlanMediaSummary }> = ({ title, summary }) => {
  const video = streamParts(summary.video, 'video');
  const audio = streamParts(summary.audio, 'audio');
  const footer = [summary.container?.toUpperCase(), formatBytes(summary.estimatedBytes)].filter(Boolean);
  const hasDetails = video.length > 0 || audio.length > 0 || footer.length > 0;

  return (
    <div className="min-w-0 rounded-xl border border-zinc-800 bg-zinc-950/45 p-3">
      <h3 className="mb-2 text-[10px] font-semibold uppercase tracking-widest text-zinc-500">{title}</h3>
      {hasDetails ? (
        <div className="space-y-1 text-xs text-zinc-300">
          {video.length > 0 && <p><span className="text-zinc-500">Video:</span> {video.join(' · ')}</p>}
          {audio.length > 0 && <p><span className="text-zinc-500">Audio:</span> {audio.join(' · ')}</p>}
          {footer.length > 0 && <p><span className="text-zinc-500">File:</span> {footer.join(' · ')}</p>}
        </div>
      ) : (
        <p className="text-xs text-zinc-500">Stream details unavailable</p>
      )}
    </div>
  );
};

const processingLabels: Record<ProcessingClass, string> = {
  'source-preserved': 'Source preserved · No re-encoding',
  'merge-only': 'Merge only · No re-encoding',
  'remux-only': 'Remux only · No re-encoding',
  'audio-transcode': 'Audio transcode',
  'video-transcode': 'Video transcode',
  'full-transcode': 'Full transcode',
  unknown: 'Processing details unavailable',
};

export const DownloadPlanCard: React.FC<DownloadPlanCardProps> = ({ plan, isLoading = false, error = null }) => {
  if (isLoading && !plan) {
    return (
      <div className="flex items-center gap-2 rounded-xl border border-zinc-800 bg-zinc-900 p-4 text-xs text-zinc-400" role="status">
        <Loader2 className="h-4 w-4 animate-spin text-blue-400" aria-hidden="true" />
        Resolving download plan…
      </div>
    );
  }
  if (error) {
    return (
      <div className="rounded-xl border border-zinc-800 bg-zinc-900 p-4 text-xs text-zinc-400" role="status">
        Processing details unavailable. You can still download this media.
      </div>
    );
  }
  if (!plan) return null;

  const isTranscode = plan.processing.class.includes('transcode');
  return (
    <div className="space-y-3 rounded-2xl border border-zinc-800 bg-zinc-900 p-4" aria-label="Download plan">
      <div className="flex items-center justify-between">
        <h2 className="text-xs font-semibold uppercase tracking-wider text-zinc-300">Quality & processing plan</h2>
        <span className="text-[10px] font-medium uppercase tracking-wide text-zinc-500">Planned, not verified</span>
      </div>
      <div className="grid gap-2 sm:grid-cols-[1fr_auto_1fr] sm:items-center">
        <Summary title="Source" summary={plan.source} />
        <ArrowRight className="mx-auto hidden h-4 w-4 text-zinc-600 sm:block" aria-hidden="true" />
        <Summary title="Planned output" summary={plan.output} />
      </div>
      <div className={`rounded-xl border p-3 ${isTranscode ? 'border-amber-900/60 bg-amber-950/20' : 'border-emerald-900/50 bg-emerald-950/15'}`}>
        <div className="flex items-start gap-2">
          {isTranscode ? (
            <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-amber-400" aria-hidden="true" />
          ) : (
            <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0 text-emerald-400" aria-hidden="true" />
          )}
          <div>
            <p className={`text-xs font-semibold ${isTranscode ? 'text-amber-300' : 'text-emerald-300'}`}>
              {processingLabels[plan.processing.class]}
            </p>
            {plan.processing.explanation && (
              <p className="mt-1 text-xs leading-relaxed text-zinc-400">{plan.processing.explanation}</p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

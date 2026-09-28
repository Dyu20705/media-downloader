import React from 'react';
import {
  AlertTriangle,
  ArrowRight,
  Clock3,
  FileOutput,
  Loader2,
  RadioTower,
  Settings2,
  Wrench,
} from 'lucide-react';
import type { AcquisitionPlan, AppError, MediaFormatSpec, MediaMetadata, ProcessingClass } from '../types';

interface DownloadPlanCardProps {
  metadata: MediaMetadata;
  plan: AcquisitionPlan | null;
  isPlanning: boolean;
  error: AppError | null;
}

const PROCESSING_LABELS: Record<ProcessingClass, { title: string; detail: string; tone: string }> = {
  SOURCE_PRESERVED: {
    title: 'Source preserved',
    detail: 'No re-encoding planned',
    tone: 'text-emerald-300 bg-emerald-950/40 border-emerald-800/60',
  },
  MERGE_ONLY: {
    title: 'Merge only',
    detail: 'Streams preserved; container assembly only',
    tone: 'text-emerald-300 bg-emerald-950/40 border-emerald-800/60',
  },
  REMUX_ONLY: {
    title: 'Remux only',
    detail: 'Streams preserved; container changed',
    tone: 'text-sky-300 bg-sky-950/40 border-sky-800/60',
  },
  AUDIO_TRANSCODE: {
    title: 'Audio transcode',
    detail: 'Audio will be re-encoded',
    tone: 'text-amber-300 bg-amber-950/40 border-amber-800/60',
  },
  VIDEO_TRANSCODE: {
    title: 'Video transcode',
    detail: 'Video will be re-encoded',
    tone: 'text-amber-300 bg-amber-950/40 border-amber-800/60',
  },
  FULL_TRANSCODE: {
    title: 'Full transcode',
    detail: 'Video and audio will be re-encoded',
    tone: 'text-orange-300 bg-orange-950/40 border-orange-800/60',
  },
  UNKNOWN: {
    title: 'Processing unknown',
    detail: 'The engine could not classify processing',
    tone: 'text-zinc-300 bg-zinc-900 border-zinc-700',
  },
};

function codecLabel(codec?: string | null): string | null {
  if (!codec || codec === 'none') return null;
  return codec.split('.')[0].toUpperCase();
}

function fpsLabel(fps?: number | null): string | null {
  if (!fps || fps <= 0) return null;
  return `${Number.isInteger(fps) ? fps : fps.toFixed(2)} fps`;
}

function sizeLabel(bytes?: number | null): string | null {
  if (!bytes || bytes <= 0) return null;
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value >= 10 || unit === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unit]}`;
}

function joinKnown(values: Array<string | null | undefined>): string {
  const known = values.filter((value): value is string => Boolean(value));
  return known.length > 0 ? known.join(' · ') : 'Unknown';
}

function findFormat(metadata: MediaMetadata, id?: string | null): MediaFormatSpec | null {
  if (!id) return null;
  return metadata.formats?.find((format) => format.formatId === id) ?? null;
}

function StreamSummary({ format, kind }: { format: MediaFormatSpec | null; kind: 'video' | 'audio' }) {
  if (!format) return <span className="text-zinc-500">Unknown {kind} stream</span>;

  if (kind === 'video') {
    const resolution = format.width && format.height
      ? `${format.width}×${format.height}`
      : format.resolution || (format.height ? `${format.height}p` : null);
    const dynamicRange = format.hdr === true ? 'HDR' : format.dynamicRange || null;
    return <>{joinKnown([resolution, fpsLabel(format.fps), codecLabel(format.vcodec), dynamicRange])}</>;
  }

  const bitrate = format.abr ? `~${Math.round(format.abr)} kbps` : null;
  return <>{joinKnown([codecLabel(format.acodec), bitrate])}</>;
}

export const DownloadPlanCard: React.FC<DownloadPlanCardProps> = ({
  metadata,
  plan,
  isPlanning,
  error,
}) => {
  if (isPlanning) {
    return (
      <div className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-4" role="status" aria-live="polite">
        <div className="flex items-center gap-2 text-sm text-zinc-300">
          <Loader2 className="h-4 w-4 animate-spin text-blue-400" aria-hidden="true" />
          <span>Resolving authoritative download plan…</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-2xl border border-red-900/60 bg-red-950/30 p-4" role="alert">
        <div className="flex items-start gap-2.5">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-red-400" aria-hidden="true" />
          <div>
            <p className="text-sm font-semibold text-red-200">{error.userMessage}</p>
            {error.technicalDetails && <p className="mt-1 text-xs text-red-300/80">{error.technicalDetails}</p>}
          </div>
        </div>
      </div>
    );
  }

  if (!plan) return null;
  if (['THUMBNAIL_ONLY', 'SUBTITLES_ONLY'].includes(plan.operation.type)) {
    return <article className="rounded-2xl border border-zinc-800 bg-zinc-900 p-4 space-y-2">
      <h2 className="text-sm font-semibold">Download plan</h2>
      <p className="text-sm">Source: {plan.source.title}</p>
      <p className="text-sm text-blue-300">Planned output: {plan.output.container.toUpperCase()} {plan.selectedStreams.subtitleLanguages.join(', ')}</p>
      <p className="text-xs text-zinc-400">{plan.processing.steps.join('; ')}</p>
      <p className="text-xs text-zinc-400">Output profiles do not apply to image and subtitle exports.</p>
    </article>;
  }

  const videoFormat = findFormat(metadata, plan.selectedStreams.videoStreamId);
  const audioFormat = findFormat(metadata, plan.selectedStreams.audioStreamId);
  const sourceContainer = videoFormat?.ext || audioFormat?.ext || null;
  const processing = PROCESSING_LABELS[plan.processing.class];
  const plannedResolution = plan.output.width && plan.output.height
    ? `${plan.output.width}×${plan.output.height}`
    : null;

  return (
    <article className="overflow-hidden rounded-2xl border border-zinc-800 bg-zinc-900/70" aria-labelledby="download-plan-title">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-zinc-800 px-4 py-3">
        <div className="flex items-center gap-2">
          <Settings2 className="h-4 w-4 text-blue-400" aria-hidden="true" />
          <h2 id="download-plan-title" className="text-sm font-semibold text-zinc-100">Download plan</h2>
          {plan.operation.type === 'CLIP' && <span className="text-xs text-blue-300">Clip: {plan.operation.startMs / 1000}s → {plan.operation.endMs / 1000}s</span>}
          <span className="rounded border border-zinc-700 bg-zinc-950 px-1.5 py-0.5 font-mono text-[10px] text-zinc-500">
            {plan.id}
          </span>
        </div>
        {plan.estimatedSize && (
          <span className="text-xs text-zinc-400">
            Estimated {sizeLabel(plan.estimatedSize.bytes)} · {plan.estimatedSize.confidence.toLowerCase()} confidence
          </span>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-[1fr_auto_1fr]">
        <section className="space-y-2 p-4" aria-label="Selected source streams">
          <div className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-zinc-500">
            <RadioTower className="h-3.5 w-3.5" aria-hidden="true" />
            <span>Source</span>
          </div>
          {!plan.output.audioOnly && (
            <p className="text-sm text-zinc-200"><StreamSummary format={videoFormat} kind="video" /></p>
          )}
          <p className="text-sm text-zinc-200"><StreamSummary format={audioFormat} kind="audio" /></p>
          <p className="text-xs uppercase text-zinc-500">{sourceContainer || 'Container unknown'}</p>
        </section>

        <div className="hidden items-center border-x border-zinc-800 px-3 md:flex" aria-hidden="true">
          <ArrowRight className="h-4 w-4 text-zinc-600" />
        </div>

        <section className="space-y-2 border-t border-zinc-800 p-4 md:border-t-0" aria-label="Planned output">
          <div className="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-zinc-500">
            <FileOutput className="h-3.5 w-3.5" aria-hidden="true" />
            <span>Planned output</span>
          </div>
          {!plan.output.audioOnly && (
            <p className="text-sm text-zinc-200">
              {joinKnown([plannedResolution, fpsLabel(plan.output.fps), codecLabel(plan.output.videoCodec)])}
            </p>
          )}
          <p className="text-sm text-zinc-200">{codecLabel(plan.output.audioCodec) || 'No audio stream'}</p>
          <p className="text-xs font-semibold uppercase text-blue-300">{plan.output.container}</p>
        </section>
      </div>

      <div className="border-t border-zinc-800 p-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <div className="mb-1.5 flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider text-zinc-500">
              <Wrench className="h-3.5 w-3.5" aria-hidden="true" />
              <span>Processing</span>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <span className={`rounded-lg border px-2 py-1 text-xs font-semibold ${processing.tone}`}>{processing.title}</span>
              <span className="text-xs text-zinc-400">{processing.detail}</span>
            </div>
          </div>
          <div className="flex flex-wrap gap-1.5">
            {plan.processing.steps.map((step) => (
              <span key={step} className="rounded-md border border-zinc-800 bg-zinc-950/70 px-2 py-1 text-[11px] text-zinc-400">
                {step}
              </span>
            ))}
          </div>
        </div>

        {(plan.warnings.length > 0 || plan.requirements.length > 0) && (
          <div className="mt-3 grid gap-2 border-t border-zinc-800/80 pt-3 sm:grid-cols-2">
            {plan.warnings.map((warning) => (
              <div key={warning.code} className="flex items-start gap-2 text-xs text-amber-300">
                <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" aria-hidden="true" />
                <span>{warning.message}</span>
              </div>
            ))}
            {plan.requirements.map((requirement) => (
              <div key={requirement.code} className="flex items-start gap-2 text-xs text-zinc-400">
                {requirement.code === 'FFMPEG'
                  ? <Clock3 className="mt-0.5 h-3.5 w-3.5 shrink-0 text-sky-400" aria-hidden="true" />
                  : <Wrench className="mt-0.5 h-3.5 w-3.5 shrink-0 text-zinc-500" aria-hidden="true" />}
                <span>{requirement.message}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </article>
  );
};

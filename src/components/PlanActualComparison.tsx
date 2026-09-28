import React from 'react';
import { AlertTriangle, CheckCircle2, FileCheck2 } from 'lucide-react';
import type { AcquisitionPlan, MediaInspection } from '../types';

interface PlanActualComparisonProps {
  plan: AcquisitionPlan;
  inspection: MediaInspection;
}

interface Difference {
  field: string;
  planned: string;
  actual: string;
}

function formatBytes(bytes: number): string {
  if (bytes <= 0) return 'Unknown size';
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value >= 10 || unit === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unit]}`;
}

function codecFamily(value?: string | null): string | null {
  if (!value) return null;
  const normalized = value.toLowerCase();
  if (normalized.includes('h264') || normalized.includes('avc')) return 'H.264';
  if (normalized.includes('h265') || normalized.includes('hevc')) return 'H.265';
  if (normalized.includes('vp9') || normalized.includes('vp09')) return 'VP9';
  if (normalized.includes('av1') || normalized.includes('av01')) return 'AV1';
  if (normalized.includes('aac')) return 'AAC';
  if (normalized.includes('opus')) return 'Opus';
  if (normalized.includes('vorbis')) return 'Vorbis';
  if (normalized.includes('mp3')) return 'MP3';
  if (normalized.includes('flac')) return 'FLAC';
  return value.toUpperCase();
}

function containerMatches(planned: string, actual: string): boolean {
  const wanted = planned.toLowerCase();
  const found = actual.toLowerCase();
  if (wanted === 'jpg') return found === 'image2' || found === 'jpeg_pipe' || found === 'mjpeg';
  if (wanted === 'mp4') return found.includes('mp4') || found.includes('mov');
  if (wanted === 'mkv') return found.includes('matroska');
  if (wanted === 'webm') return found.includes('webm');
  return found.split(',').some((part) => part.trim() === wanted) || found.includes(wanted);
}

function collectDifferences(plan: AcquisitionPlan, actual: MediaInspection): Difference[] {
  const differences: Difference[] = [];
  if (plan.timeRangeMs && actual.durationSeconds != null) {
    const requested = (plan.timeRangeMs[1] - plan.timeRangeMs[0]) / 1000;
    if (Math.abs(requested - actual.durationSeconds) > 0.1) {
      differences.push({ field: 'Clip duration', planned: `${requested.toFixed(3)} s`, actual: `${actual.durationSeconds.toFixed(3)} s` });
    }
  }
  const plannedVideo = codecFamily(plan.output.videoCodec);
  const actualVideo = codecFamily(actual.videoCodec);
  const plannedAudio = codecFamily(plan.output.audioCodec);
  const actualAudio = codecFamily(actual.audioCodec);

  if (actual.containerFormat && !containerMatches(plan.output.container, actual.containerFormat)) {
    differences.push({ field: 'Container', planned: plan.output.container.toUpperCase(), actual: actual.containerFormat.toUpperCase() });
  }
  if (plannedVideo && actualVideo && plannedVideo !== actualVideo) {
    differences.push({ field: 'Video codec', planned: plannedVideo, actual: actualVideo });
  }
  if (plan.output.audioOnly && actualVideo) {
    differences.push({ field: 'Video stream', planned: 'None', actual: actualVideo });
  }
  if (plannedAudio && actualAudio && plannedAudio !== actualAudio) {
    differences.push({ field: 'Audio codec', planned: plannedAudio, actual: actualAudio });
  }
  if (plan.output.width && plan.output.height && actual.width && actual.height
      && (plan.output.width !== actual.width || plan.output.height !== actual.height)) {
    differences.push({
      field: 'Resolution',
      planned: `${plan.output.width}×${plan.output.height}`,
      actual: `${actual.width}×${actual.height}`,
    });
  }
  if (plan.output.fps && actual.fps && Math.abs(plan.output.fps - actual.fps) > 0.1) {
    differences.push({
      field: 'Frame rate',
      planned: `${plan.output.fps.toFixed(2)} fps`,
      actual: `${actual.fps.toFixed(2)} fps`,
    });
  }

  return differences;
}

function known(values: Array<string | null | undefined>): string {
  const present = values.filter((value): value is string => Boolean(value));
  return present.length > 0 ? present.join(' · ') : 'Unknown';
}

export const PlanActualComparison: React.FC<PlanActualComparisonProps> = ({ plan, inspection }) => {
  const differences = collectDifferences(plan, inspection);
  const sidecar = ['THUMBNAIL_ONLY', 'SUBTITLES_ONLY'].includes(plan.operation.type);
  const resolution = inspection.width && inspection.height ? `${inspection.width}×${inspection.height}` : null;
  const fps = inspection.fps ? `${inspection.fps.toFixed(Number.isInteger(inspection.fps) ? 0 : 2)} fps` : null;
  const bitrate = inspection.audioBitrateKbps ? `${inspection.audioBitrateKbps} kbps` : null;

  return (
    <section className="mb-4 rounded-xl border border-slate-800/80 bg-slate-950/50 p-3.5" aria-labelledby="actual-output-title">
      <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
        <h4 id="actual-output-title" className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-slate-400">
          <FileCheck2 className="h-4 w-4 text-emerald-400" aria-hidden="true" />
          <span>Actual output</span>
        </h4>
        <span className="rounded border border-emerald-800/60 bg-emerald-950/40 px-2 py-0.5 text-[10px] font-semibold uppercase text-emerald-300">
          {inspection.verificationLevel.replaceAll('_', ' ')}
        </span>
      </div>

      <div className="grid gap-1 text-sm text-slate-200 sm:grid-cols-2">
        {inspection.durationSeconds != null && <p>Duration: {inspection.durationSeconds.toFixed(3)} s</p>}
        <p>{sidecar ? (resolution || 'Timed subtitle cues validated') : known([resolution, codecFamily(inspection.videoCodec), fps])}</p>
        {!sidecar && <p>{known([codecFamily(inspection.audioCodec), bitrate])}</p>}
        <p className="text-xs font-semibold uppercase text-blue-300">{inspection.containerFormat || 'Container unknown'}</p>
        <p className="text-xs text-slate-400 sm:text-right">{formatBytes(inspection.fileSizeBytes)}</p>
      </div>

      <div className="mt-3 border-t border-slate-800/80 pt-3">
        {differences.length === 0 ? (
          <div className="flex items-center gap-2 text-xs text-emerald-300">
            <CheckCircle2 className="h-4 w-4 shrink-0" aria-hidden="true" />
            <span>Verified properties match the download plan.</span>
          </div>
        ) : (
          <div>
            <div className="mb-2 flex items-center gap-2 text-xs font-semibold text-amber-300">
              <AlertTriangle className="h-4 w-4 shrink-0" aria-hidden="true" />
              <span>Plan vs actual differences</span>
            </div>
            <div className="grid gap-1.5">
              {differences.map((difference) => (
                <div key={difference.field} className="grid grid-cols-[minmax(5rem,1fr)_1fr_auto_1fr] items-center gap-2 rounded-lg border border-amber-900/40 bg-amber-950/20 px-2.5 py-2 text-xs">
                  <span className="font-medium text-slate-300">{difference.field}</span>
                  <span className="truncate font-mono text-slate-400" title={difference.planned}>{difference.planned}</span>
                  <span className="text-amber-500" aria-hidden="true">→</span>
                  <span className="truncate font-mono text-amber-200" title={difference.actual}>{difference.actual}</span>
                </div>
              ))}
            </div>
            <p className="mt-2 text-[11px] text-slate-500">The verified actual output is authoritative.</p>
          </div>
        )}
      </div>
    </section>
  );
};

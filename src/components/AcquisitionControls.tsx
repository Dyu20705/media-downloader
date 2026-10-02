import React from 'react';
import {
  AudioLines,
  Captions,
  Film,
  Image,
  Layers3,
  Scissors,
  Sparkles,
  type LucideIcon,
} from 'lucide-react';
import type { AcquisitionOperation, FormatRecommendation, OutputProfile, MediaMetadata } from '../types';

interface AcquisitionControlsProps {
  metadata: MediaMetadata | null;
  subtitleLanguage: string;
  onSubtitleLanguage: (language: string) => void;
  includeAutoSubtitles: boolean;
  onIncludeAutoSubtitles: (include: boolean) => void;
  operation: AcquisitionOperation;
  outputProfile: OutputProfile;
  onChangeOperation: (operation: AcquisitionOperation) => void;
  onChangeOutputProfile: (profile: OutputProfile) => void;
  disabled?: boolean;
  recommendation?: FormatRecommendation | null;
}

type BasicOperation = AcquisitionOperation['type'];

const OPERATIONS: Array<{
  type: AcquisitionOperation['type'];
  title: string;
  subtitle: string;
  icon: LucideIcon;
  available: boolean;
}> = [
  { type: 'ENTIRE_MEDIA', title: 'Entire media', subtitle: 'Video and audio', icon: Film, available: true },
  { type: 'AUDIO_ONLY', title: 'Audio only', subtitle: 'Extract audio track', icon: AudioLines, available: true },
  { type: 'CLIP', title: 'Clip', subtitle: 'Time range', icon: Scissors, available: true },
  { type: 'THUMBNAIL_ONLY', title: 'Thumbnail', subtitle: 'JPEG image', icon: Image, available: true },
  { type: 'CHAPTER', title: 'Chapter', subtitle: 'One chapter', icon: Layers3, available: true },
  { type: 'SUBTITLES_ONLY', title: 'Subtitles', subtitle: 'WebVTT captions', icon: Captions, available: true },
];

const PROFILES: Array<{
  id: Exclude<OutputProfile, 'CUSTOM'>;
  title: string;
  subtitle: string;
}> = [
  { id: 'BEST_SOURCE', title: 'Best Source', subtitle: 'Preserve the strongest available streams' },
  { id: 'UNIVERSAL', title: 'Universal', subtitle: 'Prioritize broad playback compatibility' },
  { id: 'EDITING', title: 'Editing', subtitle: 'Editing-friendly output format' },
  { id: 'SMALL', title: 'Small', subtitle: 'Prefer the smallest suitable source streams' },
];

function operationFromRecommendation(recommendation?: FormatRecommendation | null): BasicOperation | null {
  if (!recommendation) return null;
  return ['best-audio', 'mp3', 'flac'].includes(recommendation.preset) ? 'AUDIO_ONLY' : 'ENTIRE_MEDIA';
}

function profileFromRecommendation(recommendation?: FormatRecommendation | null): OutputProfile | null {
  if (!recommendation) return null;
  if (recommendation.preset === 'best-video' || recommendation.preset === 'best-audio') return 'BEST_SOURCE';
  if (recommendation.preset === 'flac') return 'EDITING';
  return 'UNIVERSAL';
}

function operationValue(type: BasicOperation): AcquisitionOperation {
  if (type === 'CHAPTER') return { type, chapterIndex: 0 };
  if (type === 'CLIP') return { type, startMs: 0, endMs: 10000 };
  return { type };
}

export const AcquisitionControls: React.FC<AcquisitionControlsProps> = ({
  metadata, subtitleLanguage, onSubtitleLanguage, includeAutoSubtitles, onIncludeAutoSubtitles,
  operation,
  outputProfile,
  onChangeOperation,
  onChangeOutputProfile,
  disabled = false,
  recommendation = null,
}) => {
  const recommendedOperation = operationFromRecommendation(recommendation);
  const recommendedProfile = profileFromRecommendation(recommendation);
  const audioGuidance = operation.type === 'AUDIO_ONLY' && outputProfile === 'UNIVERSAL'
    ? 'Universal audio creates an MP3 compatibility copy. Re-encoding cannot restore detail missing from the source.'
    : operation.type === 'AUDIO_ONLY' && outputProfile === 'EDITING'
      ? 'Editing audio creates FLAC output. A lossy source does not become an original lossless recording.'
      : operation.type === 'AUDIO_ONLY' && outputProfile === 'BEST_SOURCE'
        ? 'Best Source preserves the selected source audio stream when possible.'
        : null;

  return (
    <div className="space-y-4">
      {operation.type === 'CHAPTER' && (
        <label className="block text-sm">Chapter
          <select disabled={disabled} value={operation.chapterIndex} onChange={e => onChangeOperation({ type: 'CHAPTER', chapterIndex: Number(e.target.value) })} className="block w-full bg-zinc-900 p-2">
            {!metadata?.chapters?.length && <option value={0}>No chapters available</option>}
            {metadata?.chapters?.map((chapter, index) => <option key={index} value={index}>{index + 1}. {chapter.title} ({chapter.startTime}s – {chapter.endTime}s)</option>)}
          </select>
          <span className="text-xs text-amber-300">Fast cut; boundaries may align to nearby keyframes.</span>
        </label>
      )}
      {operation.type === 'SUBTITLES_ONLY' && (
        <fieldset disabled={disabled} className="space-y-2">
          <label className="block text-sm">Subtitle language (one file per download)
            <select value={subtitleLanguage} onChange={e => onSubtitleLanguage(e.target.value)} className="block w-full bg-zinc-900 p-2">
              <option value="">Select a language</option>
              {[...new Set([...(metadata?.subtitles || []), ...(includeAutoSubtitles ? metadata?.automaticCaptions || [] : [])].map(t => t.language))].map(language => <option key={language} value={language}>{language}</option>)}
            </select>
          </label>
          <label className="text-sm"><input type="checkbox" checked={includeAutoSubtitles} onChange={e => onIncludeAutoSubtitles(e.target.checked)} /> Include automatic captions</label>
        </fieldset>
      )}
      {operation.type === 'CLIP' && (
        <fieldset disabled={disabled} className="rounded-xl border border-zinc-800 p-3 space-y-2">
          <legend className="text-xs text-zinc-400">Clip range (seconds)</legend>
          <div className="grid grid-cols-2 gap-3">
            <label className="text-xs text-zinc-400">Start
              <input type="number" min="0" step="0.001" value={operation.startMs / 1000}
                onChange={(event) => onChangeOperation({ ...operation, startMs: Math.round(Number(event.target.value) * 1000) })}
                className="block w-full rounded bg-zinc-950 p-2 text-zinc-100" />
            </label>
            <label className="text-xs text-zinc-400">End
              <input type="number" min="0" step="0.001" value={operation.endMs / 1000}
                onChange={(event) => onChangeOperation({ ...operation, endMs: Math.round(Number(event.target.value) * 1000) })}
                className="block w-full rounded bg-zinc-950 p-2 text-zinc-100" />
            </label>
          </div>
          <p className="text-xs text-amber-300">Fast cut: boundaries may follow nearby keyframes. Review the actual duration after download.</p>
        </fieldset>
      )}
      <div className="space-y-2">
        <div className="flex items-center justify-between gap-2">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-zinc-400">What to acquire</h2>
          <span className="text-[11px] text-zinc-600">Operation</span>
        </div>
        <div role="radiogroup" aria-label="Acquisition operation" className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-6">
          {OPERATIONS.map((option) => {
            const Icon = option.icon;
            const selected = operation.type === option.type;
            const recommended = recommendedOperation === option.type;
            const unavailable = disabled || !option.available;
            return (
              <button
                key={option.type}
                id={`operation-${option.type.toLowerCase()}`}
                type="button"
                role="radio"
                aria-checked={selected}
                disabled={unavailable}
                onClick={() => option.available && onChangeOperation(operationValue(option.type as BasicOperation))}
                className={`relative rounded-xl border p-3 text-left transition-all focus-visible:ring-2 focus-visible:ring-blue-500 ${
                  selected
                    ? 'border-blue-500 bg-blue-600/15 text-zinc-100 ring-1 ring-blue-500/50'
                    : 'border-zinc-800 bg-zinc-900 text-zinc-300 hover:border-zinc-700'
                } ${unavailable ? 'cursor-not-allowed opacity-45' : 'cursor-pointer'}`}
              >
                <div className="mb-2 flex items-center justify-between">
                  <Icon className={`h-4 w-4 ${selected ? 'text-blue-400' : 'text-zinc-500'}`} aria-hidden="true" />
                  {recommended && <Sparkles className="h-3 w-3 text-blue-400" aria-hidden="true" />}
                </div>
                <div className="text-xs font-semibold">{option.title}</div>
                <div className="mt-0.5 text-[10px] leading-tight text-zinc-500">
                  {option.available ? option.subtitle : 'Coming next'}
                </div>
              </button>
            );
          })}
        </div>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between gap-2">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-zinc-400">How the output should behave</h2>
          <span className="text-[11px] text-zinc-600">Output profile</span>
        </div>
        <div role="radiogroup" aria-label="Output profile" className="grid grid-cols-2 gap-2 lg:grid-cols-4">
          {PROFILES.map((profile) => {
            const selected = outputProfile === profile.id;
            const recommended = recommendedProfile === profile.id;
            return (
              <button
                key={profile.id}
                id={`profile-${profile.id.toLowerCase()}`}
                type="button"
                role="radio"
                aria-checked={selected}
                disabled={disabled || ['THUMBNAIL_ONLY', 'SUBTITLES_ONLY'].includes(operation.type)}
                onClick={() => onChangeOutputProfile(profile.id)}
                aria-disabled={['THUMBNAIL_ONLY', 'SUBTITLES_ONLY'].includes(operation.type) || disabled}
                className={`rounded-xl border p-3 text-left transition-all focus-visible:ring-2 focus-visible:ring-blue-500 ${
                  selected
                    ? 'border-blue-500 bg-blue-600/15 ring-1 ring-blue-500/50'
                    : recommended
                      ? 'border-blue-500/40 bg-zinc-900 hover:border-blue-500/70'
                      : 'border-zinc-800 bg-zinc-900 hover:border-zinc-700'
                } ${disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'}`}
              >
                <div className="flex items-center justify-between gap-2">
                  <span className="text-sm font-semibold text-zinc-100">{profile.title}</span>
                  {recommended && <Sparkles className="h-3 w-3 shrink-0 text-blue-400" aria-hidden="true" />}
                </div>
                <p className="mt-1 text-xs leading-snug text-zinc-500">{profile.subtitle}</p>
              </button>
            );
          })}
        </div>
      </div>

      {audioGuidance && (
        <div className="rounded-lg border border-amber-900/50 bg-amber-950/15 px-3 py-2 text-[11px] leading-relaxed text-zinc-400">
          <span className="font-semibold text-amber-300">Quality note: </span>
          {audioGuidance}
        </div>
      )}
    </div>
  );
};

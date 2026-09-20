import React from 'react';
import { Sparkles } from 'lucide-react';
import { PresetType, FormatRecommendation } from '../types';

interface FormatSelectorProps {
  selectedPreset: PresetType;
  onSelectPreset: (preset: PresetType) => void;
  disabled?: boolean;
  recommendation?: FormatRecommendation | null;
}

interface FormatOption {
  id: PresetType;
  title: string;
  subtitle: string;
  guidance?: string;
}

const FORMAT_OPTIONS: FormatOption[] = [
  {
    id: 'mp4-compatible',
    title: 'MP4',
    subtitle: 'Compatible with most devices',
  },
  {
    id: 'best-video',
    title: 'Best Video',
    subtitle: 'Highest available quality',
  },
  {
    id: 'best-audio',
    title: 'Best Audio',
    subtitle: 'Preserve best available source',
    guidance: 'Preserves the source audio stream when possible, avoiding an unnecessary lossy-to-lossy conversion.',
  },
  {
    id: 'mp3',
    title: 'MP3',
    subtitle: 'Universal compatibility · lossy transcode',
    guidance: 'Converts the source to MP3 using the encoder\'s highest-quality VBR setting. Re-encoding cannot restore detail missing from the source stream.',
  },
  {
    id: 'flac',
    title: 'FLAC',
    subtitle: 'FLAC output · no quality restoration',
    guidance: 'FLAC encoding is lossless relative to decoded audio, but converting a lossy source cannot recover information already discarded upstream.',
  },
];

export const FormatSelector: React.FC<FormatSelectorProps> = ({
  selectedPreset,
  onSelectPreset,
  disabled = false,
  recommendation = null,
}) => {
  const selectedGuidance = FORMAT_OPTIONS.find(
    (option) => option.id === selectedPreset,
  )?.guidance;

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <label className="block text-xs font-semibold uppercase tracking-wider text-zinc-400">
          Format
        </label>
        {recommendation && (
          <div className="flex items-center gap-1 text-[11px] text-blue-400 font-medium">
            <Sparkles className="w-3 h-3" aria-hidden="true" />
            <span>Recommended: {recommendation.label}</span>
          </div>
        )}
      </div>

      <div 
        role="radiogroup" 
        aria-label="Media format presets"
        className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-2"
      >
        {FORMAT_OPTIONS.map((option) => {
          const isSelected = selectedPreset === option.id;
          const isRecommended = recommendation?.preset === option.id;

          return (
            <button
              key={option.id}
              id={`preset-btn-${option.id}`}
              type="button"
              role="radio"
              aria-checked={isSelected}
              disabled={disabled}
              onClick={() => onSelectPreset(option.id)}
              className={`p-3 rounded-xl text-left transition-all border cursor-pointer focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:outline-none ${
                isSelected
                  ? 'bg-blue-600/15 border-blue-500 text-zinc-100 ring-1 ring-blue-500/50'
                  : isRecommended
                  ? 'bg-zinc-900 border-blue-500/40 text-zinc-200 hover:border-blue-500/70 hover:bg-zinc-800/80'
                  : 'bg-zinc-900 border-zinc-800 text-zinc-300 hover:border-zinc-700 hover:bg-zinc-800/80'
              } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
            >
              <div className="flex items-center justify-between">
                <span className="font-semibold text-sm text-zinc-100 flex items-center gap-1.5">
                  {option.title}
                  {isRecommended && !isSelected && (
                    <Sparkles className="w-2.5 h-2.5 text-blue-400" aria-hidden="true" />
                  )}
                </span>
                <span 
                  className={`w-2 h-2 rounded-full ${isSelected ? 'bg-blue-500' : 'bg-transparent'}`}
                  aria-hidden="true" 
                />
              </div>
              <p className="text-xs text-zinc-400 mt-1 line-clamp-1 leading-tight">
                {option.subtitle}
              </p>
            </button>
          );
        })}
      </div>
      {selectedGuidance && (
        <div className="rounded-lg border border-amber-900/50 bg-amber-950/15 px-3 py-2 text-[11px] leading-relaxed text-zinc-400">
          <span className="font-semibold text-amber-300">Quality note: </span>
          {selectedGuidance}
        </div>
      )}
    </div>
  );
};

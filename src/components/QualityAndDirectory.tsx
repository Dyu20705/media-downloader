import React from 'react';
import { FolderOpen } from 'lucide-react';
import { PresetType } from '../types';
import { ipc } from '../services/ipc';

interface QualityAndDirectoryProps {
  selectedPreset: PresetType;
  selectedQuality: string;
  onSelectQuality: (quality: string) => void;
  outputDirectory: string;
  onChangeOutputDirectory: (dir: string) => void;
  availableResolutions: number[];
  disabled?: boolean;
}

export const QualityAndDirectory: React.FC<QualityAndDirectoryProps> = ({
  selectedPreset,
  selectedQuality,
  onSelectQuality,
  outputDirectory,
  onChangeOutputDirectory,
  availableResolutions,
  disabled = false
}) => {
  const isAudioPreset = selectedPreset === 'best-audio' || selectedPreset === 'mp3' || selectedPreset === 'flac';

  const handleBrowse = async () => {
    try {
      const selected = await ipc.selectDirectory();
      if (selected) {
        onChangeOutputDirectory(selected);
      }
    } catch {
      // ignore
    }
  };

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
      {/* Quality Selection */}
      <div className="space-y-1.5">
        <label 
          htmlFor="select-media-quality" 
          className="block text-xs font-semibold uppercase tracking-wider text-zinc-400"
        >
          Quality
        </label>
        <div className="relative">
          <select
            id="select-media-quality"
            value={selectedQuality}
            onChange={(e) => onSelectQuality(e.target.value)}
            disabled={disabled || isAudioPreset}
            aria-label="Target Video Quality"
            className="w-full px-3.5 py-2.5 bg-zinc-900 border border-zinc-800 focus:border-blue-500 rounded-xl text-zinc-100 text-sm appearance-none cursor-pointer focus-visible:ring-2 focus-visible:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isAudioPreset ? (
              <option value="auto">Best Available Audio Bitrate</option>
            ) : (
              <>
                <option value="auto">Best Available (Auto)</option>
                <option value="2160">4K Ultra HD (2160p)</option>
                <option value="1440">1440p QHD</option>
                <option value="1080">1080p Full HD</option>
                <option value="720">720p HD</option>
                <option value="480">480p SD</option>
              </>
            )}
          </select>
          <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-3 text-zinc-400">
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M19 9l-7 7-7-7" />
            </svg>
          </div>
        </div>
      </div>

      {/* Save to Location */}
      <div className="space-y-1.5">
        <label 
          htmlFor="input-save-folder" 
          className="block text-xs font-semibold uppercase tracking-wider text-zinc-400"
        >
          Save to
        </label>
        <div className="flex gap-2">
          <input
            id="input-save-folder"
            type="text"
            value={outputDirectory}
            onChange={(e) => onChangeOutputDirectory(e.target.value)}
            disabled={disabled}
            placeholder="Download destination folder"
            aria-label="Download destination folder path"
            className="flex-1 px-3.5 py-2.5 bg-zinc-900 border border-zinc-800 focus:border-blue-500 rounded-xl text-zinc-200 text-sm font-mono transition-all focus-visible:ring-2 focus-visible:ring-blue-500"
          />
          <button
            type="button"
            id="btn-browse-folder"
            onClick={handleBrowse}
            disabled={disabled}
            className="px-3.5 py-2.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-sm font-medium rounded-xl border border-zinc-700 flex items-center gap-1.5 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
            aria-label="Browse destination directory"
          >
            <FolderOpen className="w-4 h-4 text-zinc-400" aria-hidden="true" />
            <span>Browse</span>
          </button>
        </div>
      </div>
    </div>
  );
};

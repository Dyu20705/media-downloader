import React, { useState, useEffect } from 'react';
import { X, Settings as SettingsIcon, Save, Wrench } from 'lucide-react';
import { AppSettings, PresetType } from '../types';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  settings: AppSettings;
  onSaveSettings: (newSettings: AppSettings) => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  settings,
  onSaveSettings,
}) => {
  const [form, setForm] = useState<AppSettings>(settings);

  useEffect(() => {
    setForm(settings);
  }, [settings]);

  if (!isOpen) return null;

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault();
    onSaveSettings(form);
    onClose();
  };

  return (
    <div 
      role="dialog" 
      aria-modal="true" 
      aria-labelledby="settings-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
    >
      <div className="w-full max-w-lg bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
        {/* Header */}
        <div className="px-5 py-4 bg-zinc-950 border-b border-zinc-800 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-lg bg-zinc-800 text-zinc-300">
              <SettingsIcon className="w-4 h-4 text-blue-400" aria-hidden="true" />
            </div>
            <div>
              <h2 id="settings-title" className="font-semibold text-base text-zinc-100">
                Settings
              </h2>
              <p className="text-xs text-zinc-400">
                Preferences and tool configuration
              </p>
            </div>
          </div>
          <button
            type="button"
            id="btn-close-settings"
            onClick={onClose}
            className="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
            aria-label="Close Settings"
          >
            <X className="w-5 h-5" aria-hidden="true" />
          </button>
        </div>

        {/* Content Form */}
        <form onSubmit={handleSave} className="p-6 overflow-y-auto space-y-5 text-sm">
          {/* Output Directory */}
          <div className="space-y-1.5">
            <label htmlFor="settings-download-dir" className="block text-xs font-semibold text-zinc-300 uppercase tracking-wider">
              Default Download Folder
            </label>
            <input
              id="settings-download-dir"
              type="text"
              value={form.downloadDirectory}
              onChange={(e) => setForm({ ...form, downloadDirectory: e.target.value })}
              className="w-full px-3.5 py-2.5 bg-zinc-950 border border-zinc-800 rounded-xl text-xs font-mono text-zinc-200 focus:border-blue-500 focus-visible:ring-2 focus-visible:ring-blue-500"
            />
          </div>

          {/* Default Preset */}
          <div className="space-y-1.5">
            <label htmlFor="settings-default-preset" className="block text-xs font-semibold text-zinc-300 uppercase tracking-wider">
              Default Format
            </label>
            <select
              id="settings-default-preset"
              value={form.lastPreset}
              onChange={(e) => setForm({ ...form, lastPreset: e.target.value as PresetType })}
              className="w-full px-3.5 py-2.5 bg-zinc-950 border border-zinc-800 rounded-xl text-xs text-zinc-200 focus:border-blue-500 focus-visible:ring-2 focus-visible:ring-blue-500"
            >
              <option value="mp4-compatible">MP4 — Compatible (Default Video)</option>
              <option value="best-video">Best Video (Highest Quality MKV)</option>
              <option value="best-audio">Best Audio (Source Preservation)</option>
              <option value="mp3">MP3 (Universal 320 kbps)</option>
              <option value="flac">FLAC (Lossless)</option>
            </select>
          </div>

          {/* Custom Tool Executable Path Overrides */}
          <div className="p-4 rounded-xl bg-zinc-950 border border-zinc-800 space-y-3">
            <div className="text-xs font-semibold text-zinc-200 flex items-center gap-1.5 uppercase tracking-wider">
              <Wrench className="w-3.5 h-3.5 text-zinc-400" aria-hidden="true" />
              <span>Custom Executable Paths (Optional)</span>
            </div>

            <div className="space-y-2 text-xs">
              <div>
                <label htmlFor="settings-custom-ytdlp" className="block text-[11px] text-zinc-400 font-mono mb-1">Custom yt-dlp path</label>
                <input
                  id="settings-custom-ytdlp"
                  type="text"
                  placeholder="Managed engine (default)"
                  value={form.customYtdlpPath || ''}
                  onChange={(e) => setForm({ ...form, customYtdlpPath: e.target.value })}
                  className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-200 focus:border-blue-500"
                />
              </div>

              <div>
                <label htmlFor="settings-custom-ffmpeg" className="block text-[11px] text-zinc-400 font-mono mb-1">Custom FFmpeg path</label>
                <input
                  id="settings-custom-ffmpeg"
                  type="text"
                  placeholder="Managed engine (default)"
                  value={form.customFfmpegPath || ''}
                  onChange={(e) => setForm({ ...form, customFfmpegPath: e.target.value })}
                  className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-200 focus:border-blue-500"
                />
              </div>

              <div>
                <label htmlFor="settings-custom-mediainfo" className="block text-[11px] text-zinc-400 font-mono mb-1">Custom MediaInfo path</label>
                <input
                  id="settings-custom-mediainfo"
                  type="text"
                  placeholder="Managed engine (default)"
                  value={form.customMediainfoPath || ''}
                  onChange={(e) => setForm({ ...form, customMediainfoPath: e.target.value })}
                  className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-200 focus:border-blue-500"
                />
              </div>
            </div>
          </div>

          {/* SponsorBlock Settings */}
          <div className="space-y-1.5">
            <label htmlFor="settings-sponsorblock" className="block text-xs font-semibold text-zinc-300 uppercase tracking-wider">
              SponsorBlock Integration
            </label>
            <select
              id="settings-sponsorblock"
              value={form.sponsorBlockMode || 'off'}
              onChange={(e) => setForm({ ...form, sponsorBlockMode: e.target.value as any })}
              className="w-full px-3.5 py-2.5 bg-zinc-950 border border-zinc-800 rounded-xl text-xs text-zinc-200 focus:border-blue-500 focus-visible:ring-2 focus-visible:ring-blue-500"
            >
              <option value="off">Off — Keep all original segments</option>
              <option value="mark-chapters">Mark Sponsor Sections as Chapters (Skip easily in players)</option>
              <option value="remove-segments">Remove Sponsor Segments (Auto-cut with FFmpeg)</option>
            </select>
            <p className="text-[11px] text-zinc-500">
              Detects sponsors, intros, and self-promotions using community SponsorBlock markers.
            </p>
          </div>

          {/* Subtitles & Captions */}
          <div className="space-y-2 p-3.5 rounded-xl bg-zinc-950 border border-zinc-800">
            <label htmlFor="settings-subtitles-mode" className="block text-xs font-semibold text-zinc-300 uppercase tracking-wider">
              Subtitles & Closed Captions
            </label>
            <select
              id="settings-subtitles-mode"
              value={form.subtitleMode || 'none'}
              onChange={(e) => setForm({ ...form, subtitleMode: e.target.value as any })}
              className="w-full px-3.5 py-2 bg-zinc-900 border border-zinc-800 rounded-lg text-xs text-zinc-200 focus:border-blue-500"
            >
              <option value="none">Do not download subtitles</option>
              <option value="embed">Embed subtitle tracks into video container</option>
              <option value="download-separate">Save as separate subtitle files (.vtt / .srt)</option>
            </select>

            {form.subtitleMode && form.subtitleMode !== 'none' && (
              <div className="pt-1.5 space-y-1">
                <label htmlFor="settings-sub-lang" className="block text-[11px] text-zinc-400 font-mono">
                  Preferred Languages (yt-dlp regex / comma-separated)
                </label>
                <input
                  id="settings-sub-lang"
                  type="text"
                  placeholder="en.*,en (or 'all' for all languages)"
                  value={form.preferredSubtitleLanguage || 'en'}
                  onChange={(e) => setForm({ ...form, preferredSubtitleLanguage: e.target.value })}
                  className="w-full px-3 py-1.5 bg-zinc-900 border border-zinc-800 rounded-lg text-xs font-mono text-zinc-200 focus:border-blue-500"
                />
              </div>
            )}
          </div>

          {/* Metadata Checkboxes */}
          <div className="space-y-2 pt-1">
            <label className="flex items-center gap-2.5 text-xs text-zinc-300 cursor-pointer">
              <input
                type="checkbox"
                checked={form.embedMetadata}
                onChange={(e) => setForm({ ...form, embedMetadata: e.target.checked })}
                className="rounded border-zinc-800 text-blue-600 focus:ring-0"
              />
              <span>Embed Title, Artist & Tags into file</span>
            </label>

            <label className="flex items-center gap-2.5 text-xs text-zinc-300 cursor-pointer">
              <input
                type="checkbox"
                checked={form.embedThumbnail}
                onChange={(e) => setForm({ ...form, embedThumbnail: e.target.checked })}
                className="rounded border-zinc-800 text-blue-600 focus:ring-0"
              />
              <span>Embed cover art / thumbnail</span>
            </label>

            <label className="flex items-center gap-2.5 text-xs text-zinc-300 cursor-pointer">
              <input
                type="checkbox"
                checked={form.embedChapters ?? true}
                onChange={(e) => setForm({ ...form, embedChapters: e.target.checked })}
                className="rounded border-zinc-800 text-blue-600 focus:ring-0"
              />
              <span>Embed video chapters into container</span>
            </label>

            <label className="flex items-center gap-2.5 text-xs text-zinc-300 cursor-pointer">
              <input
                type="checkbox"
                checked={form.openFolderAfterDownload}
                onChange={(e) => setForm({ ...form, openFolderAfterDownload: e.target.checked })}
                className="rounded border-zinc-800 text-blue-600 focus:ring-0"
              />
              <span>Open folder when download finishes</span>
            </label>
          </div>

          {/* Footer Save */}
          <div className="pt-4 border-t border-zinc-800 flex justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium rounded-xl transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold rounded-xl flex items-center gap-1.5 transition-colors focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer"
            >
              <Save className="w-3.5 h-3.5" aria-hidden="true" />
              <span>Save</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default SettingsModal;

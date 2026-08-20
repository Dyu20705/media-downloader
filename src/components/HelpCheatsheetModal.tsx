import React from 'react';
import { X, HelpCircle, FileText, CheckCircle2, Sliders, Keyboard } from 'lucide-react';

interface HelpCheatsheetModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const HelpCheatsheetModal: React.FC<HelpCheatsheetModalProps> = ({
  isOpen,
  onClose,
}) => {
  if (!isOpen) return null;

  return (
    <div 
      role="dialog" 
      aria-modal="true" 
      aria-labelledby="help-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
    >
      <div className="w-full max-w-3xl bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
        {/* Header */}
        <div className="px-5 py-4 bg-zinc-950 border-b border-zinc-800 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-zinc-800 text-zinc-300">
              <HelpCircle className="w-4 h-4 text-blue-400" aria-hidden="true" />
            </div>
            <div>
              <h2 id="help-title" className="font-semibold text-base text-zinc-100">
                Help & Cheatsheet
              </h2>
              <p className="text-xs text-zinc-400">
                Quick guide for format presets, quality, and tools
              </p>
            </div>
          </div>

          <button
            type="button"
            id="btn-close-help"
            onClick={onClose}
            className="p-1.5 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-lg transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
            aria-label="Close Help"
          >
            <X className="w-5 h-5" aria-hidden="true" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto space-y-6 text-sm text-zinc-300 leading-relaxed">
          {/* Format Presets Guide */}
          <section className="space-y-2.5">
            <h3 className="font-semibold text-zinc-100 flex items-center gap-2 text-sm">
              <FileText className="w-4 h-4 text-blue-400" aria-hidden="true" />
              <span>Format Presets Guide</span>
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs">
              <div className="p-3 bg-zinc-950 border border-zinc-800 rounded-xl space-y-1">
                <div className="font-semibold text-zinc-100">MP4 (Compatible)</div>
                <div className="text-zinc-400">H.264 + AAC. Compatible with virtually all TVs, phones, tablets, and editing suites.</div>
              </div>
              <div className="p-3 bg-zinc-950 border border-zinc-800 rounded-xl space-y-1">
                <div className="font-semibold text-zinc-100">Best Video</div>
                <div className="text-zinc-400">Preserves highest resolution (4K/8K, 60fps, HDR) in an MKV container.</div>
              </div>
              <div className="p-3 bg-zinc-950 border border-zinc-800 rounded-xl space-y-1">
                <div className="font-semibold text-zinc-100">Best Audio</div>
                <div className="text-zinc-400">Extracts source audio stream (Opus/AAC) directly with 0% generational loss.</div>
              </div>
              <div className="p-3 bg-zinc-950 border border-zinc-800 rounded-xl space-y-1">
                <div className="font-semibold text-zinc-100">MP3</div>
                <div className="text-zinc-400">Universal 320 kbps CBR MP3 audio compatible with legacy players.</div>
              </div>
            </div>
          </section>

          {/* Quality Options */}
          <section className="space-y-2.5">
            <h3 className="font-semibold text-zinc-100 flex items-center gap-2 text-sm">
              <Sliders className="w-4 h-4 text-emerald-400" aria-hidden="true" />
              <span>Quality Settings</span>
            </h3>
            <p className="text-xs text-zinc-400">
              When using video presets, you can choose a maximum ceiling resolution. The engine will download the highest available stream matching or below your selection:
            </p>
            <div className="p-3 bg-zinc-950 border border-zinc-800 rounded-xl text-xs space-y-1 font-mono text-zinc-300">
              <div>• Auto: Best available stream from host (up to 4K/8K)</div>
              <div>• 2160p: 4K Ultra HD (3840 × 2160)</div>
              <div>• 1080p: Full HD (1920 × 1080)</div>
              <div>• 720p: Standard HD (1280 × 720)</div>
            </div>
          </section>

          {/* Engine Tools & Security */}
          <section className="space-y-2.5">
            <h3 className="font-semibold text-zinc-100 flex items-center gap-2 text-sm">
              <CheckCircle2 className="w-4 h-4 text-purple-400" aria-hidden="true" />
              <span>Engine Tools & Security</span>
            </h3>
            <p className="text-xs text-zinc-400">
              The application uses pinned, verified binaries (<code className="text-zinc-200">yt-dlp</code>, <code className="text-zinc-200">FFmpeg</code>, <code className="text-zinc-200">MediaInfo</code>) isolated in an application folder without modifying your system environment or registry.
            </p>
          </section>

          {/* Keyboard Shortcuts */}
          <section className="space-y-2.5">
            <h3 className="font-semibold text-zinc-100 flex items-center gap-2 text-sm">
              <Keyboard className="w-4 h-4 text-amber-400" aria-hidden="true" />
              <span>Keyboard Shortcuts</span>
            </h3>
            <div className="grid grid-cols-2 gap-2 text-xs font-mono">
              <div className="p-2.5 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between">
                <span className="text-zinc-400">Paste URL</span>
                <span className="px-1.5 py-0.5 bg-zinc-800 rounded text-zinc-200">Ctrl + V</span>
              </div>
              <div className="p-2.5 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between">
                <span className="text-zinc-400">Analyze URL</span>
                <span className="px-1.5 py-0.5 bg-zinc-800 rounded text-zinc-200">Enter</span>
              </div>
              <div className="p-2.5 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between">
                <span className="text-zinc-400">Tab Focus</span>
                <span className="px-1.5 py-0.5 bg-zinc-800 rounded text-zinc-200">Tab</span>
              </div>
              <div className="p-2.5 bg-zinc-950 border border-zinc-800 rounded-lg flex items-center justify-between">
                <span className="text-zinc-400">Close Modal</span>
                <span className="px-1.5 py-0.5 bg-zinc-800 rounded text-zinc-200">Escape</span>
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
};

export default HelpCheatsheetModal;

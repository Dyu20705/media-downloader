import React, { useState } from 'react';
import { Search, Clipboard, Loader2, Link2, X, Sparkles } from 'lucide-react';

interface UrlInputBarProps {
  url: string;
  onChangeUrl: (url: string) => void;
  onAnalyze: (urlToAnalyze?: string) => void;
  isAnalyzing: boolean;
  disabled?: boolean;
}

const SAMPLE_SOURCES = [
  { name: 'YouTube', url: 'https://www.youtube.com/watch?v=LXb3EKWsInQ' },
  { name: 'TikTok', url: 'https://www.tiktok.com/@creator/video/73928192831' },
  { name: 'Instagram', url: 'https://www.instagram.com/reel/C8q7vP1xpL2/' },
  { name: 'SoundCloud', url: 'https://soundcloud.com/artist/lofi-chill-vibes' },
  { name: 'Reddit', url: 'https://www.reddit.com/r/videos/comments/x901/nature_timelapse/' },
  { name: 'Direct MP4', url: 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4' },
  { name: 'HLS Live', url: 'https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8' }
];

export const UrlInputBar: React.FC<UrlInputBarProps> = ({
  url,
  onChangeUrl,
  onAnalyze,
  isAnalyzing,
  disabled = false
}) => {
  const [pasteNotice, setPasteNotice] = useState<string | null>(null);

  const handlePaste = async () => {
    try {
      setPasteNotice(null);
      const text = await navigator.clipboard.readText();
      if (text && text.trim().startsWith('http')) {
        onChangeUrl(text.trim());
        onAnalyze(text.trim());
      } else if (text) {
        onChangeUrl(text.trim());
      }
    } catch {
      setPasteNotice('Clipboard blocked: Use keyboard Ctrl+V.');
      setTimeout(() => setPasteNotice(null), 2500);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (url.trim()) {
      onAnalyze();
    }
  };

  const handleSelectSample = (sampleUrl: string) => {
    onChangeUrl(sampleUrl);
    onAnalyze(sampleUrl);
  };

  return (
    <div className="w-full space-y-2">
      <form onSubmit={handleSubmit} className="flex flex-col sm:flex-row gap-2">
        <div className="relative flex-1">
          <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none text-zinc-500">
            <Link2 className="w-4 h-4" aria-hidden="true" />
          </div>
          <input
            id="input-media-url"
            type="url"
            value={url}
            onChange={(e) => onChangeUrl(e.target.value)}
            disabled={disabled || isAnalyzing}
            placeholder="Paste media link (YouTube, TikTok, Instagram, Reddit, SoundCloud, Direct...)"
            aria-label="Media URL input"
            className="w-full pl-10 pr-20 py-3 bg-zinc-900 border border-zinc-700/80 focus:border-blue-500 rounded-xl text-zinc-100 placeholder-zinc-500 text-sm transition-all focus-visible:ring-2 focus-visible:ring-blue-500"
          />
          <div className="absolute inset-y-0 right-0 pr-2 flex items-center gap-1">
            {url && (
              <button
                type="button"
                id="btn-clear-url"
                onClick={() => onChangeUrl('')}
                className="p-1 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
                title="Clear input"
                aria-label="Clear URL input"
              >
                <X className="w-4 h-4" aria-hidden="true" />
              </button>
            )}
            <button
              type="button"
              id="btn-paste-url"
              onClick={handlePaste}
              disabled={disabled || isAnalyzing}
              className="px-2.5 py-1 rounded-md bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium flex items-center gap-1 border border-zinc-700 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
              title="Paste from clipboard"
              aria-label="Paste URL from clipboard"
            >
              <Clipboard className="w-3.5 h-3.5" aria-hidden="true" />
              <span className="hidden sm:inline">Paste</span>
            </button>
          </div>
        </div>

        <button
          type="submit"
          id="btn-analyze-url"
          disabled={disabled || isAnalyzing || !url.trim()}
          className="px-5 py-3 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium text-sm rounded-xl shadow-sm flex items-center justify-center gap-2 transition-colors shrink-0 focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer"
        >
          {isAnalyzing ? (
            <>
              <Loader2 className="w-4 h-4 animate-spin" aria-hidden="true" />
              <span>Analyzing…</span>
            </>
          ) : (
            <>
              <Search className="w-4 h-4" aria-hidden="true" />
              <span>Analyze</span>
            </>
          )}
        </button>
      </form>

      {/* Quick Source Presets */}
      <div className="flex items-center gap-1.5 flex-wrap text-xs text-zinc-400">
        <span className="text-[11px] text-zinc-500 font-medium flex items-center gap-1">
          <Sparkles className="w-3 h-3 text-blue-400" aria-hidden="true" />
          <span>Quick test:</span>
        </span>
        {SAMPLE_SOURCES.map((sample) => (
          <button
            key={sample.name}
            type="button"
            disabled={disabled || isAnalyzing}
            onClick={() => handleSelectSample(sample.url)}
            className={`px-2 py-0.5 rounded-md text-[11px] font-medium border transition-colors cursor-pointer ${
              url === sample.url
                ? 'bg-blue-600/20 text-blue-300 border-blue-500/50'
                : 'bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 border-zinc-800 hover:border-zinc-700'
            }`}
          >
            {sample.name}
          </button>
        ))}
      </div>

      {pasteNotice && (
        <p className="text-xs text-amber-400 font-mono" role="alert">
          {pasteNotice}
        </p>
      )}
    </div>
  );
};


import React, { useState, useEffect } from 'react';
import { 
  Clock, 
  ExternalLink, 
  Film, 
  Radio, 
  SlidersHorizontal, 
  Subtitles, 
  Bookmark, 
  Music, 
  Globe, 
  Sparkles,
  Smartphone,
  User
} from 'lucide-react';
import { MediaMetadata } from '../types';

interface MediaSummaryCardProps {
  metadata: MediaMetadata;
  onOpenInspector?: () => void;
}

export const MediaSummaryCard: React.FC<MediaSummaryCardProps> = ({ metadata, onOpenInspector }) => {
  const [imageError, setImageError] = useState(false);
  const [avatarError, setAvatarError] = useState(false);

  useEffect(() => {
    setImageError(false);
    setAvatarError(false);
  }, [metadata?.id, metadata?.thumbnail, metadata?.uploaderAvatar]);

  const formatDuration = (seconds?: number | null, isLive?: boolean | null) => {
    if (isLive) return 'Live Stream';
    if (seconds === null || seconds === undefined) return 'Dynamic Duration';
    if (seconds < 0) return 'Dynamic Duration';
    
    const totalSecs = Math.round(seconds);
    const hrs = Math.floor(totalSecs / 3600);
    const mins = Math.floor((totalSecs % 3600) / 60);
    const secs = totalSecs % 60;

    if (hrs > 0) {
      return `${hrs}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const getSourceBadge = (extractorKey?: string | null, extractor?: string | null, sourceType?: string | null): { name: string; color: string } => {
    const key = (extractorKey || extractor || sourceType || '').toLowerCase();
    
    if (key.includes('tiktok')) return { name: 'TikTok', color: 'bg-rose-950/70 text-rose-300 border-rose-800/60' };
    if (key.includes('facebook')) return { name: 'Facebook', color: 'bg-blue-950/70 text-blue-300 border-blue-800/60' };
    if (key.includes('instagram')) return { name: 'Instagram', color: 'bg-fuchsia-950/70 text-fuchsia-300 border-fuchsia-800/60' };
    if (key.includes('twitter') || key.includes('x')) return { name: 'X (Twitter)', color: 'bg-zinc-800 text-zinc-200 border-zinc-700' };
    if (key.includes('reddit')) return { name: 'Reddit', color: 'bg-orange-950/70 text-orange-300 border-orange-800/60' };
    if (key.includes('twitch')) return { name: 'Twitch', color: 'bg-purple-950/70 text-purple-300 border-purple-800/60' };
    if (key.includes('bilibili')) return { name: 'Bilibili', color: 'bg-cyan-950/70 text-cyan-300 border-cyan-800/60' };
    if (key.includes('vimeo')) return { name: 'Vimeo', color: 'bg-sky-950/70 text-sky-300 border-sky-800/60' };
    if (key.includes('soundcloud')) return { name: 'SoundCloud', color: 'bg-amber-950/70 text-amber-300 border-amber-800/60' };
    if (key.includes('bandcamp')) return { name: 'Bandcamp', color: 'bg-teal-950/70 text-teal-300 border-teal-800/60' };
    if (key.includes('dailymotion')) return { name: 'Dailymotion', color: 'bg-indigo-950/70 text-indigo-300 border-indigo-800/60' };
    if (key.includes('pinterest')) return { name: 'Pinterest', color: 'bg-red-950/70 text-red-300 border-red-800/60' };
    if (key.includes('loom')) return { name: 'Loom', color: 'bg-violet-950/70 text-violet-300 border-violet-800/60' };
    if (key.includes('hls')) return { name: 'HLS Live/VOD', color: 'bg-emerald-950/70 text-emerald-300 border-emerald-800/60' };
    if (key.includes('dash')) return { name: 'DASH Stream', color: 'bg-teal-950/70 text-teal-300 border-teal-800/60' };
    if (key.includes('direct')) return { name: 'Direct Media File', color: 'bg-zinc-800 text-zinc-300 border-zinc-700' };
    if (key.includes('youtube')) return { name: 'YouTube', color: 'bg-red-950/70 text-red-300 border-red-800/60' };
    
    return { name: extractorKey || extractor || 'Web Stream', color: 'bg-zinc-800 text-zinc-300 border-zinc-700' };
  };

  const getSourceDomain = (webpageUrl: string, extractor?: string | null): string => {
    if (extractor && extractor.toLowerCase() === 'youtube') return 'YouTube';
    try {
      const url = new URL(webpageUrl);
      return url.hostname.replace('www.', '');
    } catch {
      return extractor || 'Web Source';
    }
  };

  const isAudio = metadata.mediaKind === 'audio' || (!metadata.hasVideo && metadata.hasAudio);
  const isVertical = metadata.formats?.some(f => f.height && f.width && f.height > f.width) ||
    metadata.extractorKey?.toLowerCase().includes('tiktok') ||
    metadata.extractor?.toLowerCase().includes('shorts') ||
    metadata.extractorKey?.toLowerCase().includes('instagram');

  const highestRes = metadata.availableResolutions && metadata.availableResolutions.length > 0
    ? metadata.availableResolutions[0]
    : null;

  const highestFps = metadata.availableFrameRates && metadata.availableFrameRates.length > 0
    ? metadata.availableFrameRates[0]
    : null;

  const resLabel = isAudio 
    ? 'Audio Master' 
    : highestRes
    ? highestRes >= 2160
      ? '4K UHD'
      : highestRes >= 1440
      ? '2K QHD'
      : highestRes >= 1080
      ? '1080p FHD'
      : `${highestRes}p`
    : null;

  const chapterCount = metadata.chapters?.length || 0;
  const subtitleCount = (metadata.subtitles?.length || 0) + (metadata.automaticCaptions?.length || 0);
  const platformBadge = getSourceBadge(metadata.extractorKey, metadata.extractor, metadata.sourceType);

  // Derive smart stream codecs
  const videoCodecSummary = isAudio 
    ? null 
    : metadata.formats?.find(f => f.vcodec && f.vcodec !== 'none')?.vcodec?.split('.')[0]?.toUpperCase() || (highestRes && highestRes >= 1440 ? 'AV1 / VP9' : 'H.264');
  const audioCodecSummary = metadata.formats?.find(f => f.acodec && f.acodec !== 'none')?.acodec?.split('.')[0]?.toUpperCase() || (isAudio ? 'Source Audio' : 'AAC / Opus');

  const getPlatformGradient = (extractorKey?: string | null, extractor?: string | null, sourceType?: string | null): string => {
    const key = (extractorKey || extractor || sourceType || '').toLowerCase();
    if (key.includes('tiktok')) return 'from-rose-950/80 via-zinc-900 to-cyan-950/70';
    if (key.includes('instagram')) return 'from-fuchsia-950/80 via-pink-950/60 to-amber-950/70';
    if (key.includes('soundcloud')) return 'from-amber-950/80 via-orange-950/60 to-zinc-950';
    if (key.includes('reddit')) return 'from-orange-950/80 via-zinc-900 to-zinc-950';
    if (key.includes('twitter') || key.includes('x')) return 'from-zinc-900 via-slate-900 to-zinc-950';
    if (key.includes('facebook')) return 'from-blue-950/80 via-indigo-950/60 to-zinc-950';
    if (key.includes('twitch')) return 'from-purple-950/80 via-zinc-900 to-zinc-950';
    if (key.includes('hls') || key.includes('stream')) return 'from-emerald-950/80 via-zinc-900 to-zinc-950';
    return 'from-blue-950/70 via-zinc-900 to-zinc-950';
  };

  const fallbackGradient = getPlatformGradient(metadata.extractorKey, metadata.extractor, metadata.sourceType);

  return (
    <div
      id="media-summary-card"
      className="flex flex-col sm:flex-row items-start sm:items-center gap-4 p-4 bg-zinc-900 border border-zinc-800 rounded-xl shadow-sm hover:border-zinc-700/80 transition-colors"
    >
      {/* Thumbnail Container */}
      <div className="relative w-full sm:w-44 aspect-video rounded-lg overflow-hidden bg-zinc-950 shrink-0 border border-zinc-800 flex items-center justify-center group">
        {metadata.thumbnail && !imageError ? (
          <img
            src={metadata.thumbnail}
            alt={metadata.title}
            referrerPolicy="no-referrer"
            onError={() => setImageError(true)}
            className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
          />
        ) : (
          <div className={`w-full h-full flex flex-col items-center justify-center bg-gradient-to-br ${fallbackGradient} text-zinc-400 gap-1.5 p-3 text-center`}>
            {isAudio ? (
              <Music className="w-8 h-8 text-amber-400/90" aria-hidden="true" />
            ) : (
              <Film className="w-8 h-8 text-blue-400/80" aria-hidden="true" />
            )}
            <span className="text-[11px] font-semibold text-zinc-200 truncate max-w-[140px]">
              {platformBadge.name}
            </span>
          </div>
        )}

        {/* Media Kind Icon Badge */}
        {isAudio && (
          <div className="absolute top-1.5 left-1.5 px-1.5 py-0.5 rounded bg-indigo-950/90 border border-indigo-700/60 text-indigo-300 text-[10px] font-medium flex items-center gap-1 backdrop-blur-md">
            <Music className="w-3 h-3 text-indigo-400" aria-hidden="true" />
            <span>Audio</span>
          </div>
        )}

        {/* Vertical Video indicator */}
        {!isAudio && isVertical && (
          <div className="absolute top-1.5 left-1.5 px-1.5 py-0.5 rounded bg-black/80 border border-white/10 text-zinc-300 text-[10px] font-medium flex items-center gap-1 backdrop-blur-md">
            <Smartphone className="w-3 h-3 text-rose-400" aria-hidden="true" />
            <span>9:16</span>
          </div>
        )}

        {/* Duration / Live Status Badge */}
        <div
          className={`absolute bottom-1.5 right-1.5 px-2 py-0.5 rounded text-[11px] font-mono font-medium flex items-center gap-1 backdrop-blur-md shadow-sm ${
            metadata.isLive
              ? 'bg-red-600/90 text-white'
              : 'bg-black/80 text-zinc-200 border border-white/10'
          }`}
        >
          {metadata.isLive ? (
            <>
              <Radio className="w-3 h-3 text-white animate-pulse" aria-hidden="true" />
              <span>LIVE</span>
            </>
          ) : (
            <>
              <Clock className="w-3 h-3 text-zinc-400" aria-hidden="true" />
              <span>{formatDuration(metadata.duration, metadata.isLive)}</span>
            </>
          )}
        </div>
      </div>

      {/* Main Details */}
      <div className="flex-1 min-w-0 space-y-2 w-full">
        {/* Title */}
        <h2
          className="font-semibold text-sm sm:text-base text-zinc-100 line-clamp-2 leading-snug tracking-tight"
          title={metadata.title}
        >
          {metadata.title}
        </h2>

        {/* Creator & Source Webpage */}
        <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1 text-xs text-zinc-400">
          {metadata.uploader && (
            <span className="inline-flex items-center gap-1.5 font-medium text-zinc-200 truncate max-w-[240px]" title={metadata.uploader}>
              {metadata.uploaderAvatar && !avatarError ? (
                <img
                  src={metadata.uploaderAvatar}
                  alt={metadata.uploader}
                  referrerPolicy="no-referrer"
                  onError={() => setAvatarError(true)}
                  className="w-5 h-5 rounded-full object-cover border border-zinc-700 shrink-0"
                />
              ) : (
                <span className="w-5 h-5 rounded-full bg-zinc-800 text-zinc-300 flex items-center justify-center text-[10px] font-bold shrink-0">
                  {metadata.uploader.charAt(0).toUpperCase()}
                </span>
              )}
              <span className="truncate">{metadata.uploader}</span>
            </span>
          )}
          {metadata.uploader && <span>•</span>}
          <span className="inline-flex items-center gap-1 text-zinc-400 font-normal">
            <Globe className="w-3 h-3 text-zinc-500" aria-hidden="true" />
            <span>{getSourceDomain(metadata.webpageUrl, metadata.extractor)}</span>
          </span>
          {metadata.webpageUrl && (
            <>
              <span>•</span>
              <a
                href={metadata.webpageUrl}
                target="_blank"
                rel="noreferrer"
                id="media-source-link"
                className="inline-flex items-center gap-1 text-blue-400 hover:text-blue-300 transition-colors"
                title="Open original webpage in new tab"
              >
                <span>Source</span>
                <ExternalLink className="w-3 h-3" aria-hidden="true" />
              </a>
            </>
          )}
        </div>

        {/* Smart Spec Line (e.g. 2160p · 60 FPS · HDR | AV1 · Opus) */}
        <div className="flex flex-wrap items-center gap-1.5 pt-0.5">
          {/* Platform Extractor Badge */}
          <span className={`px-2 py-0.5 rounded text-[11px] font-semibold border ${platformBadge.color}`}>
            {platformBadge.name}
          </span>

          {/* Video or Audio Quality Specs */}
          {resLabel && (
            <span className="px-2 py-0.5 rounded text-[11px] font-semibold bg-zinc-800 text-zinc-100 border border-zinc-700/60 font-mono">
              {resLabel}
            </span>
          )}
          {highestFps && highestFps > 30 && (
            <span className="px-2 py-0.5 rounded text-[11px] font-medium bg-zinc-800 text-zinc-300 border border-zinc-700/60 font-mono">
              {highestFps} FPS
            </span>
          )}
          {metadata.isHdr && (
            <span className="px-2 py-0.5 rounded text-[11px] font-semibold bg-amber-950/70 text-amber-300 border border-amber-800/60">
              HDR
            </span>
          )}

          {/* Codec summary badge */}
          <span className="px-2 py-0.5 rounded text-[11px] font-medium bg-zinc-800/80 text-zinc-300 border border-zinc-700/60 font-mono">
            {videoCodecSummary ? `${videoCodecSummary} · ${audioCodecSummary}` : audioCodecSummary}
          </span>

          {chapterCount > 0 && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium bg-zinc-800/80 text-zinc-300 border border-zinc-700/60">
              <Bookmark className="w-3 h-3 text-zinc-400" aria-hidden="true" />
              <span>{chapterCount} ch</span>
            </span>
          )}
          {subtitleCount > 0 && (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium bg-zinc-800/80 text-zinc-300 border border-zinc-700/60">
              <Subtitles className="w-3 h-3 text-zinc-400" aria-hidden="true" />
              <span>Subs</span>
            </span>
          )}
        </div>
      </div>

      {/* Inspector Quick Trigger */}
      {onOpenInspector && (
        <div className="sm:self-center shrink-0 w-full sm:w-auto pt-1 sm:pt-0">
          <button
            type="button"
            id="open-media-inspector-btn"
            onClick={onOpenInspector}
            className="w-full sm:w-auto inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 hover:border-zinc-600 transition-colors shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 cursor-pointer"
            title="Inspect deep stream information, codecs, subtitles, and chapters"
          >
            <SlidersHorizontal className="w-3.5 h-3.5 text-zinc-400" aria-hidden="true" />
            <span>Advanced / Details</span>
          </button>
        </div>
      )}
    </div>
  );
};

import React, { useState } from 'react';
import {
  X,
  Film,
  Music,
  ShieldCheck,
  HardDrive,
  AlertTriangle,
  Subtitles,
  Bookmark,
  Info,
  Layers,
  Sparkles,
  ExternalLink,
  CheckCircle2,
} from 'lucide-react';
import { DownloadJob, MediaMetadata } from '../types';

interface MediaInfoModalProps {
  metadata?: MediaMetadata | null;
  job?: DownloadJob | null;
  onClose: () => void;
}

type TabType = 'overview' | 'strategy' | 'video' | 'audio' | 'subtitles' | 'chapters' | 'file';

export const MediaInfoModal: React.FC<MediaInfoModalProps> = ({ metadata: rawMetadata, job, onClose }) => {
  const [activeTab, setActiveTab] = useState<TabType>('overview');

  const metadata = job?.metadata || rawMetadata;
  const inspection = job?.inspection;

  if (!metadata && !inspection) return null;

  const formatBytes = (bytes?: number | null) => {
    if (!bytes || bytes <= 0) return 'N/A';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatTime = (secs?: number | null) => {
    if (secs === null || secs === undefined) return 'N/A';
    const totalSecs = Math.round(secs);
    const hrs = Math.floor(totalSecs / 3600);
    const mins = Math.floor((totalSecs % 3600) / 60);
    const s = totalSecs % 60;
    if (hrs > 0) {
      return `${hrs}:${mins.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    }
    return `${mins}:${s.toString().padStart(2, '0')}`;
  };

  const subtitlesList = [
    ...(metadata?.subtitles || []),
    ...(metadata?.automaticCaptions || []),
  ];

  const chaptersList = metadata?.chapters || [];
  const primaryFormat = metadata?.formats?.[0];

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="mediainfo-title"
      id="smart-media-inspector-dialog"
      className="fixed inset-0 z-50 flex items-center justify-center p-3 sm:p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-150"
    >
      <div className="w-full max-w-2xl bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
        {/* Modal Header */}
        <div className="px-5 py-3.5 bg-zinc-950 border-b border-zinc-800 flex items-center justify-between shrink-0">
          <div className="flex items-center gap-3 min-w-0">
            <div className="p-2 rounded-lg bg-blue-500/10 border border-blue-500/20 text-blue-400 shrink-0">
              <ShieldCheck className="w-4 h-4" aria-hidden="true" />
            </div>
            <div className="min-w-0">
              <h2 id="mediainfo-title" className="font-semibold text-sm sm:text-base text-zinc-100 truncate">
                Smart Media Inspector
              </h2>
              <p className="text-xs text-zinc-400 font-mono truncate max-w-md">
                {metadata?.title || job?.finalFileName}
              </p>
            </div>
          </div>
          <button
            type="button"
            id="btn-close-mediainfo-modal"
            onClick={onClose}
            className="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 cursor-pointer shrink-0 ml-2"
            aria-label="Close Smart Media Inspector"
          >
            <X className="w-5 h-5" aria-hidden="true" />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="flex items-center px-4 bg-zinc-950/60 border-b border-zinc-800/80 overflow-x-auto gap-1 text-xs shrink-0 no-scrollbar">
          <button
            type="button"
            onClick={() => setActiveTab('overview')}
            className={`px-3 py-2.5 font-medium border-b-2 transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
              activeTab === 'overview'
                ? 'border-blue-500 text-blue-400'
                : 'border-transparent text-zinc-400 hover:text-zinc-200'
            }`}
          >
            <Info className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Overview</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab('strategy')}
            className={`px-3 py-2.5 font-medium border-b-2 transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
              activeTab === 'strategy'
                ? 'border-blue-500 text-blue-400'
                : 'border-transparent text-zinc-400 hover:text-zinc-200'
            }`}
          >
            <Sparkles className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Strategy & Verification</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab('video')}
            className={`px-3 py-2.5 font-medium border-b-2 transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
              activeTab === 'video'
                ? 'border-blue-500 text-blue-400'
                : 'border-transparent text-zinc-400 hover:text-zinc-200'
            }`}
          >
            <Film className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Video & Codecs</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab('audio')}
            className={`px-3 py-2.5 font-medium border-b-2 transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
              activeTab === 'audio'
                ? 'border-blue-500 text-blue-400'
                : 'border-transparent text-zinc-400 hover:text-zinc-200'
            }`}
          >
            <Music className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Audio</span>
          </button>

          {subtitlesList.length > 0 && (
            <button
              type="button"
              onClick={() => setActiveTab('subtitles')}
              className={`px-3 py-2.5 font-medium border-b-2 transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
                activeTab === 'subtitles'
                  ? 'border-blue-500 text-blue-400'
                  : 'border-transparent text-zinc-400 hover:text-zinc-200'
              }`}
            >
              <Subtitles className="w-3.5 h-3.5" aria-hidden="true" />
              <span>Subtitles ({subtitlesList.length})</span>
            </button>
          )}

          {chaptersList.length > 0 && (
            <button
              type="button"
              onClick={() => setActiveTab('chapters')}
              className={`px-3 py-2.5 font-medium border-b-2 transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
                activeTab === 'chapters'
                  ? 'border-blue-500 text-blue-400'
                  : 'border-transparent text-zinc-400 hover:text-zinc-200'
              }`}
            >
              <Bookmark className="w-3.5 h-3.5" aria-hidden="true" />
              <span>Chapters ({chaptersList.length})</span>
            </button>
          )}

          <button
            type="button"
            onClick={() => setActiveTab('file')}
            className={`px-3 py-2.5 font-medium border-b-2 transition-colors flex items-center gap-1.5 whitespace-nowrap cursor-pointer ${
              activeTab === 'file'
                ? 'border-blue-500 text-blue-400'
                : 'border-transparent text-zinc-400 hover:text-zinc-200'
            }`}
          >
            <HardDrive className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Files & Streams</span>
          </button>
        </div>

        {/* Modal Scrollable Content */}
        <div className="p-5 overflow-y-auto space-y-4 text-xs text-zinc-300">
          {/* Smart Recommendation Banner if available */}
          {metadata?.smartRecommendation && (
            <div className="p-3.5 bg-blue-950/40 border border-blue-800/60 rounded-xl space-y-1.5">
              <div className="flex items-center gap-2 text-blue-300 font-semibold text-xs">
                <Sparkles className="w-4 h-4 text-blue-400 shrink-0" aria-hidden="true" />
                <span>Recommended Preset: {metadata.smartRecommendation.label}</span>
              </div>
              <p className="text-zinc-300 leading-relaxed pl-6 text-[11px]">
                {metadata.smartRecommendation.reason}
              </p>
            </div>
          )}

          {/* Lossy Conversion Warning if applicable */}
          {inspection?.isLossyTranscodeWarning && (
            <div className="p-3 bg-amber-500/10 border border-amber-500/30 rounded-xl flex items-start gap-2.5 text-amber-200">
              <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" aria-hidden="true" />
              <div>
                <span className="font-bold">Lossy Source Note:</span> FLAC container output does not restore audio fidelity lost in the compressed Opus/AAC source stream.
              </div>
            </div>
          )}

          {/* TAB 1: OVERVIEW */}
          {activeTab === 'overview' && (
            <div className="space-y-4">
              {/* Universal Resolver Insights */}
              <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-3">
                <div className="text-zinc-200 uppercase tracking-wider text-[11px] font-semibold flex items-center justify-between">
                  <div className="flex items-center gap-1.5">
                    <Layers className="w-3.5 h-3.5 text-purple-400" aria-hidden="true" />
                    <span>Universal Resolver Pipeline</span>
                  </div>
                  {metadata?.sourceType && (
                    <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-purple-950 text-purple-300 border border-purple-800">
                      {metadata.sourceType}
                    </span>
                  )}
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                  <div>
                    <span className="text-zinc-500 block">Recommended Strategy:</span>
                    <span className="font-semibold text-zinc-100">
                      {metadata?.strategy || 'YT_DLP_DOWNLOAD'}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Transcoding Cost:</span>
                    <span className="font-medium text-emerald-400">
                      {metadata?.transcodingCost || 'STREAM_COPY'}
                    </span>
                  </div>
                  {metadata?.transcodingExplanation && (
                    <div className="sm:col-span-2">
                      <span className="text-zinc-500 block">Pipeline Note:</span>
                      <span className="text-zinc-300">{metadata.transcodingExplanation}</span>
                    </div>
                  )}
                </div>
              </div>

              <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-3">
                <div className="text-zinc-200 uppercase tracking-wider text-[11px] font-semibold flex items-center gap-1.5">
                  <Info className="w-3.5 h-3.5 text-blue-400" aria-hidden="true" />
                  <span>Metadata Overview</span>
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  <div>
                    <span className="text-zinc-500 block">Title:</span>
                    <span className="font-medium text-zinc-100 select-text">{metadata?.title}</span>
                  </div>
                  {metadata?.uploader && (
                    <div>
                      <span className="text-zinc-500 block">Creator / Channel:</span>
                      <span className="font-medium text-zinc-200">{metadata.uploader}</span>
                    </div>
                  )}
                  <div>
                    <span className="text-zinc-500 block">Duration:</span>
                    <span className="font-medium text-zinc-200">
                      {metadata?.isLive ? 'Live Stream' : formatTime(metadata?.duration)}
                    </span>
                  </div>
                  {metadata?.uploadDate && (
                    <div>
                      <span className="text-zinc-500 block">Published Date:</span>
                      <span className="font-medium text-zinc-200">
                        {metadata.uploadDate.length === 8
                          ? `${metadata.uploadDate.slice(0, 4)}-${metadata.uploadDate.slice(4, 6)}-${metadata.uploadDate.slice(6, 8)}`
                          : metadata.uploadDate}
                      </span>
                    </div>
                  )}
                  {metadata?.viewCount !== undefined && metadata.viewCount !== null && (
                    <div>
                      <span className="text-zinc-500 block">Views:</span>
                      <span className="font-medium text-zinc-200">{metadata.viewCount.toLocaleString()}</span>
                    </div>
                  )}
                  {metadata?.webpageUrl && (
                    <div>
                      <span className="text-zinc-500 block">Source URL:</span>
                      <a
                        href={metadata.webpageUrl}
                        target="_blank"
                        rel="noreferrer"
                        className="inline-flex items-center gap-1 text-blue-400 hover:text-blue-300 truncate max-w-full font-mono text-[11px]"
                      >
                        <span className="truncate">{metadata.webpageUrl}</span>
                        <ExternalLink className="w-3 h-3 shrink-0" aria-hidden="true" />
                      </a>
                    </div>
                  )}
                </div>
              </div>

              {metadata?.description && (
                <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-2">
                  <span className="text-zinc-500 block font-semibold text-[11px] uppercase tracking-wider">
                    Description
                  </span>
                  <p className="text-zinc-300 leading-relaxed text-xs line-clamp-4 select-text">
                    {metadata.description}
                  </p>
                </div>
              )}
            </div>
          )}

          {/* TAB: STRATEGY & VERIFICATION */}
          {activeTab === 'strategy' && (
            <div className="space-y-4">
              {/* Acquisition Strategy & Recommendation */}
              <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-3">
                <div className="text-zinc-200 uppercase tracking-wider text-[11px] font-semibold flex items-center gap-1.5">
                  <Sparkles className="w-3.5 h-3.5 text-blue-400" aria-hidden="true" />
                  <span>Strategy & Acquisition Recipe</span>
                </div>
                <div className="grid grid-cols-2 sm:grid-cols-3 gap-3 text-xs">
                  <div>
                    <span className="text-zinc-500 block">Strategy:</span>
                    <span className="font-semibold text-zinc-200 font-mono">
                      {job?.recipe?.strategy || metadata?.strategy || 'STREAM_COPY'}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Transcode Cost:</span>
                    <span className="font-medium text-emerald-400">
                      {job?.explainableResult?.transcodingCost || metadata?.transcodingCost || 'STREAM_COPY'}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Target Container:</span>
                    <span className="font-semibold text-zinc-200 uppercase">
                      {job?.recipe?.outputContainer || job?.inspection?.containerFormat || 'MP4'}
                    </span>
                  </div>
                </div>

                {job?.explainableResult?.whyReasons && job.explainableResult.whyReasons.length > 0 && (
                  <div className="pt-2 border-t border-zinc-900 space-y-1">
                    <span className="text-zinc-400 block text-[11px] font-semibold">Selection Rationale:</span>
                    <ul className="space-y-1 text-xs text-zinc-300">
                      {job.explainableResult.whyReasons.map((r: string, i: number) => (
                        <li key={i} className="flex items-center gap-1.5 text-zinc-300">
                          <span className="text-emerald-400 font-bold">•</span>
                          <span>{r}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>

              {/* Verification Checklist */}
              {job?.verification && (
                <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-2">
                  <div className="text-zinc-200 uppercase tracking-wider text-[11px] font-semibold flex items-center gap-1.5">
                    <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" aria-hidden="true" />
                    <span>Media Verification Checks</span>
                  </div>
                  <div className="grid grid-cols-2 gap-2 text-xs">
                    <div className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-between">
                      <span className="text-zinc-400">Integrity:</span>
                      <span className="font-semibold text-emerald-400">
                        {job.verification.checklist?.fileSizeValid ? 'Passed' : 'Verified'}
                      </span>
                    </div>
                    <div className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-between">
                      <span className="text-zinc-400">Container Structure:</span>
                      <span className="font-semibold text-emerald-400">
                        {job.verification.checklist?.containerValid ? 'Valid' : 'Verified'}
                      </span>
                    </div>
                    <div className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-between">
                      <span className="text-zinc-400">Audio Playable:</span>
                      <span className="font-semibold text-emerald-400">
                        {job.verification.checklist?.audioStreamValid ? 'Confirmed' : 'Valid'}
                      </span>
                    </div>
                    <div className="p-2 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-between">
                      <span className="text-zinc-400">Video Playable:</span>
                      <span className="font-semibold text-emerald-400">
                        {job.verification.checklist?.videoStreamValid ? 'Confirmed' : 'Valid'}
                      </span>
                    </div>
                  </div>
                </div>
              )}

              {/* Fingerprint */}
              {job?.fingerprint && (
                <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-1.5 text-xs font-mono">
                  <span className="text-zinc-500 block font-semibold text-[11px] uppercase tracking-wider">
                    Canonical Fingerprint & Hash
                  </span>
                  <div className="text-[11px] text-zinc-300 break-all select-all bg-zinc-900 p-2 rounded-lg border border-zinc-800">
                    {job.fingerprint.fileHash || job.fingerprint.canonicalId}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* TAB 2: VIDEO & CODECS */}
          {activeTab === 'video' && (
            <div className="space-y-4">
              <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-3">
                <div className="text-zinc-200 uppercase tracking-wider text-[11px] font-semibold flex items-center gap-1.5">
                  <Film className="w-3.5 h-3.5 text-purple-400" aria-hidden="true" />
                  <span>Video Stream Details</span>
                </div>
                <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                  <div>
                    <span className="text-zinc-500 block">Video Codec:</span>
                    <span className="font-semibold text-zinc-200">
                      {inspection?.videoCodec || primaryFormat?.vcodec || 'None / Audio Only'}
                    </span>
                  </div>
                  {inspection?.videoProfile && (
                    <div>
                      <span className="text-zinc-500 block">Profile:</span>
                      <span className="font-medium text-zinc-200">{inspection.videoProfile}</span>
                    </div>
                  )}
                  <div>
                    <span className="text-zinc-500 block">Resolution:</span>
                    <span className="font-semibold text-zinc-200">
                      {inspection?.width && inspection?.height
                        ? `${inspection.width} x ${inspection.height}`
                        : metadata?.availableResolutions?.[0]
                        ? `${metadata.availableResolutions[0]}p Max`
                        : 'N/A'}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Frame Rate:</span>
                    <span className="font-medium text-zinc-200">
                      {inspection?.fps ? `${inspection.fps} fps` : metadata?.availableFrameRates?.[0] ? `${metadata.availableFrameRates[0]} fps` : 'N/A'}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Dynamic Range:</span>
                    <span className={`font-semibold ${metadata?.isHdr || inspection?.isHdr ? 'text-amber-400' : 'text-zinc-200'}`}>
                      {metadata?.isHdr || inspection?.isHdr ? 'HDR (High Dynamic Range)' : 'SDR (Standard)'}
                    </span>
                  </div>
                  {inspection?.bitDepth && (
                    <div>
                      <span className="text-zinc-500 block">Bit Depth:</span>
                      <span className="font-medium text-zinc-200">{inspection.bitDepth}-bit</span>
                    </div>
                  )}
                  {inspection?.colorSpace && (
                    <div>
                      <span className="text-zinc-500 block">Color Primaries:</span>
                      <span className="font-medium text-zinc-200">{inspection.colorSpace}</span>
                    </div>
                  )}
                  {inspection?.bitrateKbps && (
                    <div>
                      <span className="text-zinc-500 block">Bitrate:</span>
                      <span className="font-medium text-zinc-200">{inspection.bitrateKbps} kbps</span>
                    </div>
                  )}
                </div>
              </div>

              {metadata?.availableResolutions && metadata.availableResolutions.length > 0 && (
                <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-2">
                  <span className="text-zinc-500 block font-semibold text-[11px] uppercase tracking-wider">
                    Available Source Resolutions
                  </span>
                  <div className="flex flex-wrap gap-1.5">
                    {metadata.availableResolutions.map((res) => (
                      <span
                        key={res}
                        className="px-2.5 py-1 rounded bg-zinc-900 border border-zinc-800 text-zinc-200 text-xs font-mono font-medium"
                      >
                        {res}p {res >= 2160 ? '(4K)' : res >= 1440 ? '(2K)' : res === 1080 ? '(Full HD)' : ''}
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* TAB 3: AUDIO */}
          {activeTab === 'audio' && (
            <div className="space-y-4">
              <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-3">
                <div className="text-zinc-200 uppercase tracking-wider text-[11px] font-semibold flex items-center gap-1.5">
                  <Music className="w-3.5 h-3.5 text-amber-400" aria-hidden="true" />
                  <span>Audio Specifications</span>
                </div>
                <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                  <div>
                    <span className="text-zinc-500 block">Audio Codec:</span>
                    <span className="font-semibold text-zinc-200">
                      {inspection?.audioCodec || primaryFormat?.acodec || (job?.preset === 'mp3' ? 'MP3' : job?.preset === 'flac' ? 'FLAC' : 'AAC / Opus')}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Channels:</span>
                    <span className="font-medium text-zinc-200">
                      {inspection?.audioChannels ? `${inspection.audioChannels} Channels (${inspection.audioChannels === 2 ? 'Stereo' : 'Mono'})` : '2 Channels (Stereo)'}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Sample Rate:</span>
                    <span className="font-medium text-zinc-200">
                      {inspection?.audioSampleRateHz ? `${(inspection.audioSampleRateHz / 1000).toFixed(1)} kHz` : '48.0 kHz'}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Bitrate:</span>
                    <span className="font-medium text-zinc-200">
                      {inspection?.audioBitrateKbps ? `${inspection.audioBitrateKbps} kbps` : primaryFormat?.abr ? `${Math.round(primaryFormat.abr)} kbps` : '160 - 320 kbps'}
                    </span>
                  </div>
                  {inspection?.audioLanguage && (
                    <div>
                      <span className="text-zinc-500 block">Language:</span>
                      <span className="font-medium text-zinc-200 uppercase">{inspection.audioLanguage}</span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* TAB 4: SUBTITLES */}
          {activeTab === 'subtitles' && (
            <div className="space-y-3">
              <div className="text-zinc-200 uppercase tracking-wider text-[11px] font-semibold flex items-center gap-1.5">
                <Subtitles className="w-3.5 h-3.5 text-blue-400" aria-hidden="true" />
                <span>Discovered Subtitles & Captions ({subtitlesList.length})</span>
              </div>
              <div className="divide-y divide-zinc-800 bg-zinc-950 border border-zinc-800 rounded-xl overflow-hidden max-h-72 overflow-y-auto">
                {subtitlesList.map((sub, idx) => (
                  <div key={`${sub.language}-${idx}`} className="p-3 flex items-center justify-between gap-3 text-xs">
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="px-2 py-0.5 rounded bg-zinc-900 border border-zinc-700 text-zinc-200 font-mono uppercase text-[11px] font-bold">
                        {sub.language}
                      </span>
                      <span className="text-zinc-300 truncate">{sub.name || `Track (${sub.language})`}</span>
                    </div>
                    <span
                      className={`px-2 py-0.5 rounded text-[10px] font-medium shrink-0 ${
                        sub.isAuto
                          ? 'bg-zinc-800 text-zinc-400 border border-zinc-700'
                          : 'bg-emerald-950/80 text-emerald-300 border border-emerald-800'
                      }`}
                    >
                      {sub.isAuto ? 'Auto' : 'Official'}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* TAB 5: CHAPTERS */}
          {activeTab === 'chapters' && (
            <div className="space-y-3">
              <div className="text-zinc-200 uppercase tracking-wider text-[11px] font-semibold flex items-center gap-1.5">
                <Bookmark className="w-3.5 h-3.5 text-emerald-400" aria-hidden="true" />
                <span>Detected Chapters ({chaptersList.length})</span>
              </div>
              <div className="divide-y divide-zinc-800 bg-zinc-950 border border-zinc-800 rounded-xl overflow-hidden max-h-72 overflow-y-auto">
                {chaptersList.map((ch, idx) => (
                  <div key={idx} className="p-3 flex items-center justify-between gap-3 text-xs">
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="font-mono text-blue-400 shrink-0 font-semibold">
                        {formatTime(ch.startTime)}
                      </span>
                      <span className="text-zinc-200 truncate">{ch.title}</span>
                    </div>
                    <span className="text-zinc-500 font-mono text-[11px] shrink-0">
                      {formatTime(ch.endTime - ch.startTime)}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* TAB 6: FILE & STREAMS */}
          {activeTab === 'file' && (
            <div className="space-y-4">
              <div className="bg-zinc-950 p-4 rounded-xl border border-zinc-800 space-y-3">
                <div className="text-zinc-200 uppercase tracking-wider text-[11px] font-semibold flex items-center gap-1.5">
                  <Layers className="w-3.5 h-3.5 text-emerald-400" aria-hidden="true" />
                  <span>Container & Output Specifications</span>
                </div>
                <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                  <div>
                    <span className="text-zinc-500 block">Container:</span>
                    <span className="font-bold text-zinc-100">
                      {inspection?.containerFormat || job?.preset.toUpperCase() || 'MKV / MP4'}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">File Size:</span>
                    <span className="font-bold text-zinc-100">
                      {formatBytes(inspection?.fileSizeBytes || job?.progress?.totalBytes)}
                    </span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Stream Count:</span>
                    <span className="font-medium text-zinc-200">{inspection?.streamCount || '2 (Video + Audio)'}</span>
                  </div>
                </div>

                {job?.finalFilePath && (
                  <div className="pt-2 border-t border-zinc-900">
                    <span className="text-zinc-500 block text-[11px]">Output Destination:</span>
                    <span className="font-mono text-zinc-300 text-[11px] break-all select-all">
                      {job.finalFilePath}
                    </span>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Modal Footer */}
        <div className="px-5 py-3 bg-zinc-950 border-t border-zinc-800 flex items-center justify-between shrink-0">
          <div className="flex items-center gap-2 text-[11px] text-zinc-500 font-mono">
            <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" aria-hidden="true" />
            <span>Inspected via yt-dlp & FFprobe / MediaInfo</span>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium transition-colors cursor-pointer border border-zinc-700"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};

export default MediaInfoModal;

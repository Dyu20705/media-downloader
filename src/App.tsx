import React, { useState, useEffect, useCallback, Suspense, lazy } from 'react';
import { 
  MediaMetadata, 
  PresetType, 
  DownloadJob, 
  AppSettings, 
  ToolHealth, 
  ToolStatusInfo,
  DiagnosticLog,
  UserIntent,
  FormatRecommendation,
  DEFAULT_SETTINGS 
} from './types';
import { ipc } from './services/ipc';
import { mapErrorToUserFriendly, FormattedError } from './services/errorMapper';
import { Header } from './components/Header';
import { UrlInputBar } from './components/UrlInputBar';
import { MediaSummaryCard } from './components/MediaSummaryCard';
import { RecommendationCard } from './components/RecommendationCard';
import { FormatSelector } from './components/FormatSelector';
import { QualityAndDirectory } from './components/QualityAndDirectory';
import { DownloadProgressState } from './components/DownloadProgressState';
import { AlertCircle, ChevronDown, ChevronUp } from 'lucide-react';

// Lazy load secondary surfaces to minimize initial bundle size and execution
const SettingsModal = lazy(() => import('./components/SettingsModal'));
const DiagnosticsDrawer = lazy(() => import('./components/DiagnosticsDrawer'));
const ToolsModal = lazy(() => import('./components/ToolsModal'));
const MediaInfoModal = lazy(() => import('./components/MediaInfoModal'));
const HelpCheatsheetModal = lazy(() => import('./components/HelpCheatsheetModal'));
const DownloadHistoryModal = lazy(() => import('./components/DownloadHistoryModal'));

export default function App() {
  // Primary Workspace State
  const [url, setUrl] = useState<string>('https://www.youtube.com/watch?v=LXb3EKWsInQ');
  const [metadata, setMetadata] = useState<MediaMetadata | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState<boolean>(false);
  const [analysisError, setAnalysisError] = useState<FormattedError | null>(null);
  const [showErrorDetails, setShowErrorDetails] = useState<boolean>(false);

  // Intent & Strategy Planning
  const [intent, setIntent] = useState<UserIntent>('balanced');
  const [preset, setPreset] = useState<PresetType>('mp4-compatible');
  const [quality, setQuality] = useState<string>('auto');
  const [outputDirectory, setOutputDirectory] = useState<string>('D:\\Videos');

  // Active Job & History
  const [activeJob, setActiveJob] = useState<DownloadJob | null>(null);
  const [allJobs, setAllJobs] = useState<DownloadJob[]>([]);

  // Modals & Drawers
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [isDiagnosticsOpen, setIsDiagnosticsOpen] = useState<boolean>(false);
  const [isToolsOpen, setIsToolsOpen] = useState<boolean>(false);
  const [isHelpOpen, setIsHelpOpen] = useState<boolean>(false);
  const [isHistoryOpen, setIsHistoryOpen] = useState<boolean>(false);
  const [inspectedJob, setInspectedJob] = useState<DownloadJob | null>(null);
  const [isInspectingMetadata, setIsInspectingMetadata] = useState<boolean>(false);

  // Diagnostics & Tools
  const [logs, setLogs] = useState<DiagnosticLog[]>([]);
  const [tools, setTools] = useState<ToolHealth[]>([]);
  const [toolStatuses, setToolStatuses] = useState<ToolStatusInfo[]>([]);
  const [isRefreshingTools, setIsRefreshingTools] = useState<boolean>(false);
  const [settings, setSettings] = useState<AppSettings>(() => {
    try {
      const saved = localStorage.getItem('ocmd_settings');
      return saved ? JSON.parse(saved) : DEFAULT_SETTINGS;
    } catch {
      return DEFAULT_SETTINGS;
    }
  });

  // Calculate recommendation adaptively based on active user intent & metadata
  const currentRecommendation: FormatRecommendation | undefined = React.useMemo(() => {
    if (!metadata) return undefined;
    const isAudio = metadata.mediaKind === 'audio';
    const maxRes = metadata.availableResolutions?.[0] || 1080;
    const maxFps = metadata.availableFrameRates?.[0] || 30;
    const isHdr = Boolean(metadata.isHdr);

    if (isAudio) {
      if (intent === 'best-compatibility') {
        return {
          preset: 'mp3',
          label: 'MP3 · Universal Audio (320kbps)',
          targetQuality: 'auto',
          reason: 'Standard MP3 format for universal device compatibility',
          whyReasons: [
            '✓ compatible with all media players and car stereos',
            '✓ auto-transcoded to VBR MP3 (highest quality)',
            '✓ lightweight file footprint'
          ],
          isTranscodeFree: false,
          transcodingCost: 'TRANSCODE',
          container: 'mp3',
          details: 'Encodes audio to universal MP3 format'
        };
      }
      if (intent === 'max-quality') {
        const hasFlac = metadata.formats?.some(f => f.ext === 'flac');
        return {
          preset: hasFlac ? 'flac' : 'best-audio',
          label: hasFlac ? 'FLAC · Lossless Master' : 'M4A · Source Audio (Opus/AAC)',
          targetQuality: 'auto',
          reason: 'Preserve original audio bitstream without lossy re-encoding',
          whyReasons: [
            '✓ original unaltered audio bitstream',
            '✓ zero generational transcoding loss',
            '✓ highest source bitrate'
          ],
          isTranscodeFree: true,
          transcodingCost: 'STREAM_COPY',
          container: hasFlac ? 'flac' : 'm4a',
          details: 'Direct stream copy of uncompressed audio tracks'
        };
      }
      return {
        preset: 'best-audio',
        label: 'Best Audio (Source Copy)',
        targetQuality: 'auto',
        reason: 'Extracts untouched source audio stream without lossy transcoding',
        whyReasons: [
          '✓ untouched source stream',
          '✓ no transcoding required',
          '✓ fast instant acquisition'
        ],
        isTranscodeFree: true,
        transcodingCost: 'STREAM_COPY',
        container: 'm4a',
        details: 'Direct stream copy (Opus/AAC)'
      };
    }

    if (intent === 'max-quality') {
      const fpsLabel = maxFps > 30 ? `${maxFps}fps ` : '';
      const hdrLabel = isHdr ? ' HDR' : '';
      const label = maxRes >= 2160 ? `MKV · 4K ${fpsLabel}${hdrLabel}`.trim() : `MKV · ${maxRes}p Master`;
      return {
        preset: maxRes > 1080 ? 'best-video' : 'mp4-compatible',
        label,
        targetQuality: String(maxRes),
        reason: 'Preserves pristine source bitrates, wide-color HDR, and modern VP9/AV1 codecs in MKV container',
        whyReasons: [
          '✓ highest available source resolution',
          '✓ preserves native bit depth & dynamic range',
          ...(isHdr ? ['✓ HDR stream multiplexing enabled'] : []),
          '✓ multiplexed without lossy downscaling'
        ],
        isTranscodeFree: true,
        transcodingCost: 'MERGE',
        container: maxRes > 1080 ? 'mkv' : 'mp4',
        details: 'Multiplexes highest resolution video stream with master audio stream'
      };
    }

    if (intent === 'smallest-size') {
      return {
        preset: 'mp4-compatible',
        label: 'MP4 · 720p Space-Saver',
        targetQuality: '720',
        reason: 'Balanced efficiency for storage and quick mobile transfers',
        whyReasons: [
          '✓ lowest storage footprint',
          '✓ efficient AVC/AAC stream selection',
          '✓ no CPU-heavy video transcoding',
          '✓ fast download throughput'
        ],
        isTranscodeFree: true,
        transcodingCost: 'STREAM_COPY',
        container: 'mp4',
        details: 'Selects compact 720p stream with efficient bitrates'
      };
    }

    if (intent === 'best-compatibility') {
      return {
        preset: 'mp4-compatible',
        label: `MP4 · ${maxRes >= 1080 ? 1080 : 720}p Universal`,
        targetQuality: String(maxRes >= 1080 ? 1080 : 720),
        reason: 'Standard MP4 container with H.264/AAC for all devices and smart TVs',
        whyReasons: [
          '✓ universal H.264 / AAC compatibility',
          '✓ plays on iOS, Android, macOS, Windows, TVs',
          '✓ standard MP4 container',
          '✓ zero transcoding artifacting'
        ],
        isTranscodeFree: true,
        transcodingCost: 'MERGE',
        container: 'mp4',
        details: 'Standard MP4 with wide hardware acceleration support'
      };
    }

    // Default Balanced
    const fpsSuffix = maxFps > 30 ? ` ${maxFps}fps` : '';
    return {
      preset: 'mp4-compatible',
      label: `MP4 · ${maxRes}p${fpsSuffix} (Best Balance)`,
      targetQuality: String(maxRes),
      reason: 'Optimal balance of visual fidelity, file size, and compatibility',
      whyReasons: [
        '✓ native source quality',
        '✓ MP4 universal playback',
        '✓ no video transcoding',
        '✓ balanced download speed and clarity'
      ],
      isTranscodeFree: true,
      transcodingCost: 'MERGE',
      container: 'mp4',
      details: 'Universal H.264 / AAC MP4 container'
    };
  }, [metadata, intent]);

  // Handle Intent Change and auto-apply recommendation
  const handleSelectIntent = (newIntent: UserIntent) => {
    setIntent(newIntent);
  };

  // Apply recommendation settings
  const handleApplyRecommendation = () => {
    if (currentRecommendation) {
      setPreset(currentRecommendation.preset as PresetType);
      if (currentRecommendation.targetQuality) {
        setQuality(currentRecommendation.targetQuality);
      }
    }
  };

  const isRecommendationApplied = 
    currentRecommendation && 
    preset === currentRecommendation.preset && 
    (quality === currentRecommendation.targetQuality || currentRecommendation.targetQuality === 'auto');

  // Fetch Tool Status
  const fetchTools = useCallback(async () => {
    try {
      setIsRefreshingTools(true);
      const [toolList, detailedStatuses] = await Promise.all([
        ipc.getToolStatus(),
        ipc.getDetailedToolStatus()
      ]);
      setTools(toolList);
      setToolStatuses(detailedStatuses);
    } catch {
      // ignore
    } finally {
      setIsRefreshingTools(false);
    }
  }, []);

  // Tool actions
  const handleInstallTool = async (name: string) => {
    await ipc.installTool(name);
    await fetchTools();
  };

  const handleRepairTool = async (name: string) => {
    await ipc.repairTool(name);
    await fetchTools();
  };

  const handleInstallAllTools = async () => {
    await ipc.installAllMissingTools();
    await fetchTools();
  };

  // Fetch Diagnostics on Demand (Zero constant polling)
  const fetchDiagnostics = useCallback(async () => {
    try {
      const logList = await ipc.getDiagnostics();
      setLogs(logList);
    } catch {
      // ignore
    }
  }, []);

  // Initial Load: Settings & Tools
  useEffect(() => {
    async function init() {
      try {
        const loaded = await ipc.getSettings();
        setSettings(loaded);
        if (loaded.downloadDirectory) {
          setOutputDirectory(loaded.downloadDirectory);
        }
        if (loaded.lastPreset) {
          setPreset(loaded.lastPreset);
        }
      } catch {
        // ignore
      }
      await fetchTools();
    }
    init();
  }, [fetchTools]);

  // On-Demand Generated Command for Diagnostics / Details drawer
  const handleFetchCommandPreview = async (): Promise<string> => {
    try {
      const res = await ipc.buildCommand({
        preset,
        quality,
        outputDirectory,
        url: url || 'URL',
        settings
      });
      return res.command;
    } catch {
      return 'yt-dlp [URL]';
    }
  };

  // Analyze Action
  const handleAnalyze = async (urlToAnalyze?: string) => {
    const targetUrl = urlToAnalyze || url;
    if (!targetUrl.trim()) return;

    setIsAnalyzing(true);
    setAnalysisError(null);
    setShowErrorDetails(false);

    try {
      const meta = await ipc.analyzeMedia(targetUrl.trim());
      setMetadata(meta);
    } catch (err: any) {
      setAnalysisError(mapErrorToUserFriendly(err));
    } finally {
      setIsAnalyzing(false);
    }
  };

  // Trigger initial analyze for default sample URL
  useEffect(() => {
    handleAnalyze('https://www.youtube.com/watch?v=LXb3EKWsInQ');
  }, []);

  // Start Download Action
  const handleStartDownload = async () => {
    if (!metadata || !url) return;

    try {
      const job = await ipc.startDownload({
        url,
        metadata,
        preset,
        quality,
        outputDirectory
      });

      setActiveJob(job);
      setAllJobs(prev => [job, ...prev.filter(j => j.id !== job.id)]);
    } catch (err: any) {
      setAnalysisError(mapErrorToUserFriendly(err));
    }
  };

  // Event-Driven Active Job Subscription (Replaces Polling with SSE / Tauri Events)
  useEffect(() => {
    if (!activeJob || activeJob.status === 'COMPLETED' || activeJob.status === 'CANCELLED' || activeJob.status === 'FAILED') {
      return;
    }

    const unsubscribe = ipc.subscribeToJob(activeJob.id, (updatedJob) => {
      setActiveJob(updatedJob);
      setAllJobs(prev => prev.map(j => j.id === updatedJob.id ? updatedJob : j));
      if (updatedJob.status === 'COMPLETED' || updatedJob.status === 'FAILED' || updatedJob.status === 'CANCELLED') {
        fetchDiagnostics();
      }
    });

    return () => {
      unsubscribe();
    };
  }, [activeJob?.id, activeJob?.status, fetchDiagnostics]);

  // Cancel Job Action
  const handleCancelJob = async (jobId: string) => {
    try {
      const job = await ipc.cancelDownload(jobId);
      setActiveJob(job);
      setAllJobs(prev => prev.map(j => j.id === jobId ? job : j));
      fetchDiagnostics();
    } catch {
      // ignore
    }
  };

  // Save Settings
  const handleSaveSettings = async (newSettings: AppSettings) => {
    try {
      const saved = await ipc.saveSettings(newSettings);
      setSettings(saved);
      setOutputDirectory(saved.downloadDirectory);
      setPreset(saved.lastPreset);
      await fetchTools();
    } catch {
      setSettings(newSettings);
    }
  };

  // Reset workspace for another download
  const handleResetForAnother = () => {
    setActiveJob(null);
    setUrl('');
    setMetadata(null);
    setAnalysisError(null);
  };

  const isJobRunning = activeJob && (
    activeJob.status === 'DOWNLOADING' || 
    activeJob.status === 'POST_PROCESSING' || 
    activeJob.status === 'VERIFYING'
  );

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 flex flex-col antialiased">
      {/* Header */}
      <Header
        tools={tools}
        historyCount={allJobs.length}
        onOpenSettings={() => setIsSettingsOpen(true)}
        onOpenDiagnostics={() => {
          fetchDiagnostics();
          setIsDiagnosticsOpen(true);
        }}
        onOpenTools={() => setIsToolsOpen(true)}
        onOpenHistory={() => setIsHistoryOpen(true)}
        onOpenHelp={() => setIsHelpOpen(true)}
      />

      {/* Main Workspace (Single focused container) */}
      <main className="flex-1 max-w-2xl w-full mx-auto px-4 sm:px-6 py-6 sm:py-8 space-y-5">
        <div className="space-y-1">
          <h1 className="text-xl sm:text-2xl font-bold tracking-tight text-zinc-100">
            Download media
          </h1>
          <p className="text-xs text-zinc-400">
            Paste a link from any supported platform to save videos or music.
          </p>
        </div>

        {/* 1. Paste URL */}
        <section aria-label="URL input section">
          <UrlInputBar
            url={url}
            onChangeUrl={setUrl}
            onAnalyze={handleAnalyze}
            isAnalyzing={isAnalyzing}
            disabled={Boolean(isJobRunning)}
          />
          {analysisError && (
            <div role="alert" className="mt-2 p-3 rounded-xl bg-red-950/40 border border-red-800/50 text-red-300 text-xs font-mono space-y-1.5">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <AlertCircle className="w-4 h-4 shrink-0 text-red-400" aria-hidden="true" />
                  <span className="font-sans font-semibold text-red-200">{analysisError.userMessage}</span>
                </div>
                {analysisError.technicalDetails && (
                  <button
                    type="button"
                    onClick={() => setShowErrorDetails(prev => !prev)}
                    className="text-[11px] text-red-400 hover:text-red-300 underline font-sans cursor-pointer flex items-center gap-0.5"
                  >
                    <span>{showErrorDetails ? 'Hide' : 'Details'}</span>
                    {showErrorDetails ? <ChevronUp className="w-3 h-3" /> : <ChevronDown className="w-3 h-3" />}
                  </button>
                )}
              </div>
              {showErrorDetails && analysisError.technicalDetails && (
                <div className="mt-1.5 p-2 bg-black/60 rounded border border-red-900/60 text-[11px] font-mono text-zinc-400 break-all select-all">
                  {analysisError.technicalDetails}
                </div>
              )}
            </div>
          )}
        </section>

        {/* 2. Media Summary */}
        {metadata && (
          <section aria-label="Media summary" className="space-y-3">
            <MediaSummaryCard 
              metadata={metadata} 
              onOpenInspector={() => setIsInspectingMetadata(true)}
            />

            {/* Smart Strategy Recommendation Banner */}
            {currentRecommendation && (
              <RecommendationCard 
                recommendation={currentRecommendation}
                onApply={handleApplyRecommendation}
                isApplied={Boolean(isRecommendationApplied)}
              />
            )}
          </section>
        )}

        {/* 3. Format Presets */}
        <section aria-label="Format selection">
          <FormatSelector
            selectedPreset={preset}
            onSelectPreset={setPreset}
            recommendation={currentRecommendation}
            disabled={Boolean(isJobRunning)}
          />
        </section>

        {/* 4. Quality and Save Location */}
        <section aria-label="Quality and destination">
          <QualityAndDirectory
            selectedPreset={preset}
            selectedQuality={quality}
            onSelectQuality={setQuality}
            outputDirectory={outputDirectory}
            onChangeOutputDirectory={setOutputDirectory}
            availableResolutions={metadata?.availableResolutions || []}
            disabled={Boolean(isJobRunning)}
          />
        </section>

        {/* 5. Download Button & Progress / Completion State */}
        <section aria-label="Download and progress state" className="pt-1">
          <DownloadProgressState
            isAnalyzing={isAnalyzing}
            hasUrl={Boolean(url.trim())}
            hasMetadata={Boolean(metadata)}
            activeJob={activeJob}
            onStartDownload={handleStartDownload}
            onCancelDownload={handleCancelJob}
            onReset={handleResetForAnother}
            onOpenDetails={(job) => setInspectedJob(job)}
            onOpenDiagnostics={() => {
              fetchDiagnostics();
              setIsDiagnosticsOpen(true);
            }}
          />
        </section>
      </main>

      {/* Footer */}
      <footer className="border-t border-zinc-900 py-4 text-center text-xs text-zinc-500 font-mono">
        <div className="max-w-2xl mx-auto px-4 flex items-center justify-between">
          <span>One-Click Media Downloader</span>
          <span>Engine: yt-dlp + FFmpeg + MediaInfo</span>
        </div>
      </footer>

      {/* Lazy-Loaded Secondary Surfaces (Settings, Diagnostics, Tools, History, Specs) */}
      <Suspense fallback={null}>
        {isSettingsOpen && (
          <SettingsModal
            isOpen={isSettingsOpen}
            onClose={() => setIsSettingsOpen(false)}
            settings={settings}
            onSaveSettings={handleSaveSettings}
          />
        )}

        {isDiagnosticsOpen && (
          <DiagnosticsDrawer
            isOpen={isDiagnosticsOpen}
            onClose={() => setIsDiagnosticsOpen(false)}
            logs={logs}
            onClear={() => {
              ipc.clearDiagnostics();
              setLogs([]);
            }}
            onFetchCommandPreview={handleFetchCommandPreview}
          />
        )}

        {isToolsOpen && (
          <ToolsModal
            isOpen={isToolsOpen}
            onClose={() => setIsToolsOpen(false)}
            tools={toolStatuses}
            onRefreshTools={fetchTools}
            onInstallTool={handleInstallTool}
            onRepairTool={handleRepairTool}
            onInstallAll={handleInstallAllTools}
            isRefreshing={isRefreshingTools}
          />
        )}

        {isHelpOpen && (
          <HelpCheatsheetModal
            isOpen={isHelpOpen}
            onClose={() => setIsHelpOpen(false)}
          />
        )}

        {isHistoryOpen && (
          <DownloadHistoryModal
            isOpen={isHistoryOpen}
            onClose={() => setIsHistoryOpen(false)}
            jobs={allJobs}
            onInspect={(job) => setInspectedJob(job)}
          />
        )}

        {(inspectedJob || isInspectingMetadata) && (
          <MediaInfoModal
            job={inspectedJob}
            metadata={!inspectedJob ? metadata : undefined}
            onClose={() => {
              setInspectedJob(null);
              setIsInspectingMetadata(false);
            }}
          />
        )}
      </Suspense>
    </div>
  );
}

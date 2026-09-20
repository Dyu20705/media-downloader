import React, { useState, useCallback } from 'react';
import { Header } from './components/Header';
import { MediaEngineSetupCard } from './components/MediaEngineSetupCard';
import { UrlInputBar } from './components/UrlInputBar';
import { MediaSummaryCard } from './components/MediaSummaryCard';
import { RecommendationCard } from './components/RecommendationCard';
import { FormatSelector } from './components/FormatSelector';
import { QualityAndDirectory } from './components/QualityAndDirectory';
import { DownloadProgressState } from './components/DownloadProgressState';

import { SettingsModal } from './components/SettingsModal';
import { DownloadHistoryModal } from './components/DownloadHistoryModal';
import { ToolsModal } from './components/ToolsModal';
import { HelpCheatsheetModal } from './components/HelpCheatsheetModal';
import { DiagnosticsDrawer } from './components/DiagnosticsDrawer';
import { MediaInfoModal } from './components/MediaInfoModal';

import { useModals } from './hooks/useModals';
import { useAppSettings } from './hooks/useAppSettings';
import { useToolEngine } from './hooks/useToolEngine';
import { useMediaAnalysis } from './hooks/useMediaAnalysis';
import { useRecommendation } from './hooks/useRecommendation';
import { useDownloadEngine } from './hooks/useDownloadEngine';
import { useDiagnostics } from './hooks/useDiagnostics';

import { DownloadJob } from './types';
import { ipc } from './services/ipc';

export const App: React.FC = () => {
  // 1. Modals state manager
  const modals = useModals();

  // 2. Diagnostics state manager
  const { logs, fetchDiagnostics, clearDiagnostics } = useDiagnostics();

  // 3. Application Settings manager
  const {
    settings,
    outputDirectory,
    setOutputDirectory,
    preset,
    setPreset,
    quality,
    setQuality,
    intent,
    saveSettings,
  } = useAppSettings();

  // 4. Media Tools & Auto-Bootstrap Engine
  const {
    tools,
    toolStatuses,
    isRefreshingTools,
    isAutoBootstrapping,
    fetchTools,
    installTool,
    repairTool,
    installAllTools,
  } = useToolEngine();

  // 5. Media URL & Metadata Analysis Engine
  const {
    url,
    setUrl,
    metadata,
    isAnalyzing,
    analysisError,
    showErrorDetails,
    setShowErrorDetails,
    handleAnalyze,
  } = useMediaAnalysis();

  // 6. Dynamic Recommendation Engine
  const {
    currentRecommendation,
    isRecommendationApplied,
    handleApplyRecommendation,
  } = useRecommendation({
    metadata,
    intent,
    preset,
    quality,
    setPreset,
    setQuality,
  });

  // 7. Active Download & Queue Lifecycle Engine
  const {
    activeJob,
    allJobs,
    isJobRunning,
    startDownload,
    cancelDownload,
    resetActiveJob,
    downloadError,
  } = useDownloadEngine({ onDiagnosticsUpdate: fetchDiagnostics });

  // Inspection Target for Technical Drawer / Modal
  const [selectedJobForDetails, setSelectedJobForDetails] = useState<DownloadJob | null>(null);

  // Computed Engine State
  const requiredTools = toolStatuses.filter((t) => t.isRequired);
  const allRequiredReady = requiredTools.length > 0 && requiredTools.every((t) => t.status === 'READY');
  const hasOutdated = toolStatuses.some((t) => t.status === 'OUTDATED');
  const completedJobsCount = allJobs.filter((j) => j.status === 'COMPLETED').length;

  const handleStartDownload = useCallback(() => {
    if (!metadata || !url) return;
    startDownload({
      url,
      metadata,
      preset,
      quality,
      outputDirectory,
    });
  }, [metadata, url, preset, quality, outputDirectory, startDownload]);

  const handleFetchCommandPreview = useCallback(async (): Promise<string> => {
    try {
      const res = await ipc.buildCommand({
        preset,
        quality,
        outputDirectory,
        url: url || 'URL',
        settings,
      });
      return res.command;
    } catch {
      return 'yt-dlp [URL]';
    }
  }, [preset, quality, outputDirectory, url, settings]);

  const handleOpenJobDetails = (job: DownloadJob) => {
    setSelectedJobForDetails(job);
    modals.setIsInspectingMetadata(true);
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 flex flex-col font-sans selection:bg-blue-600 selection:text-white antialiased">
      {/* 1. Header Bar */}
      <Header
        tools={tools}
        historyCount={completedJobsCount}
        onOpenSettings={() => modals.setIsSettingsOpen(true)}
        onOpenHistory={() => modals.setIsHistoryOpen(true)}
        onOpenTools={() => modals.setIsToolsOpen(true)}
        onOpenHelp={() => modals.setIsCheatsheetOpen(true)}
        onOpenDiagnostics={() => modals.setIsDiagnosticsOpen(true)}
        isUpdatingTools={isRefreshingTools || isAutoBootstrapping}
        hasToolUpdate={hasOutdated}
      />

      {/* 2. Main Workstation Body */}
      <main className="flex-1 max-w-4xl w-full mx-auto p-4 sm:p-6 flex flex-col gap-6">
        {/* Media Engine Component Setup Card (Shows if components need attention) */}
        {(!allRequiredReady || hasOutdated) && (
          <section aria-label="Engine Setup">
            <MediaEngineSetupCard
              toolStatuses={toolStatuses}
              isLoading={isRefreshingTools}
              onInstallTool={installTool}
              onRepairTool={repairTool}
              onInstallAll={installAllTools}
              onRefresh={fetchTools}
              onOpenAdvancedModal={() => modals.setIsToolsOpen(true)}
            />
          </section>
        )}

        {/* Media URL Input Bar */}
        <section aria-label="URL Input">
          <UrlInputBar
            url={url}
            onChangeUrl={setUrl}
            onAnalyze={handleAnalyze}
            isAnalyzing={isAnalyzing}
            disabled={isJobRunning}
          />

          {/* Analysis / Extraction Error Banner */}
          {analysisError && (
            <div
              role="alert"
              className="mt-3 p-4 rounded-xl bg-red-950/40 border border-red-800/60 text-xs text-red-300 space-y-1.5 animate-fadeIn"
            >
              <div className="flex items-center justify-between">
                <span className="font-semibold text-red-200">{analysisError.userMessage}</span>
                {analysisError.technicalDetails && (
                  <button
                    type="button"
                    onClick={() => setShowErrorDetails(!showErrorDetails)}
                    className="text-[11px] text-red-400 hover:text-red-300 underline font-medium cursor-pointer"
                  >
                    {showErrorDetails ? 'Hide logs' : 'View logs'}
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

        {/* Media Summary & Smart Recommendation Banner */}
        {metadata && (
          <section aria-label="Media summary" className="space-y-3">
            <MediaSummaryCard
              metadata={metadata}
              onOpenInspector={() => modals.setIsInspectingMetadata(true)}
            />

            {currentRecommendation && (
              <RecommendationCard
                recommendation={currentRecommendation}
                onApply={handleApplyRecommendation}
                isApplied={isRecommendationApplied}
              />
            )}
          </section>
        )}

        {/* Format Selection Presets */}
        <section aria-label="Format selection">
          <FormatSelector
            selectedPreset={preset}
            onSelectPreset={setPreset}
            recommendation={currentRecommendation}
            disabled={isJobRunning}
          />
        </section>

        {/* Quality and Destination Folder */}
        <section aria-label="Quality and destination">
          <QualityAndDirectory
            selectedPreset={preset}
            selectedQuality={quality}
            onSelectQuality={setQuality}
            outputDirectory={outputDirectory}
            onChangeOutputDirectory={setOutputDirectory}
            availableResolutions={metadata?.availableResolutions || []}
            disabled={isJobRunning}
          />
        </section>

        {/* Download Execution & Progress State */}
        <section aria-label="Execution controls" className="pt-2">
          {downloadError && (
            <div className="mb-3 p-3 rounded-xl bg-red-950/40 border border-red-800/60 text-xs text-red-300">
              {downloadError.userMessage}
            </div>
          )}

          <DownloadProgressState
            isAnalyzing={isAnalyzing}
            hasUrl={Boolean(url)}
            hasMetadata={Boolean(metadata)}
            activeJob={activeJob}
            onStartDownload={handleStartDownload}
            onCancelDownload={cancelDownload}
            onReset={resetActiveJob}
            onOpenDetails={handleOpenJobDetails}
            onOpenDiagnostics={() => modals.setIsDiagnosticsOpen(true)}
          />
        </section>
      </main>

      {/* 3. Footer Bar */}
      <footer className="py-4 border-t border-zinc-900 text-center text-xs text-zinc-600 flex items-center justify-between px-6">
        <span>One-Click Media Downloader</span>
        <span>Engine: yt-dlp + FFmpeg + MediaInfo</span>
      </footer>

      {/* 4. Modals & Drawers */}
      <SettingsModal
        isOpen={modals.isSettingsOpen}
        onClose={() => modals.setIsSettingsOpen(false)}
        settings={settings}
        onSaveSettings={saveSettings}
      />

      <DownloadHistoryModal
        isOpen={modals.isHistoryOpen}
        onClose={() => modals.setIsHistoryOpen(false)}
        jobs={allJobs}
        onInspect={handleOpenJobDetails}
      />

      <ToolsModal
        isOpen={modals.isToolsOpen}
        onClose={() => modals.setIsToolsOpen(false)}
        tools={toolStatuses}
        onRefreshTools={fetchTools}
        onInstallTool={installTool}
        onRepairTool={repairTool}
        onInstallAll={installAllTools}
        isRefreshing={isRefreshingTools}
      />

      <HelpCheatsheetModal
        isOpen={modals.isCheatsheetOpen}
        onClose={() => modals.setIsCheatsheetOpen(false)}
      />

      <DiagnosticsDrawer
        isOpen={modals.isDiagnosticsOpen}
        onClose={() => modals.setIsDiagnosticsOpen(false)}
        logs={logs}
        onClear={clearDiagnostics}
        onFetchCommandPreview={handleFetchCommandPreview}
      />

      {modals.isInspectingMetadata && (metadata || selectedJobForDetails) && (
        <MediaInfoModal
          metadata={metadata || selectedJobForDetails?.metadata}
          job={selectedJobForDetails}
          onClose={() => {
            modals.setIsInspectingMetadata(false);
            setSelectedJobForDetails(null);
          }}
        />
      )}
    </div>
  );
};

export default App;

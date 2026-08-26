import React, { useState } from 'react';
import { 
  CheckCircle2, 
  AlertCircle, 
  RefreshCw, 
  Download, 
  ShieldCheck, 
  ChevronDown, 
  ChevronUp, 
  Wrench, 
  HardDrive,
  FileCode2
} from 'lucide-react';
import { ToolStatusInfo } from '../types';

interface MediaEngineSetupCardProps {
  toolStatuses: ToolStatusInfo[];
  isLoading: boolean;
  onInstallTool: (name: string) => Promise<void>;
  onRepairTool: (name: string) => Promise<void>;
  onInstallAll: () => Promise<void>;
  onRefresh: () => void;
  onOpenAdvancedModal?: () => void;
}

export const MediaEngineSetupCard: React.FC<MediaEngineSetupCardProps> = ({
  toolStatuses,
  isLoading,
  onInstallTool,
  onRepairTool,
  onInstallAll,
  onRefresh,
  onOpenAdvancedModal,
}) => {
  const [showTechnicalDetails, setShowTechnicalDetails] = useState(false);
  const [actionInProgress, setActionInProgress] = useState<string | null>(null);

  const requiredTools = toolStatuses.filter(t => t.isRequired);
  const allRequiredReady = requiredTools.length > 0 && requiredTools.every(t => t.status === 'READY');
  const hasMissingOrBroken = toolStatuses.some(t => t.status !== 'READY');
  const hasOutdated = toolStatuses.some(t => t.status === 'OUTDATED');
  const failedTool = toolStatuses.find(t => t.status === 'ERROR' || t.status === 'INVALID');

  const handleAction = async (toolName: string, isRepair: boolean) => {
    try {
      setActionInProgress(toolName);
      if (isRepair) {
        await onRepairTool(toolName);
      } else {
        await onInstallTool(toolName);
      }
    } finally {
      setActionInProgress(null);
    }
  };

  const handleInstallAll = async () => {
    try {
      setActionInProgress('all');
      await onInstallAll();
    } finally {
      setActionInProgress(null);
    }
  };

  return (
    <div className="w-full bg-zinc-900/90 border border-zinc-800 rounded-2xl p-4 sm:p-5 shadow-lg transition-all duration-200">
      {/* Top Banner Row */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div className="flex items-start sm:items-center gap-3">
          <div className={`p-2.5 rounded-xl flex-shrink-0 ${
            allRequiredReady 
              ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' 
              : hasOutdated
              ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
              : 'bg-red-500/10 text-red-400 border border-red-500/20'
          }`}>
            {allRequiredReady ? (
              <ShieldCheck className="w-5 h-5" />
            ) : (
              <Wrench className="w-5 h-5 animate-pulse" />
            )}
          </div>

          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-semibold text-zinc-100 text-sm sm:text-base">
                Media engine
              </h3>
              {allRequiredReady ? (
                <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 text-xs font-medium border border-emerald-500/20">
                  Ready
                </span>
              ) : hasOutdated ? (
                <span className="px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-400 text-xs font-medium border border-amber-500/20">
                  Update Available
                </span>
              ) : null}
            </div>
            <p className="text-xs text-zinc-400 mt-0.5">
              {allRequiredReady
                ? 'Everything is ready.'
                : failedTool
                ? `${failedTool.name} could not be verified or installed.`
                : hasOutdated
                ? 'An updated pinned build is recommended for optimal compatibility.'
                : 'Required media components need setup.'}
            </p>
          </div>
        </div>

        {/* Action Controls */}
        <div className="flex items-center gap-2 self-end sm:self-center">
          {hasMissingOrBroken && (
            <button
              type="button"
              id="btn-install-all-tools"
              onClick={handleInstallAll}
              disabled={isLoading || actionInProgress !== null}
              className={`px-3.5 py-1.5 disabled:opacity-50 text-white text-xs font-semibold rounded-lg transition-colors flex items-center gap-1.5 shadow-sm cursor-pointer ${
                hasOutdated && !failedTool
                  ? 'bg-amber-600 hover:bg-amber-500'
                  : 'bg-red-600 hover:bg-red-500'
              }`}
            >
              {actionInProgress === 'all' ? (
                <>
                  <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                  <span>Installing...</span>
                </>
              ) : (
                <>
                  <Download className="w-3.5 h-3.5" />
                  <span>{failedTool ? 'Retry Installation' : hasOutdated ? 'Update All Components' : 'Install Components'}</span>
                </>
              )}
            </button>
          )}

          <button
            type="button"
            onClick={() => setShowTechnicalDetails(!showTechnicalDetails)}
            className="px-3 py-1.5 bg-zinc-800/80 hover:bg-zinc-800 text-zinc-300 hover:text-zinc-100 text-xs rounded-lg transition-colors flex items-center gap-1 border border-zinc-700/50 cursor-pointer"
            title="Toggle technical details"
          >
            <span>{showTechnicalDetails ? 'Hide details' : 'Show details'}</span>
            {showTechnicalDetails ? (
              <ChevronUp className="w-3.5 h-3.5" />
            ) : (
              <ChevronDown className="w-3.5 h-3.5" />
            )}
          </button>
        </div>
      </div>

      {/* Checklist Overview */}
      <div className="mt-4 pt-3.5 border-t border-zinc-800/80 grid grid-cols-2 sm:grid-cols-4 gap-2">
        {toolStatuses.map((tool) => {
          const isReady = tool.status === 'READY';
          const isInstalling = actionInProgress === tool.name || actionInProgress === 'all';
          return (
            <div
              key={tool.name}
              className={`p-2.5 rounded-xl border flex items-center justify-between text-xs transition-colors ${
                isReady
                  ? 'bg-zinc-950/40 border-zinc-800/80 text-zinc-200'
                  : tool.status === 'INVALID' || tool.status === 'ERROR'
                  ? 'bg-red-950/20 border-red-900/40 text-red-300'
                  : 'bg-zinc-950/60 border-zinc-800 text-zinc-400'
              }`}
            >
              <div className="flex items-center gap-2 truncate">
                {isReady ? (
                  <CheckCircle2 className="w-4 h-4 text-emerald-400 flex-shrink-0" />
                ) : (
                  <AlertCircle className="w-4 h-4 text-amber-400 flex-shrink-0" />
                )}
                <span className="font-mono font-medium truncate">{tool.name}</span>
              </div>

              {!isReady && (
                <button
                  type="button"
                  onClick={() => handleAction(tool.name, tool.status === 'INVALID')}
                  disabled={isLoading || isInstalling}
                  className="px-2 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-[11px] font-semibold transition-colors disabled:opacity-50 ml-1 flex-shrink-0 cursor-pointer"
                >
                  {isInstalling ? '...' : tool.status === 'INVALID' ? 'Repair' : 'Install'}
                </button>
              )}
            </div>
          );
        })}
      </div>

      {/* Expandable Technical Details (Strictly Hidden by Default per Spec) */}
      {showTechnicalDetails && (
        <div className="mt-4 pt-4 border-t border-zinc-800/80 space-y-3 text-xs font-mono animate-in fade-in duration-150">
          <div className="flex items-center justify-between text-zinc-400 text-[11px]">
            <span className="font-semibold uppercase tracking-wider text-zinc-300">Supply-Chain & Cryptographic Verification</span>
            <button
              type="button"
              onClick={onRefresh}
              className="text-zinc-400 hover:text-zinc-200 flex items-center gap-1 cursor-pointer"
            >
              <RefreshCw className={`w-3 h-3 ${isLoading ? 'animate-spin' : ''}`} />
              <span>Rescan</span>
            </button>
          </div>

          <div className="space-y-2">
            {toolStatuses.map((t) => (
              <div
                key={t.name}
                className="p-3 rounded-xl bg-zinc-950/60 border border-zinc-800 space-y-1.5"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-zinc-200 uppercase">{t.name}</span>
                    <span className="px-1.5 py-0.5 rounded bg-zinc-800 text-[10px] text-zinc-300">
                      v{t.pinnedVersion}
                    </span>
                    {t.managed && (
                      <span className="px-1.5 py-0.5 rounded bg-blue-500/10 text-blue-400 border border-blue-500/20 text-[10px]">
                        App-Local
                      </span>
                    )}
                  </div>
                  <span className={`text-[11px] font-semibold ${
                    t.status === 'READY' ? 'text-emerald-400' : 'text-amber-400'
                  }`}>
                    {t.status}
                  </span>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-1 text-[11px] text-zinc-400">
                  <div className="truncate">
                    <span className="text-zinc-500">Path: </span>
                    <span className="text-zinc-300">{t.path || 'Not installed'}</span>
                  </div>
                  <div className="truncate">
                    <span className="text-zinc-500">License: </span>
                    <span className="text-zinc-300">{t.license}</span>
                  </div>
                </div>

                {t.sha256 && (
                  <div className="text-[10px] text-zinc-500 truncate">
                    <span className="text-zinc-600">SHA-256: </span>
                    <span className="text-zinc-400 font-mono">{t.sha256}</span>
                  </div>
                )}

                {t.errorMessage && (
                  <div className="text-red-400 text-[11px] bg-red-950/30 p-2 rounded border border-red-900/30 mt-1">
                    {t.errorMessage}
                  </div>
                )}
              </div>
            ))}
          </div>

          <div className="p-3 rounded-xl bg-zinc-950/30 border border-zinc-800/60 text-zinc-500 text-[11px] leading-relaxed">
            <span className="font-medium text-zinc-400">Resolution Security: </span>
            Tools are stored safely in an application-local location (<code className="text-zinc-400">%LOCALAPPDATA%\OneClickMediaDownloader\tools\</code>). System PATH and Windows registry are never modified. Incomplete temporary files never become active.
          </div>
        </div>
      )}
    </div>
  );
};

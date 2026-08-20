import React, { useState } from 'react';
import { 
  X, 
  Wrench, 
  CheckCircle2, 
  AlertCircle, 
  RefreshCw, 
  Download, 
  Layers
} from 'lucide-react';
import { ToolStatusInfo } from '../types';

interface ToolsModalProps {
  isOpen: boolean;
  onClose: () => void;
  tools: ToolStatusInfo[];
  onRefreshTools: () => void;
  onInstallTool: (name: string) => Promise<void>;
  onRepairTool: (name: string) => Promise<void>;
  onInstallAll: () => Promise<void>;
  isRefreshing: boolean;
}

export const ToolsModal: React.FC<ToolsModalProps> = ({
  isOpen,
  onClose,
  tools,
  onRefreshTools,
  onInstallTool,
  onRepairTool,
  onInstallAll,
  isRefreshing,
}) => {
  const [activeAction, setActiveAction] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleInstall = async (name: string) => {
    try {
      setActiveAction(name);
      await onInstallTool(name);
    } finally {
      setActiveAction(null);
    }
  };

  const handleRepair = async (name: string) => {
    try {
      setActiveAction(name);
      await onRepairTool(name);
    } finally {
      setActiveAction(null);
    }
  };

  const handleInstallAll = async () => {
    try {
      setActiveAction('all');
      await onInstallAll();
    } finally {
      setActiveAction(null);
    }
  };

  return (
    <div 
      role="dialog" 
      aria-modal="true" 
      aria-labelledby="tools-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
    >
      <div className="w-full max-w-2xl bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
        {/* Header */}
        <div className="px-5 py-4 bg-zinc-950 border-b border-zinc-800 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-lg bg-zinc-800 text-zinc-300">
              <Wrench className="w-4 h-4 text-blue-400" aria-hidden="true" />
            </div>
            <div>
              <h2 id="tools-title" className="font-semibold text-base text-zinc-100">
                Media Engine Management
              </h2>
              <p className="text-xs text-zinc-400">
                Isolated application-local tools & cryptographic validation
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              id="btn-refresh-tools"
              onClick={onRefreshTools}
              disabled={isRefreshing || activeAction !== null}
              className="p-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded-xl transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
              title="Rescan tools"
              aria-label="Rescan tool statuses"
            >
              <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin text-blue-400' : ''}`} aria-hidden="true" />
            </button>
            <button
              type="button"
              id="btn-close-tools"
              onClick={onClose}
              className="p-1.5 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-lg transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
              aria-label="Close Media Engine Management"
            >
              <X className="w-5 h-5" aria-hidden="true" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto space-y-4 text-xs font-mono">
          <div className="grid grid-cols-1 gap-3">
            {tools.map((t) => {
              const isReady = t.status === 'READY';
              const isBusy = activeAction === t.name || activeAction === 'all';

              return (
                <div
                  key={t.name}
                  className="p-4 rounded-xl bg-zinc-950 border border-zinc-800 flex flex-col gap-2.5"
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-sm text-zinc-100 uppercase">{t.name}</span>
                      <span className="px-2 py-0.5 rounded bg-zinc-800 text-[10px] text-zinc-300 font-mono">
                        Pinned v{t.pinnedVersion}
                      </span>
                      {isReady ? (
                        <span className="px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 text-[11px] font-semibold flex items-center gap-1">
                          <CheckCircle2 className="w-3 h-3" aria-hidden="true" />
                          Ready
                        </span>
                      ) : (
                        <span className="px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/30 text-[11px] font-semibold flex items-center gap-1">
                          <AlertCircle className="w-3 h-3" aria-hidden="true" />
                          {t.status}
                        </span>
                      )}
                    </div>

                    <div className="flex items-center gap-2">
                      {!isReady ? (
                        <button
                          type="button"
                          onClick={() => t.status === 'INVALID' ? handleRepair(t.name) : handleInstall(t.name)}
                          disabled={isBusy}
                          className="px-3 py-1.5 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white rounded-xl font-sans font-semibold text-xs transition-colors flex items-center gap-1 cursor-pointer focus-visible:ring-2 focus-visible:ring-blue-400"
                        >
                          {isBusy ? (
                            <RefreshCw className="w-3 h-3 animate-spin" aria-hidden="true" />
                          ) : (
                            <Download className="w-3 h-3" aria-hidden="true" />
                          )}
                          <span>{t.status === 'INVALID' ? 'Repair' : 'Install'}</span>
                        </button>
                      ) : (
                        <button
                          type="button"
                          onClick={() => handleRepair(t.name)}
                          disabled={isBusy}
                          className="px-2.5 py-1 bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-zinc-300 rounded-lg text-xs font-sans transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-blue-500"
                          title="Force reinstallation and integrity recheck"
                        >
                          Reinstall
                        </button>
                      )}
                    </div>
                  </div>

                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-1.5 text-zinc-400 text-[11px]">
                    <div>
                      <span className="text-zinc-500">Active Path: </span>
                      <span className="text-zinc-300 truncate">{t.path || 'Managed'}</span>
                    </div>
                    <div>
                      <span className="text-zinc-500">License: </span>
                      <span className="text-zinc-300">{t.license}</span>
                    </div>
                  </div>

                  {t.sha256 && (
                    <div className="text-[10px] text-zinc-500 font-mono truncate">
                      <span className="text-zinc-600">SHA-256 Checksum: </span>
                      <span className="text-zinc-400">{t.sha256}</span>
                    </div>
                  )}

                  {t.errorMessage && (
                    <div className="text-amber-400 text-[11px] bg-amber-950/30 p-2.5 rounded-lg border border-amber-800/40">
                      {t.errorMessage}
                    </div>
                  )}
                </div>
              );
            })}
          </div>

          {/* Resolution Hierarchy */}
          <div className="p-4 rounded-xl bg-zinc-950 border border-zinc-800 text-zinc-400 leading-relaxed space-y-1.5">
            <div className="font-semibold text-zinc-300 text-xs flex items-center gap-1.5">
              <Layers className="w-3.5 h-3.5 text-blue-400" aria-hidden="true" />
              <span>Resolution Hierarchy:</span>
            </div>
            <ol className="list-decimal list-inside space-y-0.5 text-[11px] text-zinc-400">
              <li>Configured override path in Settings</li>
              <li>Project-local tool (<code className="text-zinc-300">./bin/</code>, <code className="text-zinc-300">./tools/</code>)</li>
              <li>System Environment <code className="text-zinc-300">PATH</code></li>
              <li>Application-local managed directory (<code className="text-zinc-300">%LOCALAPPDATA%\OneClickMediaDownloader\tools\</code>)</li>
            </ol>
          </div>
        </div>

        {/* Footer */}
        <div className="px-5 py-3.5 bg-zinc-950 border-t border-zinc-800 flex items-center justify-between">
          <button
            type="button"
            onClick={handleInstallAll}
            disabled={activeAction !== null}
            className="px-3.5 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-semibold rounded-xl transition-colors flex items-center gap-1.5 cursor-pointer focus-visible:ring-2 focus-visible:ring-blue-500"
          >
            <Download className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Install All Missing</span>
          </button>

          <button
            type="button"
            onClick={onClose}
            className="px-4 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold rounded-xl transition-colors focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
};

export default ToolsModal;

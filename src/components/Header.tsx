import React from 'react';
import { Download, Terminal, Settings, Wrench, HelpCircle, History } from 'lucide-react';
import { ToolHealth } from '../types';

interface HeaderProps {
  tools: ToolHealth[];
  historyCount: number;
  onOpenSettings: () => void;
  onOpenDiagnostics: () => void;
  onOpenTools: () => void;
  onOpenHistory: () => void;
  onOpenHelp: () => void;
  isUpdatingTools?: boolean;
  hasToolUpdate?: boolean;
}

export const Header: React.FC<HeaderProps> = ({
  tools,
  historyCount,
  onOpenSettings,
  onOpenDiagnostics,
  onOpenTools,
  onOpenHistory,
  onOpenHelp,
  isUpdatingTools = false,
  hasToolUpdate = false,
}) => {
  const readyCount = tools.filter(t => t.available).length;
  const isAllReady = readyCount >= 3;

  return (
    <header className="border-b border-zinc-800 bg-zinc-950/95 sticky top-0 z-30">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 h-14 flex items-center justify-between">
        {/* Brand */}
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-blue-600 text-white flex items-center justify-center font-bold shadow-sm">
            <Download className="w-4 h-4" aria-hidden="true" />
          </div>
          <span className="font-semibold text-sm sm:text-base text-zinc-100 tracking-tight">
            Media Downloader
          </span>
        </div>

        {/* Secondary Actions */}
        <nav aria-label="Quick actions" className="flex items-center gap-1.5 sm:gap-2">
          {/* History */}
          <button
            id="nav-btn-history"
            type="button"
            onClick={onOpenHistory}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium text-zinc-300 hover:text-zinc-100 hover:bg-zinc-800 border border-transparent hover:border-zinc-700 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
            title="Download History"
            aria-label={`Download History (${historyCount} items)`}
          >
            <History className="w-3.5 h-3.5 text-zinc-400" aria-hidden="true" />
            <span className="hidden sm:inline">History</span>
            {historyCount > 0 && (
              <span className="px-1.5 py-0.2 rounded-full bg-zinc-800 text-[11px] font-mono text-zinc-300">
                {historyCount}
              </span>
            )}
          </button>

          {/* Engine Status */}
          <button
            id="nav-btn-tools"
            type="button"
            onClick={onOpenTools}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium text-zinc-300 hover:text-zinc-100 hover:bg-zinc-800 border border-transparent hover:border-zinc-700 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
            title="Media Engine Status & Tools"
            aria-label="Engine Tools"
          >
            <Wrench className="w-3.5 h-3.5 text-zinc-400" aria-hidden="true" />
            <span className="hidden sm:inline">Engine</span>
            <span
              className={`w-2 h-2 rounded-full ${
                isUpdatingTools
                  ? 'bg-blue-400 animate-pulse'
                  : hasToolUpdate
                  ? 'bg-amber-400'
                  : isAllReady
                  ? 'bg-emerald-500'
                  : 'bg-red-500'
              }`}
              aria-label={
                isUpdatingTools
                  ? 'Engine Updating'
                  : hasToolUpdate
                  ? 'Engine Update Available'
                  : isAllReady
                  ? 'Engine Ready'
                  : 'Engine Needs Setup'
              }
            />
          </button>

          {/* Diagnostics */}
          <button
            id="nav-btn-diagnostics"
            type="button"
            onClick={onOpenDiagnostics}
            className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-medium text-zinc-300 hover:text-zinc-100 hover:bg-zinc-800 border border-transparent hover:border-zinc-700 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
            title="Diagnostics & Logs"
            aria-label="Diagnostics"
          >
            <Terminal className="w-3.5 h-3.5 text-zinc-400" aria-hidden="true" />
            <span className="hidden md:inline">Diagnostics</span>
          </button>

          {/* Settings */}
          <button
            id="nav-btn-settings"
            type="button"
            onClick={onOpenSettings}
            className="p-1.5 rounded-lg text-zinc-300 hover:text-zinc-100 hover:bg-zinc-800 border border-transparent hover:border-zinc-700 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
            title="Settings"
            aria-label="Settings"
          >
            <Settings className="w-4 h-4" aria-hidden="true" />
          </button>

          {/* Help / Cheatsheet */}
          <button
            id="nav-btn-help"
            type="button"
            onClick={onOpenHelp}
            className="p-1.5 rounded-lg text-zinc-300 hover:text-zinc-100 hover:bg-zinc-800 border border-transparent hover:border-zinc-700 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
            title="Help & Cheatsheet"
            aria-label="Help and Documentation"
          >
            <HelpCircle className="w-4 h-4" aria-hidden="true" />
          </button>
        </nav>
      </div>
    </header>
  );
};

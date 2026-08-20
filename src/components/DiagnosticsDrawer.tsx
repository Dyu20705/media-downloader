import React, { useState } from 'react';
import { X, Terminal, Trash2, Copy, Check, Filter, Code } from 'lucide-react';
import { DiagnosticLog } from '../types';

interface DiagnosticsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  logs: DiagnosticLog[];
  onClear: () => void;
  commandPreview?: string;
  onFetchCommandPreview?: () => Promise<string>;
}

export const DiagnosticsDrawer: React.FC<DiagnosticsDrawerProps> = ({
  isOpen,
  onClose,
  logs,
  onClear,
  commandPreview: initialCmd = '',
  onFetchCommandPreview,
}) => {
  const [copiedLogs, setCopiedLogs] = useState(false);
  const [copiedCmd, setCopiedCmd] = useState(false);
  const [filterLevel, setFilterLevel] = useState<string>('all');
  const [activeTab, setActiveTab] = useState<'logs' | 'command'>('logs');
  const [commandPreview, setCommandPreview] = useState<string>(initialCmd);
  const [isLoadingCmd, setIsLoadingCmd] = useState<boolean>(false);

  const handleSwitchToCommand = async () => {
    setActiveTab('command');
    if (onFetchCommandPreview && !commandPreview) {
      try {
        setIsLoadingCmd(true);
        const cmd = await onFetchCommandPreview();
        setCommandPreview(cmd);
      } catch {
        // ignore
      } finally {
        setIsLoadingCmd(false);
      }
    }
  };

  if (!isOpen) return null;

  const filteredLogs = filterLevel === 'all'
    ? logs
    : logs.filter(l => l.level === filterLevel);

  const handleCopyLogs = async () => {
    const text = logs.map(l => `[${l.timestamp}] [${l.level.toUpperCase()}] ${l.message}`).join('\n');
    try {
      await navigator.clipboard.writeText(text);
      setCopiedLogs(true);
      setTimeout(() => setCopiedLogs(false), 2000);
    } catch {
      // ignore
    }
  };

  const handleCopyCmd = async () => {
    if (!commandPreview) return;
    try {
      await navigator.clipboard.writeText(commandPreview);
      setCopiedCmd(true);
      setTimeout(() => setCopiedCmd(false), 2000);
    } catch {
      // ignore
    }
  };

  return (
    <div 
      role="dialog" 
      aria-modal="true" 
      aria-labelledby="diagnostics-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
    >
      <div className="w-full max-w-3xl bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col h-[80vh]">
        {/* Header */}
        <div className="px-5 py-3.5 bg-zinc-950 border-b border-zinc-800 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-zinc-800 text-zinc-300">
              <Terminal className="w-4 h-4 text-blue-400" aria-hidden="true" />
            </div>
            <div>
              <h2 id="diagnostics-title" className="font-semibold text-sm sm:text-base text-zinc-100">
                Technical Diagnostics
              </h2>
              <p className="text-xs text-zinc-400 font-mono">
                Ring buffer ({logs.length}/256 retained lines)
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              id="btn-close-diagnostics"
              onClick={onClose}
              className="p-1.5 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-lg transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
              aria-label="Close Diagnostics"
            >
              <X className="w-5 h-5" aria-hidden="true" />
            </button>
          </div>
        </div>

        {/* Tab switcher */}
        <div className="px-5 py-2 bg-zinc-950 border-b border-zinc-800/80 flex items-center justify-between text-xs">
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setActiveTab('logs')}
              className={`px-3 py-1 rounded-lg font-medium transition-colors cursor-pointer ${
                activeTab === 'logs' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400 hover:text-zinc-200'
              }`}
            >
              Diagnostic Logs ({logs.length})
            </button>
            <button
              type="button"
              onClick={handleSwitchToCommand}
              className={`px-3 py-1 rounded-lg font-medium transition-colors cursor-pointer flex items-center gap-1 ${
                activeTab === 'command' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-400 hover:text-zinc-200'
              }`}
            >
              <Code className="w-3.5 h-3.5" aria-hidden="true" />
              <span>Generated CLI Command</span>
            </button>
          </div>

          {activeTab === 'logs' ? (
            <div className="flex items-center gap-2">
              <button
                type="button"
                id="btn-copy-diagnostics"
                onClick={handleCopyLogs}
                className="px-2.5 py-1 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium rounded-md flex items-center gap-1.5 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-blue-500"
              >
                {copiedLogs ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{copiedLogs ? 'Copied' : 'Copy'}</span>
              </button>
              <button
                type="button"
                id="btn-clear-diagnostics"
                onClick={onClear}
                className="p-1 bg-zinc-800 hover:bg-zinc-700 text-zinc-400 hover:text-red-400 text-xs rounded-md transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-blue-500"
                title="Clear logs"
                aria-label="Clear diagnostic logs"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          ) : (
            <button
              type="button"
              onClick={handleCopyCmd}
              className="px-2.5 py-1 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium rounded-md flex items-center gap-1.5 transition-colors cursor-pointer focus-visible:ring-2 focus-visible:ring-blue-500"
            >
              {copiedCmd ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              <span>{copiedCmd ? 'Copied Command' : 'Copy Command'}</span>
            </button>
          )}
        </div>

        {/* Tab: CLI Command Preview */}
        {activeTab === 'command' && (
          <div className="flex-1 bg-zinc-950 p-4 font-mono text-xs text-zinc-200 overflow-y-auto space-y-3">
            <p className="text-zinc-400 text-xs font-sans">
              This is the exact underlying CLI command compiled by the engine:
            </p>
            <div className="p-3 bg-zinc-900 border border-zinc-800 rounded-xl text-emerald-400 select-all break-all leading-relaxed">
              {commandPreview || 'yt-dlp [URL]'}
            </div>
          </div>
        )}

        {/* Tab: Logs Stream */}
        {activeTab === 'logs' && (
          <>
            {/* Filter Bar */}
            <div className="px-5 py-2 bg-zinc-950/40 border-b border-zinc-800/80 flex items-center gap-2 text-xs font-mono">
              <Filter className="w-3 h-3 text-zinc-500" aria-hidden="true" />
              <span className="text-zinc-500">Level:</span>
              {['all', 'info', 'stdout', 'warn', 'error'].map((lvl) => (
                <button
                  key={lvl}
                  type="button"
                  onClick={() => setFilterLevel(lvl)}
                  className={`px-2 py-0.5 rounded border text-[11px] transition-colors capitalize cursor-pointer ${
                    filterLevel === lvl
                      ? 'bg-zinc-700 text-white border-zinc-600 font-semibold'
                      : 'bg-zinc-900 text-zinc-400 border-zinc-800 hover:text-zinc-200'
                  }`}
                >
                  {lvl}
                </button>
              ))}
            </div>

            {/* Console Log Area */}
            <div className="flex-1 bg-zinc-950 p-4 overflow-y-auto font-mono text-xs space-y-1 select-text">
              {filteredLogs.length === 0 ? (
                <div className="text-zinc-500 italic py-8 text-center">
                  No diagnostic records in buffer.
                </div>
              ) : (
                filteredLogs.map((log) => {
                  const color =
                    log.level === 'error' || log.level === 'stderr'
                      ? 'text-red-400'
                      : log.level === 'warn'
                      ? 'text-amber-400'
                      : log.level === 'stdout'
                      ? 'text-emerald-400'
                      : 'text-zinc-300';

                  return (
                    <div key={log.id} className="flex items-start gap-2 leading-relaxed hover:bg-zinc-900 px-1 py-0.5 rounded">
                      <span className="text-zinc-600 shrink-0 select-none">
                        {log.timestamp.split('T')[1]?.slice(0, 8)}
                      </span>
                      <span className={`uppercase font-bold text-[10px] px-1 py-0.2 rounded shrink-0 select-none ${
                        log.level === 'error' ? 'bg-red-950 text-red-400 border border-red-800' :
                        log.level === 'warn' ? 'bg-amber-950 text-amber-400 border border-amber-800' :
                        log.level === 'stdout' ? 'bg-emerald-950 text-emerald-400 border border-emerald-800' :
                        'bg-zinc-800 text-zinc-400'
                      }`}>
                        {log.level}
                      </span>
                      <span className={`${color} break-all flex-1`}>{log.message}</span>
                    </div>
                  );
                })
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default DiagnosticsDrawer;

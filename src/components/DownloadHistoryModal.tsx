import React from 'react';
import { X, History, FolderOpen, Play, Info, CheckCircle2, XCircle, Ban } from 'lucide-react';
import { DownloadJob } from '../types';
import { ipc } from '../services/ipc';

interface DownloadHistoryModalProps {
  isOpen: boolean;
  onClose: () => void;
  jobs: DownloadJob[];
  onInspect: (job: DownloadJob) => void;
}

export const DownloadHistoryModal: React.FC<DownloadHistoryModalProps> = ({
  isOpen,
  onClose,
  jobs,
  onInspect,
}) => {
  if (!isOpen) return null;

  const handleOpenFolder = async (path?: string) => {
    if (path) await ipc.openFolder(path);
  };

  const handleOpenFile = async (path?: string) => {
    if (path) await ipc.openFile(path);
  };

  return (
    <div 
      role="dialog" 
      aria-modal="true" 
      aria-labelledby="history-title"
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm"
    >
      <div className="w-full max-w-2xl bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[80vh]">
        {/* Header */}
        <div className="px-5 py-4 bg-zinc-950 border-b border-zinc-800 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-zinc-800 text-zinc-300">
              <History className="w-4 h-4 text-blue-400" aria-hidden="true" />
            </div>
            <div>
              <h2 id="history-title" className="font-semibold text-base text-zinc-100">
                Download History
              </h2>
              <p className="text-xs text-zinc-400">
                {jobs.length} total downloads recorded
              </p>
            </div>
          </div>

          <button
            type="button"
            id="btn-close-history"
            onClick={onClose}
            className="p-1.5 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 rounded-lg transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
            aria-label="Close History"
          >
            <X className="w-5 h-5" aria-hidden="true" />
          </button>
        </div>

        {/* Content list */}
        <div className="p-4 overflow-y-auto divide-y divide-zinc-800">
          {jobs.length === 0 ? (
            <div className="py-12 text-center text-xs text-zinc-500">
              No previous downloads yet. Completed downloads will appear here.
            </div>
          ) : (
            jobs.map((job) => (
              <div key={job.id} className="py-3 flex items-center justify-between gap-3 first:pt-0 last:pb-0">
                <div className="min-w-0 space-y-1">
                  <div className="flex items-center gap-2">
                    {job.status === 'COMPLETED' ? (
                      <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" aria-hidden="true" />
                    ) : job.status === 'FAILED' ? (
                      <XCircle className="w-3.5 h-3.5 text-red-400 shrink-0" aria-hidden="true" />
                    ) : (
                      <Ban className="w-3.5 h-3.5 text-zinc-500 shrink-0" aria-hidden="true" />
                    )}
                    <h3 className="font-medium text-xs sm:text-sm text-zinc-200 truncate" title={job.metadata?.title}>
                      {job.metadata?.title || 'Media File'}
                    </h3>
                  </div>
                  <div className="text-[11px] text-zinc-400 flex items-center gap-2">
                    <span className="font-mono uppercase">{job.preset}</span>
                    <span>•</span>
                    <span className="font-mono text-zinc-500 truncate max-w-[200px] sm:max-w-xs">
                      {job.finalFileName || job.url}
                    </span>
                  </div>
                </div>

                <div className="flex items-center gap-1.5 shrink-0">
                  {job.status === 'COMPLETED' && job.finalFilePath && (
                    <>
                      <button
                        type="button"
                        onClick={() => handleOpenFile(job.finalFilePath)}
                        className="p-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
                        title="Open file"
                        aria-label="Open downloaded file"
                      >
                        <Play className="w-3.5 h-3.5 fill-current" aria-hidden="true" />
                      </button>
                      <button
                        type="button"
                        onClick={() => handleOpenFolder(job.finalFilePath)}
                        className="p-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
                        title="Show in folder"
                        aria-label="Show file in folder"
                      >
                        <FolderOpen className="w-3.5 h-3.5" aria-hidden="true" />
                      </button>
                    </>
                  )}
                  {job.inspection && (
                    <button
                      type="button"
                      onClick={() => onInspect(job)}
                      className="p-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white transition-colors focus-visible:ring-2 focus-visible:ring-blue-500"
                      title="Technical specs"
                      aria-label="View media specifications"
                    >
                      <Info className="w-3.5 h-3.5" aria-hidden="true" />
                    </button>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};

export default DownloadHistoryModal;

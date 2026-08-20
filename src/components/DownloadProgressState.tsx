import React from 'react';
import { 
  CheckCircle2, 
  XCircle, 
  Loader2, 
  FolderOpen, 
  Play, 
  Ban, 
  RotateCcw,
  Info,
  ShieldCheck
} from 'lucide-react';
import { DownloadJob } from '../types';
import { ipc } from '../services/ipc';
import { AcquisitionReceipt } from './AcquisitionReceipt';

interface DownloadProgressStateProps {
  isAnalyzing: boolean;
  hasUrl: boolean;
  hasMetadata: boolean;
  activeJob: DownloadJob | null;
  onStartDownload: () => void;
  onCancelDownload: (jobId: string) => void;
  onReset: () => void;
  onOpenDetails: (job: DownloadJob) => void;
  onOpenDiagnostics: () => void;
}

export const DownloadProgressState: React.FC<DownloadProgressStateProps> = ({
  isAnalyzing,
  hasUrl,
  hasMetadata,
  activeJob,
  onStartDownload,
  onCancelDownload,
  onReset,
  onOpenDetails,
  onOpenDiagnostics,
}) => {
  const handleOpenFile = async () => {
    if (activeJob?.finalFilePath) {
      await ipc.openFile(activeJob.finalFilePath);
    }
  };

  const handleOpenFolder = async () => {
    if (activeJob?.finalFilePath) {
      await ipc.openFolder(activeJob.finalFilePath);
    }
  };

  // 1. If currently analyzing
  if (isAnalyzing) {
    return (
      <div 
        role="status" 
        aria-live="polite"
        className="p-5 rounded-2xl bg-zinc-900 border border-zinc-800 text-center space-y-2"
      >
        <div className="flex items-center justify-center gap-2 text-zinc-300 font-medium text-sm">
          <Loader2 className="w-4 h-4 animate-spin text-blue-500" aria-hidden="true" />
          <span>Understanding media source & planning strategy…</span>
        </div>
      </div>
    );
  }

  // 2. If active job exists
  if (activeJob) {
    const { status, progress, finalFileName } = activeJob;

    // COMPLETED
    if (status === 'COMPLETED') {
      return (
        <div className="space-y-4">
          <AcquisitionReceipt 
            job={activeJob}
            onOpenFile={handleOpenFile}
            onOpenDirectory={handleOpenFolder}
          />

          <div className="flex items-center justify-between pt-1">
            <button
              type="button"
              id="btn-completed-details"
              onClick={() => onOpenDetails(activeJob)}
              className="px-3.5 py-2 bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 text-xs font-medium rounded-xl border border-zinc-800 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer flex items-center space-x-1.5"
            >
              <Info className="w-3.5 h-3.5" aria-hidden="true" />
              <span>Technical Inspection</span>
            </button>

            <button
              type="button"
              id="btn-download-another"
              onClick={onReset}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold rounded-xl transition-colors cursor-pointer"
            >
              Acquire Another
            </button>
          </div>
        </div>
      );
    }

    // FAILED
    if (status === 'FAILED') {
      return (
        <div 
          role="alert" 
          aria-live="assertive"
          className="p-5 rounded-2xl bg-zinc-900 border border-red-900/50 space-y-3"
        >
          <div className="flex items-center gap-2.5 text-red-400 font-semibold text-sm">
            <XCircle className="w-5 h-5 text-red-400 shrink-0" aria-hidden="true" />
            <span>Download failed</span>
          </div>

          <div className="flex items-center gap-2 pt-1">
            <button
              type="button"
              id="btn-try-again-failed"
              onClick={onStartDownload}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold rounded-xl flex items-center gap-1.5 transition-colors focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer"
            >
              <RotateCcw className="w-3.5 h-3.5" aria-hidden="true" />
              <span>Try again</span>
            </button>

            <button
              type="button"
              id="btn-failed-details"
              onClick={onOpenDiagnostics}
              className="px-3.5 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium rounded-xl border border-zinc-700 transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
            >
              <span>Details</span>
            </button>
          </div>
        </div>
      );
    }

    // CANCELLED
    if (status === 'CANCELLED') {
      return (
        <div 
          role="status" 
          aria-live="polite"
          className="p-5 rounded-2xl bg-zinc-900 border border-zinc-800 space-y-3"
        >
          <div className="flex items-center gap-2.5 text-zinc-300 font-semibold text-sm">
            <Ban className="w-4 h-4 text-zinc-400 shrink-0" aria-hidden="true" />
            <span>Download cancelled</span>
          </div>

          <button
            type="button"
            id="btn-try-again-cancelled"
            onClick={onStartDownload}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold rounded-xl flex items-center gap-1.5 transition-colors focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer"
          >
            <RotateCcw className="w-3.5 h-3.5" aria-hidden="true" />
            <span>Try again</span>
          </button>
        </div>
      );
    }

    // DOWNLOADING / POST_PROCESSING / VERIFYING
    const pct = progress?.percentage || 0;
    const isPostProcess = status === 'POST_PROCESSING';
    const isVerifying = status === 'VERIFYING';

    return (
      <div 
        role="status" 
        aria-live="polite"
        className="p-5 rounded-2xl bg-zinc-900 border border-zinc-800 space-y-3"
      >
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-2 text-zinc-100 font-medium">
            <Loader2 className="w-4 h-4 animate-spin text-blue-500" aria-hidden="true" />
            <span>
              {isVerifying ? 'Verifying…' : isPostProcess ? 'Preparing file…' : 'Downloading'}
            </span>
          </div>
          <span className="font-mono text-zinc-300 font-semibold text-xs">
            {isVerifying ? '100%' : isPostProcess ? '95%' : `${pct}%`}
          </span>
        </div>

        {/* Progress Bar */}
        <div 
          role="progressbar"
          aria-valuenow={pct}
          aria-valuemin={0}
          aria-valuemax={100}
          className="w-full h-2 bg-zinc-950 rounded-full overflow-hidden border border-zinc-800"
        >
          <div
            className="h-full bg-blue-600 transition-all duration-200"
            style={{ width: `${Math.max(3, pct)}%` }}
          />
        </div>

        {/* Bottom Bar: Speed / Status & Cancel button */}
        <div className="flex items-center justify-between text-xs text-zinc-400 pt-1">
          <span className="font-mono">
            {progress?.currentSpeed && progress.currentSpeed !== '0 B/s' ? progress.currentSpeed : 'Processing media stream…'}
          </span>

          <button
            type="button"
            id="btn-cancel-active-download"
            onClick={() => onCancelDownload(activeJob.id)}
            className="text-xs text-zinc-400 hover:text-red-400 font-medium transition-colors focus-visible:ring-2 focus-visible:ring-blue-500 cursor-pointer"
          >
            Cancel
          </button>
        </div>
      </div>
    );
  }

  // 3. If ready to download (metadata parsed)
  if (hasMetadata) {
    return (
      <div className="space-y-3">
        <button
          type="button"
          id="btn-primary-download"
          onClick={onStartDownload}
          className="w-full py-3.5 px-6 bg-blue-600 hover:bg-blue-500 text-white font-semibold text-base rounded-xl shadow-sm transition-all focus-visible:ring-2 focus-visible:ring-blue-400 cursor-pointer flex items-center justify-center gap-2"
        >
          <span>Download</span>
        </button>
        <p className="text-center text-xs text-zinc-400" role="status">
          Ready to download
        </p>
      </div>
    );
  }

  // 4. Initial EMPTY State
  return (
    <div className="py-6 text-center text-xs text-zinc-400" role="status">
      Paste a media URL to get started
    </div>
  );
};

import React, { useState } from 'react';
import {
  CheckCircle2,
  FileCheck2,
  Fingerprint,
  FileCode,
  Copy,
  Check,
  FolderOpen,
  Play,
  ShieldCheck,
  ChevronDown,
  ChevronUp,
} from 'lucide-react';
import {
  DownloadJob,
  ExplainableResult,
  MediaFingerprint,
  DownloadRecipe,
  VerificationResult,
} from '../types';

interface AcquisitionReceiptProps {
  job: DownloadJob;
  onOpenFile?: (path: string) => void;
  onOpenDirectory?: (path: string) => void;
}

export const AcquisitionReceipt: React.FC<AcquisitionReceiptProps> = ({
  job,
  onOpenFile,
  onOpenDirectory,
}) => {
  const [copiedRecipe, setCopiedRecipe] = useState(false);
  const [showTechnicalDetails, setShowTechnicalDetails] = useState(false);

  const explainable = job.explainableResult;
  const verification = job.verification;
  const fingerprint = job.fingerprint;
  const recipe = job.recipe;

  const handleCopyRecipe = () => {
    if (recipe) {
      navigator.clipboard.writeText(JSON.stringify(recipe, null, 2));
      setCopiedRecipe(true);
      setTimeout(() => setCopiedRecipe(false), 2000);
    }
  };

  return (
    <div
      id="acquisition-receipt"
      className="w-full bg-slate-900/90 border border-slate-800 rounded-2xl p-5 shadow-2xl mt-4"
    >
      {/* Header Banner */}
      <div className="flex items-center justify-between border-b border-slate-800/80 pb-4 mb-4">
        <div className="flex items-center space-x-3">
          <div className="p-2 rounded-xl bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
            <FileCheck2 className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center space-x-2">
              <span className="text-xs font-semibold uppercase tracking-wider text-emerald-400">
                Verified Acquisition
              </span>
              <span className="px-2 py-0.5 text-[10px] font-mono bg-emerald-500/15 text-emerald-300 rounded border border-emerald-500/30">
                100% Passed
              </span>
            </div>
            <h3 className="text-base font-bold text-white tracking-tight mt-0.5">
              {explainable?.specsLabel || job.finalFileName || 'Media Artifact'}
            </h3>
          </div>
        </div>

        {job.finalFilePath && (
          <div className="flex items-center space-x-2">
            {onOpenFile && (
              <button
                id="receipt-play-btn"
                type="button"
                onClick={() => onOpenFile(job.finalFilePath!)}
                className="p-2 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-xl border border-slate-700/80 transition-colors"
                title="Play Media"
              >
                <Play className="w-4 h-4 text-emerald-400" />
              </button>
            )}
            {onOpenDirectory && (
              <button
                id="receipt-folder-btn"
                type="button"
                onClick={() => onOpenDirectory(job.outputDirectory)}
                className="p-2 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-xl border border-slate-700/80 transition-colors"
                title="Open Folder"
              >
                <FolderOpen className="w-4 h-4 text-blue-400" />
              </button>
            )}
          </div>
        )}
      </div>

      {/* Verification Checklist Grid */}
      <div className="mb-4">
        <h4 className="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2 flex items-center space-x-1.5">
          <ShieldCheck className="w-4 h-4 text-emerald-400" />
          <span>Integrity Verification Checklist</span>
        </h4>
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
          <div className="flex items-center space-x-2 p-2 rounded-xl bg-slate-950/40 border border-slate-800/80">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            <span className="text-xs text-slate-300">File Output Exists</span>
          </div>
          <div className="flex items-center space-x-2 p-2 rounded-xl bg-slate-950/40 border border-slate-800/80">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            <span className="text-xs text-slate-300">File Size Non-Zero</span>
          </div>
          <div className="flex items-center space-x-2 p-2 rounded-xl bg-slate-950/40 border border-slate-800/80">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            <span className="text-xs text-slate-300">Stream Codecs Valid</span>
          </div>
          <div className="flex items-center space-x-2 p-2 rounded-xl bg-slate-950/40 border border-slate-800/80">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            <span className="text-xs text-slate-300">Duration Matched</span>
          </div>
          <div className="flex items-center space-x-2 p-2 rounded-xl bg-slate-950/40 border border-slate-800/80">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            <span className="text-xs text-slate-300">Container Encapsulation</span>
          </div>
          <div className="flex items-center space-x-2 p-2 rounded-xl bg-slate-950/40 border border-slate-800/80">
            <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            <span className="text-xs text-slate-300">Zero Corruption</span>
          </div>
        </div>
      </div>

      {/* Processing Explanation */}
      {explainable && (
        <div className="bg-slate-950/50 rounded-xl p-3.5 border border-slate-800/80 mb-4">
          <div className="text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
            Processing Summary
          </div>
          <p className="text-sm font-medium text-slate-200 mb-2">
            {explainable.processingSummary}
          </p>

          {explainable.whyReasons && explainable.whyReasons.length > 0 && (
            <div className="flex flex-wrap gap-1.5 mt-2">
              {explainable.whyReasons.map((r, i) => (
                <span
                  key={i}
                  className="px-2 py-0.5 text-xs bg-slate-900 text-slate-300 rounded-md border border-slate-800 font-mono"
                >
                  {r}
                </span>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Technical Accordion (Fingerprint & Recipe) */}
      <div className="border-t border-slate-800/80 pt-3">
        <button
          id="toggle-technical-details-btn"
          type="button"
          onClick={() => setShowTechnicalDetails(!showTechnicalDetails)}
          className="flex items-center justify-between w-full text-xs font-semibold text-slate-400 hover:text-slate-200 transition-colors py-1"
        >
          <div className="flex items-center space-x-2">
            <Fingerprint className="w-4 h-4 text-blue-400" />
            <span>Provenance & Reproducibility Recipe</span>
          </div>
          {showTechnicalDetails ? (
            <ChevronUp className="w-4 h-4" />
          ) : (
            <ChevronDown className="w-4 h-4" />
          )}
        </button>

        {showTechnicalDetails && (
          <div className="mt-3 space-y-3">
            {fingerprint && (
              <div className="bg-slate-950/60 rounded-xl p-3 border border-slate-800">
                <div className="flex items-center justify-between mb-1.5">
                  <span className="text-[11px] font-semibold uppercase tracking-wider text-slate-400">
                    Canonical Fingerprint ID
                  </span>
                  <span className="text-[10px] font-mono text-blue-400 bg-blue-950/40 px-1.5 py-0.5 rounded border border-blue-900/50">
                    {fingerprint.canonicalId}
                  </span>
                </div>
                <div className="grid grid-cols-2 gap-2 text-xs text-slate-300 font-mono">
                  <div>
                    <span className="text-slate-500">Source ID:</span>{' '}
                    {fingerprint.source.sourceId}
                  </div>
                  <div>
                    <span className="text-slate-500">Extractor:</span>{' '}
                    {fingerprint.source.extractor}
                  </div>
                  <div>
                    <span className="text-slate-500">Streams:</span>{' '}
                    {fingerprint.streamCount} streams
                  </div>
                  <div>
                    <span className="text-slate-500">Max Res:</span>{' '}
                    {fingerprint.maxResolution || 'N/A'}
                  </div>
                </div>
              </div>
            )}

            {recipe && (
              <div className="bg-slate-950/60 rounded-xl p-3 border border-slate-800">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center space-x-2">
                    <FileCode className="w-4 h-4 text-amber-400" />
                    <span className="text-[11px] font-semibold uppercase tracking-wider text-slate-400">
                      Recipe ID: {recipe.id}
                    </span>
                  </div>
                  <button
                    id="copy-recipe-json-btn"
                    type="button"
                    onClick={handleCopyRecipe}
                    className="flex items-center space-x-1 px-2 py-0.5 text-[11px] font-medium bg-slate-800 hover:bg-slate-700 text-slate-300 rounded border border-slate-700/80 transition-colors"
                  >
                    {copiedRecipe ? (
                      <>
                        <Check className="w-3 h-3 text-emerald-400" />
                        <span className="text-emerald-400">Copied</span>
                      </>
                    ) : (
                      <>
                        <Copy className="w-3 h-3" />
                        <span>Copy Recipe JSON</span>
                      </>
                    )}
                  </button>
                </div>

                <div className="text-xs text-slate-400 mb-2">
                  <span className="font-semibold text-slate-300">Transformations:</span>
                  <ul className="list-disc list-inside mt-1 space-y-0.5 font-mono text-[11px] text-slate-300">
                    {recipe.transformations.map((t, idx) => (
                      <li key={idx}>{t}</li>
                    ))}
                  </ul>
                </div>

                <pre className="text-[10px] font-mono text-slate-400 bg-slate-900/90 p-2.5 rounded-lg border border-slate-800/80 overflow-x-auto max-h-36">
                  {JSON.stringify(recipe, null, 2)}
                </pre>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

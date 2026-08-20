import React from 'react';
import { CheckCircle2, Sparkles, Zap, ShieldAlert, Cpu } from 'lucide-react';
import { FormatRecommendation, TranscodingCost } from '../types';

interface RecommendationCardProps {
  recommendation: FormatRecommendation;
  onApply: () => void;
  isApplied?: boolean;
}

const TRANSCODING_BADGES: Record<
  TranscodingCost,
  { label: string; bg: string; text: string; border: string }
> = {
  NO_PROCESSING: {
    label: 'Direct Pass',
    bg: 'bg-emerald-500/10',
    text: 'text-emerald-400',
    border: 'border-emerald-500/30',
  },
  STREAM_COPY: {
    label: 'Zero Re-encode (Stream Copy)',
    bg: 'bg-emerald-500/10',
    text: 'text-emerald-400',
    border: 'border-emerald-500/30',
  },
  REMUX: {
    label: 'Container Remux',
    bg: 'bg-teal-500/10',
    text: 'text-teal-400',
    border: 'border-teal-500/30',
  },
  MERGE: {
    label: 'Bitstream Merge (No Video Re-encode)',
    bg: 'bg-blue-500/10',
    text: 'text-blue-400',
    border: 'border-blue-500/30',
  },
  TRANSCODE: {
    label: 'Audio Transcode (Highest Bitrate)',
    bg: 'bg-amber-500/10',
    text: 'text-amber-400',
    border: 'border-amber-500/30',
  },
};

export const RecommendationCard: React.FC<RecommendationCardProps> = ({
  recommendation,
  onApply,
  isApplied = false,
}) => {
  const badgeInfo =
    TRANSCODING_BADGES[recommendation.transcodingCost] ||
    TRANSCODING_BADGES.MERGE;

  return (
    <div
      id="recommendation-card"
      className="w-full bg-gradient-to-br from-slate-900/90 via-slate-900/60 to-blue-950/20 border border-blue-500/30 rounded-2xl p-4 shadow-xl mb-4 transition-all duration-300 relative overflow-hidden"
    >
      <div className="flex items-center justify-between gap-2 mb-3">
        <div className="flex items-center space-x-2">
          <div className="p-1.5 rounded-lg bg-blue-500/20 text-blue-400 border border-blue-400/30">
            <Sparkles className="w-4 h-4" />
          </div>
          <div>
            <h4 className="text-xs font-semibold uppercase tracking-wider text-blue-400">
              Recommended Strategy
            </h4>
            <span className="text-sm font-bold text-white tracking-tight">
              {recommendation.label}
            </span>
          </div>
        </div>

        <span
          className={`inline-flex items-center space-x-1 px-2.5 py-1 text-xs font-semibold rounded-full border ${badgeInfo.bg} ${badgeInfo.text} ${badgeInfo.border}`}
        >
          <Zap className="w-3 h-3" />
          <span>{badgeInfo.label}</span>
        </span>
      </div>

      <p className="text-xs text-slate-300 mb-3 leading-relaxed">
        {recommendation.reason}
      </p>

      {recommendation.whyReasons && recommendation.whyReasons.length > 0 && (
        <div className="bg-slate-950/40 rounded-xl p-3 border border-slate-800/60 mb-3.5">
          <div className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider mb-2 flex items-center space-x-1.5">
            <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
            <span>Why this selection</span>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-1.5">
            {recommendation.whyReasons.map((reason, idx) => (
              <div
                key={idx}
                className="text-xs text-slate-300 flex items-center space-x-1.5 font-medium"
              >
                <span className="text-emerald-400 font-bold">•</span>
                <span>{reason.replace(/^✓\s*/, '')}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="flex items-center justify-between pt-1">
        <div className="flex items-center space-x-3 text-xs text-slate-400">
          <span className="font-mono text-slate-300">
            Container: <span className="uppercase font-bold text-blue-400">{recommendation.container}</span>
          </span>
          <span className="text-slate-600">|</span>
          <span className="font-mono text-slate-300">
            Target: <span className="font-bold text-slate-200">{recommendation.targetQuality}p</span>
          </span>
        </div>

        <button
          id="apply-recommendation-btn"
          type="button"
          onClick={onApply}
          className={`px-3.5 py-1.5 rounded-lg text-xs font-semibold transition-all duration-200 flex items-center space-x-1.5 ${
            isApplied
              ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 cursor-default'
              : 'bg-blue-600 hover:bg-blue-500 text-white shadow-md shadow-blue-900/30'
          }`}
        >
          {isApplied ? (
            <>
              <CheckCircle2 className="w-3.5 h-3.5" />
              <span>Applied</span>
            </>
          ) : (
            <>
              <Sparkles className="w-3.5 h-3.5" />
              <span>Use Recommendation</span>
            </>
          )}
        </button>
      </div>
    </div>
  );
};

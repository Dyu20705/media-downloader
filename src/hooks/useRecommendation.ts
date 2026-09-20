import { useCallback, useMemo, useState } from 'react';
import type {
  FormatRecommendation,
  MediaMetadata,
  PresetType,
  UserIntent,
} from '../types';

interface RecommendationOptions {
  metadata: MediaMetadata | null;
  intent: UserIntent;
  preset: PresetType;
  quality: string;
  setPreset: (preset: PresetType) => void;
  setQuality: (quality: string) => void;
}

export function useRecommendation({
  metadata,
  preset,
  quality,
  setPreset,
  setQuality,
}: RecommendationOptions) {
  const [appliedSignature, setAppliedSignature] = useState<string | null>(null);
  const currentRecommendation = metadata?.smartRecommendation ?? null;
  const signature = currentRecommendation
    ? `${currentRecommendation.preset}:${currentRecommendation.targetQuality}`
    : null;

  const handleApplyRecommendation = useCallback(() => {
    if (!currentRecommendation) return;
    setPreset(currentRecommendation.preset);
    setQuality(currentRecommendation.targetQuality || 'auto');
    setAppliedSignature(signature);
  }, [currentRecommendation, setPreset, setQuality, signature]);

  const isRecommendationApplied = useMemo(() => {
    if (!currentRecommendation || !signature) return false;
    return appliedSignature === signature
      && preset === currentRecommendation.preset
      && quality === (currentRecommendation.targetQuality || 'auto');
  }, [appliedSignature, currentRecommendation, preset, quality, signature]);

  return {
    currentRecommendation: currentRecommendation as FormatRecommendation | null,
    isRecommendationApplied,
    handleApplyRecommendation,
  };
}

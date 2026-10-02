import { useCallback, useMemo, useState } from 'react';
import type {
  FormatRecommendation,
  AcquisitionOperation,
  MediaMetadata,
  OutputProfile,
  UserIntent,
} from '../types';
import { selectionFromPreset } from '../acquisition';

interface RecommendationOptions {
  metadata: MediaMetadata | null;
  intent: UserIntent;
  operation: AcquisitionOperation;
  outputProfile: OutputProfile;
  quality: string;
  setOperation: (operation: AcquisitionOperation) => void;
  setOutputProfile: (profile: OutputProfile) => void;
  setQuality: (quality: string) => void;
}

export function useRecommendation({
  metadata,
  operation,
  outputProfile,
  quality,
  setOperation,
  setOutputProfile,
  setQuality,
}: RecommendationOptions) {
  const [appliedSignature, setAppliedSignature] = useState<string | null>(null);
  const currentRecommendation = metadata?.smartRecommendation ?? null;
  const signature = currentRecommendation
    ? `${currentRecommendation.preset}:${currentRecommendation.targetQuality}`
    : null;
  const recommendedSelection = currentRecommendation
    ? selectionFromPreset(currentRecommendation.preset)
    : null;

  const handleApplyRecommendation = useCallback(() => {
    if (!currentRecommendation) return;
    const selection = selectionFromPreset(currentRecommendation.preset);
    setOperation(selection.operation);
    setOutputProfile(selection.outputProfile);
    setQuality(currentRecommendation.targetQuality || 'auto');
    setAppliedSignature(signature);
  }, [currentRecommendation, setOperation, setOutputProfile, setQuality, signature]);

  const isRecommendationApplied = useMemo(() => {
    if (!currentRecommendation || !signature) return false;
    return appliedSignature === signature
      && operation.type === recommendedSelection?.operation.type
      && outputProfile === recommendedSelection?.outputProfile
      && quality === (currentRecommendation.targetQuality || 'auto');
  }, [appliedSignature, currentRecommendation, operation.type, outputProfile, quality, recommendedSelection, signature]);

  return {
    currentRecommendation: currentRecommendation as FormatRecommendation | null,
    isRecommendationApplied,
    handleApplyRecommendation,
  };
}

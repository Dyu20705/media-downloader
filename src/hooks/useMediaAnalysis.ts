import { useCallback, useRef, useState } from 'react';
import { ipc } from '../services/ipc';
import type { AppError, MediaMetadata } from '../types';

function normalizeError(error: unknown, fallback: string): AppError {
  const technicalDetails = error instanceof Error ? error.message : String(error);
  return { userMessage: fallback, technicalDetails };
}

export function useMediaAnalysis() {
  const [url, setUrlState] = useState('');
  const [metadata, setMetadata] = useState<MediaMetadata | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [analysisError, setAnalysisError] = useState<AppError | null>(null);
  const [showErrorDetails, setShowErrorDetails] = useState(false);
  const requestId = useRef(0);

  const setUrl = useCallback((next: string) => {
    requestId.current += 1;
    setUrlState(next);
    setMetadata(null);
    setAnalysisError(null);
    setShowErrorDetails(false);
  }, []);

  const handleAnalyze = useCallback(async (urlToAnalyze?: string) => {
    const target = (urlToAnalyze ?? url).trim();
    if (!target) return;

    const currentRequest = ++requestId.current;
    if (urlToAnalyze !== undefined) setUrlState(target);
    setIsAnalyzing(true);
    setMetadata(null);
    setAnalysisError(null);
    setShowErrorDetails(false);
    try {
      const result = await ipc.analyzeMedia(target);
      if (currentRequest === requestId.current) setMetadata(result);
    } catch (error) {
      if (currentRequest === requestId.current) {
        setAnalysisError(normalizeError(error, 'Could not analyze this media source.'));
      }
    } finally {
      if (currentRequest === requestId.current) setIsAnalyzing(false);
    }
  }, [url]);

  return {
    url,
    setUrl,
    metadata,
    isAnalyzing,
    analysisError,
    showErrorDetails,
    setShowErrorDetails,
    handleAnalyze,
  };
}

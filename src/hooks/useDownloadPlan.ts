import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { DownloadPlan, QualityPreset } from '../quality-transparency';

interface UseDownloadPlanOptions {
  metadata: unknown | null;
  preset: QualityPreset;
  quality: string;
}

export const useDownloadPlan = ({ metadata, preset, quality }: UseDownloadPlanOptions) => {
  const [plan, setPlan] = useState<DownloadPlan | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let isCurrent = true;

    if (!metadata) {
      setPlan(null);
      setError(null);
      setIsLoading(false);
      return () => {
        isCurrent = false;
      };
    }

    setIsLoading(true);
    setError(null);
    invoke<DownloadPlan>('get_download_plan', { metadata, preset, quality })
      .then((nextPlan) => {
        if (isCurrent) setPlan(nextPlan);
      })
      .catch((reason: unknown) => {
        if (!isCurrent) return;
        setPlan(null);
        setError(reason instanceof Error ? reason.message : String(reason));
      })
      .finally(() => {
        if (isCurrent) setIsLoading(false);
      });

    return () => {
      isCurrent = false;
    };
  }, [metadata, preset, quality]);

  return { plan, isLoading, error };
};

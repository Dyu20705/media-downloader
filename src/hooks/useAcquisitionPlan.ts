import { useEffect, useRef, useState } from 'react';
import { ipc } from '../services/ipc';
import type { AcquisitionPlan, AcquisitionRequest, AppError, MediaMetadata } from '../types';

interface UseAcquisitionPlanResult {
  plan: AcquisitionPlan | null;
  isPlanning: boolean;
  planningError: AppError | null;
}

export function useAcquisitionPlan(
  metadata: MediaMetadata | null,
  acquisition: AcquisitionRequest | null,
): UseAcquisitionPlanResult {
  const [plan, setPlan] = useState<AcquisitionPlan | null>(null);
  const [isPlanning, setIsPlanning] = useState(false);
  const [planningError, setPlanningError] = useState<AppError | null>(null);
  const requestId = useRef(0);

  useEffect(() => {
    const currentRequest = ++requestId.current;

    if (!metadata || !acquisition) {
      setPlan(null);
      setIsPlanning(false);
      setPlanningError(null);
      return;
    }

    setPlan(null);
    setIsPlanning(true);
    setPlanningError(null);

    void ipc.planAcquisition(metadata, acquisition)
      .then((nextPlan) => {
        if (currentRequest === requestId.current) setPlan(nextPlan);
      })
      .catch((error: unknown) => {
        if (currentRequest !== requestId.current) return;
        const technicalDetails = error instanceof Error ? error.message : String(error);
        setPlanningError({
          userMessage: 'A download plan could not be created for these choices.',
          technicalDetails,
        });
      })
      .finally(() => {
        if (currentRequest === requestId.current) setIsPlanning(false);
      });
  }, [metadata, acquisition]);

  return { plan, isPlanning, planningError };
}

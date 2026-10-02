import { useEffect, useRef, useState } from 'react';
import { ipc } from '../services/ipc';
import { sanitizeTechnicalError } from '../utils/sanitizeError';
import type { AcquisitionPlan, AcquisitionRequest, AppError, MediaMetadata } from '../types';

interface UseAcquisitionPlanResult {
  plan: AcquisitionPlan | null;
  isPlanning: boolean;
  planningError: AppError | null;
}

export function useAcquisitionPlan(
  metadata: MediaMetadata | null,
  acquisition: AcquisitionRequest | null,
  settingsRevision?: unknown,
): UseAcquisitionPlanResult {
  const [plan, setPlan] = useState<AcquisitionPlan | null>(null);
  const [isPlanning, setIsPlanning] = useState(false);
  const [planningError, setPlanningError] = useState<AppError | null>(null);
  const requestId = useRef(0);
  const [resolvedInput, setResolvedInput] = useState<{ metadata: MediaMetadata; acquisition: AcquisitionRequest; settingsRevision: unknown } | null>(null);

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
        if (currentRequest === requestId.current) {
          setPlan(nextPlan);
          setResolvedInput({ metadata, acquisition, settingsRevision });
        }
      })
      .catch((error: unknown) => {
        if (currentRequest !== requestId.current) return;
        const technicalDetails = sanitizeTechnicalError(error);
        setPlanningError({
          userMessage: 'A download plan could not be created for these choices.',
          technicalDetails,
        });
      })
      .finally(() => {
        if (currentRequest === requestId.current) setIsPlanning(false);
      });
    return () => { requestId.current += 1; };
  }, [metadata, acquisition, settingsRevision]);

  const current = resolvedInput?.metadata === metadata && resolvedInput?.acquisition === acquisition && resolvedInput?.settingsRevision === settingsRevision;
  return { plan: current ? plan : null, isPlanning: isPlanning || Boolean(metadata && acquisition && !current && !planningError), planningError };
}

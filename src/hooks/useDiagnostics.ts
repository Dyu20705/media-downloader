import { useCallback, useState } from 'react';
import { ipc } from '../services/ipc';
import type { DiagnosticLog } from '../types';

export function useDiagnostics() {
  const [logs, setLogs] = useState<DiagnosticLog[]>([]);

  const fetchDiagnostics = useCallback(async () => {
    const next = await ipc.getDiagnostics();
    setLogs(next);
  }, []);

  const clearDiagnostics = useCallback(async () => {
    await ipc.clearDiagnostics();
    setLogs([]);
  }, []);

  return { logs, fetchDiagnostics, clearDiagnostics };
}

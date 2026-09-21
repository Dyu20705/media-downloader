import { useCallback, useEffect, useState } from 'react';
import { ipc } from '../services/ipc';
import type { ToolHealth, ToolStatusInfo } from '../types';

export function useToolEngine() {
  const [tools, setTools] = useState<ToolHealth[]>([]);
  const [toolStatuses, setToolStatuses] = useState<ToolStatusInfo[]>([]);
  const [isRefreshingTools, setIsRefreshingTools] = useState(false);
  const [isAutoBootstrapping] = useState(false);

  const fetchTools = useCallback(async () => {
    setIsRefreshingTools(true);
    try {
      const detailed = await ipc.getDetailedToolStatus();
      setToolStatuses(detailed);
      setTools(
        detailed.map((tool) => ({
          name: tool.name,
          available: tool.status === 'READY',
          path: tool.path,
          version: tool.version,
          repairMessage:
            tool.status === 'READY'
              ? null
              : tool.errorMessage ?? `Click Repair to install ${tool.name}`,
        })),
      );
    } finally {
      setIsRefreshingTools(false);
    }
  }, []);

  useEffect(() => {
    void fetchTools().catch(() => {
      // Setup controls remain visible when the engine cannot be inspected.
    });
  }, [fetchTools]);

  const refreshAfter = useCallback(async (action: () => Promise<unknown>) => {
    await action();
    await fetchTools();
  }, [fetchTools]);

  const installTool = useCallback((name: string) =>
    refreshAfter(() => ipc.installTool(name)), [refreshAfter]);
  const repairTool = useCallback((name: string) =>
    refreshAfter(() => ipc.repairTool(name)), [refreshAfter]);
  const installAllTools = useCallback(() =>
    refreshAfter(() => ipc.installAllTools()), [refreshAfter]);

  return {
    tools,
    toolStatuses,
    isRefreshingTools,
    isAutoBootstrapping,
    fetchTools,
    installTool,
    repairTool,
    installAllTools,
  };
}

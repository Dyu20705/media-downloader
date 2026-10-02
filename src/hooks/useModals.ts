import { useState } from 'react';

export function useModals() {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isHistoryOpen, setIsHistoryOpen] = useState(false);
  const [isToolsOpen, setIsToolsOpen] = useState(false);
  const [isCheatsheetOpen, setIsCheatsheetOpen] = useState(false);
  const [isDiagnosticsOpen, setIsDiagnosticsOpen] = useState(false);
  const [isInspectingMetadata, setIsInspectingMetadata] = useState(false);

  return {
    isSettingsOpen,
    setIsSettingsOpen,
    isHistoryOpen,
    setIsHistoryOpen,
    isToolsOpen,
    setIsToolsOpen,
    isCheatsheetOpen,
    setIsCheatsheetOpen,
    isDiagnosticsOpen,
    setIsDiagnosticsOpen,
    isInspectingMetadata,
    setIsInspectingMetadata,
  };
}

import { useCallback, useEffect, useState } from 'react';
import { ipc } from '../services/ipc';
import type { AcquisitionOperation, AppSettings, OutputProfile, UserIntent } from '../types';
import { selectionFromPreset } from '../acquisition';

const DEFAULT_SETTINGS: AppSettings = {
  downloadDirectory: '',
  lastPreset: 'mp4-compatible',
  defaultQuality: 'auto',
  openFolderAfterDownload: false,
  autoAnalyzeOnPaste: true,
  embedMetadata: true,
  embedThumbnail: true,
  embedChapters: true,
  concurrentFragments: 1,
  trimFilenames: 180,
  sponsorBlockMode: 'off',
  subtitleMode: 'none',
  preferredSubtitleLanguage: 'en',
  customYtdlpPath: null,
  customFfmpegPath: null,
  customFfprobePath: null,
  customMediainfoPath: null,
};

export function useAppSettings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [outputDirectory, setOutputDirectory] = useState('');
  const [operation, setOperation] = useState<AcquisitionOperation>({ type: 'ENTIRE_MEDIA' });
  const [outputProfile, setOutputProfile] = useState<OutputProfile>('UNIVERSAL');
  const [quality, setQuality] = useState('auto');
  const [intent] = useState<UserIntent>('balanced');

  useEffect(() => {
    let active = true;
    ipc.getSettings().then((loaded) => {
      if (!active) return;
      setSettings(loaded);
      setOutputDirectory(loaded.downloadDirectory);
      const selection = selectionFromPreset(loaded.lastPreset);
      setOperation(selection.operation);
      setOutputProfile(selection.outputProfile);
      setQuality(loaded.defaultQuality);
    }).catch(() => {
      // The UI remains usable with safe defaults if settings cannot be read.
    });
    return () => {
      active = false;
    };
  }, []);

  const saveSettings = useCallback((next: AppSettings) => {
    ipc.saveSettings(next).then((saved) => {
      setSettings(saved);
      setOutputDirectory(saved.downloadDirectory);
      const selection = selectionFromPreset(saved.lastPreset);
      setOperation(selection.operation);
      setOutputProfile(selection.outputProfile);
      setQuality(saved.defaultQuality);
    }).catch(() => {
      // Keep the last confirmed settings when persistence fails.
    });
  }, []);

  return {
    settings,
    outputDirectory,
    setOutputDirectory,
    operation,
    setOperation,
    outputProfile,
    setOutputProfile,
    quality,
    setQuality,
    intent,
    saveSettings,
  };
}

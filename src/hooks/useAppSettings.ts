import { useCallback, useEffect, useState } from 'react';
import { ipc } from '../services/ipc';
import type { AppSettings, PresetType, UserIntent } from '../types';

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
  const [preset, setPreset] = useState<PresetType>('mp4-compatible');
  const [quality, setQuality] = useState('auto');
  const [intent] = useState<UserIntent>('balanced');

  useEffect(() => {
    let active = true;
    ipc.getSettings().then((loaded) => {
      if (!active) return;
      setSettings(loaded);
      setOutputDirectory(loaded.downloadDirectory);
      setPreset(loaded.lastPreset);
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
      setPreset(saved.lastPreset);
      setQuality(saved.defaultQuality);
    }).catch(() => {
      // Keep the last confirmed settings when persistence fails.
    });
  }, []);

  return {
    settings,
    outputDirectory,
    setOutputDirectory,
    preset,
    setPreset,
    quality,
    setQuality,
    intent,
    saveSettings,
  };
}

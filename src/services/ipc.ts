import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  AppSettings,
  BuildCommandRequest,
  BuildCommandResponse,
  DiagnosticLog,
  DownloadJob,
  MediaMetadata,
  StartDownloadRequest,
  ToolHealth,
  ToolStatusInfo,
} from '../types';

export const ipc = {
  analyzeMedia: (url: string) => invoke<MediaMetadata>('analyze_media', { url }),
  buildCommand: (request: BuildCommandRequest) =>
    invoke<BuildCommandResponse>('build_command', { request }),
  startDownload: (request: StartDownloadRequest) =>
    invoke<DownloadJob>('start_download', { request }),
  cancelDownload: (jobId: string) =>
    invoke<DownloadJob>('cancel_download', { jobId }),
  getActiveJob: () => invoke<DownloadJob | null>('get_active_job'),
  getToolStatus: () => invoke<ToolHealth[]>('get_tool_status'),
  getDetailedToolStatus: () => invoke<ToolStatusInfo[]>('get_detailed_tool_status'),
  installTool: (name: string) => invoke<ToolStatusInfo>('install_tool', { name }),
  repairTool: (name: string) => invoke<ToolStatusInfo>('repair_tool', { name }),
  installAllTools: () => invoke<ToolStatusInfo[]>('install_all_missing_tools'),
  autoBootstrapTools: () => invoke<ToolStatusInfo[]>('auto_bootstrap_tools'),
  getSettings: () => invoke<AppSettings>('get_settings'),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>('save_settings', { settings }),
  getDiagnostics: () => invoke<DiagnosticLog[]>('get_diagnostics'),
  clearDiagnostics: () => invoke<void>('clear_diagnostics'),
  openFolder: (path: string) => invoke<void>('open_folder', { path }),
  openFile: (path: string) => invoke<void>('open_file', { path }),
  selectDirectory: async (): Promise<string | null> => {
    const selection = await open({ directory: true, multiple: false });
    return typeof selection === 'string' ? selection : null;
  },
};

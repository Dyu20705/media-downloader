import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  AppSettings,
  AcquisitionPlan,
  AcquisitionRequest,
  BuildCommandRequest,
  BuildCommandResponse,
  DiagnosticLog,
  DownloadJob,
  MediaMetadata,
  StartDownloadRequest,
  ToolHealth,
  ToolStatusInfo,
} from '../types';

function ensureTauriRuntime(): void {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    throw new Error(
      'The desktop app bridge is unavailable. Open openDownloader from the installed desktop app and try again.',
    );
  }
}

function invokeTauri<T>(command: string, args?: object): Promise<T> {
  try {
    ensureTauriRuntime();
  } catch (error) {
    return Promise.reject(error);
  }

  return invoke<T>(command, args as Record<string, unknown> | undefined);
}

export const ipc = {
  analyzeMedia: (url: string) => invokeTauri<MediaMetadata>('analyze_media', { url }),
  planAcquisition: (metadata: MediaMetadata, acquisition: AcquisitionRequest) =>
    invokeTauri<AcquisitionPlan>('plan_acquisition', { metadata, acquisition }),
  buildCommand: (request: BuildCommandRequest) =>
    invokeTauri<BuildCommandResponse>('build_command', { request }),
  startDownload: (request: StartDownloadRequest) =>
    invokeTauri<DownloadJob>('start_download', { request }),
  cancelDownload: (jobId: string) =>
    invokeTauri<DownloadJob>('cancel_download', { jobId }),
  getActiveJob: () => invokeTauri<DownloadJob | null>('get_active_job'),
  getDownloadHistory: () => invokeTauri<DownloadJob[]>('get_download_history'),
  retryDownload: (jobId: string) => invokeTauri<DownloadJob>('retry_download', { jobId }),
  getToolStatus: () => invokeTauri<ToolHealth[]>('get_tool_status'),
  getDetailedToolStatus: () => invokeTauri<ToolStatusInfo[]>('get_detailed_tool_status'),
  installTool: (name: string) => invokeTauri<ToolStatusInfo>('install_tool', { name }),
  repairTool: (name: string) => invokeTauri<ToolStatusInfo>('repair_tool', { name }),
  installAllTools: () => invokeTauri<ToolStatusInfo[]>('install_all_missing_tools'),
  autoBootstrapTools: () => invokeTauri<ToolStatusInfo[]>('auto_bootstrap_tools'),
  getSettings: () => invokeTauri<AppSettings>('get_settings'),
  saveSettings: (settings: AppSettings) => invokeTauri<AppSettings>('save_settings', { settings }),
  getDiagnostics: () => invokeTauri<DiagnosticLog[]>('get_diagnostics'),
  clearDiagnostics: () => invokeTauri<void>('clear_diagnostics'),
  openFolder: (path: string) => invokeTauri<void>('open_folder', { path }),
  openFile: (path: string) => invokeTauri<void>('open_file', { path }),
  selectDirectory: async (): Promise<string | null> => {
    ensureTauriRuntime();
    const selection = await open({ directory: true, multiple: false });
    return typeof selection === 'string' ? selection : null;
  },
};

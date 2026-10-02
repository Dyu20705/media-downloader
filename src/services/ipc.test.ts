import { afterEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { ipc } from './ipc';

vi.mock('@tauri-apps/api/core', async (importOriginal) => ({
  ...await importOriginal<typeof import('@tauri-apps/api/core')>(),
  invoke: vi.fn(),
}));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));

afterEach(() => {
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

describe('desktop bridge runtime detection', () => {
  it('rejects IPC and dialogs with a friendly error outside Tauri, without window', async () => {
    vi.stubGlobal('window', undefined);
    vi.stubGlobal('isTauri', undefined);
    await expect(ipc.getSettings()).rejects.toThrow('Open openDownloader from the installed desktop app');
    await expect(ipc.selectDirectory()).rejects.toThrow('desktop app bridge is unavailable');
    expect(invoke).not.toHaveBeenCalled();
    expect(open).not.toHaveBeenCalled();
  });

  it('rejects a normal browser without the public runtime marker', async () => {
    vi.stubGlobal('window', {});
    vi.stubGlobal('isTauri', false);
    await expect(ipc.getSettings()).rejects.toThrow('desktop app bridge is unavailable');
    expect(invoke).not.toHaveBeenCalled();
  });

  it('invokes IPC and directory selection in Tauri', async () => {
    vi.stubGlobal('isTauri', true);
    vi.mocked(invoke).mockResolvedValue([]);
    vi.mocked(open).mockResolvedValue('/selected folder');
    await expect(ipc.getDownloadHistory()).resolves.toEqual([]);
    expect(invoke).toHaveBeenCalledWith('get_download_history', undefined);
    await expect(ipc.selectDirectory()).resolves.toBe('/selected folder');
    expect(open).toHaveBeenCalledWith({ directory: true, multiple: false });
  });
});

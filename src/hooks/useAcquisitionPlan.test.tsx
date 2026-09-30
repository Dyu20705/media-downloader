import React from 'react';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ipc } from '../services/ipc';
import { useAcquisitionPlan } from './useAcquisitionPlan';
import type { AcquisitionPlan, AcquisitionRequest, MediaMetadata } from '../types';

vi.mock('../services/ipc', () => ({ ipc: { planAcquisition: vi.fn() } }));

const metadata = { id: 'source' } as MediaMetadata;
const request: AcquisitionRequest = {
  sourceScope: 'SINGLE_MEDIA', operation: { type: 'THUMBNAIL_ONLY' },
  outputProfile: 'BEST_SOURCE', outputDirectory: '/tmp/output', duplicatePolicy: 'RENAME',
  trackSelection: { subtitleLanguages: [], includeAutoSubtitles: false },
};
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}
let result: ReturnType<typeof useAcquisitionPlan>;
let root: ReactTestRenderer | undefined;
function Probe({ revision = 0, acquisition = request }: { revision?: number; acquisition?: AcquisitionRequest }) {
  result = useAcquisitionPlan(metadata, acquisition, revision);
  return null;
}
afterEach(() => { act(() => root?.unmount()); root = undefined; vi.resetAllMocks(); });

describe('reviewed acquisition plan', () => {
  it('discards a response from an earlier request', async () => {
    const first = deferred<AcquisitionPlan>();
    const second = deferred<AcquisitionPlan>();
    vi.mocked(ipc.planAcquisition).mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    act(() => { root = create(<Probe />); });
    act(() => root!.update(<Probe acquisition={{ ...request, operation: { type: 'SUBTITLES_ONLY' } }} />));
    await act(async () => { second.resolve({ id: 'new' } as AcquisitionPlan); });
    await act(async () => { first.resolve({ id: 'old' } as AcquisitionPlan); });
    expect(result.plan?.id).toBe('new');
    expect(result.isPlanning).toBe(false);
  });

  it('invalidates an accepted plan when semantic settings change', async () => {
    vi.mocked(ipc.planAcquisition).mockResolvedValueOnce({ id: 'old' } as AcquisitionPlan);
    await act(async () => { root = create(<Probe />); });
    expect(result.plan?.id).toBe('old');
    const pending = deferred<AcquisitionPlan>();
    vi.mocked(ipc.planAcquisition).mockReturnValueOnce(pending.promise);
    act(() => root!.update(<Probe revision={1} />));
    expect(result.plan).toBeNull();
    expect(result.isPlanning).toBe(true);
    await act(async () => { pending.resolve({ id: 'updated' } as AcquisitionPlan); });
    expect(result.plan?.id).toBe('updated');
  });

  it('exposes planning failure without leaving Start enabled', async () => {
    vi.mocked(ipc.planAcquisition).mockRejectedValue(new Error('Selected language unavailable'));
    await act(async () => { root = create(<Probe />); });
    expect(result.plan).toBeNull();
    expect(result.isPlanning).toBe(false);
    expect(result.planningError?.technicalDetails).toBe('Selected language unavailable');
  });
});

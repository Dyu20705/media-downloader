import { useCallback, useEffect, useRef, useState } from 'react';
import { ipc } from '../services/ipc';
import type { AppError, DownloadJob, StartDownloadRequest } from '../types';

const TERMINAL_STATUSES = new Set(['COMPLETED', 'FAILED', 'CANCELLED']);

function normalizeError(error: unknown, userMessage: string): AppError {
  const technicalDetails = error instanceof Error ? error.message : String(error);
  return { userMessage, technicalDetails };
}

function upsertJob(jobs: DownloadJob[], job: DownloadJob): DownloadJob[] {
  const existing = jobs.findIndex((candidate) => candidate.id === job.id);
  if (existing < 0) return [job, ...jobs];
  const next = [...jobs];
  next[existing] = job;
  return next;
}

interface DownloadEngineOptions {
  onDiagnosticsUpdate: () => Promise<void>;
}

export function useDownloadEngine({ onDiagnosticsUpdate }: DownloadEngineOptions) {
  const [activeJob, setActiveJob] = useState<DownloadJob | null>(null);
  const [allJobs, setAllJobs] = useState<DownloadJob[]>([]);
  const [downloadError, setDownloadError] = useState<AppError | null>(null);
  const mounted = useRef(true);

  const acceptJob = useCallback((job: DownloadJob | null) => {
    if (!job) return;
    setActiveJob(job);
    setAllJobs((current) => upsertJob(current, job));
  }, []);

  useEffect(() => {
    mounted.current = true;
    ipc.getActiveJob().then(acceptJob).catch(() => undefined);
    return () => {
      mounted.current = false;
    };
  }, [acceptJob]);

  const isJobRunning = Boolean(activeJob && !TERMINAL_STATUSES.has(activeJob.status));

  useEffect(() => {
    if (!isJobRunning) return;
    const timer = window.setInterval(() => {
      ipc.getActiveJob().then((job) => {
        if (!mounted.current || !job) return;
        acceptJob(job);
        if (TERMINAL_STATUSES.has(job.status)) {
          void onDiagnosticsUpdate().catch(() => undefined);
        }
      }).catch(() => undefined);
    }, 400);
    return () => window.clearInterval(timer);
  }, [acceptJob, isJobRunning, onDiagnosticsUpdate]);

  const startDownload = useCallback(async (request: StartDownloadRequest) => {
    setDownloadError(null);
    try {
      acceptJob(await ipc.startDownload(request));
    } catch (error) {
      setDownloadError(normalizeError(error, 'The download could not be started.'));
      void onDiagnosticsUpdate().catch(() => undefined);
    }
  }, [acceptJob, onDiagnosticsUpdate]);

  const cancelDownload = useCallback(async (jobId: string) => {
    try {
      acceptJob(await ipc.cancelDownload(jobId));
    } catch (error) {
      setDownloadError(normalizeError(error, 'The download could not be cancelled.'));
    }
  }, [acceptJob]);

  const resetActiveJob = useCallback(() => {
    if (!isJobRunning) {
      setActiveJob(null);
      setDownloadError(null);
    }
  }, [isJobRunning]);

  return {
    activeJob,
    allJobs,
    isJobRunning,
    startDownload,
    cancelDownload,
    resetActiveJob,
    downloadError,
  };
}

import React from 'react';
import type { MediaMetadata } from '../types';
import { getPlaylistIntentState } from '../utils/playlistIntent';

interface PlaylistIntentGuardProps {
  url: string;
  metadata: MediaMetadata;
}

export const PlaylistIntentGuard: React.FC<PlaylistIntentGuardProps> = ({ url, metadata }) => {
  const intent = getPlaylistIntentState(url, metadata);
  if (!intent.showGuard) return null;

  return (
    <fieldset className="rounded-xl border border-amber-800/70 bg-amber-950/20 p-4" aria-label="Playlist intent">
      <legend className="px-2 text-sm font-semibold text-amber-200">This URL has playlist context</legend>
      <p className="mb-3 text-xs text-zinc-300">
        {intent.videoAvailable
          ? 'The default is to acquire this video only. Playlist downloads are not available yet.'
          : 'This appears to be a playlist URL. Playlist downloads are not available yet.'}
      </p>
      <div className="space-y-2 text-sm">
        <label className={`flex items-center gap-2 ${intent.videoAvailable ? 'text-zinc-100' : 'text-zinc-500'}`}>
          <input type="radio" name="playlist-intent" checked={intent.videoAvailable} disabled={!intent.videoAvailable} readOnly />
          This video
        </label>
        <label className="flex items-center gap-2 text-zinc-500">
          <input type="radio" name="playlist-intent" disabled />
          Entire playlist <span className="text-xs">(not available yet)</span>
        </label>
      </div>
    </fieldset>
  );
};

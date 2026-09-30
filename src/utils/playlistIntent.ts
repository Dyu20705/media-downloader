import type { MediaMetadata } from '../types';

type PlaylistMetadata = Pick<MediaMetadata,
  'playlistTitle' | 'playlistIndex' | 'playlistCount' | 'hasVideo' | 'hasAudio'>;

export interface PlaylistIntentState {
  showGuard: boolean;
  videoAvailable: boolean;
}

/** Playlist execution is intentionally unavailable; ambiguous URLs may only acquire one video. */
export function getPlaylistIntentState(url: string, metadata?: PlaylistMetadata | null): PlaylistIntentState {
  let hasListParameter = false;
  let hasVideoParameter = false;
  try {
    const parsed = new URL(url);
    hasListParameter = Boolean(parsed.searchParams.get('list'));
    hasVideoParameter = Boolean(parsed.searchParams.get('v'));
  } catch {
    // Invalid URLs are handled by analysis; metadata can still identify extractor playlist context.
  }

  const extractorHasPlaylistContext = Boolean(
    metadata?.playlistTitle || metadata?.playlistIndex != null || metadata?.playlistCount != null,
  );
  const hasPlaylistContext = hasListParameter || extractorHasPlaylistContext;
  const hasPlayableMedia = Boolean(metadata?.hasVideo || metadata?.hasAudio);
  const playlistOnly = hasListParameter
    ? !hasVideoParameter && !metadata?.playlistIndex && !hasPlayableMedia
    : extractorHasPlaylistContext && metadata?.playlistIndex == null && !hasPlayableMedia;

  return {
    showGuard: hasPlaylistContext,
    videoAvailable: hasPlaylistContext && !playlistOnly,
  };
}

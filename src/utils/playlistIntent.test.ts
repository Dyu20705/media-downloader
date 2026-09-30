import { describe, expect, it } from 'vitest';
import { getPlaylistIntentState } from './playlistIntent';

const video = { hasVideo: true, hasAudio: true, playlistTitle: null, playlistIndex: null, playlistCount: null };

describe('playlist intent guard', () => {
  it('does not show playlist choices for a plain video', () => {
    expect(getPlaylistIntentState('https://example.test/watch?v=video', video))
      .toEqual({ showGuard: false, videoAvailable: false });
  });

  it('marks a plain playlist unavailable when no playable entry is analyzed', () => {
    expect(getPlaylistIntentState('https://example.test/playlist?list=collection', {
      ...video, hasVideo: false, hasAudio: false, playlistTitle: 'Collection', playlistCount: 8,
    })).toEqual({ showGuard: true, videoAvailable: false });
  });

  it('defaults an ambiguous video plus list URL to this video', () => {
    expect(getPlaylistIntentState('https://example.test/watch?v=video&list=collection', video))
      .toEqual({ showGuard: true, videoAvailable: true });
  });

  it('uses extractor playlist metadata as an ambiguity signal', () => {
    expect(getPlaylistIntentState('https://example.test/item/one', {
      ...video, playlistTitle: 'Collection', playlistIndex: 1, playlistCount: 8,
    })).toEqual({ showGuard: true, videoAvailable: true });
  });

  it('keeps entire-playlist execution unavailable for every playlist context', () => {
    const intent = getPlaylistIntentState('https://example.test/watch?v=video&list=collection', video);
    expect(intent.showGuard).toBe(true);
    // The UI exposes This video as the only enabled radio choice; playlist execution has no request path.
    expect(intent.videoAvailable).toBe(true);
  });
});

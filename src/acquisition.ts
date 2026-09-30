import type { AcquisitionOperation, OutputProfile, PresetType } from './types';

export function selectionFromPreset(preset: PresetType): {
  operation: AcquisitionOperation;
  outputProfile: OutputProfile;
} {
  if (preset === 'best-audio') return { operation: { type: 'AUDIO_ONLY' }, outputProfile: 'BEST_SOURCE' };
  if (preset === 'mp3') return { operation: { type: 'AUDIO_ONLY' }, outputProfile: 'UNIVERSAL' };
  if (preset === 'flac') return { operation: { type: 'AUDIO_ONLY' }, outputProfile: 'EDITING' };
  if (preset === 'best-video') return { operation: { type: 'ENTIRE_MEDIA' }, outputProfile: 'BEST_SOURCE' };
  return { operation: { type: 'ENTIRE_MEDIA' }, outputProfile: 'UNIVERSAL' };
}

export function operationLabel(operation: AcquisitionOperation): string {
  switch (operation.type) {
    case 'ENTIRE_MEDIA': return 'Entire media';
    case 'AUDIO_ONLY': return 'Audio only';
    case 'CLIP': return 'Clip';
    case 'THUMBNAIL_ONLY': return 'Thumbnail';
    case 'CHAPTER': return `Chapter ${operation.chapterIndex + 1}`;
    case 'SUBTITLES_ONLY': return 'Subtitles';
  }
}

export function profileLabel(profile: OutputProfile): string {
  return profile.toLowerCase().split('_').map((part) => part[0].toUpperCase() + part.slice(1)).join(' ');
}

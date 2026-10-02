export interface PlatformDefinition {
  id: string;
  name: string;
  description: string;
  sampleUrl: string;
}

export const PLATFORM_REGISTRY: PlatformDefinition[] = [
  {
    id: 'youtube',
    name: 'YouTube',
    description: 'Try a public YouTube video',
    sampleUrl: 'https://www.youtube.com/watch?v=BaW_jenozKc',
  },
  {
    id: 'vimeo',
    name: 'Vimeo',
    description: 'Try a public Vimeo video',
    sampleUrl: 'https://vimeo.com/76979871',
  },
  {
    id: 'soundcloud',
    name: 'SoundCloud',
    description: 'Try a public SoundCloud track',
    sampleUrl: 'https://soundcloud.com/iameden/eden-end-credits',
  },
];

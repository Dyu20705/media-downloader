export * from './generated/ipc';

export interface AppError {
  userMessage: string;
  technicalDetails?: string;
}

// apps/website/src/shared/constants/storage.ts

/**
 * Storage keys used across the application
 */
export const STORAGE_KEYS = {
  // Viewer-specific
  VIEWER_DSL: 'sruja:viewer:dsl',
  VIEWER_PANE: 'sruja:viewer:pane',
  
  // Progress tracking
  COMPLETED_CHALLENGES: 'sruja:completedChallenges',
  
  // Add more storage keys here as needed
} as const;

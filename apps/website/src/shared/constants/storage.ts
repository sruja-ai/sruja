// apps/website/src/shared/constants/storage.ts

/**
 * Storage keys used across the application
 */
export const STORAGE_KEYS = {
  // Playground-specific
  PLAYGROUND_DSL: 'sruja:playground:dsl',
  PLAYGROUND_PANE: 'sruja:playground:pane',

  // Progress tracking
  COMPLETED_CHALLENGES: 'sruja:completedChallenges',

  // Add more storage keys here as needed
} as const;

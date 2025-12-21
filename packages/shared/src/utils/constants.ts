// packages/shared/src/utils/constants.ts
// Application-wide constants to avoid magic numbers and strings

/**
 * Default project identifier used when project ID is not specified.
 * 
 * @public
 */
export const DEFAULT_PROJECT_ID = "sruja-project";

/**
 * Default project name used when project name is not specified.
 * 
 * @public
 */
export const DEFAULT_PROJECT_NAME = "sruja-project";

/**
 * Percentage validation constants.
 * 
 * @public
 */
export const PERCENTAGE = {
  /** Minimum valid percentage value */
  MIN: 0,
  /** Maximum valid percentage value */
  MAX: 100,
} as const;

/**
 * Reading time calculation constants.
 * 
 * @public
 */
export const READING_TIME = {
  /** Default words per minute for reading time calculation */
  DEFAULT_WPM: 200,
} as const;

/**
 * Retry and timeout constants for async operations.
 * 
 * @public
 */
export const RETRY = {
  /** Default retry delay in milliseconds */
  DEFAULT_DELAY_MS: 50,
  /** Default maximum retries for WASM operations */
  MAX_RETRIES: 150,
  /** Default timeout in milliseconds */
  DEFAULT_TIMEOUT_MS: 10000,
} as const;

/**
 * Storage key prefixes for browser storage.
 * 
 * @public
 */
export const STORAGE_KEYS = {
  /** Prefix for architecture data storage */
  ARCHITECTURE: "sruja-architecture-data",
  /** Prefix for UI settings storage */
  UI_SETTINGS: "sruja-ui-settings",
  /** Prefix for grid settings storage */
  GRID_SETTINGS: "sruja-grid-settings",
  /** Prefix for share data storage */
  SHARES: "sruja-shares",
} as const;

/**
 * Error stack trace truncation limit.
 * 
 * @public
 */
export const ERROR_STACK_TRUNCATE_LENGTH = 500;

/**
 * Memoization cache size limits.
 * 
 * @public
 */
export const CACHE = {
  /** Default maximum cache size for memoization */
  DEFAULT_MAX_SIZE: 100,
} as const;


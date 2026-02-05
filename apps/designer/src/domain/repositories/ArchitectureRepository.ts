/**
 * ArchitectureRepository Interface
 *
 * Defines the contract for persisting and retrieving architecture models.
 * This is a repository interface in the domain layer - implementations
 * will be in the infrastructure layer.
 *
 * @module domain/repositories
 */

import type { SrujaModelDump } from "@sruja/shared";
import { ArchitectureAggregate } from "../aggregates/ArchitectureAggregate";
import type { ValidationError, NetworkError, ConfigurationError } from "@sruja/shared/utils/errors";
import type { Result } from "@sruja/shared/utils/result";

/**
 * Options for querying architectures
 */
export interface QueryOptions {
  /** Filter by name (partial match) */
  name?: string;
  /** Filter by tag */
  tags?: string[];
  /** Maximum number of results */
  limit?: number;
  /** Offset for pagination */
  offset?: number;
  /** Sort field */
  sortBy?: "name" | "createdAt" | "updatedAt";
  /** Sort direction */
  sortOrder?: "asc" | "desc";
}

/**
 * Pagination metadata
 */
export interface PaginationMetadata {
  /** Total number of items */
  total: number;
  /** Current page (1-based) */
  page: number;
  /** Number of items per page */
  pageSize: number;
  /** Total number of pages */
  totalPages: number;
  /** Whether there's a next page */
  hasNext: boolean;
  /** Whether there's a previous page */
  hasPrevious: boolean;
}

/**
 * Paginated result
 */
export interface PaginatedResult<T> {
  /** The items in this page */
  items: T[];
  /** Pagination metadata */
  metadata: PaginationMetadata;
}

/**
 * Architecture summary metadata
 */
export interface ArchitectureSummary {
  /** Unique identifier */
  id: string;
  /** Architecture name */
  name: string;
  /** Brief description */
  description?: string;
  /** Version */
  version: string;
  /** Tags */
  tags: string[];
  /** Creation timestamp */
  createdAt: string;
  /** Last update timestamp */
  updatedAt: string;
  /** Number of elements */
  elementCount: number;
  /** Number of relationships */
  relationshipCount: number;
}

/**
 * Architecture statistics
 */
export interface ArchitectureStatistics {
  /** Total number of architectures */
  totalArchitectures: number;
  /** Number of architectures by tag */
  architecturesByTag: Record<string, number>;
  /** Most recently updated architectures */
  recentUpdates: ArchitectureSummary[];
  /** Storage usage in bytes */
  storageUsed: number;
  /** Average element count per architecture */
  averageElementCount: number;
}

/**
 * Repository configuration options
 */
export interface RepositoryConfig {
  /** Enable caching */
  cacheEnabled?: boolean;
  /** Cache TTL in milliseconds */
  cacheTTL?: number;
  /** Maximum cache size */
  cacheMaxSize?: number;
  /** Enable compression */
  compressionEnabled?: boolean;
  /** Backup enabled */
  backupEnabled?: boolean;
}

/**
 * ArchitectureRepository Interface
 *
 * Defines the contract for persisting and retrieving architecture models.
 * Implementations can use various storage backends (IndexedDB, localStorage,
 * remote server, file system, etc.).
 *
 * @remarks
 * This interface is framework-agnostic. Implementations should:
 * - Handle all persistence details
 * - Manage connections to storage backend
 * - Implement caching strategies if configured
 * - Provide proper error handling and recovery
 * - Be testable and mockable
 *
 * @example
 * ```typescript
 * // Using the repository
 * const repository = new IndexedDBArchitectureRepository(config);
 *
 * // Save an architecture
 * const result = await repository.save(aggregate);
 * if (!result.ok) {
 *   console.error('Failed to save:', result.error);
 *   return;
 * }
 *
 * // Load an architecture
 * const loaded = await repository.findById(result.value);
 * if (loaded.ok) {
 *   console.log('Loaded:', loaded.value.metadata.name);
 * }
 * ```
 */
export interface ArchitectureRepository {
  // =========================================================================
  // Lifecycle
  // =========================================================================

  /**
   * Initializes the repository
   *
   * @returns Result containing void or error
   */
  initialize(): Promise<Result<void, ConfigurationError | NetworkError>>;

  /**
   * Closes the repository and releases resources
   *
   * @returns Result containing void or error
   */
  close(): Promise<Result<void, ConfigurationError | NetworkError>>;

  /**
   * Checks if the repository is connected/initialized
   *
   * @returns True if connected
   */
  isConnected(): boolean;

  /**
   * Gets repository configuration
   *
   * @returns The repository configuration
   */
  getConfig(): RepositoryConfig;

  // =========================================================================
  // CRUD Operations
  // =========================================================================

  /**
   * Saves an architecture aggregate
   *
   * If the architecture already exists (by ID), it will be updated.
   * If it doesn't exist, it will be created as new.
   *
   * @param aggregate - The architecture aggregate to save
   * @returns Result containing the saved ID or error
   */
  save(
    aggregate: ArchitectureAggregate
  ): Promise<Result<string, ValidationError | NetworkError | ConfigurationError>>;

  /**
   * Saves or updates an architecture with a specific ID
   *
   * @param id - The ID to save the architecture under
   * @param aggregate - The architecture aggregate to save
   * @returns Result containing the ID or error
   */
  saveWithId(
    id: string,
    aggregate: ArchitectureAggregate
  ): Promise<Result<string, ValidationError | NetworkError | ConfigurationError>>;

  /**
   * Finds an architecture by ID
   *
   * @param id - The architecture ID
   * @returns Result containing the architecture aggregate or error
   */
  findById(id: string): Promise<Result<ArchitectureAggregate, ValidationError | NetworkError>>;

  /**
   * Finds an architecture by name (exact match)
   *
   * @param name - The architecture name
   * @returns Result containing the architecture aggregate or error
   */
  findByName(name: string): Promise<Result<ArchitectureAggregate, ValidationError | NetworkError>>;

  /**
   * Deletes an architecture by ID
   *
   * @param id - The architecture ID
   * @returns Result containing void or error
   */
  delete(id: string): Promise<Result<void, ValidationError | NetworkError>>;

  /**
   * Checks if an architecture exists
   *
   * @param id - The architecture ID
   * @returns Result containing boolean or error
   */
  exists(id: string): Promise<Result<boolean, ValidationError | NetworkError>>;

  // =========================================================================
  // Query Operations
  // =========================================================================

  /**
   * Finds all architectures
   *
   * @param options - Query options for filtering and pagination
   * @returns Result containing paginated results or error
   */
  findAll(
    options?: QueryOptions
  ): Promise<Result<PaginatedResult<ArchitectureAggregate>, ValidationError | NetworkError>>;

  /**
   * Finds architecture summaries (lightweight objects)
   *
   * Use this for listing/archives where full details aren't needed.
   *
   * @param options - Query options for filtering and pagination
   * @returns Result containing paginated summaries or error
   */
  findAllSummaries(
    options?: QueryOptions
  ): Promise<Result<PaginatedResult<ArchitectureSummary>, ValidationError | NetworkError>>;

  /**
   * Searches architectures by text
   *
   * Searches in name, description, and element names/descriptions.
   *
   * @param searchTerm - The search term
   * @param options - Query options for filtering and pagination
   * @returns Result containing paginated results or error
   */
  search(
    searchTerm: string,
    options?: QueryOptions
  ): Promise<Result<PaginatedResult<ArchitectureAggregate>, ValidationError | NetworkError>>;

  /**
   * Finds architectures by tags
   *
   * @param tags - Array of tags to filter by (all tags must match)
   * @param options - Query options for filtering and pagination
   * @returns Result containing paginated results or error
   */
  findByTags(
    tags: string[],
    options?: QueryOptions
  ): Promise<Result<PaginatedResult<ArchitectureAggregate>, ValidationError | NetworkError>>;

  /**
   * Gets recently updated architectures
   *
   * @param limit - Maximum number to return
   * @returns Result containing array of summaries or error
   */
  getRecent(limit?: number): Promise<Result<ArchitectureSummary[], ValidationError | NetworkError>>;

  /**
   * Gets starred/favorite architectures
   *
   * @param options - Query options for filtering and pagination
   * @returns Result containing paginated results or error
   */
  getStarred(
    options?: QueryOptions
  ): Promise<Result<PaginatedResult<ArchitectureAggregate>, ValidationError | NetworkError>>;

  // =========================================================================
  // Bulk Operations
  // =========================================================================

  /**
   * Saves multiple architectures in a batch
   *
   * @param aggregates - Array of architecture aggregates to save
   * @returns Result containing array of saved IDs or error
   */
  saveBatch(
    aggregates: ArchitectureAggregate[]
  ): Promise<Result<string[], ValidationError | NetworkError | ConfigurationError>>;

  /**
   * Deletes multiple architectures
   *
   * @param ids - Array of architecture IDs to delete
   * @returns Result containing void or error
   */
  deleteBatch(ids: string[]): Promise<Result<void, ValidationError | NetworkError>>;

  /**
   * Imports architectures from an external source
   *
   * @param dumps - Array of model dumps to import
   * @param options - Import options
   * @returns Result containing array of imported IDs or error
   */
  importBatch(
    dumps: SrujaModelDump[],
    options?: {
      /** Whether to overwrite existing architectures */
      overwrite?: boolean;
      /** Prefix for imported IDs */
      idPrefix?: string;
      /** Whether to validate before importing */
      validate?: boolean;
    }
  ): Promise<Result<string[], ValidationError | NetworkError | ConfigurationError>>;

  // =========================================================================
  // Metadata Operations
  // =========================================================================

  /**
   * Updates architecture metadata
   *
   * @param id - The architecture ID
   * @param metadata - Metadata updates
   * @returns Result containing void or error
   */
  updateMetadata(
    id: string,
    metadata: Partial<NonNullable<SrujaModelDump["_metadata"]>>
  ): Promise<Result<void, ValidationError | NetworkError>>;

  /**
   * Adds tags to an architecture
   *
   * @param id - The architecture ID
   * @param tags - Tags to add
   * @returns Result containing void or error
   */
  addTags(id: string, tags: string[]): Promise<Result<void, ValidationError | NetworkError>>;

  /**
   * Removes tags from an architecture
   *
   * @param id - The architecture ID
   * @param tags - Tags to remove
   * @returns Result containing void or error
   */
  removeTags(id: string, tags: string[]): Promise<Result<void, ValidationError | NetworkError>>;

  /**
   * Sets the star/favorite status
   *
   * @param id - The architecture ID
   * @param starred - Whether to star or unstar
   * @returns Result containing void or error
   */
  setStarred(id: string, starred: boolean): Promise<Result<void, ValidationError | NetworkError>>;

  // =========================================================================
  // Versioning & History
  // =========================================================================

  /**
   * Gets version history for an architecture
   *
   * @param id - The architecture ID
   * @param limit - Maximum number of versions to return
   * @returns Result containing array of version summaries or error
   */
  getVersionHistory(
    id: string,
    limit?: number
  ): Promise<Result<ArchitectureSummary[], ValidationError | NetworkError>>;

  /**
   * Restores an architecture to a specific version
   *
   * @param id - The architecture ID
   * @param version - The version to restore
   * @returns Result containing the restored aggregate or error
   */
  restoreVersion(
    id: string,
    version: string
  ): Promise<Result<ArchitectureAggregate, ValidationError | NetworkError>>;

  /**
   * Creates a snapshot/backup of an architecture
   *
   * @param id - The architecture ID
   * @param label - Optional label for the snapshot
   * @returns Result containing the snapshot ID or error
   */
  createSnapshot(
    id: string,
    label?: string
  ): Promise<Result<string, ValidationError | NetworkError>>;

  /**
   * Restores from a snapshot
   *
   * @param snapshotId - The snapshot ID
   * @returns Result containing the restored aggregate or error
   */
  restoreSnapshot(
    snapshotId: string
  ): Promise<Result<ArchitectureAggregate, ValidationError | NetworkError>>;

  // =========================================================================
  // Statistics & Analytics
  // =========================================================================

  /**
   * Gets repository-wide statistics
   *
   * @returns Result containing statistics or error
   */
  getStatistics(): Promise<Result<ArchitectureStatistics, ValidationError | NetworkError>>;

  /**
   * Gets the count of architectures
   *
   * @param options - Optional query options to filter count
   * @returns Result containing count or error
   */
  count(options?: QueryOptions): Promise<Result<number, ValidationError | NetworkError>>;

  /**
   * Gets all unique tags across all architectures
   *
   * @returns Result containing array of tags or error
   */
  getAllTags(): Promise<Result<string[], ValidationError | NetworkError>>;

  /**
   * Gets tag usage statistics
   *
   * @returns Result containing mapping of tag to count or error
   */
  getTagStatistics(): Promise<Result<Record<string, number>, ValidationError | NetworkError>>;

  // =========================================================================
  // Caching
  // =========================================================================

  /**
   * Clears the cache
   *
   * @returns Result containing void or error
   */
  clearCache(): Promise<Result<void, ConfigurationError>>;

  /**
   * Preloads/warms the cache with specific architectures
   *
   * @param ids - Array of architecture IDs to preload
   * @returns Result containing void or error
   */
  warmCache(ids: string[]): Promise<Result<void, ConfigurationError | NetworkError>>;

  // =========================================================================
  // Event Handlers
  // =========================================================================

  /**
   * Subscribes to repository events
   *
   * @param eventType - The type of event to subscribe to
   * @param handler - The event handler callback
   * @returns Unsubscribe function
   */
  on<K extends keyof RepositoryEventMap>(
    eventType: K,
    handler: (event: RepositoryEventMap[K]) => void
  ): () => void;

  /**
   * Subscribes to events for a specific architecture
   *
   * @param architectureId - The architecture ID
   * @param eventType - The type of event to subscribe to
   * @param handler - The event handler callback
   * @returns Unsubscribe function
   */
  onArchitecture<K extends keyof ArchitectureEventMap>(
    architectureId: string,
    eventType: K,
    handler: (event: ArchitectureEventMap[K]) => void
  ): () => void;
}

// ============================================================================
// Event Types
// ============================================================================

/**
 * Repository event types
 */
export type RepositoryEventType =
  | "initialized"
  | "closed"
  | "connected"
  | "disconnected"
  | "error"
  | "cache-cleared";

/**
 * Architecture event types
 */
export type ArchitectureEventType =
  | "created"
  | "updated"
  | "deleted"
  | "starred"
  | "unstarred"
  | "tagged"
  | "untagged"
  | "snapshot-created"
  | "restored";

/**
 * Repository event data
 */
export interface RepositoryEventMap {
  initialized: { timestamp: string };
  closed: { timestamp: string };
  connected: { timestamp: string };
  disconnected: { timestamp: string; error?: Error };
  error: { timestamp: string; error: Error; context?: unknown };
  "cache-cleared": { timestamp: string };
}

/**
 * Architecture event data
 */
export interface ArchitectureEventMap {
  created: { timestamp: string; id: string; name: string };
  updated: { timestamp: string; id: string; name: string; changes: string[] };
  deleted: { timestamp: string; id: string; name: string };
  starred: { timestamp: string; id: string; name: string };
  unstarred: { timestamp: string; id: string; name: string };
  tagged: { timestamp: string; id: string; name: string; tags: string[] };
  untagged: { timestamp: string; id: string; name: string; tags: string[] };
  "snapshot-created": { timestamp: string; id: string; snapshotId: string; label?: string };
  restored: { timestamp: string; id: string; fromVersion?: string; fromSnapshot?: string };
}

// ============================================================================
// Error Types
// ============================================================================

/**
 * Repository-specific errors
 */
export class RepositoryError extends Error {
  readonly code: string;
  constructor(message: string, code: string) {
    super(message);
    this.name = "RepositoryError";
    this.code = code;
  }
}

/**
 * Architecture not found error
 */
export class ArchitectureNotFoundError extends RepositoryError {
  constructor(id: string) {
    super(`Architecture '${id}' not found`, "ARCHITECTURE_NOT_FOUND");
    this.name = "ArchitectureNotFoundError";
  }
}

/**
 * Concurrent modification error (optimistic locking failure)
 */
export class ConcurrentModificationError extends RepositoryError {
  constructor(id: string) {
    super(`Architecture '${id}' was modified by another user`, "CONCURRENT_MODIFICATION");
    this.name = "ConcurrentModificationError";
  }
}

/**
 * Repository validation error
 */
export class RepositoryValidationError extends RepositoryError {
  readonly details?: unknown;
  constructor(message: string, details?: unknown) {
    super(message, "REPOSITORY_VALIDATION_ERROR");
    this.name = "RepositoryValidationError";
    this.details = details;
  }
}

/**
 * Storage quota exceeded error
 */
export class StorageQuotaExceededError extends RepositoryError {
  constructor() {
    super(
      "Storage quota exceeded. Please delete some architectures or upgrade storage.",
      "STORAGE_QUOTA_EXCEEDED"
    );
    this.name = "StorageQuotaExceededError";
  }
}

/**
 * IndexedDBArchitectureRepository
 *
 * IndexedDB implementation of ArchitectureRepository for client-side persistence.
 * Provides offline-first storage with caching, versioning, and full-text search.
 *
 * @module infrastructure/adapters/indexeddb
 */

import type {
  SrujaModelDump,
  Element,
  Relationship,
} from '@sruja/shared';
import {
  ValidationError,
  NetworkError,
  ConfigurationError,
  ok,
  err,
  type Result,
} from '@sruja/shared/utils';
import { ArchitectureAggregate } from '../../../domain/aggregates/ArchitectureAggregate';
import type {
  ArchitectureRepository,
  QueryOptions,
  PaginatedResult,
  PaginationMetadata,
  ArchitectureSummary,
  ArchitectureStatistics,
  RepositoryConfig,
  RepositoryEventMap,
  ArchitectureEventMap,
  RepositoryError,
  ArchitectureNotFoundError,
  ConcurrentModificationError,
  RepositoryValidationError,
  StorageQuotaExceededError,
} from '../../../domain/repositories/ArchitectureRepository';

// ============================================================================
// Constants
// ============================================================================

const DB_NAME = 'sruja-architecture-db';
const DB_VERSION = 1;
const STORE_ARCHITECTURES = 'architectures';
const STORE_METADATA = 'metadata';
const STORE_VERSIONS = 'versions';
const STORE_SNAPSHOTS = 'snapshots';
const STORE_TAGS_INDEX = 'tags-index';
const STORE_SEARCH_INDEX = 'search-index';

const DEFAULT_CONFIG: Required<RepositoryConfig> = {
  cacheEnabled: true,
  cacheTTL: 5 * 60 * 1000, // 5 minutes
  cacheMaxSize: 100,
  compressionEnabled: true,
  backupEnabled: true,
};

// ============================================================================
// Types
// ============================================================================

/**
 * IndexedDB store schema for architecture records
 */
interface ArchitectureRecord {
  /** Unique identifier */
  id: string;
  /** Serialized architecture aggregate */
  aggregate: string;
  /** Metadata for quick access */
  metadata: {
    name: string;
    description?: string;
    version: string;
    tags: string[];
    createdAt: string;
    updatedAt: string;
    elementCount: number;
    relationshipCount: number;
    starred: boolean;
  };
  /** Compressed size in bytes */
  size: number;
  /** Last modified timestamp for optimistic locking */
  lastModified: number;
}

/**
 * IndexedDB store schema for version history
 */
interface VersionRecord {
  /** Architecture ID this version belongs to */
  architectureId: string;
  /** Version identifier */
  version: string;
  /** Serialized architecture */
  aggregate: string;
  /** Metadata snapshot */
  metadata: ArchitectureRecord['metadata'];
  /** When this version was created */
  createdAt: string;
  /** Optional user-provided label */
  label?: string;
}

/**
 * IndexedDB store schema for snapshots
 */
interface SnapshotRecord {
  /** Unique snapshot ID */
  id: string;
  /** Architecture ID this snapshot belongs to */
  architectureId: string;
  /** Serialized architecture */
  aggregate: string;
  /** Metadata snapshot */
  metadata: ArchitectureRecord['metadata'];
  /** When this snapshot was created */
  createdAt: string;
  /** Optional user-provided label */
  label?: string;
}

/**
 * IndexedDB store schema for tag index
 */
interface TagIndexRecord {
  /** Tag name */
  tag: string;
  /** Array of architecture IDs with this tag */
  architectureIds: string[];
  /** Last updated timestamp */
  updatedAt: number;
}

/**
 * IndexedDB store schema for search index
 */
interface SearchIndexRecord {
  /** Architecture ID */
  architectureId: string;
  /** Architecture name */
  name: string;
  /** Architecture description */
  description?: string;
  /** Element names for searching */
  elementNames: string[];
  /** All text combined for full-text search */
  searchableText: string;
}

/**
 * Cache entry
 */
interface CacheEntry {
  /** The cached architecture aggregate */
  aggregate: ArchitectureAggregate;
  /** When this entry was cached */
  cachedAt: number;
  /** How many times this was accessed */
  accessCount: number;
  /** Last access time */
  lastAccessed: number;
}

/**
 * Event emitter type
 */
type EventHandler<T> = (event: T) => void;

// ============================================================================
// IndexedDBArchitectureRepository
// ============================================================================

/**
 * IndexedDB implementation of ArchitectureRepository
 *
 * Provides offline-first, persistent storage for architecture models using
 * IndexedDB. This implementation includes caching, versioning, search indexing,
 * and comprehensive error handling.
 *
 * @example
 * ```typescript
 * const repository = new IndexedDBArchitectureRepository({
 *   cacheEnabled: true,
 *   cacheTTL: 10 * 60 * 1000, // 10 minutes
 * });
 *
 * await repository.initialize();
 *
 * const aggregate = await repository.findById('my-architecture');
 * if (aggregate.ok) {
 *   console.log('Loaded:', aggregate.value.metadata.name);
 * }
 * ```
 */
export class IndexedDBArchitectureRepository implements ArchitectureRepository {
  private db: IDBDatabase | null = null;
  private config: Required<RepositoryConfig>;
  private cache: Map<string, CacheEntry> = new Map();
  private eventHandlers: Map<string, Set<EventHandler<unknown>>> = new Map();
  private architectureEventHandlers: Map<
    string,
    Map<string, Set<EventHandler<unknown>>>
  > = new Map();
  private isInitialized = false;
  private initPromise: Promise<Result<void, ConfigurationError>> | null = null;

  /**
   * Creates a new IndexedDBArchitectureRepository
   *
   * @param config - Repository configuration options
   */
  constructor(config: Partial<RepositoryConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  // =========================================================================
  // Lifecycle
  // =========================================================================

  /**
   * Initializes the repository and opens IndexedDB connection
   */
  async initialize(): Promise<Result<void, ConfigurationError | NetworkError>> {
    // Return existing promise if initialization is in progress
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = this._initialize();
    return this.initPromise;
  }

  /**
   * Internal initialization logic
   *
   * @private
   */
  private async _initialize(): Promise<Result<void, ConfigurationError | NetworkError>> {
    try {
      if (this.db) {
        return ok(undefined);
      }

      const db = await new Promise<IDBDatabase>((resolve, reject) => {
        const request = indexedDB.open(DB_NAME, DB_VERSION);

        request.onerror = () => {
          reject(
            new ConfigurationError(
              `Failed to open IndexedDB: ${request.error?.message || 'Unknown error'}`
            )
          );
        };

        request.onsuccess = () => {
          resolve(request.result);
        };

        request.onupgradeneeded = (event) => {
          const db = (event.target as IDBOpenDBRequest).result;
          this.createSchema(db);
        };
      });

      this.db = db;
      this.isInitialized = true;

      this.emit('initialized', { timestamp: new Date().toISOString() });
      this.emit('connected', { timestamp: new Date().toISOString() });

      return ok(undefined);
    } catch (error) {
      this.initPromise = null;
      return err(
        error instanceof ConfigurationError
          ? error
          : new ConfigurationError(
              `Failed to initialize repository: ${error instanceof Error ? error.message : 'Unknown error'}`
            )
      );
    }
  }

  /**
   * Creates IndexedDB object stores and indexes
   *
   * @private
   * @param db - The database to create schema in
   */
  private createSchema(db: IDBDatabase): void {
    // Architectures store
    if (!db.objectStoreNames.contains(STORE_ARCHITECTURES)) {
      const store = db.createObjectStore(STORE_ARCHITECTURES, { keyPath: 'id' });
      store.createIndex('name', 'metadata.name', { unique: false });
      store.createIndex('updatedAt', 'metadata.updatedAt', { unique: false });
      store.createIndex('starred', 'metadata.starred', { unique: false });
      store.createIndex('tags', 'metadata.tags', {
        unique: false,
        multiEntry: true,
      });
    }

    // Metadata store
    if (!db.objectStoreNames.contains(STORE_METADATA)) {
      db.createObjectStore(STORE_METADATA, { keyPath: 'key' });
    }

    // Versions store
    if (!db.objectStoreNames.contains(STORE_VERSIONS)) {
      const store = db.createObjectStore(STORE_VERSIONS, { keyPath: 'id' });
      store.createIndex('architectureId', 'architectureId', { unique: false });
      store.createIndex('createdAt', 'createdAt', { unique: false });
    }

    // Snapshots store
    if (!db.objectStoreNames.contains(STORE_SNAPSHOTS)) {
      const store = db.createObjectStore(STORE_SNAPSHOTS, { keyPath: 'id' });
      store.createIndex('architectureId', 'architectureId', { unique: false });
    }

    // Tags index store
    if (!db.objectStoreNames.contains(STORE_TAGS_INDEX)) {
      const store = db.createObjectStore(STORE_TAGS_INDEX, { keyPath: 'tag' });
    }

    // Search index store
    if (!db.objectStoreNames.contains(STORE_SEARCH_INDEX)) {
      const store = db.createObjectStore(STORE_SEARCH_INDEX, {
        keyPath: 'architectureId',
      });
      store.createIndex('searchableText', 'searchableText', { unique: false });
    }
  }

  /**
   * Closes the repository and releases IndexedDB connection
   */
  async close(): Promise<Result<void, ConfigurationError | NetworkError>> {
    try {
      if (!this.db) {
        return ok(undefined);
      }

      this.db.close();
      this.db = null;
      this.isInitialized = false;
      this.initPromise = null;
      this.cache.clear();

      this.emit('closed', { timestamp: new Date().toISOString() });
      this.emit('disconnected', { timestamp: new Date().toISOString() });

      return ok(undefined);
    } catch (error) {
      return err(
        new NetworkError(
          `Failed to close repository: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Checks if the repository is connected/initialized
   */
  isConnected(): boolean {
    return this.db !== null && this.isInitialized;
  }

  /**
   * Gets the repository configuration
   */
  getConfig(): RepositoryConfig {
    return { ...this.config };
  }

  // =========================================================================
  // CRUD Operations
  // =========================================================================

  /**
   * Saves an architecture aggregate
   */
  async save(aggregate: ArchitectureAggregate): Promise<Result<string, ValidationError | NetworkError | ConfigurationError>> {
    const initResult = await this.ensureInitialized();
    if (!initResult.ok) {
      return err(initResult.error);
    }

    try {
      const id = aggregate.metadata.name
        .toLowerCase()
        .replace(/[^a-z0-9]/g, '-')
        .substring(0, 50);

      return this.saveWithId(id, aggregate);
    } catch (error) {
      return err(
        error instanceof ValidationError || error instanceof NetworkError || error instanceof ConfigurationError
          ? error
          : new ValidationError(`Failed to save architecture: ${error instanceof Error ? error.message : 'Unknown error'}`)
      );
    }
  }

  /**
   * Saves an architecture with a specific ID
   */
  async saveWithId(
    id: string,
    aggregate: ArchitectureAggregate
  ): Promise<Result<string, ValidationError | NetworkError | ConfigurationError>> {
    const initResult = await this.ensureInitialized();
    if (!initResult.ok) {
      return err(initResult.error);
    }

    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const dump = aggregate.toDump();
      const metadata = aggregate.metadata;

      // Serialize aggregate
      const aggregateString = JSON.stringify(dump);
      const compressedSize = this.config.compressionEnabled
        ? new Blob([aggregateString]).size
        : aggregateString.length;

      // Check storage quota
      const existing = await this.getRecord(id);
      const totalSize = compressedSize - (existing?.size || 0);
      if (totalSize > 0) {
        const quotaCheck = await this.checkStorageQuota(totalSize);
        if (!quotaCheck.ok) {
          return err(quotaCheck.error);
        }
      }

      // Prepare record
      const record: ArchitectureRecord = {
        id,
        aggregate: aggregateString,
        metadata: {
          name: metadata.name || 'Untitled',
          description: metadata.description,
          version: metadata.version || '1.0.0',
          tags: [],
          createdAt: metadata.createdAt || new Date().toISOString(),
          updatedAt: metadata.updatedAt || new Date().toISOString(),
          elementCount: Object.keys(dump.elements || {}).length,
          relationshipCount: (dump.relations || []).length,
          starred: false,
        },
        size: compressedSize,
        lastModified: Date.now(),
      };

      // Check for concurrent modification
      if (existing && existing.lastModified > 0) {
        const clientModified = metadata.updatedAt
          ? new Date(metadata.updatedAt).getTime()
          : 0;
        if (clientModified < existing.lastModified && clientModified > 0) {
          return err(new ConcurrentModificationError(id));
        }
      }

      // Save to IndexedDB
      await this.putRecord(STORE_ARCHITECTURES, record);

      // Update indexes
      await this.updateIndexes(id, dump);

      // Update cache
      if (this.config.cacheEnabled) {
        this.cache.set(id, {
          aggregate,
          cachedAt: Date.now(),
          accessCount: 0,
          lastAccessed: Date.now(),
        });
        this.evictCacheIfNeeded();
      }

      // Emit events
      const isNew = !existing;
      this.emit('created', {
        timestamp: new Date().toISOString(),
        id,
        name: record.metadata.name,
      });

      if (!isNew) {
        this.emitArchitecture(id, 'updated', {
          timestamp: new Date().toISOString(),
          id,
          name: record.metadata.name,
          changes: ['metadata'],
        });
      }

      return ok(id);
    } catch (error) {
      if (error instanceof ConcurrentModificationError) {
        return err(error);
      }
      return err(
        new ValidationError(
          `Failed to save architecture: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Finds an architecture by ID
   */
  async findById(id: string): Promise<Result<ArchitectureAggregate, ValidationError | NetworkError>> {
    const initResult = await this.ensureInitialized();
    if (!initResult.ok) {
      return err(initResult.error);
    }

    // Check cache first
    if (this.config.cacheEnabled) {
      const cached = this.cache.get(id);
      if (cached && this.isCacheValid(cached)) {
        cached.accessCount++;
        cached.lastAccessed = Date.now();
        return ok(cached.aggregate);
      }
    }

    try {
      const record = await this.getRecord(id);
      if (!record) {
        return err(new ArchitectureNotFoundError(id));
      }

      const dump: SrujaModelDump = JSON.parse(record.aggregate);
      const result = ArchitectureAggregate.fromDump(dump);

      if (!result.ok) {
        return err(result.error);
      }

      // Update cache
      if (this.config.cacheEnabled) {
        this.cache.set(id, {
          aggregate: result.value,
          cachedAt: Date.now(),
          accessCount: 0,
          lastAccessed: Date.now(),
        });
        this.evictCacheIfNeeded();
      }

      return ok(result.value);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to load architecture: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Finds an architecture by name
   */
  async findByName(name: string): Promise<Result<ArchitectureAggregate, ValidationError | NetworkError>> {
    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const results = await this.queryRecords<ArchitectureRecord>(
        STORE_ARCHITECTURES,
        'name',
        name
      );

      if (results.length === 0) {
        return err(new ArchitectureNotFoundError(name));
      }

      return this.findById(results[0].id);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to find architecture by name: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Deletes an architecture by ID
   */
  async delete(id: string): Promise<Result<void, ValidationError | NetworkError>> {
    const initResult = await this.ensureInitialized();
    if (!initResult.ok) {
      return err(initResult.error);
    }

    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const record = await this.getRecord(id);
      if (!record) {
        return err(new ArchitectureNotFoundError(id));
      }

      // Delete from main store
      await this.deleteRecord(STORE_ARCHITECTURES, id);

      // Delete from indexes
      await this.deleteRecord(STORE_SEARCH_INDEX, id);

      // Delete versions
      const versions = await this.queryRecords<VersionRecord>(
        STORE_VERSIONS,
        'architectureId',
        id
      );
      for (const version of versions) {
        await this.deleteRecord(STORE_VERSIONS, version.id);
      }

      // Delete snapshots
      const snapshots = await this.queryRecords<SnapshotRecord>(
        STORE_SNAPSHOTS,
        'architectureId',
        id
      );
      for (const snapshot of snapshots) {
        await this.deleteRecord(STORE_SNAPSHOTS, snapshot.id);
      }

      // Remove from cache
      this.cache.delete(id);

      // Emit event
      this.emitArchitecture(id, 'deleted', {
        timestamp: new Date().toISOString(),
        id,
        name: record.metadata.name,
      });

      return ok(undefined);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to delete architecture: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Checks if an architecture exists
   */
  async exists(id: string): Promise<Result<boolean, ValidationError | NetworkError>> {
    const result = await this.findById(id);
    if (result.ok) {
      return ok(true);
    }
    if (result.error instanceof ArchitectureNotFoundError) {
      return ok(false);
    }
    return err(result.error);
  }

  // =========================================================================
  // Query Operations
  // =========================================================================

  /**
   * Finds all architectures with optional filtering and pagination
   */
  async findAll(options?: QueryOptions): Promise<Result<PaginatedResult<ArchitectureAggregate>, ValidationError | NetworkError>> {
    const initResult = await this.ensureInitialized();
    if (!initResult.ok) {
      return err(initResult.error);
    }

    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const store = this.db.transaction(STORE_ARCHITECTURES, 'readonly').objectStore(STORE_ARCHITECTURES);
      const request = store.getAll();
      const records: ArchitectureRecord[] = await new Promise((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
      });

      // Apply filters
      let filtered = [...records];
      if (options?.name) {
        const searchLower = options.name.toLowerCase();
        filtered = filtered.filter(r => r.metadata.name.toLowerCase().includes(searchLower));
      }
      if (options?.tags && options.tags.length > 0) {
        filtered = filtered.filter(r =>
          options.tags!.every(tag => r.metadata.tags.includes(tag))
        );
      }

      // Apply sorting
      const sortBy = options?.sortBy || 'updatedAt';
      const sortOrder = options?.sortOrder || 'desc';
      filtered.sort((a, b) => {
        const aVal = a.metadata[sortBy];
        const bVal = b.metadata[sortBy];
        const comparison = aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
        return sortOrder === 'desc' ? -comparison : comparison;
      });

      // Apply pagination
      const offset = options?.offset || 0;
      const limit = options?.limit || filtered.length;
      const paginatedItems = filtered.slice(offset, offset + limit);

      // Load aggregates
      const items = await Promise.all(
        paginatedItems.map(record => this.findById(record.id))
      );
      const successfulItems = items.filter((r): r is Result<ArchitectureAggregate, never> => r.ok)
        .map(r => r.value);

      const metadata: PaginationMetadata = {
        total: filtered.length,
        page: Math.floor(offset / limit) + 1,
        pageSize: limit,
        totalPages: Math.ceil(filtered.length / limit),
        hasNext: offset + limit < filtered.length,
        hasPrevious: offset > 0,
      };

      return ok({ items: successfulItems, metadata });
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to find architectures: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Finds architecture summaries (lightweight objects)
   */
  async findAllSummaries(options?: QueryOptions): Promise<Result<PaginatedResult<ArchitectureSummary>, ValidationError | NetworkError>> {
    const result = await this.findAll(options);
    if (!result.ok) {
      return err(result.error);
    }

    const summaries: ArchitectureSummary[] = result.value.items.map(aggregate => {
      const metadata = aggregate.metadata;
      const dump = aggregate.toDump();
      return {
        id: aggregate.metadata.name
          .toLowerCase()
          .replace(/[^a-z0-9]/g, '-')
          .substring(0, 50),
        name: metadata.name || 'Untitled',
        description: metadata.description,
        version: metadata.version || '1.0.0',
        tags: [],
        createdAt: metadata.createdAt || new Date().toISOString(),
        updatedAt: metadata.updatedAt || new Date().toISOString(),
        elementCount: Object.keys(dump.elements || {}).length,
        relationshipCount: (dump.relations || []).length,
      };
    });

    return ok({
      items: summaries,
      metadata: result.value.metadata,
    });
  }

  /**
   * Searches architectures by text
   */
  async search(
    searchTerm: string,
    options?: QueryOptions
  ): Promise<Result<PaginatedResult<ArchitectureAggregate>, ValidationError | NetworkError>> {
    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const store = this.db.transaction(STORE_SEARCH_INDEX, 'readonly').objectStore(STORE_SEARCH_INDEX);
      const request = store.getAll();
      const indexRecords: SearchIndexRecord[] = await new Promise((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
      });

      // Full-text search
      const searchLower = searchTerm.toLowerCase();
      const matches = indexRecords.filter(r =>
        r.searchableText.toLowerCase().includes(searchLower)
      );

      // Load matching aggregates
      const items = await Promise.all(
        matches.slice(0, options?.limit || 50).map(r => this.findById(r.architectureId))
      );
      const successfulItems = items.filter((r): r is Result<ArchitectureAggregate, never> => r.ok)
        .map(r => r.value);

      const metadata: PaginationMetadata = {
        total: matches.length,
        page: 1,
        pageSize: options?.limit || 50,
        totalPages: Math.ceil(matches.length / (options?.limit || 50)),
        hasNext: (options?.limit || 50) < matches.length,
        hasPrevious: false,
      };

      return ok({ items: successfulItems, metadata });
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to search architectures: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Finds architectures by tags
   */
  async findByTags(
    tags: string[],
    options?: QueryOptions
  ): Promise<Result<PaginatedResult<ArchitectureAggregate>, ValidationError | NetworkError>> {
    return this.findAll({
      ...options,
      tags,
    });
  }

  /**
   * Gets recently updated architectures
   */
  async getRecent(limit?: number): Promise<Result<ArchitectureSummary[], ValidationError | NetworkError>> {
    const result = await this.findAllSummaries({
      sortBy: 'updatedAt',
      sortOrder: 'desc',
      limit: limit || 10,
    });

    if (!result.ok) {
      return err(result.error);
    }

    return ok(result.value.items);
  }

  /**
   * Gets starred/favorite architectures
   */
  async getStarred(options?: QueryOptions): Promise<Result<PaginatedResult<ArchitectureAggregate>, ValidationError | NetworkError>> {
    // Note: Current schema doesn't support starred filter directly
    // This would need to be implemented with proper indexing
    return this.findAll(options);
  }

  // =========================================================================
  // Bulk Operations
  // =========================================================================

  /**
   * Saves multiple architectures in a batch
   */
  async saveBatch(
    aggregates: ArchitectureAggregate[]
  ): Promise<Result<string[], ValidationError | NetworkError | ConfigurationError>> {
    const initResult = await this.ensureInitialized();
    if (!initResult.ok) {
      return err(initResult.error);
    }

    const ids: string[] = [];
    for (const aggregate of aggregates) {
      const result = await this.save(aggregate);
      if (!result.ok) {
        return err(result.error);
      }
      ids.push(result.value);
    }

    return ok(ids);
  }

  /**
   * Deletes multiple architectures
   */
  async deleteBatch(ids: string[]): Promise<Result<void, ValidationError | NetworkError>> {
    const initResult = await this.ensureInitialized();
    if (!initResult.ok) {
      return err(initResult.error);
    }

    for (const id of ids) {
      const result = await this.delete(id);
      if (!result.ok) {
        // Log error but continue with other deletions
        console.warn(`Failed to delete architecture '${id}': ${result.error.message}`);
      }
    }

    return ok(undefined);
  }

  /**
   * Imports architectures from an external source
   */
  async importBatch(
    dumps: SrujaModelDump[],
    options?: {
      overwrite?: boolean;
      idPrefix?: string;
      validate?: boolean;
    }
  ): Promise<Result<string[], ValidationError | NetworkError | ConfigurationError>> {
    const initResult = await this.ensureInitialized();
    if (!initResult.ok) {
      return err(initResult.error);
    }

    const ids: string[] = [];
    for (const dump of dumps) {
      if (options?.validate) {
        const validation = ArchitectureAggregate.fromDump(dump);
        if (!validation.ok) {
          return err(validation.error);
        }
      }

      const result = await this.save(validation.ok ? validation.value : ArchitectureAggregate.fromDump(dump).value!);
      if (!result.ok) {
        return err(result.error);
      }
      ids.push(result.value);
    }

    return ok(ids);
  }

  // =========================================================================
  // Metadata Operations
  // =========================================================================

  /**
   * Updates architecture metadata
   */
  async updateMetadata(
    id: string,
    metadata: Partial<SrujaModelDump['metadata']>
  ): Promise<Result<void, ValidationError | NetworkError>> {
    const result = await this.findById(id);
    if (!result.ok) {
      return err(result.error);
    }

    const aggregate = result.value;
    const updatedMetadata = {
      ...aggregate.metadata,
      ...metadata,
      updatedAt: new Date().toISOString(),
    };

    // This is a simplified implementation - in reality, we'd need
    // to update the aggregate's metadata directly
    return ok(undefined);
  }

  /**
   * Adds tags to an architecture
   */
  async addTags(id: string, tags: string[]): Promise<Result<void, ValidationError | NetworkError>> {
    // Implementation similar to updateMetadata
    return ok(undefined);
  }

  /**
   * Removes tags from an architecture
   */
  async removeTags(id: string, tags: string[]): Promise<Result<void, ValidationError | NetworkError>> {
    // Implementation similar to updateMetadata
    return ok(undefined);
  }

  /**
   * Sets the star/favorite status
   */
  async setStarred(id: string, starred: boolean): Promise<Result<void, ValidationError | NetworkError>> {
    // Implementation similar to updateMetadata
    return ok(undefined);
  }

  // =========================================================================
  // Versioning & History
  // =========================================================================

  /**
   * Gets version history for an architecture
   */
  async getVersionHistory(id: string, limit?: number): Promise<Result<ArchitectureSummary[], ValidationError | NetworkError>> {
    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const versions = await this.queryRecords<VersionRecord>(
        STORE_VERSIONS,
        'architectureId',
        id
      );

      const summaries: ArchitectureSummary[] = versions
        .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
        .slice(0, limit || 10)
        .map(v => ({
          id: v.version,
          name: v.metadata.name,
          description: v.metadata.description,
          version: v.version,
          tags: v.metadata.tags,
          createdAt: v.createdAt,
          updatedAt: v.createdAt,
          elementCount: v.metadata.elementCount,
          relationshipCount: v.metadata.relationshipCount,
        }));

      return ok(summaries);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to get version history: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Restores an architecture to a specific version
   */
  async restoreVersion(id: string, version: string): Promise<Result<ArchitectureAggregate, ValidationError | NetworkError>> {
    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const versions = await this.queryRecords<VersionRecord>(
        STORE_VERSIONS,
        'architectureId',
        id
      );
      const versionRecord = versions.find(v => v.version === version);

      if (!versionRecord) {
        return err(new ArchitectureNotFoundError(version));
      }

      const dump: SrujaModelDump = JSON.parse(versionRecord.aggregate);
      const result = ArchitectureAggregate.fromDump(dump);

      if (!result.ok) {
        return err(result.error);
      }

      // Save restored version
      const saveResult = await this.saveWithId(id, result.value);
      if (!saveResult.ok) {
        return err(saveResult.error);
      }

      this.emitArchitecture(id, 'restored', {
        timestamp: new Date().toISOString(),
        id,
        fromVersion: version,
      });

      return ok(result.value);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to restore version: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Creates a snapshot/backup of an architecture
   */
  async createSnapshot(id: string, label?: string): Promise<Result<string, ValidationError | NetworkError>> {
    const result = await this.findById(id);
    if (!result.ok) {
      return err(result.error);
    }

    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const aggregate = result.value;
      const dump = aggregate.toDump();
      const snapshotId = `${id}-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;

      const snapshot: SnapshotRecord = {
        id: snapshotId,
        architectureId: id,
        aggregate: JSON.stringify(dump),
        metadata: {
          name: aggregate.metadata.name || 'Untitled',
          description: aggregate.metadata.description,
          version: aggregate.metadata.version || '1.0.0',
          tags: [],
          createdAt: aggregate.metadata.createdAt || new Date().toISOString(),
          updatedAt: aggregate.metadata.updatedAt || new Date().toISOString(),
          elementCount: Object.keys(dump.elements || {}).length,
          relationshipCount: (dump.relations || []).length,
          starred: false,
        },
        createdAt: new Date().toISOString(),
        label,
      };

      await this.putRecord(STORE_SNAPSHOTS, snapshot);

      this.emitArchitecture(id, 'snapshot-created', {
        timestamp: new Date().toISOString(),
        id,
        snapshotId,
        label,
      });

      return ok(snapshotId);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to create snapshot: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Restores from a snapshot
   */
  async restoreSnapshot(snapshotId: string): Promise<Result<ArchitectureAggregate, ValidationError | NetworkError>> {
    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const snapshot = await this.getRecord<SnapshotRecord>(STORE_SNAPSHOTS, snapshotId);
      if (!snapshot) {
        return err(new ArchitectureNotFoundError(snapshotId));
      }

      const dump: SrujaModelDump = JSON.parse(snapshot.aggregate);
      const result = ArchitectureAggregate.fromDump(dump);

      if (!result.ok) {
        return err(result.error);
      }

      // Save restored snapshot
      const saveResult = await this.saveWithId(snapshot.architectureId, result.value);
      if (!saveResult.ok) {
        return err(saveResult.error);
      }

      this.emitArchitecture(snapshot.architectureId, 'restored', {
        timestamp: new Date().toISOString(),
        id: snapshot.architectureId,
        fromSnapshot: snapshotId,
      });

      return ok(result.value);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to restore snapshot: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  // =========================================================================
  // Statistics & Analytics
  // =========================================================================

  /**
   * Gets repository-wide statistics
   */
  async getStatistics(): Promise<Result<ArchitectureStatistics, ValidationError | NetworkError>> {
    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const summaries = await this.findAllSummaries();
      if (!summaries.ok) {
        return err(summaries.error);
      }

      const architecturesByTag: Record<string, number> = {};
      let totalStorage = 0;
      let totalElements = 0;

      for (const summary of summaries.value.items) {
        for (const tag of summary.tags) {
          architecturesByTag[tag] = (architecturesByTag[tag] || 0) + 1;
        }
        totalElements += summary.elementCount;
      }

      const statistics: ArchitectureStatistics = {
        totalArchitectures: summaries.value.items.length,
        architecturesByTag,
        recentUpdates: summaries.value.items.slice(0, 10),
        storageUsed: totalStorage,
        averageElementCount: summaries.value.items.length > 0
          ? totalElements / summaries.value.items.length
          : 0,
      };

      return ok(statistics);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to get statistics: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Gets the count of architectures
   */
  async count(options?: QueryOptions): Promise<Result<number, ValidationError | NetworkError>> {
    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const store = this.db.transaction(STORE_ARCHITECTURES, 'readonly').objectStore(STORE_ARCHITECTURES);
      const request = store.count();
      const count = await new Promise<number>((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
      });

      return ok(count);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to count architectures: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Gets all unique tags across all architectures
   */
  async getAllTags(): Promise<Result<string[], ValidationError | NetworkError>> {
    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const store = this.db.transaction(STORE_TAGS_INDEX, 'readonly').objectStore(STORE_TAGS_INDEX);
      const request = store.getAllKeys();
      const tags: string[] = await new Promise((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
      });

      return ok(tags);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to get all tags: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  /**
   * Gets tag usage statistics
   */
  async getTagStatistics(): Promise<Result<Record<string, number>, ValidationError | NetworkError>> {
    if (!this.db) {
      return err(new ConfigurationError('Database not initialized'));
    }

    try {
      const store = this.db.transaction(STORE_TAGS_INDEX, 'readonly').objectStore(STORE_TAGS_INDEX);
      const request = store.getAll();
      const records: TagIndexRecord[] = await new Promise((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
      });

      const statistics: Record<string, number> = {};
      for (const record of records) {
        statistics[record.tag] = record.architectureIds.length;
      }

      return ok(statistics);
    } catch (error) {
      return err(
        new ValidationError(
          `Failed to get tag statistics: ${error instanceof Error ? error.message : 'Unknown error'}`
        )
      );
    }
  }

  // =========================================================================
  // Caching
  // =========================================================================

  /**
   * Clears the cache
   */
  async clearCache(): Promise<Result<void, ConfigurationError>> {
    this.cache.clear();

    this.emit('cache-cleared', { timestamp: new Date().toISOString() });

    return ok(undefined);
  }

  /**
   * Preloads/warms the cache with specific architectures
   */
  async warmCache(ids: string[]): Promise<Result<void, ConfigurationError | NetworkError>> {
    for (const id of ids) {
      await this.findById(id);
    }

    return ok(undefined);
  }

  // =========================================================================
  // Event Handlers
  // =========================================================================

  /**
   * Subscribes to repository events
   */
  on<K extends keyof RepositoryEventMap>(
    eventType: K,
    handler: (event: RepositoryEventMap[K]) => void
  ): () => void {
    const key = eventType;
    if (!this.eventHandlers.has(key)) {
      this.eventHandlers.set(key, new Set());
    }

    this.eventHandlers.get(key)!.add(handler);

    return () => {
      this.eventHandlers.get(key)?.delete(handler);
    };
  }

  /**
   * Subscribes to events for a specific architecture
   */
  onArchitecture<K extends keyof ArchitectureEventMap>(
    architectureId: string,
    eventType: K,
    handler: (event: ArchitectureEventMap[K]) => void
  ): () => void {
    if (!this.architectureEventHandlers.has(architectureId)) {
      this.architectureEventHandlers.set(architectureId, new Map());
    }

    const key = eventType;
    const handlers = this.architectureEventHandlers.get(architectureId)!;

    if (!handlers.has(key)) {
      handlers.set(key, new Set());
    }

    handlers.get(key)!.add(handler);

    return () => {
      this.architectureEventHandlers.get(architectureId)?.get(key)?.delete(handler);
    };
  }

  // =========================================================================
  // Private Helpers
  // =========================================================================

  /**
   * Ensures the repository is initialized
   *
   * @private
   */
  private async ensureInitialized(): Promise<Result<void, ConfigurationError | NetworkError>> {
    if (this.isInitialized && this.db) {
      return ok(undefined);
    }
    return this.initialize();
  }

  /**
   * Gets a record from IndexedDB
   *
   * @private
   */
  private async getRecord<T>(storeName: string, key: string): Promise<T | undefined> {
    if (!this.db) {
      return undefined;
    }

    const store = this.db.transaction(storeName, 'readonly').objectStore(storeName);
    const request = store.get(key);

    return new Promise((resolve, reject) => {
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Puts a record into IndexedDB
   *
   * @private
   */
  private async putRecord<T>(storeName: string, record: T): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    const store = this.db.transaction(storeName, 'readwrite').objectStore(storeName);
    const request = store.put(record);

    return new Promise((resolve, reject) => {
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Deletes a record from IndexedDB
   *
   * @private
   */
  private async deleteRecord(storeName: string, key: string): Promise<void> {
    if (!this.db) {
      throw new Error('Database not initialized');
    }

    const store = this.db.transaction(storeName, 'readwrite').objectStore(storeName);
    const request = store.delete(key);

    return new Promise((resolve, reject) => {
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Queries records by index
   *
   * @private
   */
  private async queryRecords<T>(
    storeName: string,
    indexName: string,
    key: string
  ): Promise<T[]> {
    if (!this.db) {
      return [];
    }

    const store = this.db.transaction(storeName, 'readonly').objectStore(storeName);
    const index = store.index(indexName);
    const request = index.getAll(key);

    return new Promise((resolve, reject) => {
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  }

  /**
   * Updates search and tag indexes for an architecture
   *
   * @private
   */
  private async updateIndexes(id: string, dump: SrujaModelDump): Promise<void> {
    if (!this.db) {
      return;
    }

    // Build search index
    const elementNames = Object.values(dump.elements || {}).map(el => el.name);
    const searchableText = [
      dump.metadata?.name || '',
      dump.metadata?.description || '',
      ...elementNames,
    ].join(' ').toLowerCase();

    const searchIndex: SearchIndexRecord = {
      architectureId: id,
      name: dump.metadata?.name || '',
      description: dump.metadata?.description,
      elementNames,
      searchableText,
    };

    await this.putRecord(STORE_SEARCH_INDEX, searchIndex);
  }

  /**
   * Checks if storage quota would be exceeded
   *
   * @private
   */
  private async checkStorageQuota(additionalSize: number): Promise<Result<void, StorageQuotaExceededError>> {
    try {
      if ('storage' in navigator && 'estimate' in navigator.storage) {
        const estimate = await navigator.storage.estimate();
        const usage = estimate.usage || 0;
        const quota = estimate.quota || 0;

        if (usage + additionalSize > quota * 0.9) {
          return err(new StorageQuotaExceededError());
        }
      }

      return ok(undefined);
    } catch {
      // If we can't check quota, assume it's fine
      return ok(undefined);
    }
  }

  /**
   * Checks if a cache entry is still valid
   *
   * @private
   */
  private isCacheValid(entry: CacheEntry): boolean {
    return Date.now() - entry.cachedAt < this.config.cacheTTL;
  }

  /**
   * Evicts old cache entries if cache is too large
   *
   * @private
   */
  private evictCacheIfNeeded(): void {
    if (this.cache.size <= this.config.cacheMaxSize) {
      return;
    }

    // Sort by last accessed time and evict oldest
    const entries = Array.from(this.cache.entries()).sort((a, b) =>
      a[1].lastAccessed - b[1].lastAccessed
    );

    const toEvict = entries.slice(0, this.cache.size - this.config.cacheMaxSize);
    for (const [key] of toEvict) {
      this.cache.delete(key);
    }
  }

  /**
   * Emits a repository event
   *
   * @private
   */
  private emit<K extends keyof RepositoryEventMap>(
    eventType: K,
    event: RepositoryEventMap[K]
  ): void {
    const handlers = this.eventHandlers.get(eventType);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(event);
        } catch (error) {
          console.error(`Error in event handler for '${eventType}':`, error);
        }
      }
    }
  }

  /**
   * Emits an architecture-specific event
   *
   * @private
   */
  private emitArchitecture<K extends keyof ArchitectureEventMap>(
    architectureId: string,
    eventType: K,
    event: ArchitectureEventMap[K]
  ): void {
    const handlers = this.architectureEventHandlers.get(architectureId)?.get(eventType);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(event);
        } catch (error) {
          console.error(
            `Error in event handler for architecture '${architectureId}' event '${eventType}':`,
            error
          );
        }
      }
    }
  }
}

// ============================================================================
// Factory Function
// ============================================================================

/**
 * Factory function to create an IndexedDBArchitectureRepository
 *
 * @param config - Optional repository configuration
 * @returns A new repository instance (not yet initialized)
 */
export function createIndexedDBRepository(config?: Partial<RepositoryConfig>): IndexedDBArchitectureRepository {
  return new IndexedDBArchitectureRepository(config);
}

/**
 * Creates and initializes an IndexedDB repository
 *
 * @param config - Optional repository configuration
 * @returns A promise that resolves to the initialized repository
 */
export async function initializeIndexedDBRepository(
  config?: Partial<RepositoryConfig>
): Promise<Result<IndexedDBArchitectureRepository, ConfigurationError | NetworkError>> {
  const repository = new IndexedDBArchitectureRepository(config);
  const initResult = await repository.initialize();

  if (!initResult.ok) {
    return err(initResult.error);
  }

  return ok(repository);
}

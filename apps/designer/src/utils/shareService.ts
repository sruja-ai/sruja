// apps/playground/src/utils/shareService.ts
import LZString from "lz-string";
import { logger } from "@sruja/shared";

const SHARE_STORAGE_KEY = "sruja-shares";

export interface ShareEntry {
  id: string;
  dsl: string;
  createdAt: number;
  updatedAt: number;
}

/**
 * Storage adapter interface for share persistence
 */
export interface ShareStorageAdapter {
  get(shareId: string): Promise<ShareEntry | null>;
  set(shareId: string, entry: ShareEntry): Promise<void>;
  delete(shareId: string): Promise<void>;
  getAll(): Promise<Record<string, ShareEntry>>;
  has(shareId: string): Promise<boolean>;
}

/**
 * LocalStorage adapter for share persistence
 */
class LocalStorageAdapter implements ShareStorageAdapter {
  private readonly storageKey: string;

  constructor(storageKey: string = SHARE_STORAGE_KEY) {
    this.storageKey = storageKey;
  }

  async get(shareId: string): Promise<ShareEntry | null> {
    try {
      const shares = this.getAllSync();
      return shares[shareId] || null;
    } catch {
      return null;
    }
  }

  async set(_shareId: string, entry: ShareEntry): Promise<void> {
    try {
      const shares = this.getAllSync();
      shares[entry.id] = entry;
      localStorage.setItem(this.storageKey, JSON.stringify(shares));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.error("Failed to save share to localStorage", {
        component: "shareService",
        action: "set",
        adapter: "LocalStorage",
        error:
          error instanceof Error
            ? {
                message: error.message,
                name: error.name,
                stack: error.stack,
              }
            : errorMessage,
      });
      throw error;
    }
  }

  async delete(shareId: string): Promise<void> {
    try {
      const shares = this.getAllSync();
      delete shares[shareId];
      localStorage.setItem(this.storageKey, JSON.stringify(shares));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.error("Failed to delete share from localStorage", {
        component: "shareService",
        action: "delete",
        adapter: "LocalStorage",
        shareId,
        error:
          error instanceof Error
            ? {
                message: error.message,
                name: error.name,
                stack: error.stack,
              }
            : errorMessage,
      });
      throw error;
    }
  }

  async getAll(): Promise<Record<string, ShareEntry>> {
    return this.getAllSync();
  }

  async has(shareId: string): Promise<boolean> {
    const shares = this.getAllSync();
    return shareId in shares;
  }

  private getAllSync(): Record<string, ShareEntry> {
    try {
      const stored = localStorage.getItem(this.storageKey);
      return stored ? JSON.parse(stored) : {};
    } catch {
      return {};
    }
  }
}

/**
 * IndexedDB adapter for share persistence (for larger storage)
 */
class IndexedDBAdapter implements ShareStorageAdapter {
  private dbName = "sruja-shares";
  private storeName = "shares";
  private db: IDBDatabase | null = null;

  private async initDB(): Promise<IDBDatabase> {
    if (this.db) return this.db;

    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, 1);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve(request.result);
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        if (!db.objectStoreNames.contains(this.storeName)) {
          const store = db.createObjectStore(this.storeName, { keyPath: "id" });
          store.createIndex("createdAt", "createdAt", { unique: false });
          store.createIndex("updatedAt", "updatedAt", { unique: false });
        }
      };
    });
  }

  async get(shareId: string): Promise<ShareEntry | null> {
    try {
      const db = await this.initDB();
      return new Promise((resolve, reject) => {
        const transaction = db.transaction([this.storeName], "readonly");
        const store = transaction.objectStore(this.storeName);
        const request = store.get(shareId);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result || null);
      });
    } catch {
      return null;
    }
  }

  async set(_shareId: string, entry: ShareEntry): Promise<void> {
    try {
      const db = await this.initDB();
      return new Promise((resolve, reject) => {
        const transaction = db.transaction([this.storeName], "readwrite");
        const store = transaction.objectStore(this.storeName);
        const request = store.put(entry);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve();
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.error("Failed to save share to IndexedDB", {
        component: "shareService",
        action: "set",
        adapter: "IndexedDB",
        error:
          error instanceof Error
            ? {
                message: error.message,
                name: error.name,
                stack: error.stack,
              }
            : errorMessage,
      });
      throw error;
    }
  }

  async delete(shareId: string): Promise<void> {
    try {
      const db = await this.initDB();
      return new Promise((resolve, reject) => {
        const transaction = db.transaction([this.storeName], "readwrite");
        const store = transaction.objectStore(this.storeName);
        const request = store.delete(shareId);

        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve();
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.error("Failed to delete share from IndexedDB", {
        component: "shareService",
        action: "delete",
        adapter: "IndexedDB",
        shareId,
        error:
          error instanceof Error
            ? {
                message: error.message,
                name: error.name,
                stack: error.stack,
              }
            : errorMessage,
      });
      throw error;
    }
  }

  async getAll(): Promise<Record<string, ShareEntry>> {
    try {
      const db = await this.initDB();
      return new Promise((resolve, reject) => {
        const transaction = db.transaction([this.storeName], "readonly");
        const store = transaction.objectStore(this.storeName);
        const request = store.getAll();

        request.onerror = () => reject(request.error);
        request.onsuccess = () => {
          const entries = request.result as ShareEntry[];
          const result: Record<string, ShareEntry> = {};
          entries.forEach((entry) => {
            result[entry.id] = entry;
          });
          resolve(result);
        };
      });
    } catch {
      return {};
    }
  }

  async has(shareId: string): Promise<boolean> {
    const entry = await this.get(shareId);
    return entry !== null;
  }
}

/**
 * Backend API adapter for share persistence (for cross-device sharing)
 */
class BackendAPIAdapter implements ShareStorageAdapter {
  private readonly baseUrl: string;

  constructor(baseUrl: string = "/api/shares") {
    this.baseUrl = baseUrl;
  }

  async get(shareId: string): Promise<ShareEntry | null> {
    try {
      const response = await fetch(`${this.baseUrl}/${shareId}`);
      if (!response.ok) {
        if (response.status === 404) return null;
        throw new Error(`Failed to load share: ${response.statusText}`);
      }
      const data = await response.json();
      return {
        id: data.id || shareId,
        dsl: data.dsl,
        createdAt: new Date(data.created_at || data.createdAt).getTime(),
        updatedAt: new Date(data.updated_at || data.updatedAt).getTime(),
      };
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.error("Failed to load share from backend", {
        component: "shareService",
        action: "get",
        adapter: "Backend",
        shareId,
        error:
          error instanceof Error
            ? {
                message: error.message,
                name: error.name,
                stack: error.stack,
              }
            : errorMessage,
      });
      return null;
    }
  }

  async set(shareId: string, entry: ShareEntry): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/${shareId}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          id: shareId,
          dsl: entry.dsl,
          created_at: new Date(entry.createdAt).toISOString(),
          updated_at: new Date(entry.updatedAt).toISOString(),
        }),
      });
      if (!response.ok) {
        throw new Error(`Failed to save share: ${response.statusText}`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.error("Failed to save share to backend", {
        component: "shareService",
        action: "set",
        adapter: "Backend",
        error:
          error instanceof Error
            ? {
                message: error.message,
                name: error.name,
                stack: error.stack,
              }
            : errorMessage,
      });
      throw error;
    }
  }

  async delete(shareId: string): Promise<void> {
    try {
      const response = await fetch(`${this.baseUrl}/${shareId}`, {
        method: "DELETE",
      });
      if (!response.ok && response.status !== 404) {
        throw new Error(`Failed to delete share: ${response.statusText}`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.error("Failed to delete share from backend", {
        component: "shareService",
        action: "delete",
        adapter: "Backend",
        shareId,
        error:
          error instanceof Error
            ? {
                message: error.message,
                name: error.name,
                stack: error.stack,
              }
            : errorMessage,
      });
      throw error;
    }
  }

  async getAll(): Promise<Record<string, ShareEntry>> {
    // Backend typically doesn't support getAll, return empty
    // Individual gets can be done via get(shareId)
    return {};
  }

  async has(shareId: string): Promise<boolean> {
    const entry = await this.get(shareId);
    return entry !== null;
  }
}

/**
 * Composite adapter that tries multiple storage backends in order
 */
class CompositeStorageAdapter implements ShareStorageAdapter {
  private adapters: ShareStorageAdapter[];

  constructor(adapters: ShareStorageAdapter[]) {
    this.adapters = adapters;
  }

  async get(shareId: string): Promise<ShareEntry | null> {
    // Try each adapter in order until one succeeds
    for (const adapter of this.adapters) {
      try {
        const entry = await adapter.get(shareId);
        if (entry) return entry;
      } catch {
        // Continue to next adapter
      }
    }
    return null;
  }

  async set(shareId: string, entry: ShareEntry): Promise<void> {
    // Write to all adapters (best effort)
    const promises = this.adapters.map((adapter) =>
      adapter.set(shareId, entry).catch((err) => {
        const errorMessage = err instanceof Error ? err.message : String(err);
        logger.warn("Failed to write to storage adapter", {
          component: "shareService",
          action: "set",
          shareId,
          error: errorMessage,
        });
      })
    );
    await Promise.allSettled(promises);
  }

  async delete(shareId: string): Promise<void> {
    // Delete from all adapters
    const promises = this.adapters.map((adapter) =>
      adapter.delete(shareId).catch((err) => {
        const errorMessage = err instanceof Error ? err.message : String(err);
        logger.warn("Failed to delete from storage adapter", {
          component: "shareService",
          action: "delete",
          shareId,
          error: errorMessage,
        });
      })
    );
    await Promise.allSettled(promises);
  }

  async getAll(): Promise<Record<string, ShareEntry>> {
    // Merge results from all adapters (localStorage is source of truth for local)
    const allEntries: Record<string, ShareEntry> = {};
    for (const adapter of this.adapters) {
      try {
        const entries = await adapter.getAll();
        Object.assign(allEntries, entries);
      } catch {
        // Continue to next adapter
      }
    }
    return allEntries;
  }

  async has(shareId: string): Promise<boolean> {
    for (const adapter of this.adapters) {
      try {
        if (await adapter.has(shareId)) return true;
      } catch {
        // Continue to next adapter
      }
    }
    return false;
  }
}

/**
 * Share service for generating stable URLs that don't change with mutations.
 * Supports pluggable storage backends (localStorage, IndexedDB, Backend API, etc.)
 */
class ShareService {
  private storage: ShareStorageAdapter;
  /**
   * Generate a unique share ID (UUID v4)
   */
  private generateShareId(): string {
    // Use crypto.randomUUID if available, fallback to manual generation
    if (typeof crypto !== "undefined" && crypto.randomUUID) {
      return crypto.randomUUID();
    }
    // Fallback for older browsers
    return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === "x" ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
  }

  constructor(storageAdapter?: ShareStorageAdapter) {
    // Default to localStorage, but can be configured
    this.storage = storageAdapter || new LocalStorageAdapter();
  }

  /**
   * Configure storage adapter(s)
   * @param adapters One or more storage adapters (composite adapter if multiple)
   */
  setStorageAdapter(...adapters: ShareStorageAdapter[]): void {
    this.storage = adapters.length === 1 ? adapters[0] : new CompositeStorageAdapter(adapters);
  }

  /**
   * Clean up old shares (keep last N)
   */
  async cleanupOldShares(maxEntries: number = 50): Promise<void> {
    const shares = await this.storage.getAll();
    const entries = Object.entries(shares);
    if (entries.length <= maxEntries) return;

    // Sort by updatedAt (newest first) and keep top N
    entries.sort((a, b) => b[1].updatedAt - a[1].updatedAt);
    const toDelete = entries.slice(maxEntries);

    // Delete old entries
    await Promise.all(toDelete.map(([id]) => this.storage.delete(id)));
  }

  /**
   * Create a new share with stable ID
   * @param dsl The DSL content to share
   * @param includeCodeInUrl If true, include compressed code in URL for first-time sharing
   * @returns Share URL with stable ID
   */
  async createShare(dsl: string, includeCodeInUrl: boolean = false): Promise<string> {
    const id = this.generateShareId();
    const now = Date.now();

    // Store in configured storage
    await this.storage.set(id, {
      id,
      dsl,
      createdAt: now,
      updatedAt: now,
    });

    // Generate URL with share ID
    const url = new URL(window.location.href);
    url.searchParams.set("share", id);

    // Optionally include code in URL for first-time sharing
    if (includeCodeInUrl) {
      const compressed = LZString.compressToBase64(dsl);
      const encoded = encodeURIComponent(compressed);
      url.searchParams.set("code", encoded);
    }

    return url.toString();
  }

  /**
   * Update an existing share (mutations)
   * @param shareId The share ID
   * @param dsl The updated DSL content
   */
  async updateShare(shareId: string, dsl: string): Promise<void> {
    const existing = await this.storage.get(shareId);
    if (!existing) {
      throw new Error(`Share ${shareId} not found`);
    }

    await this.storage.set(shareId, {
      ...existing,
      dsl,
      updatedAt: Date.now(),
    });
  }

  /**
   * Load share data by ID
   * @param shareId The share ID
   * @param fallbackCode Optional compressed code from URL (for first-time loads)
   * @returns DSL content or null if not found
   */
  async loadShare(shareId: string, fallbackCode?: string): Promise<string | null> {
    // Try configured storage first (for mutations/updates)
    const entry = await this.storage.get(shareId);
    if (entry) {
      return entry.dsl;
    }

    // Fallback to URL code (for first-time sharing from another device)
    if (fallbackCode) {
      try {
        const decoded = decodeURIComponent(fallbackCode);
        const decompressed = LZString.decompressFromBase64(decoded);
        if (decompressed) {
          // Store it in configured storage for future loads
          try {
            await this.updateShare(shareId, decompressed);
          } catch {
            // Share doesn't exist yet, create it
            const now = Date.now();
            await this.storage.set(shareId, {
              id: shareId,
              dsl: decompressed,
              createdAt: now,
              updatedAt: now,
            });
          }
          return decompressed;
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        logger.error("Failed to decompress code from URL", {
          component: "shareService",
          action: "decompressFromUrl",
          error:
            error instanceof Error
              ? {
                  message: error.message,
                  name: error.name,
                  stack: error.stack,
                }
              : errorMessage,
        });
      }
    }

    return null;
  }

  /**
   * Check if a share ID exists in storage
   */
  async hasShare(shareId: string): Promise<boolean> {
    return await this.storage.has(shareId);
  }

  /**
   * Delete a share
   */
  async deleteShare(shareId: string): Promise<void> {
    await this.storage.delete(shareId);
  }

  /**
   * Get share metadata (for debugging/admin)
   */
  async getShareInfo(shareId: string): Promise<ShareEntry | null> {
    return await this.storage.get(shareId);
  }
}

// Singleton instance (defaults to localStorage)
export const shareService = new ShareService();

// Export adapters for custom configuration
export { LocalStorageAdapter, IndexedDBAdapter, BackendAPIAdapter, CompositeStorageAdapter };

/**
 * Parse share ID and code from URL
 * - share=<id>: Share ID (stable, updates don't change URL)
 * - code=<compressed>: Compressed DSL code (for direct sharing)
 */
export function parseShareUrl(url: string | URL): { shareId: string | null; code: string | null } {
  const urlObj = typeof url === "string" ? new URL(url) : url;
  return {
    shareId: urlObj.searchParams.get("share"),
    code: urlObj.searchParams.get("code"),
  };
}

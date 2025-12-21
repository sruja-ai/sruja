// packages/shared/src/storage/indexedStore.ts
// Browser storage utilities with IndexedDB fallback

import { isBrowser } from '../utils/env';

/**
 * Key-value storage interface.
 * 
 * @internal
 */
interface KV {
  /** Get a value by key */
  get(key: string): Promise<string | null>;
  /** Set a value by key */
  set(key: string, value: string): Promise<void>;
}

/**
 * Create a localStorage-based KV store.
 * 
 * @internal
 * @returns KV store using localStorage
 * 
 * @remarks
 * Falls back to no-op if window is undefined (SSR).
 */
function createLocal(): KV {
  return {
    async get(key: string): Promise<string | null> {
      if (!isBrowser()) {
        return null;
      }
      return window.localStorage.getItem(key);
    },
    async set(key: string, value: string): Promise<void> {
      if (!isBrowser()) {
        return;
      }
      window.localStorage.setItem(key, value);
    },
  };
}

/**
 * Create an IndexedDB-based KV store.
 * 
 * @internal
 * @returns Promise resolving to KV store using IndexedDB
 * 
 * @remarks
 * Falls back to localStorage if IndexedDB is unavailable.
 * Database name: 'sruja', version: 1, object store: 'kv'.
 */
function createIndexed(): Promise<KV> {
  return new Promise((resolve) => {
    if (typeof indexedDB === 'undefined') {
      return resolve(createLocal());
    }
    const req = indexedDB.open('sruja', 1);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains('kv')) {
        db.createObjectStore('kv');
      }
    };
    req.onsuccess = () => {
      const db = req.result;
      resolve({
        async get(key: string): Promise<string | null> {
          return new Promise((res) => {
            const tx = db.transaction('kv', 'readonly');
            const store = tx.objectStore('kv');
            const g = store.get(key);
            g.onsuccess = () => {
              res((g.result as string) || null);
            };
            g.onerror = () => {
              res(null);
            };
          });
        },
        async set(key: string, value: string): Promise<void> {
          return new Promise((res) => {
            const tx = db.transaction('kv', 'readwrite');
            const store = tx.objectStore('kv');
            const p = store.put(value, key);
            p.onsuccess = () => {
              res();
            };
            p.onerror = () => {
              res();
            };
          });
        },
      });
    };
    req.onerror = () => {
      resolve(createLocal());
    };
  });
}

/**
 * Singleton promise for the KV store instance.
 * 
 * @internal
 */
let kvPromise: Promise<KV> | null = null;

/**
 * Get or create the storage store instance.
 * 
 * @public
 * @returns Promise resolving to the KV store
 * 
 * @remarks
 * Uses IndexedDB if available, falls back to localStorage.
 * The store is created lazily on first access.
 * 
 * @example
 * const store = await getStore();
 * await store.set('key', 'value');
 * const value = await store.get('key');
 */
export async function getStore(): Promise<KV> {
  if (!kvPromise) {
    kvPromise = createIndexed();
  }
  return kvPromise;
}

/**
 * Set a value in storage.
 * 
 * @public
 * @param key - Storage key
 * @param value - Value to store
 * 
 * @example
 * await storeSet('user-preferences', JSON.stringify({ theme: 'dark' }));
 */
export async function storeSet(key: string, value: string): Promise<void> {
  const s = await getStore();
  return s.set(key, value);
}

/**
 * Get a value from storage.
 * 
 * @public
 * @param key - Storage key
 * @returns Stored value or null if not found
 * 
 * @example
 * const value = await storeGet('user-preferences');
 * if (value) { const prefs = JSON.parse(value); }
 */
export async function storeGet(key: string): Promise<string | null> {
  const s = await getStore();
  return s.get(key);
}

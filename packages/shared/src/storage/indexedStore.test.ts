// packages/shared/src/storage/indexedStore.test.ts
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { storeSet, storeGet } from './indexedStore';

// Mock IndexedDB and localStorage
const mockStorage = new Map<string, string>();

beforeEach(() => {
  mockStorage.clear();
  
  // Mock localStorage
  global.localStorage = {
    getItem: vi.fn((key: string) => mockStorage.get(key) || null),
    setItem: vi.fn((key: string, value: string) => {
      mockStorage.set(key, value);
    }),
    removeItem: vi.fn((key: string) => {
      mockStorage.delete(key);
    }),
    clear: vi.fn(() => {
      mockStorage.clear();
    }),
    length: 0,
    key: vi.fn(() => null),
  } as any;
  
  // Mock indexedDB to fallback to localStorage
  global.indexedDB = undefined as any;
});

afterEach(() => {
  mockStorage.clear();
});

describe('IndexedStore', () => {
  describe('storeSet and storeGet', () => {
    it('should store and retrieve a value', async () => {
      await storeSet('key1', 'value1');
      const value = await storeGet('key1');
      expect(value).toBe('value1');
    });

    it('should store and retrieve objects', async () => {
      const obj = { name: 'test', count: 42 };
      await storeSet('key2', JSON.stringify(obj));
      const value = await storeGet('key2');
      expect(JSON.parse(value!)).toEqual(obj);
    });

    it('should return null for non-existent keys', async () => {
      const value = await storeGet('non-existent');
      expect(value).toBeNull();
    });

    it('should overwrite existing values', async () => {
      await storeSet('key3', 'value1');
      await storeSet('key3', 'value2');
      const value = await storeGet('key3');
      expect(value).toBe('value2');
    });
  });
});


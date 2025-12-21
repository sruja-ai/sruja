// apps/website/src/shared/utils/storage.test.ts
import { describe, it, expect, beforeEach } from 'vitest';
import {
  setStorageItem,
  getStorageItem,
  removeStorageItem,
  createStorage,
  createStringStorage,
} from './storage';

describe('storage utilities', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe('setStorageItem', () => {
    it('sets a value in localStorage', () => {
      expect(setStorageItem('test-key', 'test-value')).toBe(true);
      expect(localStorage.getItem('test-key')).toBe('test-value');
    });

    it('returns false if localStorage is not available', () => {
      const originalWindow = global.window;
      // @ts-expect-error - testing localStorage unavailability
      delete global.window;
      expect(setStorageItem('test-key', 'test-value')).toBe(false);
      global.window = originalWindow;
    });
  });

  describe('getStorageItem', () => {
    it('gets a value from localStorage', () => {
      localStorage.setItem('test-key', 'test-value');
      expect(getStorageItem('test-key')).toBe('test-value');
    });

    it('returns null if key does not exist', () => {
      expect(getStorageItem('non-existent')).toBeNull();
    });
  });

  describe('removeStorageItem', () => {
    it('removes an item from localStorage', () => {
      localStorage.setItem('test-key', 'test-value');
      expect(removeStorageItem('test-key')).toBe(true);
      expect(localStorage.getItem('test-key')).toBeNull();
    });
  });

  describe('createStorage', () => {
    it('stores and retrieves JSON values', () => {
      const storage = createStorage<{ name: string }>('test-json', { name: 'default' });
      
      expect(storage.get()).toEqual({ name: 'default' });
      
      storage.set({ name: 'test' });
      expect(storage.get()).toEqual({ name: 'test' });
    });

    it('returns default value for invalid JSON', () => {
      localStorage.setItem('test-json', 'invalid json');
      const storage = createStorage<{ name: string }>('test-json', { name: 'default' });
      expect(storage.get()).toEqual({ name: 'default' });
    });
  });

  describe('createStringStorage', () => {
    it('stores and retrieves string values', () => {
      const storage = createStringStorage('test-string', 'default');
      
      expect(storage.get()).toBe('default');
      
      storage.set('test-value');
      expect(storage.get()).toBe('test-value');
    });

    it('uses empty string as default if not provided', () => {
      const storage = createStringStorage('test-empty');
      expect(storage.get()).toBe('');
    });
  });
});

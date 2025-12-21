// apps/website/src/shared/utils/storage.ts

/**
 * Generic localStorage helper that handles errors gracefully
 */
export function setStorageItem(key: string, value: string): boolean {
  if (typeof window === 'undefined') return false;
  try {
    localStorage.setItem(key, value);
    return true;
  } catch (e) {
    if (e instanceof DOMException && e.name === 'QuotaExceededError') {
      console.warn(`localStorage quota exceeded for key: ${key}`);
    } else {
      console.warn(`Failed to save to localStorage (${key}):`, e);
    }
    return false;
  }
}

/**
 * Generic localStorage getter that handles errors gracefully
 */
export function getStorageItem(key: string): string | null {
  if (typeof window === 'undefined') return null;
  try {
    return localStorage.getItem(key);
  } catch (e) {
    console.warn(`Failed to load from localStorage (${key}):`, e);
    return null;
  }
}

/**
 * Generic localStorage remove that handles errors gracefully
 */
export function removeStorageItem(key: string): boolean {
  if (typeof window === 'undefined') return false;
  try {
    localStorage.removeItem(key);
    return true;
  } catch (e) {
    console.warn(`Failed to remove from localStorage (${key}):`, e);
    return false;
  }
}

/**
 * Create a typed storage interface for a specific key
 */
export function createStorage<T>(key: string, defaultValue: T) {
  return {
    get: (): T => {
      const item = getStorageItem(key);
      if (item === null) return defaultValue;
      try {
        return JSON.parse(item) as T;
      } catch {
        return defaultValue;
      }
    },
    set: (value: T): boolean => {
      try {
        return setStorageItem(key, JSON.stringify(value));
      } catch {
        return false;
      }
    },
    remove: (): boolean => removeStorageItem(key),
  };
}

/**
 * String storage (non-JSON) for simple string values
 */
export function createStringStorage(key: string, defaultValue: string = '') {
  return {
    get: (): string => {
      return getStorageItem(key) ?? defaultValue;
    },
    set: (value: string): boolean => {
      return setStorageItem(key, value);
    },
    remove: (): boolean => removeStorageItem(key),
  };
}

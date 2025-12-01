// Storage utilities with error handling and validation

/**
 * Safe localStorage wrapper with error handling
 */
export class StorageError extends Error {
  constructor(message: string, public readonly originalError?: unknown) {
    super(message);
    this.name = 'StorageError';
  }
}

/**
 * Safely get an item from localStorage
 */
export function getStorageItem(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch (error) {
    console.warn(`Failed to read from localStorage for key "${key}":`, error);
    return null;
  }
}

/**
 * Safely set an item in localStorage
 */
export function setStorageItem(key: string, value: string): boolean {
  try {
    localStorage.setItem(key, value);
    return true;
  } catch (error) {
    if (error instanceof DOMException) {
      if (error.name === 'QuotaExceededError') {
        console.warn('localStorage quota exceeded');
        // Try to clear old entries
        try {
          clearOldStorageEntries();
          localStorage.setItem(key, value);
          return true;
        } catch (retryError) {
          console.error('Failed to save after clearing old entries:', retryError);
        }
      }
    }
    console.warn(`Failed to write to localStorage for key "${key}":`, error);
    return false;
  }
}

/**
 * Safely remove an item from localStorage
 */
export function removeStorageItem(key: string): boolean {
  try {
    localStorage.removeItem(key);
    return true;
  } catch (error) {
    console.warn(`Failed to remove from localStorage for key "${key}":`, error);
    return false;
  }
}

/**
 * Get a JSON object from localStorage
 */
export function getStorageJSON<T>(key: string, defaultValue: T | null = null): T | null {
  const item = getStorageItem(key);
  if (item === null) {
    return defaultValue;
  }
  try {
    return JSON.parse(item) as T;
  } catch (error) {
    console.warn(`Failed to parse JSON from localStorage for key "${key}":`, error);
    return defaultValue;
  }
}

/**
 * Set a JSON object in localStorage
 */
export function setStorageJSON<T>(key: string, value: T): boolean {
  try {
    const serialized = JSON.stringify(value);
    return setStorageItem(key, serialized);
  } catch (error) {
    console.warn(`Failed to serialize JSON for localStorage key "${key}":`, error);
    return false;
  }
}

/**
 * Check if localStorage is available
 */
export function isStorageAvailable(): boolean {
  try {
    const test = '__storage_test__';
    localStorage.setItem(test, test);
    localStorage.removeItem(test);
    return true;
  } catch {
    return false;
  }
}

/**
 * Clear old storage entries (simple implementation - clears entries older than 30 days)
 */
function clearOldStorageEntries(): void {
  const maxAge = 30 * 24 * 60 * 60 * 1000; // 30 days
  const now = Date.now();
  
  try {
    const keys: string[] = [];
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key && key.startsWith('sruja-')) {
        keys.push(key);
      }
    }
    
    keys.forEach(key => {
      try {
        const item = localStorage.getItem(key);
        if (item) {
          // Simple heuristic: if it's a timestamped entry, check age
          // This is a basic implementation - can be enhanced
          const data = JSON.parse(item);
          if (data && typeof data === 'object' && data.timestamp) {
            const age = now - data.timestamp;
            if (age > maxAge) {
              localStorage.removeItem(key);
            }
          }
        }
      } catch {
        // Ignore individual item errors
      }
    });
  } catch (error) {
    console.warn('Failed to clear old storage entries:', error);
  }
}


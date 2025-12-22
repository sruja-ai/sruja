// apps/website/src/shared/hooks/useLocalStorage.ts
import { useState, useCallback } from 'react';
import { createStorage } from '../utils/storage';

/**
 * React hook for localStorage with JSON serialization
 */
export function useLocalStorage<T>(key: string, defaultValue: T) {
  const storage = createStorage<T>(key, defaultValue);

  const [storedValue, setStoredValue] = useState<T>(() => {
    if (typeof window === 'undefined') {
      return defaultValue;
    }
    return storage.get();
  });

  const setValue = useCallback(
    (value: T | ((val: T) => T)) => {
      try {
        const valueToStore = value instanceof Function ? value(storedValue) : value;
        setStoredValue(valueToStore);
        if (typeof window !== 'undefined') {
          storage.set(valueToStore);
        }
      } catch (error) {
        console.error(`Error setting localStorage key "${key}":`, error);
      }
    },
    [key, storedValue, storage]
  );

  const removeValue = useCallback(() => {
    try {
      setStoredValue(defaultValue);
      if (typeof window !== 'undefined') {
        storage.remove();
      }
    } catch (error) {
      console.error(`Error removing localStorage key "${key}":`, error);
    }
  }, [key, defaultValue, storage]);

  return [storedValue, setValue, removeValue] as const;
}

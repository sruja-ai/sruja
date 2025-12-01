// React hook for WASM ready state
import { useState, useEffect } from 'react';
import { initSrujaWasm } from '../utils/wasm';

/**
 * Hook to check if WASM is ready
 * Automatically initializes WASM if not already initialized
 */
export function useWasmReady(): boolean {
  const [isReady, setIsReady] = useState(false);

  useEffect(() => {
    // Check if already ready (check both namespaced and legacy)
    const isReady = window.sruja?.wasmReady || window.srujaWasmReady;
    if (isReady) {
      setIsReady(true);
      return;
    }

    // Initialize WASM if not already initializing
    const isInitializing = window.sruja?.wasmInitializing || window.srujaWasmInitializing;
    if (!isInitializing) {
      initSrujaWasm();
    }

    // Poll for ready state (check both namespaced and legacy)
    const checkInterval = setInterval(() => {
      const ready = window.sruja?.wasmReady || window.srujaWasmReady;
      if (ready) {
        setIsReady(true);
        clearInterval(checkInterval);
      }
    }, 100);

    return () => clearInterval(checkInterval);
  }, []);

  return isReady;
}


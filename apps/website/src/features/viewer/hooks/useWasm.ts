import { useState, useEffect, useRef } from 'react';
import { initWasm, type WasmApi } from '@sruja/shared';

export function useWasm() {
  const [isWasmLoading, setIsWasmLoading] = useState(true);
  const wasmApiRef = useRef<WasmApi | null>(null);

  useEffect(() => {
    setIsWasmLoading(true);
    const base = typeof window !== 'undefined' ? '/' : '';
    initWasm({ base })
      .then((api) => {
        wasmApiRef.current = api;
        setIsWasmLoading(false);
      })
      .catch((err) => {
        console.error('WASM init failed', err);
        setIsWasmLoading(false);
      });
  }, []);

  return { wasmApiRef, isWasmLoading };
}



import { useState, useEffect } from 'react';
import LZString from 'lz-string';
import { loadDslFromStorage } from '../utils/storage';

export function useDslState(initialDsl?: string) {
  const [dsl, setDsl] = useState<string>(() => {
    // Priority: initialDsl > URL params/hash > localStorage
    if (initialDsl) return initialDsl;
    if (typeof window !== 'undefined') {
      const hash = window.location.hash || '';
      if (hash.startsWith('#code=')) {
        const b64 = hash.substring('#code='.length);
        try {
          const decompressed = LZString.decompressFromBase64(decodeURIComponent(b64));
          if (decompressed) return decompressed;
        } catch (_) {}
      }
      const params = new URLSearchParams(window.location.search);
      const urlCode = params.get('code');
      if (urlCode) return decodeURIComponent(urlCode);
    }
    return loadDslFromStorage();
  });

  return [dsl, setDsl] as const;
}



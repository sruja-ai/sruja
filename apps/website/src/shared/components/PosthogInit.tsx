// apps/website/src/shared/components/PosthogInit.tsx
import { useEffect } from 'react';
import { initPosthog, enableAutoTracking } from '@sruja/shared';

interface PosthogInitProps {
  apiKey?: string;
  host?: string;
}

export function PosthogInit({ apiKey, host }: PosthogInitProps) {
  useEffect(() => {
    if (apiKey) {
      (async () => {
        await initPosthog({ apiKey, host });
        enableAutoTracking();
      })();
    }
  }, [apiKey, host]);

  return null;
}

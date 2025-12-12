// apps/website/src/shared/scripts/initPosthog.ts
import { envConfig } from '../../config/env';
import { initPosthog, enableAutoTracking } from '@sruja/shared';

(async () => {
  if (envConfig.posthog?.apiKey) {
    await initPosthog({ apiKey: envConfig.posthog.apiKey, host: envConfig.posthog.host });
    enableAutoTracking();
  }
})();

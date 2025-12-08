// apps/website/src/config/env.ts
// Environment configuration with Algolia index selection
// Each environment uses a different Algolia index:
// - development: sruja_docs_dev
// - staging: sruja_docs_staging
// - production: sruja_docs

type Environment = 'development' | 'staging' | 'production'

interface EnvConfig {
  env: Environment
  siteUrl: string
  baseUrl: string
  posthog?: {
    apiKey: string
    host: string
  }
  algolia?: {
    appId: string
    apiKey: string
    indexName: string
  }
}

function getEnvironment(): Environment {
  // Check for explicit environment variable first
  if (import.meta.env.PUBLIC_ENV) {
    const explicitEnv = import.meta.env.PUBLIC_ENV.toLowerCase();
    if (explicitEnv === 'production' || explicitEnv === 'staging' || explicitEnv === 'development') {
      return explicitEnv as Environment;
    }
  }

  // Check NODE_ENV for production builds
  const isProduction = import.meta.env.MODE === 'production' || import.meta.env.PROD;
  
  if (isProduction) {
    const url = import.meta.env.SITE || '';
    // Check URL for staging indicators
    if (url.includes('staging') || url.includes('stage') || url.includes('stg')) {
      return 'staging';
    }
    // Default to production if no staging indicators
    return 'production';
  }
  
  // Default to development for dev mode
  return 'development';
}

function getEnvConfig(): EnvConfig {
  const env = getEnvironment()
  const siteUrl = import.meta.env.SITE || (env === 'production' ? 'https://sruja.ai' : 'http://localhost:4321')
  const baseUrl = import.meta.env.BASE || '/'
  
  const config: EnvConfig = {
    env,
    siteUrl,
    baseUrl,
  }

  // PostHog configuration
  if (import.meta.env.PUBLIC_POSTHOG_API_KEY) {
    config.posthog = {
      apiKey: import.meta.env.PUBLIC_POSTHOG_API_KEY,
      host: import.meta.env.PUBLIC_POSTHOG_HOST || 'https://app.posthog.com',
    }
  }

  // Algolia configuration
  // Application ID is hardcoded (safe to expose publicly, same across all environments)
  const algoliaAppId = 'N6FUL0KI3V';
  
  // Search-Only API Key - keep in env vars for flexibility
  // Set in .env.local: PUBLIC_ALGOLIA_SEARCH_API_KEY=966344a0eccf1872c34d1fee70ff7f7d
  const algoliaApiKey = import.meta.env.PUBLIC_ALGOLIA_SEARCH_API_KEY;
  
  if (algoliaAppId && algoliaApiKey) {
    // Determine index name based on environment
    // Can be overridden with PUBLIC_ALGOLIA_INDEX_NAME env var
    const defaultIndexName = import.meta.env.PUBLIC_ALGOLIA_INDEX_NAME || 
      (env === 'production' ? 'sruja_docs' : 
       env === 'staging' ? 'sruja_docs_staging' : 
       'sruja_docs_dev')
    
    config.algolia = {
      appId: algoliaAppId,
      apiKey: algoliaApiKey,
      indexName: defaultIndexName,
    }

    // Log index name in development for debugging
    if (env === 'development' && typeof console !== 'undefined') {
      console.log(`[Algolia] Using index: ${defaultIndexName} (env: ${env})`);
    }
  }

  return config
}

export const envConfig = getEnvConfig()
export type { EnvConfig, Environment }




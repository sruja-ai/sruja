// apps/website/src/config/env.ts
// Environment configuration with simplified Algolia index selection
// Uses a single Algolia index across all environments (overridable via env)

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
  sentry?: {
    dsn: string
    environment: string
    tracesSampleRate?: number
    replaysSessionSampleRate?: number
    replaysOnErrorSampleRate?: number
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

  // Algolia configuration (single index)
  // Application ID is hardcoded (safe to expose publicly, same across all environments)
  const algoliaAppId = 'N6FUL0KI3V';
  
  // Search-Only API Key - keep in env vars for flexibility
  // Set in .env.local: PUBLIC_ALGOLIA_SEARCH_API_KEY=966344a0eccf1872c34d1fee70ff7f7d
  const algoliaApiKey = import.meta.env.PUBLIC_ALGOLIA_SEARCH_API_KEY;
  
  if (algoliaAppId && algoliaApiKey) {
    // Single index name, overridable via PUBLIC_ALGOLIA_INDEX_NAME
    const defaultIndexName = import.meta.env.PUBLIC_ALGOLIA_INDEX_NAME || 'sruja_docs'
    
    config.algolia = {
      appId: algoliaAppId,
      apiKey: algoliaApiKey,
      indexName: defaultIndexName,
    }

    if (env === 'development' && typeof console !== 'undefined') {
      console.info(`[Algolia] Using index: ${defaultIndexName}`);
    }
  }

  // Sentry configuration
  const sentryDsn = import.meta.env.PUBLIC_SENTRY_DSN;
  if (sentryDsn) {
    config.sentry = {
      dsn: sentryDsn,
      environment: env,
      // Lower sample rates for staging, higher for production
      tracesSampleRate: env === 'production' ? 0.1 : env === 'staging' ? 0.5 : 1.0,
      // Session replay: capture all errors, sample sessions
      replaysSessionSampleRate: env === 'production' ? 0.1 : env === 'staging' ? 0.5 : 1.0,
      replaysOnErrorSampleRate: 1.0, // Always capture replays on errors
    };
  }

  return config
}

export const envConfig = getEnvConfig()
export type { EnvConfig, Environment }


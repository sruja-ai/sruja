let client: any = null
let ready = false

export type PosthogInit = {
  apiKey: string
  host?: string
  options?: Record<string, any>
}

export async function initPosthog(cfg: PosthogInit) {
  if (typeof window === 'undefined') return null
  
  // Don't initialize if API key is missing or empty
  if (!cfg.apiKey || cfg.apiKey.trim() === '') {
    console.warn('[PostHog] Skipping initialization: API key is missing or empty')
    return null
  }
  
  if ((window as any).posthog && typeof (window as any).posthog.init === 'function') {
    (window as any).posthog.init(cfg.apiKey, { api_host: cfg.host, ...(cfg.options || {}) })
    client = (window as any).posthog
    ready = true
    return client
  }
  const mod = await import('posthog-js')
  if (mod.default && typeof mod.default.init === 'function') {
    mod.default.init(cfg.apiKey, { api_host: cfg.host, ...(cfg.options || {}) })
    client = mod.default
  } else if (mod.posthog && typeof mod.posthog.init === 'function') {
    mod.posthog.init(cfg.apiKey, { api_host: cfg.host, ...(cfg.options || {}) })
    client = mod.posthog
  } else {
    console.warn('posthog-js module does not have init function')
    return null
  }
  ready = true
  return client
}

export function getPosthog() {
  if (typeof window !== 'undefined' && (window as any).posthog) return (window as any).posthog
  return client
}

export function capture(event: string, properties?: Record<string, any>) {
  const ph = getPosthog()
  if (ph && typeof ph.capture === 'function') ph.capture(event, properties || {})
}

export function identify(id: string, properties?: Record<string, any>) {
  const ph = getPosthog()
  if (ph && typeof ph.identify === 'function') ph.identify(id, properties || {})
}

export function isReady() {
  return ready || (typeof window !== 'undefined' && !!(window as any).posthog)
}

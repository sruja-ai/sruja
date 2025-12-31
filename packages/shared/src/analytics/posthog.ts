interface PosthogClient {
  init(apiKey: string, options?: Record<string, unknown>): void;
  capture(event: string, properties?: Record<string, unknown>): void;
  identify(id: string, properties?: Record<string, unknown>): void;
}

let client: PosthogClient | null = null;
let ready = false;
let environment: string | undefined = undefined;

export type PosthogInit = {
  apiKey: string;
  host?: string;
  environment?: string;
  options?: Record<string, unknown>;
};

export async function initPosthog(cfg: PosthogInit) {
  if (typeof window === "undefined") return null;

  // Don't initialize if API key is missing or empty
  if (!cfg.apiKey || cfg.apiKey.trim() === "") {
    console.warn("[PostHog] Skipping initialization: API key is missing or empty");
    return null;
  }

  const win = window as unknown as Window & { posthog?: PosthogClient };

  // Store environment for adding to all events
  environment = cfg.environment;

  // Build PostHog options with environment
  const posthogOptions: Record<string, unknown> = {
    api_host: cfg.host,
    ...(cfg.options || {}),
  };

  // Add environment to PostHog options if provided
  if (cfg.environment) {
    posthogOptions.environment = cfg.environment;
  }

  if (win.posthog && typeof win.posthog.init === "function") {
    win.posthog.init(cfg.apiKey, posthogOptions);
    client = win.posthog;
    ready = true;
    return client;
  }

  const mod = await import("posthog-js");
  const ph = ((mod as unknown as Record<string, unknown>).default ||
    (mod as unknown as Record<string, unknown>).posthog) as PosthogClient | undefined;

  if (ph && typeof ph.init === "function") {
    ph.init(cfg.apiKey, posthogOptions);
    client = ph;
    ready = true;
    return client;
  } else {
    console.warn("posthog-js module does not have init function");
    return null;
  }
}

export function getPosthog(): PosthogClient | null {
  if (typeof window !== "undefined") {
    const win = window as unknown as Window & { posthog?: PosthogClient };
    if (win.posthog) return win.posthog;
  }
  return client;
}

export function capture(event: string, properties?: Record<string, unknown>) {
  const ph = getPosthog();
  if (ph && typeof ph.capture === "function") {
    // Always include environment in event properties if set
    const propsWithEnv = {
      ...(properties || {}),
      ...(environment ? { environment } : {}),
    };
    ph.capture(event, propsWithEnv);
  }
}

export function identify(id: string, properties?: Record<string, unknown>) {
  const ph = getPosthog();
  if (ph && typeof ph.identify === "function") {
    // Always include environment in identify properties if set
    const propsWithEnv = {
      ...(properties || {}),
      ...(environment ? { environment } : {}),
    };
    ph.identify(id, propsWithEnv);
  }
}

export function isReady() {
  if (ready) return true;
  if (typeof window !== "undefined") {
    const win = window as unknown as Window & { posthog?: PosthogClient };
    return !!win.posthog;
  }
  return false;
}

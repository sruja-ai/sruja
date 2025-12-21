// packages/shared/src/utils/env.ts
// Environment detection utilities

/**
 * Check if code is running in a browser environment.
 * 
 * @public
 * @returns true if running in browser, false otherwise
 * 
 * @example
 * ```typescript
 * if (isBrowser()) {
 *   // Browser-specific code
 * }
 * ```
 */
export function isBrowser(): boolean {
  return typeof window !== 'undefined';
}

/**
 * Check if code is running in Node.js environment.
 * 
 * @public
 * @returns true if running in Node.js, false otherwise
 * 
 * @example
 * ```typescript
 * if (isNode()) {
 *   // Node.js-specific code
 * }
 * ```
 */
export function isNode(): boolean {
  return typeof process !== 'undefined' && process.versions?.node !== undefined;
}

/**
 * Check if code is running in SSR (Server-Side Rendering) environment.
 * 
 * @public
 * @returns true if running in SSR (no window/document), false otherwise
 * 
 * @example
 * ```typescript
 * if (isSSR()) {
 *   // SSR-safe code
 * }
 * ```
 */
export function isSSR(): boolean {
  return typeof window === 'undefined' || typeof document === 'undefined';
}

/**
 * Get base URL for the application.
 * 
 * @public
 * @param options - Configuration options
 * @param options.trailingSlash - Whether to include trailing slash (default: false)
 * @param options.studioPath - Special handling for /studio path (default: true)
 * @returns Base URL string
 * 
 * @remarks
 * Detects base URL from:
 * 1. Vite's import.meta.env.BASE_URL (if available)
 * 2. Current pathname (for subdirectory deployments)
 * 
 * @example
 * ```typescript
 * const base = getBaseUrl(); // Returns '/designer' or '/'
 * const baseWithSlash = getBaseUrl({ trailingSlash: true }); // Returns '/designer/' or '/'
 * ```
 */
export function getBaseUrl(options?: {
  trailingSlash?: boolean;
  studioPath?: boolean;
}): string {
  if (!isBrowser()) {
    return options?.trailingSlash ? '/' : '';
  }

  // Try to get BASE_URL from import.meta.env (Vite/Astro)
  const meta = import.meta as { env?: { BASE_URL?: string } };
  if (meta.env?.BASE_URL) {
    const base = meta.env.BASE_URL;
    if (options?.trailingSlash) {
      return base.endsWith('/') ? base : base + '/';
    }
    return base.endsWith('/') ? base.slice(0, -1) : base;
  }

  // Fallback: detect from current pathname
  const pathname = window.location.pathname;
  
  // Special handling for /studio path
  if (options?.studioPath !== false && pathname.startsWith('/studio')) {
    return options?.trailingSlash ? '/' : '';
  }

  // Extract base from pathname (e.g., /designer/index.html -> /designer)
  const match = pathname.match(/^(\/.+?)\//);
  const base = match ? match[1] : '';

  if (options?.trailingSlash) {
    return base ? base + '/' : '/';
  }
  return base;
}


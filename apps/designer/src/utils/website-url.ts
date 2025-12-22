/**
 * Get the website URL based on the current environment
 *
 * Returns the full URL for the website, respecting the environment:
 * - Production: https://sruja.ai
 * - Staging: https://staging.sruja.ai
 * - Development: http://localhost:4321 (or relative /)
 */

type Environment = "development" | "staging" | "production";

/**
 * Detect the current environment based on the hostname
 */
function getEnvironment(): Environment {
  if (typeof window === "undefined") {
    return "development";
  }

  const hostname = window.location.hostname;

  // Check for staging indicators
  if (hostname.includes("staging") || hostname.includes("stage") || hostname.includes("stg")) {
    return "staging";
  }

  // Check for localhost/development
  if (hostname === "localhost" || hostname === "127.0.0.1" || hostname.includes("localhost")) {
    return "development";
  }

  // Default to production for other domains
  return "production";
}

/**
 * Get the website URL with optional path
 *
 * @param path - Optional path to append (e.g., '/docs', '/courses')
 * @param useAbsolute - Whether to return absolute URL (default: true in production/staging)
 * @returns The website URL
 */
export function getWebsiteUrl(path: string = "", useAbsolute?: boolean): string {
  const env = getEnvironment();

  // Determine base URL based on environment
  let baseUrl: string;

  if (typeof window !== "undefined") {
    const shouldUseAbsolute = useAbsolute ?? env !== "development";

    if (shouldUseAbsolute) {
      // Extract base URL from current location
      const { protocol, hostname, port } = window.location;

      // For staging/production, use the hostname
      if (env === "staging") {
        baseUrl = "https://staging.sruja.ai";
      } else if (env === "production") {
        baseUrl = "https://sruja.ai";
      } else {
        // Development: use current origin
        baseUrl = port ? `${protocol}//${hostname}:${port}` : `${protocol}//${hostname}`;
      }
    } else {
      // Return relative path for development
      return path || "/";
    }
  } else {
    // SSR fallback
    baseUrl =
      env === "staging"
        ? "https://staging.sruja.ai"
        : env === "production"
          ? "https://sruja.ai"
          : "http://localhost:4321";
  }

  // Remove trailing slash from baseUrl if present
  baseUrl = baseUrl.replace(/\/$/, "");

  // Handle path
  if (path) {
    // Ensure path starts with /
    const normalizedPath = path.startsWith("/") ? path : `/${path}`;
    return `${baseUrl}${normalizedPath}`;
  }

  return baseUrl;
}

/**
 * Get website URL with query parameters
 *
 * @param path - Base path (e.g., '/docs', '/courses')
 * @param params - Query parameters object (e.g., { level: 'L1', tab: 'builder' })
 * @returns The website URL with query string
 */
export function getWebsiteUrlWithParams(path: string = "", params: Record<string, string>): string {
  const queryString = new URLSearchParams(params).toString();
  const fullPath = queryString ? `${path}?${queryString}` : path;
  return getWebsiteUrl(fullPath);
}

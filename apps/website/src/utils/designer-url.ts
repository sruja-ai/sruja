/**
 * Get the designer URL based on the current environment
 *
 * Returns the full URL for the designer app, respecting the environment:
 * - Production: https://sruja.ai/designer
 * - Staging: https://staging.sruja.ai/designer
 * - Development: http://localhost:4321/designer (or relative /designer)
 */

import { envConfig } from "@/config/env";

/**
 * Get the designer URL with optional path and query parameters
 *
 * @param path - Optional path to append (e.g., '?level=L1&tab=builder' or '/?level=L1&tab=builder')
 * @param useAbsolute - Whether to return absolute URL (default: true in production/staging)
 * @returns The designer URL
 */
export function getDesignerUrl(path: string = "", useAbsolute?: boolean): string {
  const basePath = "/designer";

  // Handle query string (starts with ?) or path (starts with /)
  let fullPath = basePath;
  if (path) {
    if (path.startsWith("?")) {
      // Query string: /designer?level=L1&tab=builder
      fullPath = `${basePath}${path}`;
    } else if (path.startsWith("/")) {
      // Path with query: /designer/?level=L1&tab=builder
      fullPath = `${basePath}${path}`;
    } else {
      // Just append as query: /designer?level=L1&tab=builder
      fullPath = `${basePath}?${path}`;
    }
  }

  // In browser, use absolute URL for production/staging, relative for development
  if (typeof window !== "undefined") {
    const shouldUseAbsolute = useAbsolute ?? envConfig.env !== "development";

    if (shouldUseAbsolute && envConfig.siteUrl) {
      // Remove trailing slash from siteUrl if present
      const baseUrl = envConfig.siteUrl.replace(/\/$/, "");
      return `${baseUrl}${fullPath}`;
    }
  }

  // Return relative path for development or when useAbsolute is false
  return fullPath;
}

/**
 * Get designer URL with query parameters
 *
 * @param params - Query parameters object (e.g., { level: 'L1', tab: 'builder' })
 * @returns The designer URL with query string
 */
export function getDesignerUrlWithParams(params: Record<string, string>): string {
  const queryString = new URLSearchParams(params).toString();
  return getDesignerUrl(queryString ? `?${queryString}` : "");
}

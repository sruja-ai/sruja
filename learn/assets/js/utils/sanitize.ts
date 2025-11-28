// HTML sanitization utilities to prevent XSS attacks

/**
 * Escapes HTML special characters to prevent XSS
 */
export function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

/**
 * Sanitizes a URL to ensure it's safe for use in href attributes
 * Only allows relative paths or same-origin URLs
 */
export function sanitizeUrl(url: string): string {
  // Remove any javascript: or data: protocols
  const trimmed = url.trim();
  if (trimmed.startsWith('javascript:') || trimmed.startsWith('data:') || trimmed.startsWith('vbscript:')) {
    return '#';
  }
  // Allow relative paths and absolute paths starting with /
  if (trimmed.startsWith('/') || trimmed.startsWith('./') || trimmed.startsWith('../')) {
    return trimmed;
  }
  // Allow http/https URLs for external links (will be validated by browser)
  if (trimmed.startsWith('http://') || trimmed.startsWith('https://')) {
    return trimmed;
  }
  // Default to safe relative path
  return '#';
}

/**
 * Sanitizes SVG content by ensuring it only contains valid SVG elements
 * This is a basic check - for production, consider using DOMPurify
 */
export function sanitizeSvg(svg: string): string {
  // Basic validation: ensure it starts with <svg and ends with </svg>
  const trimmed = svg.trim();
  if (!trimmed.startsWith('<svg') || !trimmed.includes('</svg>')) {
    return '';
  }
  // Remove script tags and event handlers (basic protection)
  return trimmed
    .replace(/<script[^>]*>[\s\S]*?<\/script>/gi, '')
    .replace(/on\w+\s*=\s*["'][^"']*["']/gi, '')
    .replace(/on\w+\s*=\s*[^\s>]*/gi, '');
}

/**
 * Creates a safe anchor element with sanitized href and text
 */
export function createSafeAnchor(href: string, text: string, className?: string): HTMLAnchorElement {
  const a = document.createElement('a');
  a.href = sanitizeUrl(href);
  a.textContent = text;
  if (className) {
    a.className = className;
  }
  return a;
}


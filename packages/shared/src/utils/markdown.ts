// packages/shared/src/utils/markdown.ts
// Utility functions for markdown processing
// Non-React utilities that can be used across all apps

import { isSSR } from "./env";

/**
 * Copy text to clipboard.
 *
 * @public
 * @param text - Text to copy to clipboard
 * @returns Promise resolving to true if successful, false otherwise
 *
 * @remarks
 * Only works in browser environments (not Node.js).
 * Uses modern Clipboard API with fallback to execCommand for older browsers.
 *
 * @example
 * ```typescript
 * const success = await copyToClipboard('Hello, world!');
 * if (success) {
 *   console.log('Copied to clipboard');
 * }
 * ```
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  if (!text) return false;

  // Check if we're in a browser environment
  if (isSSR()) {
    console.warn("copyToClipboard: Not available in Node.js environment");
    return false;
  }

  try {
    // Use modern Clipboard API if available
    if (navigator.clipboard && navigator.clipboard.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }

    // Fallback for older browsers
    const textArea = document.createElement("textarea");
    textArea.value = text;
    textArea.style.position = "fixed";
    textArea.style.left = "-999999px";
    textArea.style.top = "-999999px";
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();

    try {
      const successful = document.execCommand("copy");
      document.body.removeChild(textArea);
      return successful;
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (_err) {
      document.body.removeChild(textArea);
      return false;
    }
  } catch (err) {
    console.error("Failed to copy to clipboard:", err);
    return false;
  }
}

/**
 * Extract Mermaid code blocks from markdown.
 *
 * @public
 * @param markdown - Markdown content to parse
 * @returns Array of Mermaid diagram code (without the code fence markers)
 *
 * @example
 * ```typescript
 * const markdown = '```mermaid\ngraph LR\nA --> B\n```';
 * const diagrams = extractMermaidBlocks(markdown);
 * // Returns: ['graph LR\nA --> B']
 * ```
 */
export function extractMermaidBlocks(markdown: string): readonly string[] {
  const mermaidBlocks: string[] = [];
  const mermaidRegex = /```mermaid\n([\s\S]*?)```/gi;
  let match;

  while ((match = mermaidRegex.exec(markdown)) !== null) {
    mermaidBlocks.push(match[1].trim());
  }

  return mermaidBlocks;
}

/**
 * Extract code blocks from markdown.
 *
 * @public
 * @param markdown - Markdown content to parse
 * @param language - Optional language filter (e.g., 'javascript', 'typescript')
 * @returns Array of code blocks with language and code content
 *
 * @example
 * ```typescript
 * const markdown = '```typescript\nconst x = 1;\n```';
 * const blocks = extractCodeBlocks(markdown, 'typescript');
 * // Returns: [{ language: 'typescript', code: 'const x = 1;' }]
 * ```
 */
export function extractCodeBlocks(
  markdown: string,
  language?: string
): ReadonlyArray<{ readonly language: string | null; readonly code: string }> {
  const codeBlocks: Array<{ language: string | null; code: string }> = [];
  const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/gi;
  let match;

  while ((match = codeBlockRegex.exec(markdown)) !== null) {
    const blockLanguage = match[1] || null;
    const code = match[2].trim();

    if (!language || blockLanguage === language) {
      codeBlocks.push({ language: blockLanguage, code });
    }
  }

  return codeBlocks;
}

/**
 * Sanitize markdown content by removing potentially dangerous HTML.
 *
 * @public
 * @param markdown - Markdown content to sanitize
 * @returns Sanitized markdown with dangerous HTML removed
 *
 * @remarks
 * Removes:
 * - Script tags
 * - Iframe tags
 * - JavaScript: protocol links
 *
 * Preserves markdown syntax and safe HTML.
 *
 * @example
 * ```typescript
 * const unsafe = '<script>alert("xss")</script># Hello';
 * const safe = sanitizeMarkdown(unsafe);
 * // Returns: '# Hello'
 * ```
 */
export function sanitizeMarkdown(markdown: string): string {
  // Remove potentially dangerous HTML tags while preserving markdown syntax
  return markdown
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, "")
    .replace(/<iframe\b[^<]*(?:(?!<\/iframe>)<[^<]*)*<\/iframe>/gi, "")
    .replace(/javascript:/gi, "");
}

/**
 * Get word count from markdown (excluding code blocks and inline code).
 *
 * @public
 * @param markdown - Markdown content to analyze
 * @returns Word count (excluding code blocks)
 *
 * @remarks
 * Removes code blocks, inline code, and markdown syntax before counting words.
 *
 * @example
 * ```typescript
 * const count = getMarkdownWordCount('# Hello world\n```code```');
 * // Returns: 2 (only counts "Hello world")
 * ```
 */
export function getMarkdownWordCount(markdown: string): number {
  // Remove code blocks and inline code
  const text = markdown
    .replace(/```[\s\S]*?```/g, "")
    .replace(/`[^`]+`/g, "")
    .replace(/[#*_~`()[\]]/g, "")
    .trim();

  return text.split(/\s+/).filter((word) => word.length > 0).length;
}

/**
 * Get estimated reading time from markdown.
 *
 * @public
 * @param markdown - Markdown content to analyze
 * @param wordsPerMinute - Reading speed in words per minute (default: 200)
 * @returns Estimated reading time in minutes (rounded up)
 *
 * @example
 * ```typescript
 * const time = getReadingTime(markdown, 250);
 * console.log(`Estimated reading time: ${time} minutes`);
 * ```
 */
import { READING_TIME } from "./constants";

export function getReadingTime(
  markdown: string,
  wordsPerMinute: number = READING_TIME.DEFAULT_WPM
): number {
  const wordCount = getMarkdownWordCount(markdown);
  return Math.ceil(wordCount / wordsPerMinute);
}

/**
 * Format markdown for preview by normalizing line endings.
 *
 * @public
 * @param markdown - Raw markdown content
 * @returns Formatted markdown with normalized line endings
 *
 * @remarks
 * Converts Windows (CRLF) and old Mac (CR) line endings to Unix (LF) format.
 *
 * @example
 * ```typescript
 * const formatted = formatMarkdownForPreview('Line 1\r\nLine 2');
 * // Returns: 'Line 1\nLine 2'
 * ```
 */
export function formatMarkdownForPreview(markdown: string): string {
  // Normalize line endings
  return markdown.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

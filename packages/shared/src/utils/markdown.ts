// packages/shared/src/utils/markdown.ts
// Utility functions for markdown processing
// Non-React utilities that can be used across all apps

/**
 * Copy text to clipboard
 * @param text Text to copy
 * @returns Promise that resolves when copy is successful
 * @note Only works in browser environments (not Node.js)
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  if (!text) return false;

  // Check if we're in a browser environment
  if (typeof window === "undefined" || typeof document === "undefined") {
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
    } catch (err) {
      document.body.removeChild(textArea);
      return false;
    }
  } catch (err) {
    console.error("Failed to copy to clipboard:", err);
    return false;
  }
}

/**
 * Extract mermaid code blocks from markdown
 * @param markdown Markdown content
 * @returns Array of mermaid code blocks
 */
export function extractMermaidBlocks(markdown: string): string[] {
  const mermaidBlocks: string[] = [];
  const mermaidRegex = /```mermaid\n([\s\S]*?)```/gi;
  let match;

  while ((match = mermaidRegex.exec(markdown)) !== null) {
    mermaidBlocks.push(match[1].trim());
  }

  return mermaidBlocks;
}

/**
 * Extract code blocks from markdown
 * @param markdown Markdown content
 * @param language Optional language filter (e.g., 'javascript', 'typescript')
 * @returns Array of code blocks with language info
 */
export function extractCodeBlocks(
  markdown: string,
  language?: string
): Array<{ language: string | null; code: string }> {
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
 * Sanitize markdown content (basic sanitization)
 * @param markdown Markdown content
 * @returns Sanitized markdown
 */
export function sanitizeMarkdown(markdown: string): string {
  // Remove potentially dangerous HTML tags while preserving markdown syntax
  return markdown
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, "")
    .replace(/<iframe\b[^<]*(?:(?!<\/iframe>)<[^<]*)*<\/iframe>/gi, "")
    .replace(/javascript:/gi, "");
}

/**
 * Get word count from markdown (excluding code blocks)
 * @param markdown Markdown content
 * @returns Word count
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
 * Get estimated reading time from markdown
 * @param markdown Markdown content
 * @param wordsPerMinute Reading speed (default: 200)
 * @returns Estimated reading time in minutes
 */
export function getReadingTime(markdown: string, wordsPerMinute: number = 200): number {
  const wordCount = getMarkdownWordCount(markdown);
  return Math.ceil(wordCount / wordsPerMinute);
}

/**
 * Format markdown for preview (basic formatting)
 * @param markdown Raw markdown content
 * @returns Formatted markdown
 */
export function formatMarkdownForPreview(markdown: string): string {
  // Normalize line endings
  return markdown.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

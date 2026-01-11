// packages/ui/src/utils/markdown.ts
/**
 * Markdown to HTML converter for Sruja UI components.
 * Uses 'marked' library for proper markdown parsing with GFM (GitHub Flavored Markdown).
 */
import { marked } from "marked";

// Configure marked for GitHub Flavored Markdown (includes tables)
marked.setOptions({
  gfm: true, // Enable GitHub Flavored Markdown (tables, strikethrough, etc.)
  breaks: true, // Treat single newlines as <br>
});

/**
 * Convert markdown to HTML with support for:
 * - Tables (GFM)
 * - Code blocks with syntax highlighting classes
 * - Mermaid diagrams (rendered as divs, needs Mermaid.js to render)
 * - Headers, lists, bold, italic, links, etc.
 */
export async function markdownToHtml(markdown: string): Promise<string> {
  if (!markdown) return "";

  try {
    // Pre-process: Convert mermaid code blocks to divs that can be rendered by Mermaid.js
    const processedMarkdown = markdown.replace(
      /```mermaid\n([\s\S]*?)```/g,
      '<pre class="mermaid">$1</pre>'
    );

    // Parse markdown to HTML using marked
    const html = await marked.parse(processedMarkdown);
    return html;
  } catch (error) {
    console.error("Markdown parsing error:", error);
    // Fallback to escaped text
    return `<pre>${markdown.replace(/</g, "&lt;").replace(/>/g, "&gt;")}</pre>`;
  }
}

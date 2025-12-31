// packages/ui/src/utils/markdown.ts
/**
 * Simple markdown to HTML converter for Sruja UI components.
 * This is a basic implementation that handles headers, lists, code, and mermaid.
 */
export async function markdownToHtml(markdown: string): Promise<string> {
  if (!markdown) return "";

  // This is a placeholder for a real markdown parser.
  // In a real app, you'd use marked, micromark, or similar.
  // We'll do simple regex replacements for basic markdown.

  let html = markdown
    // Escape HTML special characters
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");

  // Headers
  html = html.replace(/^# (.*$)/gm, "<h1>$1</h1>");
  html = html.replace(/^## (.*$)/gm, "<h2>$1</h2>");
  html = html.replace(/^### (.*$)/gm, "<h3>$1</h3>");

  // Bold / Italic
  html = html.replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/\*(.*?)\*/g, "<em>$1</em>");

  // Lists
  html = html.replace(/^- (.*$)/gm, "<ul><li>$1</li></ul>");
  html = html.replace(/^\d+\. (.*$)/gm, "<ol><li>$1</li></ol>");
  // Cleanup adjacent lists (very basic)
  html = html.replace(/<\/ul>\n<ul>/g, "\n");
  html = html.replace(/<\/ol>\n<ol>/g, "\n");

  // Mermaid blocks
  html = html.replace(/```mermaid\n([\s\S]*?)```/g, '<div class="mermaid">$1</div>');

  // Code blocks
  html = html.replace(
    /```(\w+)?\n([\s\S]*?)```/g,
    '<pre><code class="language-$1">$2</code></pre>'
  );

  // Inline code
  html = html.replace(/`(.*?)`/g, "<code>$1</code>");

  // Paragraphs
  html = html.replace(/^(?!<[a-z])(.*)$/gm, "<p>$1</p>");
  html = html.replace(/<\/p>\n<p>/g, "<br/>");

  return html;
}

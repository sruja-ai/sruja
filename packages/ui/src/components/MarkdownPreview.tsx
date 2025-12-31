import { useEffect, useState } from "react";

export interface MarkdownPreviewProps {
  content: string;
  className?: string;
  onMermaidExpand?: (svg: string, code: string) => void;
}

export function MarkdownPreview({
  content,
  className = "",
  onMermaidExpand: _onMermaidExpand,
}: MarkdownPreviewProps) {
  const [html, setHtml] = useState<string>("");

  useEffect(() => {
    async function load() {
      try {
        const { markdownToHtml } = await import("../utils/markdown");
        const res = await markdownToHtml(content);
        setHtml(res);
      } catch {
        // Fallback to plain text if rendering fails
        setHtml(content);
      }
    }
    load();
  }, [content]);

  // Note: onMermaidExpand is used by parent components to handle diagram clicks
  // We don't use it directly here as we're just rendering the HTML
  // but we keep it in props for interface compatibility.

  return (
    <div className={`markdown-preview ${className}`} dangerouslySetInnerHTML={{ __html: html }} />
  );
}

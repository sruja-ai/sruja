import { useEffect, useRef, useState } from "react";
import mermaid from "mermaid";
import "./MarkdownPreview.css";

// Initialize mermaid with a nice theme
mermaid.initialize({
  startOnLoad: false,
  theme: "neutral",
  securityLevel: "loose",
  fontFamily: "inherit",
});

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
  const containerRef = useRef<HTMLDivElement>(null);

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

  // Render mermaid diagrams after HTML is set
  useEffect(() => {
    if (!html || !containerRef.current) return;

    const renderMermaid = async () => {
      const mermaidElements = containerRef.current?.querySelectorAll("pre.mermaid");
      if (!mermaidElements || mermaidElements.length === 0) return;

      for (let i = 0; i < mermaidElements.length; i++) {
        const element = mermaidElements[i] as HTMLElement;
        const code = element.textContent || "";

        try {
          const id = `mermaid-${Date.now()}-${i}`;
          const { svg } = await mermaid.render(id, code);
          element.innerHTML = svg;
          element.classList.remove("mermaid");
          element.classList.add("mermaid-rendered");
        } catch (error) {
          console.warn("Mermaid render failed:", error);
          // Keep the code block as-is if mermaid fails
          element.classList.add("mermaid-error");
        }
      }
    };

    renderMermaid();
  }, [html]);

  return (
    <div
      ref={containerRef}
      className={`markdown-preview ${className}`}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

import { useCallback, useEffect, useRef, useState } from "react";
import mermaid from "mermaid";
import { getIsDark, getMermaidConfig } from "../utils/mermaidTheme";
import "./MarkdownPreview.css";
import "./MermaidDiagram.css";

// Initial config (will be overridden when component runs with current theme)
mermaid.initialize(getMermaidConfig(getIsDark()));

export interface MarkdownPreviewProps {
  content: string;
  className?: string;
  onMermaidExpand?: (svg: string, code: string) => void;
}

/** Store mermaid code in a data attribute (base64) so we can re-render on theme change. */
const DATA_MERMAID_CODE = "data-mermaid-code";

function getStoredCode(container: HTMLElement): string {
  const raw = container.getAttribute(DATA_MERMAID_CODE);
  if (!raw) return "";
  try {
    return decodeURIComponent(escape(atob(raw)));
  } catch {
    return "";
  }
}

function setStoredCode(container: HTMLElement, code: string): void {
  try {
    container.setAttribute(DATA_MERMAID_CODE, btoa(unescape(encodeURIComponent(code))));
  } catch {
    // ignore if code is not encodable
  }
}

export function MarkdownPreview({
  content,
  className = "",
  onMermaidExpand: _onMermaidExpand,
}: MarkdownPreviewProps) {
  const [html, setHtml] = useState<string>("");
  const containerRef = useRef<HTMLDivElement>(null);
  const lastThemeRef = useRef<boolean | null>(null);

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

  const renderAllMermaid = useCallback(async (root: HTMLElement) => {
    const isDark = getIsDark();
    mermaid.initialize(getMermaidConfig(isDark));
    lastThemeRef.current = isDark;

    const preMermaid = root.querySelectorAll("pre.mermaid");
    const codeMermaid = root.querySelectorAll("pre > code.language-mermaid");

    const blocks: { container: HTMLElement; code: string }[] = [];

    preMermaid.forEach((el) => {
      const pre = el as HTMLElement;
      blocks.push({ container: pre, code: pre.textContent || "" });
    });

    codeMermaid.forEach((el) => {
      const codeEl = el as HTMLElement;
      const pre = codeEl.parentElement;
      if (pre && !blocks.some((b) => b.container === pre)) {
        blocks.push({ container: pre as HTMLElement, code: codeEl.textContent || "" });
      }
    });

    for (let i = 0; i < blocks.length; i++) {
      const { container, code } = blocks[i];
      if (!code.trim()) continue;

      try {
        const id = `mermaid-${Date.now()}-${i}`;
        const { svg } = await mermaid.render(id, code);
        container.innerHTML = svg;
        container.classList.remove("mermaid");
        container.classList.add("mermaid-rendered", "mermaid-diagram-wrapper");
        setStoredCode(container, code);
      } catch (error) {
        console.warn("Mermaid render failed:", error);
        container.classList.add("mermaid-error");
      }
    }
  }, []);

  // Render mermaid diagrams after HTML is set.
  // Support both: pre.mermaid (pre-processed) and pre > code.language-mermaid (marked output).
  useEffect(() => {
    if (!html || !containerRef.current) return;

    renderAllMermaid(containerRef.current);
  }, [html, renderAllMermaid]);

  // Re-render diagrams when theme changes so they use the correct theme.
  useEffect(() => {
    const handleThemeChange = () => {
      const root = containerRef.current;
      if (!root) return;

      const isDark = getIsDark();
      if (lastThemeRef.current === isDark) return;

      const rendered = root.querySelectorAll(".mermaid-rendered");
      if (rendered.length === 0) return;

      mermaid.initialize(getMermaidConfig(isDark));
      lastThemeRef.current = isDark;

      rendered.forEach(async (el, i) => {
        const container = el as HTMLElement;
        const code = getStoredCode(container);
        if (!code.trim()) return;

        try {
          const id = `mermaid-theme-${Date.now()}-${i}`;
          const { svg } = await mermaid.render(id, code);
          container.innerHTML = svg;
        } catch (error) {
          console.warn("Mermaid re-render on theme change failed:", error);
        }
      });
    };

    window.addEventListener("theme-change", handleThemeChange);
    return () => window.removeEventListener("theme-change", handleThemeChange);
  }, []);

  return (
    <div
      ref={containerRef}
      className={`markdown-preview ${className}`}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

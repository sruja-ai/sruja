import { useEffect, useMemo, useRef, useState } from "react";
import { logger } from "@sruja/shared";
import { getIsDark, getMermaidConfig } from "../utils/mermaidTheme";
import "./MermaidDiagram.css";

export interface MermaidDiagramProps {
  code: string;
  onExpand?: (svg: string, code: string) => void;
}

/** Initialize mermaid with Sruja design-system theme (light/dark). Re-initializes when theme may have changed. */
async function initMermaid() {
  try {
    const mermaidModule = await import("mermaid");
    const mermaid = mermaidModule.default || mermaidModule;
    if (typeof mermaid.initialize === "function") {
      const isDark = getIsDark();
      const config = getMermaidConfig(isDark);
      mermaid.initialize({
        ...config,
        flowchart: {
          ...config.flowchart,
          useMaxWidth: true,
          subGraphTitleMargin: { top: 10, bottom: 20 },
        },
      });
    }
  } catch (err) {
    console.error("Failed to initialize Mermaid:", err);
  }
}

export function MermaidDiagram({ code, onExpand }: MermaidDiagramProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [svgContent, setSvgContent] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const renderId = useMemo(() => `mermaid-${Math.random().toString(36).slice(2)}`, []);

  useEffect(() => {
    let active = true;
    async function render() {
      const diagramCode = code || "graph TD; A-->B;";
      setIsLoading(true);
      setError(null);

      try {
        // Initialize mermaid first
        await initMermaid();

        // Import and use mermaid
        const mermaidModule = await import("mermaid");
        const mermaid = mermaidModule.default || mermaidModule;

        // Parse and render the diagram
        await mermaid.parse(diagramCode);

        // Render the diagram - don't pass container to avoid conflicts with dangerouslySetInnerHTML
        // mermaid will create a temporary container and return the SVG
        const result = await mermaid.render(renderId, diagramCode);

        if (!active) return;
        const svg = result?.svg || "";

        if (!svg || svg.trim() === "") {
          throw new Error("Mermaid rendered empty SVG");
        }

        setSvgContent(svg);
        setIsLoading(false);
      } catch (e) {
        const errorMsg = e instanceof Error ? e.message : String(e);
        logger.error("Mermaid render error", {
          component: "mermaid",
          action: "render",
          errorType: e instanceof Error ? e.constructor.name : "unknown",
          error: errorMsg,
          code: diagramCode.substring(0, 100), // Log first 100 chars of code
        });
        setError(errorMsg);
        setSvgContent(
          `<svg xmlns="http://www.w3.org/2000/svg" width="400" height="60"><text x="10" y="35" fill="red">Mermaid render failed: ${errorMsg}</text></svg>`
        );
        setIsLoading(false);
      }
    }
    render();
    return () => {
      active = false;
    };
  }, [code, renderId]);

  const handleExpand = (e: React.MouseEvent) => {
    if (!onExpand) return;
    e.preventDefault();
    e.stopPropagation();
    onExpand(svgContent, code);
  };

  return (
    <div className="relative">
      {isLoading && (
        <div className="mermaid-loading flex items-center justify-center p-4 text-[var(--color-text-secondary)]">
          <div className="animate-spin mr-2 h-4 w-4 border-2 border-[var(--color-border)] border-t-[var(--color-primary)] rounded-full" />
          Loading diagram...
        </div>
      )}
      <div
        ref={containerRef}
        className="mermaid-diagram-wrapper"
        style={{ display: isLoading ? "none" : "block" }}
        dangerouslySetInnerHTML={{ __html: svgContent }}
      />
      {error && !isLoading && (
        <div className="mermaid-error p-4 text-red-500 text-sm">Error: {error}</div>
      )}
      {onExpand && !isLoading && !error && (
        <button
          type="button"
          onClick={handleExpand}
          className="absolute top-2 right-2 inline-flex items-center rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-2 py-1 text-xs text-[var(--color-text-primary)] shadow-sm hover:opacity-90"
        >
          Expand
        </button>
      )}
    </div>
  );
}

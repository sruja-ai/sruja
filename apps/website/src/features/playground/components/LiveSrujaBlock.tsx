// apps/website/src/features/playground/components/LiveSrujaBlock.tsx
import { useEffect, useState, useCallback, useRef } from "react";
import { SrujaMonacoEditor } from "@sruja/ui";
import { SrujaLoader } from "@sruja/ui";
import { initWasm, logger } from "@sruja/shared";
import { trackEvent, trackInteraction } from "@/shared/utils/analytics";
import type { SrujaModelDump, DotResult } from "@sruja/shared";
import { SrujaDiagramPreview } from "./SrujaDiagramPreview";

export default function LiveSrujaBlock({ initialDsl }: { initialDsl: string }) {
  const [dsl, setDsl] = useState(() => {
    // If the input is already clean (no pervasive indentation), return it
    if (!initialDsl.includes("\n")) return initialDsl;

    const lines = initialDsl.split("\n");
    // Ignore first/last empty lines often caused by template literals
    let start = 0;
    while (start < lines.length && lines[start].trim() === "") start++;
    let end = lines.length - 1;
    while (end >= start && lines[end].trim() === "") end--;

    if (start > end) return ""; // All empty

    const relevantLines = lines.slice(start, end + 1);

    // Calculate minimum indentation of non-empty lines
    const minIndent = relevantLines.reduce((min, line) => {
      if (line.trim() === "") return min;
      const match = line.match(/^(\s*)/);
      return Math.min(min, match ? match[1].length : 0);
    }, Infinity);

    // If unreasonable indent (e.g. infinite or 0), just return trimmed
    if (minIndent === Infinity || minIndent === 0) return relevantLines.join("\n");

    return relevantLines
      .map((line) => (line.length >= minIndent ? line.slice(minIndent) : line))
      .join("\n");
  });
  const [data, setData] = useState<SrujaModelDump | null>(null);
  const [dotResult, setDotResult] = useState<DotResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [errorHeader, setErrorHeader] = useState<string | null>(null);
  const isInitialRender = useRef(true);

  const renderDiagram = useCallback(async () => {
    setBusy(true);
    setErrorHeader(null);

    try {
      const normalize = (s: string) => {
        const basic = s
          .replace(/\u2192/g, "->")
          .replace(/[“”]/g, '"')
          .replace(/[’]/g, "'")
          .replace(/\u2013|\u2014/g, "-");

        // Split and trim trailing whitespace, but keep leading whitespace for indentation
        return basic
          .split(/\r?\n/)
          .map((line) => line.replace(/^\s*\d+\s*[→:.-]\s?/, ""))
          .join("\n")
          .trim();
      };
      const input = normalize(dsl);
      const api = await initWasm();

      // Get model dump for general info
      const jsonStr = await api.dslToModel(input);
      const parsed = JSON.parse(jsonStr) as SrujaModelDump;
      setData(parsed);

      // Get DOT result for layout/rendering
      try {
        const dot = await api.dslToDot(input);
        setDotResult(dot);
      } catch (mErr) {
        logger.warn("Failed to generate DOT for playground", { error: mErr });
      }

      if (!isInitialRender.current) {
        trackInteraction("success", "render_diagram", { component: "playground" });
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      setErrorHeader(msg);
      logger.error("Failed to render diagram", {
        component: "playground",
        action: "render",
        errorType: error instanceof Error ? error.constructor.name : "unknown",
        error: msg,
      });
    } finally {
      setBusy(false);
      isInitialRender.current = false;
    }
  }, [dsl]);

  // Initial render
  useEffect(() => {
    renderDiagram();
    trackEvent("live.render_view");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Debounced auto-render on change
  useEffect(() => {
    const timer = setTimeout(() => {
      if (!isInitialRender.current) {
        renderDiagram();
      }
    }, 1000); // 1s debounce for live preview
    return () => clearTimeout(timer);
  }, [dsl, renderDiagram]);

  const [theme, setTheme] = useState<"vs" | "vs-dark" | "hc-black">(() =>
    typeof document !== "undefined" &&
    document.documentElement.getAttribute("data-theme") === "dark"
      ? "vs-dark"
      : "vs"
  );
  useEffect(() => {
    const handler = () => {
      setTheme(document.documentElement.getAttribute("data-theme") === "dark" ? "vs-dark" : "vs");
    };
    try {
      window.addEventListener("theme-change", handler);
    } catch {
      void 0;
    }
    return () => {
      try {
        window.removeEventListener("theme-change", handler);
      } catch {
        void 0;
      }
    };
  }, []);

  return (
    <div
      style={{ border: "1px solid var(--color-border)", borderRadius: 8, overflow: "hidden" }}
      data-testid="viewer-editor"
    >
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr) minmax(0, 1fr)",
          gap: 0,
          minHeight: 640,
        }}
      >
        <div style={{ borderRight: "1px solid var(--color-border)", position: "relative" }}>
          <SrujaMonacoEditor
            value={dsl}
            onChange={(v) => setDsl(v || "")}
            theme={theme}
            options={{
              minimap: { enabled: false },
              wordWrap: "on",
              fontSize: 14,
              tabSize: 2,
              insertSpaces: true,
              detectIndentation: false,
            }}
            height="640px"
          />
        </div>
        <div style={{ position: "relative", height: 640 }}>
          {errorHeader && (
            <div
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                right: 0,
                padding: 8,
                background: "#fee2e2",
                color: "#b91c1c",
                fontSize: 13,
                borderBottom: "1px solid #fecaca",
                zIndex: 10,
              }}
            >
              Failed to render: {errorHeader}
            </div>
          )}
          {dotResult ? (
            <SrujaDiagramPreview model={data!} dotResult={dotResult} />
          ) : (
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                height: "100%",
                color: "var(--color-text-secondary)",
              }}
            >
              {busy ? <SrujaLoader size={32} /> : <div>Click Render to view diagram</div>}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

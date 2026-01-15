// apps/website/src/features/playground/components/LiveSrujaBlock.tsx
import { useEffect, useState, useCallback, useRef } from "react";
import { SrujaMonacoEditor } from "@sruja/ui";
import { SrujaLoader } from "@sruja/ui";
import { initWasm, logger } from "@sruja/shared";
import { trackEvent, trackInteraction } from "@/shared/utils/analytics";
import type { SrujaModelDump, DotResult } from "@sruja/shared";
import { SrujaDiagramPreview } from "./SrujaDiagramPreview";

export default function LiveSrujaBlock({ initialDsl }: { initialDsl: string }) {
  // Helper to normalize indentation
  const normalizeDsl = useCallback((input: string) => {
    // If the input is already clean (no pervasive indentation), return it
    if (!input.includes("\n")) return input.trim();

    const lines = input.split("\n");
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

    // Only strip indentation if ALL non-empty lines have the same leading indentation
    if (minIndent === Infinity || minIndent === 0) {
      return relevantLines.join("\n");
    }

    // Check if all non-empty lines start with at least minIndent spaces
    const allHaveMinIndent = relevantLines.every((line) => {
      if (line.trim() === "") return true;
      return line.length >= minIndent && line.slice(0, minIndent).trim() === "";
    });

    if (!allHaveMinIndent) {
      // Relative indentation exists, preserve it
      return relevantLines.join("\n");
    }

    // All lines have the same leading indentation, strip it
    return relevantLines
      .map((line) => (line.length >= minIndent ? line.slice(minIndent) : line))
      .join("\n");
  }, []);

  const [dsl, setDsl] = useState(() => normalizeDsl(initialDsl));

  // React to prop changes (e.g. HMR or parent updates)
  useEffect(() => {
    setDsl(normalizeDsl(initialDsl));
  }, [initialDsl, normalizeDsl]);
  const [data, setData] = useState<SrujaModelDump | null>(null);
  const [dotResult, setDotResult] = useState<DotResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [errorHeader, setErrorHeader] = useState<string | null>(null);
  const isInitialRender = useRef(true);
  const dslRef = useRef(dsl);

  // Keep ref in sync with state
  useEffect(() => {
    dslRef.current = dsl;
  }, [dsl]);

  const renderDiagram = useCallback(async (inputDsl: string) => {
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
      const input = normalize(inputDsl);
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
  }, []);

  // Initial render
  useEffect(() => {
    renderDiagram(dsl);
    trackEvent("live.render_view");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const pendingUpdateRef = useRef<boolean>(false);

  // Debounced auto-render on change
  useEffect(() => {
    // Flag that we have a pending update
    pendingUpdateRef.current = true;

    const timer = setTimeout(() => {
      if (busy) {
        // If busy, we rely on the effect below to pick it up later
        return;
      }

      if (!isInitialRender.current) {
        pendingUpdateRef.current = false;
        renderDiagram(dslRef.current);
      }
    }, 1500); // 1.5s delay to allow typing to finish

    return () => clearTimeout(timer);
  }, [dsl, busy, renderDiagram]);

  // Effect to retry rendering if we were busy when the timeout fired
  useEffect(() => {
    if (!busy && pendingUpdateRef.current && !isInitialRender.current) {
      pendingUpdateRef.current = false;
      renderDiagram(dslRef.current);
    }
  }, [busy, renderDiagram]);

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
              formatOnPaste: true,
              formatOnType: false,
              autoIndent: "full",
              trimAutoWhitespace: true,
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

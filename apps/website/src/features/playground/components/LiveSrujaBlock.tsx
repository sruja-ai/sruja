// apps/website/src/features/playground/components/LiveSrujaBlock.tsx
import { useEffect, useState, useCallback, useRef } from "react";
import { SrujaMonacoEditor } from "@sruja/ui";
import { SrujaLoader } from "@sruja/ui";
import { initWasm, logger } from "@sruja/shared";
import { trackEvent, trackInteraction } from "@/shared/utils/analytics";
import type { SrujaModelDump } from "@sruja/shared";
import { SrujaDiagramPreview } from "./SrujaDiagramPreview";

export default function LiveSrujaBlock({ initialDsl }: { initialDsl: string }) {
  // Dedent: strip the minimum leading indentation so the block has no excess left margin.
  // Handles template literals where the first line has no indent (e.g. `\n  code`) and the rest do.
  const normalizeDsl = useCallback((input: string) => {
    if (!input.includes("\n")) return input.trim();

    const lines = input.split("\n");
    let start = 0;
    while (start < lines.length && lines[start].trim() === "") start++;
    let end = lines.length - 1;
    while (end >= start && lines[end].trim() === "") end--;

    if (start > end) return "";

    const relevantLines = lines.slice(start, end + 1);

    // Minimum indentation among lines that have any leading space.
    const leadingCounts = relevantLines
      .filter((line) => line.trim() !== "")
      .map((line) => line.match(/^(\s*)/)?.[1].length ?? 0);
    const minIndent =
      leadingCounts.length === 0 ? 0 : Math.min(...leadingCounts.filter((n) => n > 0), Infinity);
    // Only strip template-literal indent (>= 4 spaces). Never strip 2-space code indent.
    const strip = minIndent !== Infinity && minIndent >= 4 ? minIndent : 0;

    return relevantLines
      .map((line) => {
        const leading = line.match(/^(\s*)/)?.[1].length ?? 0;
        if (strip > 0 && leading >= strip) return line.slice(strip);
        return line.trimStart();
      })
      .join("\n");
  }, []);

  const [dsl, setDsl] = useState(() => normalizeDsl(initialDsl));

  // React to prop changes (e.g. HMR or parent updates)
  useEffect(() => {
    setDsl(normalizeDsl(initialDsl));
  }, [initialDsl, normalizeDsl]);
  const [data, setData] = useState<SrujaModelDump | null>(null);
  const [mermaidCode, setMermaidCode] = useState<string | null>(null);
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

      // Get Mermaid for diagram (use existing tool)
      try {
        const mermaid = await api.dslToMermaid(input);
        setMermaidCode(mermaid || null);
      } catch (mErr) {
        logger.warn("Failed to generate Mermaid for playground", { error: mErr });
        setMermaidCode(null);
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
    (document.documentElement.getAttribute("data-theme") === "dark" ||
      document.documentElement.classList.contains("dark"))
      ? "vs-dark"
      : "vs"
  );
  useEffect(() => {
    const getIsDark = () =>
      document.documentElement.getAttribute("data-theme") === "dark" ||
      document.documentElement.classList.contains("dark");

    const handler = () => {
      setTheme(getIsDark() ? "vs-dark" : "vs");
    };
    try {
      window.addEventListener("theme-change", handler);
    } catch {
      void 0;
    }

    // Also react to class/attribute toggles (e.g. `html.dark`)
    let observer: MutationObserver | null = null;
    try {
      observer = new MutationObserver(() => handler());
      observer.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ["class", "data-theme"],
      });
    } catch {
      observer = null;
    }

    return () => {
      try {
        window.removeEventListener("theme-change", handler);
      } catch {
        void 0;
      }
      try {
        observer?.disconnect();
      } catch {
        void 0;
      }
    };
  }, []);

  return (
    <div
      style={{
        border: "1px solid var(--color-border)",
        borderRadius: 8,
        overflow: "hidden",
        background: "var(--color-surface, #ffffff)",
        color: "var(--color-text-primary, #0f172a)",
      }}
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
            enableLsp={false}
            options={{
              minimap: { enabled: false },
              wordWrap: "on",
              fontSize: 14,
              tabSize: 2,
              insertSpaces: true,
              detectIndentation: false,
              formatOnPaste: false,
              formatOnType: false,
              autoIndent: "none",
              trimAutoWhitespace: true,
              glyphMargin: false,
              lineNumbersMinChars: 2,
              lineDecorationsWidth: 0,
              // Keep folding, but don’t reserve extra space for experimental features
              stickyScroll: { enabled: false },
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
          {mermaidCode ? (
            <SrujaDiagramPreview model={data!} mermaidCode={mermaidCode} />
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

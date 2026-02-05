// Mermaid-only diagram view: uses existing tool (mermaid.js), no custom layout.
import { forwardRef, useImperativeHandle, useRef, useEffect, useState } from "react";
import { useArchitectureStore } from "../../stores";
import { convertDslToMermaid } from "../../wasm";
import { convertModelToDsl } from "../../utils/modelToDsl";
import { MermaidDiagram } from "@sruja/ui";
import type { CanvasHandle } from "./types";

function downloadDataUrl(dataUrl: string, filename: string) {
  const a = document.createElement("a");
  a.href = dataUrl;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
}

export const SrujaCanvas = forwardRef<CanvasHandle | null>(function SrujaCanvas(_, ref) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [mermaidCode, setMermaidCode] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const dslSource = useArchitectureStore((s) => s.dslSource);
  const model = useArchitectureStore((s) => s.model);

  useEffect(() => {
    let cancelled = false;
    setError(null);

    const run = async () => {
      let dsl = (dslSource ?? "").trim();
      if (!dsl && model) {
        try {
          dsl = (await convertModelToDsl(model))?.trim() ?? "";
        } catch {
          dsl = "";
        }
      }
      if (!dsl) {
        if (!cancelled) setMermaidCode(null);
        return;
      }
      try {
        const code = await convertDslToMermaid(dsl);
        if (!cancelled && code) setMermaidCode(code);
        else if (!cancelled) setMermaidCode(null);
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
          setMermaidCode(null);
        }
      }
    };
    run();
    return () => {
      cancelled = true;
    };
  }, [dslSource, model]);

  useImperativeHandle(
    ref,
    () => ({
      exportAsPNG: async () => {
        const el = containerRef.current?.querySelector("svg");
        if (!el) return;
        const svgData = new XMLSerializer().serializeToString(el);
        const canvas = document.createElement("canvas");
        const ctx = canvas.getContext("2d");
        if (!ctx) return;
        const img = new Image();
        const svgBlob = new Blob([svgData], { type: "image/svg+xml;charset=utf-8" });
        const url = URL.createObjectURL(svgBlob);
        await new Promise<void>((resolve, reject) => {
          img.onload = () => {
            canvas.width = img.width;
            canvas.height = img.height;
            ctx.drawImage(img, 0, 0);
            URL.revokeObjectURL(url);
            downloadDataUrl(canvas.toDataURL("image/png"), "sruja-diagram.png");
            resolve();
          };
          img.onerror = () => {
            URL.revokeObjectURL(url);
            reject(new Error("Failed to export PNG"));
          };
          img.src = url;
        });
      },
      exportAsSVG: async () => {
        const el = containerRef.current?.querySelector("svg");
        if (!el) return;
        const svgData = new XMLSerializer().serializeToString(el);
        const blob = new Blob([svgData], { type: "image/svg+xml;charset=utf-8" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = "sruja-diagram.svg";
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
      },
      fitView: () => {},
      zoomToSelection: () => {},
      zoomToActualSize: () => {},
      focusNode: () => {},
    }),
    []
  );

  if (error) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          padding: 24,
          color: "var(--color-text-secondary)",
          background: "var(--color-background-secondary)",
        }}
      >
        <div>
          <strong>Diagram error</strong>
          <p style={{ marginTop: 8, fontSize: 14 }}>{error}</p>
        </div>
      </div>
    );
  }

  if (!mermaidCode || !mermaidCode.trim()) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          color: "var(--color-text-secondary)",
          background: "var(--color-background-secondary)",
        }}
      >
        <p>No diagram. Add architecture in Code or Builder.</p>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      style={{
        width: "100%",
        height: "100%",
        overflow: "auto",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 24,
        background: "var(--color-background-secondary)",
      }}
    >
      <div style={{ maxWidth: 900 }}>
        <MermaidDiagram code={mermaidCode} />
      </div>
    </div>
  );
});

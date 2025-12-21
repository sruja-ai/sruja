import { useImperativeHandle } from "react";
import type { RefObject } from "react";
import html2canvas from "html2canvas";
import type { SrujaModelDump } from "@sruja/shared";
import type { ArchitectureCanvasRef } from "./types";

/**
 * Configuration for export functionality.
 */
export interface ExportConfig {
  containerRef: RefObject<HTMLDivElement | null>;
  model: SrujaModelDump | null;
}

/**
 * Hook that provides export methods for LikeC4Canvas.
 * 
 * @param config - Export configuration
 * @param ref - Forwarded ref to expose methods
 * 
 * @remarks
 * Exposes methods for:
 * - PNG export (via html2canvas)
 * - SVG export (currently also uses html2canvas, outputs PNG)
 * - Viewport control methods (no-ops for API compatibility)
 */
export function useLikeC4CanvasExports(
  config: ExportConfig,
  ref: React.ForwardedRef<ArchitectureCanvasRef>
): void {
  const { containerRef, model } = config;

  useImperativeHandle(
    ref,
    () => ({
      exportAsPNG: async (filename?: string) => {
        if (!containerRef.current || !model) {
          throw new Error("Canvas not initialized");
        }
        try {
          const canvas = await html2canvas(containerRef.current, {
            useCORS: true,
            allowTaint: true,
            backgroundColor: null,
            ignoreElements: (element) => {
              const classList = element.classList;
              return (
                classList.contains("likec4-view-selector") ||
                classList.contains("animation-controls-wrapper") ||
                classList.contains("react-flow__controls") ||
                classList.contains("react-flow__panel")
              );
            },
          });
          const url = canvas.toDataURL("image/png");
          const link = document.createElement("a");
          link.download = filename || `${model._metadata?.name || "diagram"}.png`;
          link.href = url;
          link.click();
        } catch (error) {
          console.error("Export PNG Error:", error);
          throw new Error(`Failed to export PNG: ${error instanceof Error ? error.message : "Unknown error"}`);
        }
      },
      exportAsSVG: async (filename?: string) => {
        // LikeC4 renders to canvas/SVG internally, but html2canvas converts to PNG.
        // For now, we export as PNG with a .png extension (not .svg) to avoid confusion.
        if (!containerRef.current || !model) {
          throw new Error("Canvas not initialized");
        }
        try {
          const canvas = await html2canvas(containerRef.current, {
            useCORS: true,
            allowTaint: true,
            backgroundColor: null,
            ignoreElements: (element) => {
              const classList = element.classList;
              return (
                classList.contains("likec4-view-selector") ||
                classList.contains("animation-controls-wrapper") ||
                classList.contains("react-flow__controls") ||
                classList.contains("react-flow__panel")
              );
            },
          });
          const url = canvas.toDataURL("image/png");
          const link = document.createElement("a");
          const baseFilename = filename || `${model._metadata?.name || "diagram"}`;
          // Remove .svg extension if present and add .png
          link.download = baseFilename.replace(/\.svg$/, "") + ".png";
          link.href = url;
          link.click();
        } catch (error) {
          console.error("Export SVG Error:", error);
          throw new Error(`Failed to export diagram: ${error instanceof Error ? error.message : "Unknown error"}`);
        }
      },
      getReactFlowInstance: () => null, // Not available in LikeC4
      fitView: () => {
        // LikeC4 handles view fitting internally via its own viewport management.
        // This method is a no-op for API compatibility with ReactFlow-based components.
        // TODO: Implement if LikeC4 exposes viewport control API
      },
      zoomToSelection: () => {
        // LikeC4 handles zoom internally via its own viewport management.
        // This method is a no-op for API compatibility with ReactFlow-based components.
        // TODO: Implement if LikeC4 exposes selection-based zoom API
      },
      zoomToActualSize: () => {
        // LikeC4 handles zoom internally via its own viewport management.
        // This method is a no-op for API compatibility with ReactFlow-based components.
        // TODO: Implement if LikeC4 exposes zoom-to-actual-size API
      },
      focusNode: (_nodeId: string) => {
        // LikeC4 handles node focus internally via its own viewport management.
        // This method is a no-op for API compatibility with ReactFlow-based components.
        // TODO: Implement if LikeC4 exposes node focus/scroll API
      },
    }),
    [model, containerRef]
  );
}


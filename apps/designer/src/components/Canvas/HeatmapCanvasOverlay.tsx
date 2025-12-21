// apps/designer/src/components/Canvas/HeatmapCanvasOverlay.tsx
import React, { useEffect, useRef } from "react";
import { useReactFlow, useStore } from "@xyflow/react";

interface HeatmapCanvasOverlayProps {
  nodeBadness: Record<string, number>;
  visible: boolean;
}

/**
 * HeatmapCanvasOverlay - Shows quality heatmap overlay on diagram
 * 
 * NOTE: This component requires ReactFlow and is not compatible with LikeC4 mode.
 * It should only be rendered when ReactFlow is available. The parent component
 * should check for ReactFlow availability before rendering this.
 */
export const HeatmapCanvasOverlay: React.FC<HeatmapCanvasOverlayProps> = ({
  nodeBadness,
  visible,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { getViewport } = useReactFlow();
  const transform = useStore((s) => s.transform);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const draw = () => {
      if (!visible) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        return;
      }

      const { zoom } = getViewport();

      // Handle high-DPI displays
      const dpr = window.devicePixelRatio || 1;
      const width = canvas.clientWidth * dpr;
      const height = canvas.clientHeight * dpr;

      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, canvas.clientWidth, canvas.clientHeight);

      Object.entries(nodeBadness).forEach(([id, bad]) => {
        if (bad < 0.05) return;

        // Find the node's DOM element
        const nodeEl = document.querySelector(`[data-id="${id}"]`);
        if (!nodeEl) return;

        const rect = nodeEl.getBoundingClientRect();

        // Get canvas bounding rect to convert screen coords to canvas local coords
        const canvasRect = canvas.getBoundingClientRect();

        const cx = rect.left - canvasRect.left + rect.width / 2;
        const cy = rect.top - canvasRect.top + rect.height / 2;

        const radius = 40 * zoom + 60 * bad * zoom;

        const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, radius);
        // Red for bad (1.0), flowing to transparent
        // Adjust opacity based on badness
        const opacity = 0.6 * bad;
        g.addColorStop(0, `rgba(255, 0, 0, ${opacity})`);
        g.addColorStop(1, "rgba(255, 0, 0, 0)");

        ctx.fillStyle = g;
        ctx.beginPath();
        ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.fill();
      });

      // Reset scale
      ctx.setTransform(1, 0, 0, 1, 0, 0);
    };

    const renderLoop = () => {
      draw();
    };

    renderLoop();

    return () => {
      // cleanup
    };
  }, [nodeBadness, visible, transform, getViewport]);

  if (!visible) return null;

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: "absolute",
        top: 0,
        left: 0,
        width: "100%",
        height: "100%",
        pointerEvents: "none",
        zIndex: 100, // Above nodes? Or below? Above for "overlay" feel.
      }}
    />
  );
};

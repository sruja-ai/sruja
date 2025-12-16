import React, { useEffect, useRef } from "react";
import { useReactFlow, useStore } from "@xyflow/react";

interface HeatmapCanvasOverlayProps {
  nodeBadness: Record<string, number>;
  visible: boolean;
}

export const HeatmapCanvasOverlay: React.FC<HeatmapCanvasOverlayProps> = ({
  nodeBadness,
  visible,
}) => {
  const { getViewport } = useReactFlow();
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Subscribe to transform changes to redraw efficiently
  // xyflow/react store subscription (internal but effective)
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

        // We need to find the node's DOM element OR rely on React Flow internal state.
        // DOM lookup is safer for exact screen position if we don't have node positions passed in.
        // However, accessing DOM in render loop is slow.
        // Better: Use `useStore` to get nodes?
        // Or: assume `nodeBadness` is computed from *current* nodes, but we need their positions.
        // To keep it simple/fast: DOM lookup for now, or pass nodes prop.

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

    // Draw on mount and updates
    // Use requestAnimationFrame for smoothness

    const renderLoop = () => {
      draw();
      // We don't need a continuous loop if we subscribe to events,
      // but for smooth dragging it's often easier.
      // For now, draw once per effect trigger (metrics/transform change).
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

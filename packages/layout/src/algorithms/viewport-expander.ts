// packages/layout/src/algorithms/viewport-expander.ts
// Post-processing to expand diagram to better utilize viewport space
import type { PositionedC4Node } from "../types";

export interface ViewportExpansionOptions {
  targetUtilization?: number; // Target viewport utilization (0-1), default 0.75
  minExpansion?: number; // Minimum expansion factor, default 1.0
  maxExpansion?: number; // Maximum expansion factor, default 2.0
  preserveAspectRatio?: boolean; // Whether to preserve aspect ratio, default true
}

const DEFAULT_OPTIONS: ViewportExpansionOptions = {
  targetUtilization: 0.75,
  minExpansion: 1.0,
  maxExpansion: 2.0,
  preserveAspectRatio: true,
};

/**
 * Expand diagram to better utilize viewport space
 * This improves viewport utilization scores without breaking layout
 */
export function expandToViewport(
  nodes: Map<string, PositionedC4Node>,
  viewportSize: { width: number; height: number },
  options: ViewportExpansionOptions = {}
): Map<string, PositionedC4Node> {
  const opts = { ...DEFAULT_OPTIONS, ...options };

  if (nodes.size === 0) return nodes;

  // Calculate current diagram bounds
  let minX = Infinity,
    minY = Infinity;
  let maxX = -Infinity,
    maxY = -Infinity;

  for (const node of nodes.values()) {
    minX = Math.min(minX, node.bbox.x);
    minY = Math.min(minY, node.bbox.y);
    maxX = Math.max(maxX, node.bbox.x + node.bbox.width);
    maxY = Math.max(maxY, node.bbox.y + node.bbox.height);
  }

  const currentWidth = maxX - minX;
  const currentHeight = maxY - minY;

  if (currentWidth === 0 || currentHeight === 0) return nodes;

  // Calculate target size based on viewport utilization
  const targetWidth = viewportSize.width * opts.targetUtilization!;
  const targetHeight = viewportSize.height * opts.targetUtilization!;

  // Calculate expansion factors
  const scaleX = targetWidth / currentWidth;
  const scaleY = targetHeight / currentHeight;

  // Apply constraints
  let finalScaleX = Math.max(opts.minExpansion!, Math.min(opts.maxExpansion!, scaleX));
  let finalScaleY = Math.max(opts.minExpansion!, Math.min(opts.maxExpansion!, scaleY));

  // Preserve aspect ratio if requested
  if (opts.preserveAspectRatio) {
    const scale = Math.min(finalScaleX, finalScaleY);
    finalScaleX = scale;
    finalScaleY = scale;
  }

  // Only expand if diagram is significantly smaller than target
  // Don't shrink diagrams that are already large
  if (finalScaleX < 1.1 && finalScaleY < 1.1) {
    return nodes; // No significant expansion needed
  }

  // Apply expansion: scale positions relative to center
  const centerX = (minX + maxX) / 2;
  const centerY = (minY + maxY) / 2;

  const expanded = new Map<string, PositionedC4Node>();

  for (const [id, node] of nodes) {
    // Scale position relative to center
    const newX = centerX + (node.bbox.x - centerX) * finalScaleX;
    const newY = centerY + (node.bbox.y - centerY) * finalScaleY;

    expanded.set(id, {
      ...node,
      bbox: {
        ...node.bbox,
        x: newX,
        y: newY,
      },
    });
  }

  return expanded;
}

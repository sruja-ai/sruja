// apps/designer/src/types/edgeData.ts
// Type definitions for React Flow edge data

import type { XYPosition } from "@xyflow/react";

/**
 * Edge label position
 */
export interface EdgeLabelPosition {
  x: number;
  y: number;
}

/**
 * Edge data structure for React Flow edges
 */
export interface EdgeData {
  label?: string;
  technology?: string;
  labelPosition?: EdgeLabelPosition;
  points?: XYPosition[];
  [key: string]: unknown;
}

/**
 * Type guard to check if edge has label position
 */
export function hasLabelPosition(data: unknown): data is { labelPosition: EdgeLabelPosition } {
  return (
    typeof data === "object" &&
    data !== null &&
    "labelPosition" in data &&
    typeof (data as { labelPosition: unknown }).labelPosition === "object" &&
    (data as { labelPosition: { x?: unknown; y?: unknown } }).labelPosition !== null &&
    typeof (data as { labelPosition: { x?: unknown } }).labelPosition.x === "number" &&
    typeof (data as { labelPosition: { y?: unknown } }).labelPosition.y === "number"
  );
}

/**
 * Type guard to check if edge has points
 */
export function hasPoints(data: unknown): data is { points: XYPosition[] } {
  return (
    typeof data === "object" &&
    data !== null &&
    "points" in data &&
    Array.isArray((data as { points: unknown }).points)
  );
}

/**
 * Safely get label position from edge data
 */
export function getLabelPosition(data: unknown): EdgeLabelPosition | null {
  if (hasLabelPosition(data)) {
    return data.labelPosition;
  }
  return null;
}

/**
 * Safely get points from edge data
 */
export function getEdgePoints(data: unknown): XYPosition[] | null {
  if (hasPoints(data)) {
    // Return null for empty arrays (considered as no valid points)
    return data.points.length > 0 ? data.points : null;
  }
  return null;
}

import type { PositionedNode } from "./algorithms/coordinates";
import type { C4Id } from "./brand";
import type { Rect } from "./geometry/rect";

// ============================================================================
// BEAUTIFY OPTIONS
// ============================================================================

export interface BeautifyOptions {
  /** Snap nodes to grid (default: false) */
  snapToGrid?: boolean;
  /** Grid size in pixels (default: 20) */
  gridSize?: number;
  /** Align sibling nodes at same hierarchy level (default: true) */
  alignNodes?: boolean;
  /** Distribute whitespace evenly around clusters (default: true) */
  balanceWhitespace?: boolean;
  /** Adjust container aspect ratios toward golden ratio (default: false) */
  goldenRatio?: boolean;
  /** Minimum margin between nodes (default: 20) */
  minMargin?: number;
}

export interface BeautifyResult {
  nodes: Map<C4Id, PositionedNode>;
  /** Changes made during beautification */
  changes: {
    aligned: number;
    whitespaceAdjusted: number;
    aspectAdjusted: number;
    snapped: number;
  };
}

const GOLDEN_RATIO = 1.618;
const DEFAULT_MIN_MARGIN = 20;

// ============================================================================
// MAIN BEAUTIFY FUNCTION
// ============================================================================

/**
 * Beautify positioned nodes by aligning, balancing whitespace, and adjusting proportions.
 */
export function beautify(
  positioned: Map<C4Id, PositionedNode>,
  opts: BeautifyOptions = {}
): BeautifyResult {
  const {
    snapToGrid = false,
    gridSize = 20,
    alignNodes = true,
    balanceWhitespace = true,
    goldenRatio = false,
    minMargin = DEFAULT_MIN_MARGIN,
  } = opts;

  // Clone nodes for immutability
  const result = new Map<C4Id, PositionedNode>();
  for (const [id, node] of positioned) {
    result.set(id, { ...node, bbox: { ...node.bbox } });
  }

  const changes = {
    aligned: 0,
    whitespaceAdjusted: 0,
    aspectAdjusted: 0,
    snapped: 0,
  };

  // 1. Align sibling nodes
  if (alignNodes) {
    changes.aligned = alignSiblings(result);
  }

  // 2. Balance whitespace
  if (balanceWhitespace) {
    changes.whitespaceAdjusted = balanceNodeWhitespace(result, minMargin);
  }

  // 3. Adjust container aspect ratios
  if (goldenRatio) {
    changes.aspectAdjusted = adjustAspectRatios(result);
  }

  // 4. Snap to grid (last, to preserve alignment)
  if (snapToGrid) {
    changes.snapped = snapNodesToGrid(result, gridSize);
  }

  return { nodes: result, changes };
}

// ============================================================================
// SIBLING ALIGNMENT
// ============================================================================

/**
 * Align sibling nodes (nodes with the same parent) along their center axes.
 * For nodes in the same row, align vertical centers.
 * For nodes in the same column, align horizontal centers.
 */
function alignSiblings(nodes: Map<C4Id, PositionedNode>): number {
  let aligned = 0;

  // Group nodes by parent
  const byParent = new Map<C4Id | undefined, PositionedNode[]>();
  for (const node of nodes.values()) {
    const key = node.parent?.id;
    if (!byParent.has(key)) byParent.set(key, []);
    byParent.get(key)!.push(node);
  }

  for (const siblings of byParent.values()) {
    if (siblings.length < 2) continue;

    // Find nodes that are approximately on the same row (within tolerance)
    const tolerance = 30; // pixels
    const rows = groupByProximity(siblings, (n) => n.y + n.bbox.height / 2, tolerance);

    for (const row of rows) {
      if (row.length < 2) continue;

      // Align vertical centers
      const avgCenterY = row.reduce((sum, n) => sum + n.y + n.bbox.height / 2, 0) / row.length;

      for (const node of row) {
        const currentCenterY = node.y + node.bbox.height / 2;
        if (Math.abs(currentCenterY - avgCenterY) > 2) {
          const deltaY = avgCenterY - currentCenterY;
          node.y += deltaY;
          node.bbox.y = node.y;
          aligned++;
        }
      }
    }

    // Similarly for columns
    const cols = groupByProximity(siblings, (n) => n.x + n.bbox.width / 2, tolerance);

    for (const col of cols) {
      if (col.length < 2) continue;

      // Align horizontal centers
      const avgCenterX = col.reduce((sum, n) => sum + n.x + n.bbox.width / 2, 0) / col.length;

      for (const node of col) {
        const currentCenterX = node.x + node.bbox.width / 2;
        if (Math.abs(currentCenterX - avgCenterX) > 2) {
          const deltaX = avgCenterX - currentCenterX;
          node.x += deltaX;
          node.bbox.x = node.x;
          aligned++;
        }
      }
    }
  }

  return aligned;
}

/**
 * Group nodes by proximity of a computed value.
 */
function groupByProximity<T>(items: T[], getValue: (item: T) => number, tolerance: number): T[][] {
  const sorted = [...items].sort((a, b) => getValue(a) - getValue(b));
  const groups: T[][] = [];
  let currentGroup: T[] = [];
  let lastValue = -Infinity;

  for (const item of sorted) {
    const value = getValue(item);
    if (value - lastValue > tolerance && currentGroup.length > 0) {
      groups.push(currentGroup);
      currentGroup = [];
    }
    currentGroup.push(item);
    lastValue = value;
  }

  if (currentGroup.length > 0) {
    groups.push(currentGroup);
  }

  return groups;
}

// ============================================================================
// WHITESPACE BALANCING
// ============================================================================

/**
 * Balance whitespace by ensuring even margins between sibling nodes.
 */
function balanceNodeWhitespace(nodes: Map<C4Id, PositionedNode>, minMargin: number): number {
  let adjusted = 0;

  // Group by parent
  const byParent = new Map<C4Id | undefined, PositionedNode[]>();
  for (const node of nodes.values()) {
    const key = node.parent?.id;
    if (!byParent.has(key)) byParent.set(key, []);
    byParent.get(key)!.push(node);
  }

  for (const siblings of byParent.values()) {
    if (siblings.length < 2) continue;

    // Sort by x position for horizontal balancing
    const sortedByX = [...siblings].sort((a, b) => a.x - b.x);

    // Calculate gaps between consecutive nodes
    const gaps: number[] = [];
    for (let i = 1; i < sortedByX.length; i++) {
      const prev = sortedByX[i - 1];
      const curr = sortedByX[i];
      const gap = curr.x - (prev.x + prev.bbox.width);
      gaps.push(gap);
    }

    if (gaps.length === 0) continue;

    // Target: uniform gaps (but at least minMargin)
    const avgGap = Math.max(minMargin, gaps.reduce((a, b) => a + b, 0) / gaps.length);

    // Adjust positions to achieve uniform gaps, keeping first node fixed
    for (let i = 1; i < sortedByX.length; i++) {
      const node = sortedByX[i];
      const prev = sortedByX[i - 1];
      const targetX = prev.x + prev.bbox.width + avgGap;

      if (Math.abs(node.x - targetX) > 5) {
        node.x = targetX;
        node.bbox.x = node.x;
        adjusted++;
      }
    }

    // Similarly for vertical (sort by y)
    const sortedByY = [...siblings].sort((a, b) => a.y - b.y);
    const vGaps: number[] = [];
    for (let i = 1; i < sortedByY.length; i++) {
      const prev = sortedByY[i - 1];
      const curr = sortedByY[i];
      const gap = curr.y - (prev.y + prev.bbox.height);
      if (gap > 0) vGaps.push(gap); // Only consider positive gaps
    }

    if (vGaps.length > 0) {
      const avgVGap = Math.max(minMargin, vGaps.reduce((a, b) => a + b, 0) / vGaps.length);

      for (let i = 1; i < sortedByY.length; i++) {
        const node = sortedByY[i];
        const prev = sortedByY[i - 1];
        // Only adjust if nodes are in same "column" (overlapping x ranges)
        if (rectsOverlapHorizontally(prev.bbox, node.bbox)) {
          const targetY = prev.y + prev.bbox.height + avgVGap;
          if (Math.abs(node.y - targetY) > 5) {
            node.y = targetY;
            node.bbox.y = node.y;
            adjusted++;
          }
        }
      }
    }
  }

  return adjusted;
}

function rectsOverlapHorizontally(a: Rect, b: Rect): boolean {
  return !(a.x + a.width < b.x || b.x + b.width < a.x);
}

// ============================================================================
// ASPECT RATIO ADJUSTMENT
// ============================================================================

/**
 * Adjust container aspect ratios toward the golden ratio (1.618).
 * Only affects container nodes that have children.
 */
function adjustAspectRatios(nodes: Map<C4Id, PositionedNode>): number {
  let adjusted = 0;

  // Find container nodes (those with children)
  const childrenByParent = new Map<C4Id, PositionedNode[]>();
  for (const node of nodes.values()) {
    if (node.parent?.id) {
      const parentId = node.parent.id;
      if (!childrenByParent.has(parentId)) {
        childrenByParent.set(parentId, []);
      }
      childrenByParent.get(parentId)!.push(node);
    }
  }

  for (const [parentId, children] of childrenByParent) {
    const parent = nodes.get(parentId);
    if (!parent) continue;

    const currentRatio = parent.bbox.width / parent.bbox.height;

    // If already close to golden ratio, skip
    if (Math.abs(currentRatio - GOLDEN_RATIO) < 0.2) continue;

    // Calculate the bounding box of children
    const childBounds = calculateBoundingBox(children);
    if (!childBounds) continue;

    // Adjust parent to be closer to golden ratio while containing children
    const targetRatio = GOLDEN_RATIO;
    const area = parent.bbox.width * parent.bbox.height;

    // Calculate new dimensions maintaining same area
    const newWidth = Math.sqrt(area * targetRatio);
    const newHeight = newWidth / targetRatio;

    // Ensure children still fit with padding
    const padding = 20;
    const minWidth = childBounds.width + padding * 2;
    const minHeight = childBounds.height + padding * 2;

    const finalWidth = Math.max(newWidth, minWidth);
    const finalHeight = Math.max(newHeight, minHeight);

    if (
      Math.abs(finalWidth - parent.bbox.width) > 10 ||
      Math.abs(finalHeight - parent.bbox.height) > 10
    ) {
      // Center the adjustment
      const dw = finalWidth - parent.bbox.width;
      const dh = finalHeight - parent.bbox.height;

      parent.bbox.x -= dw / 2;
      parent.bbox.y -= dh / 2;
      parent.bbox.width = finalWidth;
      parent.bbox.height = finalHeight;
      parent.x = parent.bbox.x;
      parent.y = parent.bbox.y;

      adjusted++;
    }
  }

  return adjusted;
}

function calculateBoundingBox(nodes: PositionedNode[]): Rect | null {
  if (nodes.length === 0) return null;

  let minX = Infinity,
    minY = Infinity;
  let maxX = -Infinity,
    maxY = -Infinity;

  for (const node of nodes) {
    minX = Math.min(minX, node.x);
    minY = Math.min(minY, node.y);
    maxX = Math.max(maxX, node.x + node.bbox.width);
    maxY = Math.max(maxY, node.y + node.bbox.height);
  }

  return {
    x: minX,
    y: minY,
    width: maxX - minX,
    height: maxY - minY,
  };
}

// ============================================================================
// GRID SNAPPING
// ============================================================================

/**
 * Snap node positions to a grid.
 */
function snapNodesToGrid(nodes: Map<C4Id, PositionedNode>, gridSize: number): number {
  let snapped = 0;

  for (const node of nodes.values()) {
    const newX = Math.round(node.x / gridSize) * gridSize;
    const newY = Math.round(node.y / gridSize) * gridSize;

    if (newX !== node.x || newY !== node.y) {
      node.x = newX;
      node.y = newY;
      node.bbox.x = newX;
      node.bbox.y = newY;
      snapped++;
    }
  }

  return snapped;
}

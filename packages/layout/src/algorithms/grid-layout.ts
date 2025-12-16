/**
 * Grid-based child layout for containers
 *
 * Instead of laying children horizontally (which makes containers too wide),
 * this arranges children in a grid with configurable max columns.
 */

import type { C4Id } from "../brand";
import type { Size } from "../types";

export interface GridLayoutResult {
  width: number;
  height: number;
  positions: Map<C4Id, { x: number; y: number }>;
}

export interface GridLayoutOptions {
  maxColumns?: number;
  nodeSpacing?: number;
  rowSpacing?: number;
}

/**
 * Lay out children in a grid pattern
 */
export function layoutGrid(
  nodes: { id: C4Id; size: Size }[],
  options: GridLayoutOptions = {}
): GridLayoutResult {
  const { maxColumns = 3, nodeSpacing = 40, rowSpacing = 40 } = options;

  if (nodes.length === 0) {
    return { width: 0, height: 0, positions: new Map() };
  }

  const positions = new Map<C4Id, { x: number; y: number }>();

  // Calculate optimal column count
  const cols = Math.min(maxColumns, nodes.length);
  const rows = Math.ceil(nodes.length / cols);

  // Find max dimensions per column and row
  // Optimized: use for loop instead of forEach for better performance
  const colWidths: number[] = Array(cols).fill(0);
  const rowHeights: number[] = Array(rows).fill(0);
  const nodesLength = nodes.length;

  for (let idx = 0; idx < nodesLength; idx++) {
    const node = nodes[idx];
    const col = idx % cols;
    const row = Math.floor(idx / cols);
    const nodeSize = node.size;
    colWidths[col] = Math.max(colWidths[col], nodeSize.width);
    rowHeights[row] = Math.max(rowHeights[row], nodeSize.height);
  }

  // Calculate column X positions
  const colX: number[] = [];
  let x = 0;
  for (let c = 0; c < cols; c++) {
    colX.push(x);
    x += colWidths[c] + nodeSpacing;
  }

  // Calculate row Y positions
  const rowY: number[] = [];
  let y = 0;
  for (let r = 0; r < rows; r++) {
    rowY.push(y);
    y += rowHeights[r] + rowSpacing;
  }

  // Position each node
  // Optimized: use for loop instead of forEach and cache calculations
  for (let idx = 0; idx < nodesLength; idx++) {
    const node = nodes[idx];
    const col = idx % cols;
    const row = Math.floor(idx / cols);

    // Center node within its cell
    const cellWidth = colWidths[col];
    const cellHeight = rowHeights[row];
    const nodeSize = node.size;
    const offsetX = (cellWidth - nodeSize.width) * 0.5; // Use multiplication instead of division
    const offsetY = (cellHeight - nodeSize.height) * 0.5;

    positions.set(node.id, {
      x: colX[col] + offsetX,
      y: rowY[row] + offsetY,
    });
  }

  // Calculate total dimensions
  // Optimized: use for loops instead of reduce for better performance
  let totalWidth = 0;
  for (let c = 0; c < cols; c++) {
    totalWidth += colWidths[c];
  }
  totalWidth += nodeSpacing * (cols - 1);

  let totalHeight = 0;
  for (let r = 0; r < rows; r++) {
    totalHeight += rowHeights[r];
  }
  totalHeight += rowSpacing * (rows - 1);

  return {
    width: Math.max(0, totalWidth),
    height: Math.max(0, totalHeight),
    positions,
  };
}

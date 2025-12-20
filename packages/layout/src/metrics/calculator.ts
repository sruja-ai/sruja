/**
 * Metrics Calculator - Computes layout performance metrics
 */

import type { LayoutMetrics, LayoutContext, LayoutNode, LayoutEdge } from "../core/types";

export function createMetricsCalculator() {
  return (context: LayoutContext): LayoutMetrics => {
    const nodes = Array.from(context.nodes.values());
    const edges = Array.from(context.edges.values());

    return {
      totalNodes: context.graph.nodes.size,
      visibleNodes: nodes.filter((n) => n.visible).length,
      totalEdges: context.graph.relationships.length,
      edgeCrossings: calculateEdgeCrossings(edges),
      totalEdgeLength: calculateTotalEdgeLength(edges),
      averageEdgeLength: calculateAverageEdgeLength(edges),
      edgeBends: calculateTotalBends(edges),
      averageBends: calculateAverageBends(edges),
      overlaps: calculateNodeOverlaps(nodes),
      containmentViolations: calculateContainmentViolations(nodes),
      aspectRatio: calculateAspectRatio(nodes),
      coverage: calculateViewportUtilization(nodes, context.options),
      uniformity: calculateUniformity(nodes),
      balance: calculateBalance(nodes),
      compactness: calculateCompactness(nodes, edges),
      processingTime: 0, // Will be set by engine
      memoryUsage: 0, // Will be set by engine
    };
  };
}

function calculateEdgeCrossings(edges: LayoutEdge[]): number {
  let crossings = 0;
  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      if (edgesCross(edges[i], edges[j])) {
        crossings++;
      }
    }
  }
  return crossings;
}

function calculateTotalEdgeLength(edges: LayoutEdge[]): number {
  return edges.reduce((sum, edge) => sum + edge.length, 0);
}

function calculateAverageEdgeLength(edges: LayoutEdge[]): number {
  if (edges.length === 0) return 0;
  return calculateTotalEdgeLength(edges) / edges.length;
}

function calculateTotalBends(edges: LayoutEdge[]): number {
  return edges.reduce((sum, edge) => sum + edge.bendCount, 0);
}

function calculateAverageBends(edges: LayoutEdge[]): number {
  if (edges.length === 0) return 0;
  return calculateTotalBends(edges) / edges.length;
}

function calculateNodeOverlaps(nodes: LayoutNode[]): number {
  let overlaps = 0;
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      if (nodesOverlap(nodes[i], nodes[j])) {
        overlaps++;
      }
    }
  }
  return overlaps;
}

function calculateContainmentViolations(nodes: LayoutNode[]): number {
  let violations = 0;
  for (const node of nodes) {
    if (!node.parent) continue;

    const parentBounds = node.parent.bbox;
    const childBounds = node.bbox;
    const padding = 50;

    if (
      childBounds.x < parentBounds.x + padding ||
      childBounds.y < parentBounds.y + padding ||
      childBounds.x + childBounds.width > parentBounds.x + parentBounds.width - padding ||
      childBounds.y + childBounds.height > parentBounds.y + parentBounds.height - padding
    ) {
      violations++;
    }
  }
  return violations;
}

function calculateAspectRatio(nodes: LayoutNode[]): number {
  if (nodes.length === 0) return 1;

  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, node.bbox.x);
    minY = Math.min(minY, node.bbox.y);
    maxX = Math.max(maxX, node.bbox.x + node.bbox.width);
    maxY = Math.max(maxY, node.bbox.y + node.bbox.height);
  }

  const width = maxX - minX;
  const height = maxY - minY;
  return height > 0 ? width / height : 1;
}

function calculateViewportUtilization(nodes: LayoutNode[], _options: any): number {
  if (nodes.length === 0) return 0;

  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, node.bbox.x);
    minY = Math.min(minY, node.bbox.y);
    maxX = Math.max(maxX, node.bbox.x + node.bbox.width);
    maxY = Math.max(maxY, node.bbox.y + node.bbox.height);
  }

  const diagramArea = (maxX - minX) * (maxY - minY);
  const viewportArea = 1920 * 1080; // Default viewport
  return diagramArea / viewportArea;
}

function calculateUniformity(nodes: LayoutNode[]): number {
  if (nodes.length < 2) return 1;

  // Calculate variance in node sizes
  const sizes = nodes.map((n) => n.bbox.width * n.bbox.height);
  const mean = sizes.reduce((sum, size) => sum + size, 0) / sizes.length;
  const variance = sizes.reduce((sum, size) => sum + Math.pow(size - mean, 2), 0) / sizes.length;

  // Convert to uniformity score (lower variance = higher uniformity)
  return Math.max(0, 1 - variance / (mean * mean));
}

function calculateBalance(nodes: LayoutNode[]): number {
  if (nodes.length === 0) return 1;

  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, node.bbox.x);
    minY = Math.min(minY, node.bbox.y);
    maxX = Math.max(maxX, node.bbox.x + node.bbox.width);
    maxY = Math.max(maxY, node.bbox.y + node.bbox.height);
  }

  const centerX = (minX + maxX) / 2;
  const centerY = (minY + maxY) / 2;

  // Calculate center of mass
  let totalX = 0,
    totalY = 0,
    totalArea = 0;
  for (const node of nodes) {
    const area = node.bbox.width * node.bbox.height;
    const nodeCenterX = node.bbox.x + node.bbox.width / 2;
    const nodeCenterY = node.bbox.y + node.bbox.height / 2;
    totalX += nodeCenterX * area;
    totalY += nodeCenterY * area;
    totalArea += area;
  }

  const centerOfMassX = totalX / totalArea;
  const centerOfMassY = totalY / totalArea;

  // Balance is inverse of distance between geometric center and center of mass
  const distance = Math.hypot(centerX - centerOfMassX, centerY - centerOfMassY);
  const maxDistance = Math.hypot(maxX - minX, maxY - minY) / 2;

  return Math.max(0, 1 - distance / maxDistance);
}

function calculateCompactness(nodes: LayoutNode[], _edges: LayoutEdge[]): number {
  if (nodes.length === 0) return 1;

  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, node.bbox.x);
    minY = Math.min(minY, node.bbox.y);
    maxX = Math.max(maxX, node.bbox.x + node.bbox.width);
    maxY = Math.max(maxY, node.bbox.y + node.bbox.height);
  }

  const boundingArea = (maxX - minX) * (maxY - minY);
  const nodeArea = nodes.reduce((sum, node) => sum + node.bbox.width * node.bbox.height, 0);

  // Compactness is ratio of node area to bounding area
  return boundingArea > 0 ? nodeArea / boundingArea : 1;
}

function edgesCross(edge1: LayoutEdge, edge2: LayoutEdge): boolean {
  if (
    edge1.source.id === edge2.source.id ||
    edge1.source.id === edge2.target.id ||
    edge1.target.id === edge2.source.id ||
    edge1.target.id === edge2.target.id
  ) {
    return false;
  }

  for (let i = 0; i < edge1.points.length - 1; i++) {
    for (let j = 0; j < edge2.points.length - 1; j++) {
      if (
        lineSegmentsIntersect(
          edge1.points[i],
          edge1.points[i + 1],
          edge2.points[j],
          edge2.points[j + 1]
        )
      ) {
        return true;
      }
    }
  }
  return false;
}

function nodesOverlap(node1: LayoutNode, node2: LayoutNode): boolean {
  return !(
    node1.bbox.x + node1.bbox.width <= node2.bbox.x ||
    node2.bbox.x + node2.bbox.width <= node1.bbox.x ||
    node1.bbox.y + node1.bbox.height <= node2.bbox.y ||
    node2.bbox.y + node2.bbox.height <= node1.bbox.y
  );
}

function lineSegmentsIntersect(p1: any, p2: any, p3: any, p4: any): boolean {
  const det = (p2.x - p1.x) * (p4.y - p3.y) - (p4.x - p3.x) * (p2.y - p1.y);
  if (Math.abs(det) < 1e-10) return false;

  const lambda = ((p4.y - p3.y) * (p4.x - p1.x) + (p3.x - p4.x) * (p4.y - p1.y)) / det;
  const gamma = ((p1.y - p2.y) * (p4.x - p1.x) + (p2.x - p1.x) * (p4.y - p1.y)) / det;

  return 0 < lambda && lambda < 1 && 0 < gamma && gamma < 1;
}

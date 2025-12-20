// packages/layout/src/algorithms/crossing-based-positioning.ts
// Node position adjustments based on actual edge crossing analysis
import type { Point } from "../geometry/point";
import type { PositionedNode } from "./coordinates";
import type { C4Id } from "../brand";
import type { C4Relationship } from "../c4-model";

export interface EdgeRoute {
  id: string;
  sourceId: string;
  targetId: string;
  path: Point[];
}

/**
 * Analyze which nodes are involved in the most crossings
 */
export function analyzeNodeCrossings(
  _nodes: Map<C4Id, PositionedNode>,
  edges: EdgeRoute[],
  pathsCrossFn: (
    path1: Point[],
    path2: Point[],
    from1?: string,
    to1?: string,
    from2?: string,
    to2?: string
  ) => boolean
): Map<C4Id, number> {
  const nodeCrossingCounts = new Map<C4Id, number>();

  // Count crossings per node
  // Each crossing involves 2 edges, and each edge has 2 nodes
  // So each crossing contributes to 4 nodes' counts
  // Optimized: compare each pair exactly once (i < j) and cache edge references
  const edgesLength = edges.length;
  for (let i = 0; i < edgesLength; i++) {
    const e1 = edges[i];
    const e1SourceId = e1.sourceId as C4Id;
    const e1TargetId = e1.targetId as C4Id;

    for (let j = i + 1; j < edgesLength; j++) {
      const e2 = edges[j];

      if (pathsCrossFn(e1.path, e2.path, e1.sourceId, e1.targetId, e2.sourceId, e2.targetId)) {
        // Both source and target nodes of crossing edges are involved
        // Weight by how many crossings each node is involved in
        // Optimized: cache IDs to avoid repeated type casting
        const e2SourceId = e2.sourceId as C4Id;
        const e2TargetId = e2.targetId as C4Id;

        nodeCrossingCounts.set(e1SourceId, (nodeCrossingCounts.get(e1SourceId) || 0) + 1);
        nodeCrossingCounts.set(e1TargetId, (nodeCrossingCounts.get(e1TargetId) || 0) + 1);
        nodeCrossingCounts.set(e2SourceId, (nodeCrossingCounts.get(e2SourceId) || 0) + 1);
        nodeCrossingCounts.set(e2TargetId, (nodeCrossingCounts.get(e2TargetId) || 0) + 1);
      }
    }
  }

  return nodeCrossingCounts;
}

/**
 * Find crossing clusters - groups of nodes that are involved in many crossings together
 */
export function findCrossingClusters(
  _nodes: Map<C4Id, PositionedNode>,
  edges: EdgeRoute[],
  pathsCrossFn: (
    path1: Point[],
    path2: Point[],
    from1?: string,
    to1?: string,
    from2?: string,
    to2?: string
  ) => boolean
): Array<{ nodes: C4Id[]; crossingCount: number }> {
  const clusters: Array<{ nodes: Set<C4Id>; crossingCount: number }> = [];

  // Group edges that cross each other
  // Optimized: compare each pair exactly once (i < j) and cache edge references
  const edgesLength = edges.length;
  for (let i = 0; i < edgesLength; i++) {
    const e1 = edges[i];
    for (let j = i + 1; j < edgesLength; j++) {
      const e2 = edges[j];

      if (pathsCrossFn(e1.path, e2.path, e1.sourceId, e1.targetId, e2.sourceId, e2.targetId)) {
        // Find or create cluster for these nodes
        // Optimized: cache IDs to avoid repeated type casting
        const e1SourceId = e1.sourceId as C4Id;
        const e1TargetId = e1.targetId as C4Id;
        const e2SourceId = e2.sourceId as C4Id;
        const e2TargetId = e2.targetId as C4Id;

        const clusterNodes = new Set<C4Id>([e1SourceId, e1TargetId, e2SourceId, e2TargetId]);

        // Try to merge with existing clusters
        let merged = false;
        for (const cluster of clusters) {
          // If clusters share nodes, merge them
          const hasOverlap = Array.from(clusterNodes).some((n) => cluster.nodes.has(n));
          if (hasOverlap) {
            clusterNodes.forEach((n) => cluster.nodes.add(n));
            cluster.crossingCount++;
            merged = true;
            break;
          }
        }

        if (!merged) {
          clusters.push({ nodes: clusterNodes, crossingCount: 1 });
        }
      }
    }
  }

  // Return sorted by crossing count
  return clusters
    .map((c) => ({ nodes: Array.from(c.nodes), crossingCount: c.crossingCount }))
    .sort((a, b) => b.crossingCount - a.crossingCount);
}

/**
 * Apply force-directed adjustments to nodes to reduce crossings
 * Nodes involved in many crossings are pushed away from each other
 */
export function adjustNodePositionsForCrossings(
  nodes: Map<C4Id, PositionedNode>,
  crossingCounts: Map<C4Id, number>,
  relationships: C4Relationship[],
  maxIterations: number = 3,
  stepSize: number = 15
): Map<C4Id, PositionedNode> {
  const result = new Map(nodes);

  // Build adjacency map (nodes connected by edges)
  const neighbors = new Map<C4Id, Set<C4Id>>();
  for (const rel of relationships) {
    const fromId = rel.from as C4Id;
    const toId = rel.to as C4Id;
    if (!neighbors.has(fromId)) neighbors.set(fromId, new Set());
    if (!neighbors.has(toId)) neighbors.set(toId, new Set());
    neighbors.get(fromId)!.add(toId);
    neighbors.get(toId)!.add(fromId);
  }

  // Only adjust nodes with significant crossings (8+)
  // Focus on the worst offenders to avoid disrupting the entire layout
  const allNodeCrossings = Array.from(result.entries())
    .map(([id, node]) => ({ id, node, crossings: crossingCounts.get(id) || 0 }))
    .filter((n) => n.crossings > 0)
    .sort((a, b) => b.crossings - a.crossings);

  // Take top 30% of nodes with crossings, or at least top 5
  const topCount = Math.max(5, Math.ceil(allNodeCrossings.length * 0.3));
  const highCrossingNodes = allNodeCrossings.slice(0, topCount);

  if (highCrossingNodes.length === 0) return result;

  for (let iter = 0; iter < maxIterations; iter++) {
    const forces = new Map<C4Id, { fx: number; fy: number }>();

    // Initialize forces only for high-crossing nodes
    for (const { id } of highCrossingNodes) {
      forces.set(id, { fx: 0, fy: 0 });
    }

    // Calculate repulsion forces between high-crossing nodes
    for (let i = 0; i < highCrossingNodes.length; i++) {
      for (let j = i + 1; j < highCrossingNodes.length; j++) {
        const n1 = highCrossingNodes[i];
        const n2 = highCrossingNodes[j];

        // Skip if they're neighbors (connected by edge) - we want to keep them reasonably close
        if (neighbors.get(n1.id)?.has(n2.id)) continue;

        // Skip if one is parent of the other (hierarchical relationship)
        const n1Node = result.get(n1.id);
        const n2Node = result.get(n2.id);
        if (n1Node?.parent?.id === n2.id || n2Node?.parent?.id === n1.id) continue;

        // Optimized: cache bbox references and use squared distance
        const n1Bbox = n1.node.bbox;
        const n2Bbox = n2.node.bbox;
        const dx = n2Bbox.x - n1Bbox.x;
        const dy = n2Bbox.y - n1Bbox.y;
        const distSq = dx * dx + dy * dy;
        const dist = Math.sqrt(distSq) || 1;

        // Repulsion force - scale based on crossing counts
        // For nodes with many crossings, be more aggressive
        const n1Crossings = n1.crossings;
        const n2Crossings = n2.crossings;
        const baseStrength = n1Crossings * n2Crossings * 0.5;
        const totalCrossings = n1Crossings + n2Crossings;
        const strength = Math.min(
          baseStrength,
          Math.max(30, totalCrossings) * 5 // Scale with total crossings
        );
        const force = strength / Math.max(distSq, 50); // Use distSq directly, prevent infinite force

        const invDist = 1 / dist; // Cache inverse distance
        const fx = dx * invDist * force;
        const fy = dy * invDist * force;

        const f1 = forces.get(n1.id)!;
        const f2 = forces.get(n2.id)!;
        if (f1 && f2) {
          f1.fx -= fx;
          f1.fy -= fy;
          f2.fx += fx;
          f2.fy += fy;
        }
      }
    }

    // Apply forces with damping - more conservative
    // Optimized: cache damping calculation and stepSize * damping
    const damping = 0.7 - iter * 0.1; // Less aggressive damping
    const stepDamping = stepSize * damping;
    const minMoveThreshold = 1;
    const containmentPadding = 20;

    for (const [id, force] of forces) {
      const node = result.get(id)!;
      const moveX = force.fx * stepDamping;
      const moveY = force.fy * stepDamping;

      // Only move if force is significant and movement is reasonable
      if (Math.abs(moveX) > minMoveThreshold || Math.abs(moveY) > minMoveThreshold) {
        // Limit maximum movement per iteration - allow more movement for high-crossing nodes
        const nodeCrossings = crossingCounts.get(id) || 0;
        const maxMove = nodeCrossings > 8 ? 50 : nodeCrossings > 5 ? 40 : 30;
        const clampedMoveX = Math.max(-maxMove, Math.min(maxMove, moveX));
        const clampedMoveY = Math.max(-maxMove, Math.min(maxMove, moveY));

        // Optimized: cache bbox reference
        const nodeBbox = node.bbox;
        const newX = nodeBbox.x + clampedMoveX;
        const newY = nodeBbox.y + clampedMoveY;

        // Check if node has parent - if so, ensure it stays within parent bounds
        let canMove = true;
        if (node.parent) {
          const parent = result.get(node.parent.id);
          if (parent) {
            // Optimized: cache parent boundaries
            const parentBbox = parent.bbox;
            const parentRight = parentBbox.x + parentBbox.width;
            const parentBottom = parentBbox.y + parentBbox.height;
            const nodeRight = newX + nodeBbox.width;
            const nodeBottom = newY + nodeBbox.height;
            const minX = parentBbox.x + containmentPadding;
            const minY = parentBbox.y + containmentPadding;
            const maxX = parentRight - containmentPadding;
            const maxY = parentBottom - containmentPadding;

            // Check if new position would be outside parent
            if (newX < minX || newY < minY || nodeRight > maxX || nodeBottom > maxY) {
              canMove = false;
            }
          }
        }

        if (canMove) {
          // Optimized: reuse bbox object if coordinates haven't changed
          result.set(id, {
            ...node,
            x: newX,
            y: newY,
            bbox:
              nodeBbox.x === newX && nodeBbox.y === newY
                ? nodeBbox
                : { ...nodeBbox, x: newX, y: newY },
          });
        }
      }
    }
  }

  return result;
}

/**
 * Increase spacing between nodes that have crossing edges
 */
export function increaseSpacingForCrossingNodes(
  nodes: Map<C4Id, PositionedNode>,
  edges: EdgeRoute[],
  pathsCrossFn: (
    path1: Point[],
    path2: Point[],
    from1?: string,
    to1?: string,
    from2?: string,
    to2?: string
  ) => boolean,
  minSpacing: number = 50
): Map<C4Id, PositionedNode> {
  const result = new Map(nodes);
  const nodePairs = new Map<string, number>(); // Track pairs with crossings

  // Identify node pairs involved in crossings
  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      const e1 = edges[i];
      const e2 = edges[j];

      if (pathsCrossFn(e1.path, e2.path, e1.sourceId, e1.targetId, e2.sourceId, e2.targetId)) {
        // Mark all node pairs from these edges
        const pairs = [
          [e1.sourceId, e2.sourceId],
          [e1.sourceId, e2.targetId],
          [e1.targetId, e2.sourceId],
          [e1.targetId, e2.targetId],
        ];

        for (const [n1, n2] of pairs) {
          if (n1 !== n2) {
            const key = n1 < n2 ? `${n1}-${n2}` : `${n2}-${n1}`;
            nodePairs.set(key, (nodePairs.get(key) || 0) + 1);
          }
        }
      }
    }
  }

  // Adjust spacing for pairs with crossings
  for (const [key, crossingCount] of nodePairs) {
    if (!key || typeof key !== 'string') {
      console.warn(`[CrossingBased] Invalid key, skipping:`, key);
      continue;
    }

    const parts = key.split("-");
    if (parts.length !== 2) {
      console.warn(`[CrossingBased] Invalid key format, expected 'id1-id2', got:`, key);
      continue;
    }

    const [id1, id2] = parts;
    const node1 = result.get(id1 as C4Id);
    const node2 = result.get(id2 as C4Id);

    if (!node1 || !node2) continue;

    const dx = node2.bbox.x - node1.bbox.x;
    const dy = node2.bbox.y - node1.bbox.y;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const requiredDist = minSpacing + crossingCount * 20; // More spacing for more crossings

    if (dist < requiredDist && dist > 0) {
      // Check if nodes have parent-child relationship - if so, be more careful
      const isParentChild = node1.parent?.id === id2 || node2.parent?.id === id1;
      if (isParentChild) {
        // For parent-child, only push the child, not the parent
        const child = node1.parent?.id === id2 ? node1 : node2;
        const childId = node1.parent?.id === id2 ? id1 : id2;
        const parent = node1.parent?.id === id2 ? node2 : node1;

        // Only adjust if child is too close to parent edge
        const pushDist = Math.min((requiredDist - dist) / 2, 15); // Limit push distance
        const pushX = (dx / dist) * pushDist;
        const pushY = (dy / dist) * pushDist;

        const newX = child.bbox.x - pushX;
        const newY = child.bbox.y - pushY;

        // Verify child stays within parent
        const parentRight = parent.bbox.x + parent.bbox.width;
        const parentBottom = parent.bbox.y + parent.bbox.height;
        const childRight = newX + child.bbox.width;
        const childBottom = newY + child.bbox.height;

        if (
          newX >= parent.bbox.x + 20 &&
          newY >= parent.bbox.y + 20 &&
          childRight <= parentRight - 20 &&
          childBottom <= parentBottom - 20
        ) {
          result.set(childId as C4Id, {
            ...child,
            x: newX,
            y: newY,
            bbox: {
              ...child.bbox,
              x: newX,
              y: newY,
            },
          });
        }
      } else {
        // For non-parent-child pairs, push both apart
        const pushDist = Math.min((requiredDist - dist) / 2, 20); // Limit push distance
        const pushX = (dx / dist) * pushDist;
        const pushY = (dy / dist) * pushDist;

        const newX1 = node1.bbox.x - pushX;
        const newY1 = node1.bbox.y - pushY;
        const newX2 = node2.bbox.x + pushX;
        const newY2 = node2.bbox.y + pushY;

        // Verify nodes stay within their parents if they have them
        let canMove1 = true;
        let canMove2 = true;

        if (node1.parent) {
          const parent = result.get(node1.parent.id);
          if (parent) {
            const parentRight = parent.bbox.x + parent.bbox.width;
            const parentBottom = parent.bbox.y + parent.bbox.height;
            const nodeRight = newX1 + node1.bbox.width;
            const nodeBottom = newY1 + node1.bbox.height;

            if (
              newX1 < parent.bbox.x + 20 ||
              newY1 < parent.bbox.y + 20 ||
              nodeRight > parentRight - 20 ||
              nodeBottom > parentBottom - 20
            ) {
              canMove1 = false;
            }
          }
        }

        if (node2.parent) {
          const parent2 = result.get(node2.parent.id);
          if (parent2) {
            const parentRight = parent2.bbox.x + parent2.bbox.width;
            const parentBottom = parent2.bbox.y + parent2.bbox.height;
            const nodeRight = newX2 + node2.bbox.width;
            const nodeBottom = newY2 + node2.bbox.height;

            if (
              newX2 < parent2.bbox.x + 20 ||
              newY2 < parent2.bbox.y + 20 ||
              nodeRight > parentRight - 20 ||
              nodeBottom > parentBottom - 20
            ) {
              canMove2 = false;
            }
          }
        }

        if (canMove1) {
          result.set(id1 as C4Id, {
            ...node1,
            x: newX1,
            y: newY1,
            bbox: {
              ...node1.bbox,
              x: newX1,
              y: newY1,
            },
          });
        }

        if (canMove2) {
          result.set(id2 as C4Id, {
            ...node2,
            x: newX2,
            y: newY2,
            bbox: {
              ...node2.bbox,
              x: newX2,
              y: newY2,
            },
          });
        }
      }
    }
  }

  return result;
}

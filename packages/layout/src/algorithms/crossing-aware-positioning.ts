// packages/layout/src/algorithms/crossing-aware-positioning.ts
// Node position adjustments based on edge crossing analysis
import type { Point } from "../geometry/point";
import type { PositionedC4Node } from "../types";
import type { C4Id } from "../brand";
import type { C4Relationship } from "../c4-model";

export interface EdgePath {
  id: string;
  sourceId: string;
  targetId: string;
  path: Point[];
}

/**
 * Analyze which nodes are involved in the most crossings
 */
export function analyzeCrossingNodes(
  _nodes: Map<C4Id, PositionedC4Node>,
  edges: EdgePath[],
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
 * Apply force-directed adjustments to nodes to reduce crossings
 * Nodes involved in many crossings are pushed away from each other
 */
export function applyForceDirectedAdjustments(
  nodes: Map<C4Id, PositionedC4Node>,
  crossingCounts: Map<C4Id, number>,
  relationships: C4Relationship[],
  maxIterations: number = 5,
  stepSize: number = 15
): Map<C4Id, PositionedC4Node> {
  const result = new Map(nodes);

  // Build adjacency map
  const neighbors = new Map<C4Id, Set<C4Id>>();
  for (const rel of relationships) {
    if (!neighbors.has(rel.from)) neighbors.set(rel.from, new Set());
    if (!neighbors.has(rel.to)) neighbors.set(rel.to, new Set());
    neighbors.get(rel.from)!.add(rel.to);
    neighbors.get(rel.to)!.add(rel.from);
  }

  for (let iter = 0; iter < maxIterations; iter++) {
    const forces = new Map<C4Id, { fx: number; fy: number }>();

    // Initialize forces
    for (const [id] of result) {
      forces.set(id, { fx: 0, fy: 0 });
    }

    // Calculate repulsion forces between high-crossing nodes
    const highCrossingNodes = Array.from(result.entries())
      .filter(([id]) => (crossingCounts.get(id) || 0) > 2)
      .map(([id, node]) => ({ id, node }));

    for (let i = 0; i < highCrossingNodes.length; i++) {
      for (let j = i + 1; j < highCrossingNodes.length; j++) {
        const n1 = highCrossingNodes[i];
        const n2 = highCrossingNodes[j];

        // Skip if they're neighbors (connected by edge)
        if (neighbors.get(n1.id)?.has(n2.id)) continue;

        // Optimized: cache bbox references and use squared distance
        const n1Bbox = n1.node.bbox;
        const n2Bbox = n2.node.bbox;
        const dx = n2Bbox.x - n1Bbox.x;
        const dy = n2Bbox.y - n1Bbox.y;
        const distSq = dx * dx + dy * dy;
        const dist = Math.sqrt(distSq) || 1;

        // Repulsion force proportional to crossing counts
        const n1Crossings = crossingCounts.get(n1.id) || 0;
        const n2Crossings = crossingCounts.get(n2.id) || 0;
        const strength = n1Crossings * n2Crossings * 0.5;
        const force = strength / distSq; // Use distSq directly instead of dist * dist

        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;

        const f1 = forces.get(n1.id)!;
        const f2 = forces.get(n2.id)!;
        f1.fx -= fx;
        f1.fy -= fy;
        f2.fx += fx;
        f2.fy += fy;
      }
    }

    // Apply forces with damping
    // Optimized: cache damping calculation and stepSize * damping
    const damping = 0.8 - iter * 0.1; // Reduce movement over iterations
    const stepDamping = stepSize * damping;
    const minMoveThreshold = 0.5;

    for (const [id, force] of forces) {
      const node = result.get(id)!;
      const moveX = force.fx * stepDamping;
      const moveY = force.fy * stepDamping;

      // Only move if force is significant
      if (Math.abs(moveX) > minMoveThreshold || Math.abs(moveY) > minMoveThreshold) {
        // Update both x/y and bbox
        // Optimized: cache bbox reference
        const nodeBbox = node.bbox;
        const newX = nodeBbox.x + moveX;
        const newY = nodeBbox.y + moveY;
        const updatedNode = {
          ...node,
          bbox: {
            ...nodeBbox,
            x: newX,
            y: newY,
          },
        };
        // Update x and y if they exist on the node
        if ("x" in node && "y" in node) {
          (updatedNode as any).x = newX;
          (updatedNode as any).y = newY;
        }
        result.set(id, updatedNode);
      }
    }
  }

  return result;
}

/**
 * Increase spacing between nodes that have crossing edges
 */
export function adjustSpacingForCrossings(
  nodes: Map<C4Id, PositionedC4Node>,
  edges: EdgePath[],
  pathsCrossFn: (
    path1: Point[],
    path2: Point[],
    from1?: string,
    to1?: string,
    from2?: string,
    to2?: string
  ) => boolean,
  minSpacing: number = 40
): Map<C4Id, PositionedC4Node> {
  const result = new Map(nodes);
  const nodePairs = new Map<string, number>(); // Track pairs with crossings

  // Identify node pairs involved in crossings
  // Optimized: compare each pair exactly once (i < j) and cache edge references
  const edgesLength = edges.length;
  for (let i = 0; i < edgesLength; i++) {
    const e1 = edges[i];
    for (let j = i + 1; j < edgesLength; j++) {
      const e2 = edges[j];

      if (pathsCrossFn(e1.path, e2.path, e1.sourceId, e1.targetId, e2.sourceId, e2.targetId)) {
        // Mark all node pairs from these edges
        // Optimized: cache IDs to avoid repeated property access
        const e1Source = e1.sourceId;
        const e1Target = e1.targetId;
        const e2Source = e2.sourceId;
        const e2Target = e2.targetId;

        const pairs = [
          [e1Source, e2Source],
          [e1Source, e2Target],
          [e1Target, e2Source],
          [e1Target, e2Target],
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
      console.warn(`[CrossingAware] Invalid key, skipping:`, key);
      continue;
    }

    const parts = key.split("-");
    if (parts.length !== 2) {
      console.warn(`[CrossingAware] Invalid key format, expected 'id1-id2', got:`, key);
      continue;
    }

    const [id1, id2] = parts;
    const node1 = result.get(id1 as C4Id);
    const node2 = result.get(id2 as C4Id);

    if (!node1 || !node2) continue;

    // Optimized: cache bbox references
    const node1Bbox = node1.bbox;
    const node2Bbox = node2.bbox;
    const dx = node2Bbox.x - node1Bbox.x;
    const dy = node2Bbox.y - node1Bbox.y;
    const distSq = dx * dx + dy * dy;
    const dist = Math.sqrt(distSq);
    const requiredDist = minSpacing + crossingCount * 20;

    if (dist < requiredDist && dist > 0) {
      const pushDist = (requiredDist - dist) * 0.5; // Use multiplication instead of division
      const invDist = 1 / dist; // Cache inverse distance
      const pushX = dx * invDist * pushDist;
      const pushY = dy * invDist * pushDist;

      const node1Id = id1 as C4Id;
      const node2Id = id2 as C4Id;

      // Only adjust if both nodes exist
      if (result.has(node1Id) && result.has(node2Id)) {
        const newX1 = node1Bbox.x - pushX;
        const newY1 = node1Bbox.y - pushY;
        const newX2 = node2Bbox.x + pushX;
        const newY2 = node2Bbox.y + pushY;

        // Optimized: reuse bbox object if coordinates haven't changed
        const updatedNode1 = {
          ...node1,
          bbox:
            node1Bbox.x === newX1 && node1Bbox.y === newY1
              ? node1Bbox
              : { ...node1Bbox, x: newX1, y: newY1 },
        };
        if ("x" in node1 && "y" in node1) {
          (updatedNode1 as any).x = newX1;
          (updatedNode1 as any).y = newY1;
        }
        result.set(node1Id, updatedNode1);

        const updatedNode2 = {
          ...node2,
          bbox:
            node2Bbox.x === newX2 && node2Bbox.y === newY2
              ? node2Bbox
              : { ...node2Bbox, x: newX2, y: newY2 },
        };
        if ("x" in node2 && "y" in node2) {
          (updatedNode2 as any).x = newX2;
          (updatedNode2 as any).y = newY2;
        }
        result.set(node2Id, updatedNode2);
      }
    }
  }

  return result;
}

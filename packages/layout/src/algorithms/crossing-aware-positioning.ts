// packages/layout/src/algorithms/crossing-aware-positioning.ts
// Node position adjustments based on edge crossing analysis
import type { Point } from "../geometry/point";
import type { PositionedC4Node } from "../c4-layout";
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
  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      const e1 = edges[i];
      const e2 = edges[j];

      if (pathsCrossFn(e1.path, e2.path, e1.sourceId, e1.targetId, e2.sourceId, e2.targetId)) {
        // Both source and target nodes of crossing edges are involved
        nodeCrossingCounts.set(
          e1.sourceId as C4Id,
          (nodeCrossingCounts.get(e1.sourceId as C4Id) || 0) + 1
        );
        nodeCrossingCounts.set(
          e1.targetId as C4Id,
          (nodeCrossingCounts.get(e1.targetId as C4Id) || 0) + 1
        );
        nodeCrossingCounts.set(
          e2.sourceId as C4Id,
          (nodeCrossingCounts.get(e2.sourceId as C4Id) || 0) + 1
        );
        nodeCrossingCounts.set(
          e2.targetId as C4Id,
          (nodeCrossingCounts.get(e2.targetId as C4Id) || 0) + 1
        );
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

        const dx = n2.node.bbox.x - n1.node.bbox.x;
        const dy = n2.node.bbox.y - n1.node.bbox.y;
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;

        // Repulsion force proportional to crossing counts
        const strength = (crossingCounts.get(n1.id) || 0) * (crossingCounts.get(n2.id) || 0) * 0.5;
        const force = strength / (dist * dist);

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
    const damping = 0.8 - iter * 0.1; // Reduce movement over iterations
    for (const [id, force] of forces) {
      const node = result.get(id)!;
      const moveX = force.fx * stepSize * damping;
      const moveY = force.fy * stepSize * damping;

      // Only move if force is significant
      if (Math.abs(moveX) > 0.5 || Math.abs(moveY) > 0.5) {
        // Update both x/y and bbox
        const newX = node.bbox.x + moveX;
        const newY = node.bbox.y + moveY;
        const updatedNode = {
          ...node,
          bbox: {
            ...node.bbox,
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
    const [id1, id2] = key.split("-");
    const node1 = result.get(id1 as C4Id);
    const node2 = result.get(id2 as C4Id);

    if (!node1 || !node2) continue;

    const dx = node2.bbox.x - node1.bbox.x;
    const dy = node2.bbox.y - node1.bbox.y;
    const dist = Math.sqrt(dx * dx + dy * dy);
    const requiredDist = minSpacing + crossingCount * 20;

    if (dist < requiredDist && dist > 0) {
      const pushDist = (requiredDist - dist) / 2;
      const pushX = (dx / dist) * pushDist;
      const pushY = (dy / dist) * pushDist;

      const node1Id = id1 as C4Id;
      const node2Id = id2 as C4Id;

      // Only adjust if both nodes exist
      if (result.has(node1Id) && result.has(node2Id)) {
        const newX1 = node1.bbox.x - pushX;
        const newY1 = node1.bbox.y - pushY;
        const newX2 = node2.bbox.x + pushX;
        const newY2 = node2.bbox.y + pushY;

        const updatedNode1 = {
          ...node1,
          bbox: {
            ...node1.bbox,
            x: newX1,
            y: newY1,
          },
        };
        if ("x" in node1 && "y" in node1) {
          (updatedNode1 as any).x = newX1;
          (updatedNode1 as any).y = newY1;
        }
        result.set(node1Id, updatedNode1);

        const updatedNode2 = {
          ...node2,
          bbox: {
            ...node2.bbox,
            x: newX2,
            y: newY2,
          },
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

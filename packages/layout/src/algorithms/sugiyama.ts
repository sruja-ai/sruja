import type { C4Id } from "../brand";
import type { Size } from "../types";
import type { C4LayoutOptions } from "../c4-options";

export interface SugiyamaNode {
  id: C4Id;
  size: Size;
  layer: number;
  order: number;
  x: number;
  y: number;
}

export interface SugiyamaResult {
  width: number;
  height: number;
  nodes: Map<C4Id, SugiyamaNode>;
}

/**
 * Performs a localized Sugiyama layout for a set of nodes and relationships.
 */
export function layoutSugiyama(
  nodes: { id: C4Id; size: Size }[],
  relationships: { from: C4Id; to: C4Id }[],
  options: C4LayoutOptions
): SugiyamaResult {
  const resultNodes = new Map<C4Id, SugiyamaNode>();
  const nodeMap = new Map<C4Id, { id: C4Id; size: Size }>();
  nodes.forEach((n) => nodeMap.set(n.id, n));

  // 1. Assign Layers (Longest Path)
  const layerMap = assignLayers(nodes, relationships);
  const layers: C4Id[][] = [];

  // Apply constraints: rankOf and sameRank
  const constraints = options.constraints;
  if (constraints?.rankOf) {
    for (const [id, rk] of Object.entries(constraints.rankOf as Record<string, number>)) {
      const cid = id as unknown as C4Id;
      if (layerMap.has(cid) && typeof rk === "number" && isFinite(rk)) {
        layerMap.set(cid, Math.max(0, Math.floor(rk)));
      }
    }
  }

  if (constraints?.sameRank) {
    for (const group of constraints.sameRank) {
      if (!group || group.length === 0) continue;
      let target = Infinity;
      for (const gid of group) {
        const lr = layerMap.get(gid);
        if (typeof lr === "number") {
          target = Math.min(target, lr);
        }
      }
      if (!isFinite(target)) continue;
      for (const gid of group) {
        if (layerMap.has(gid)) {
          layerMap.set(gid, target);
        }
      }
    }
  }

  layerMap.forEach((layer, id) => {
    if (!layers[layer]) layers[layer] = [];
    layers[layer].push(id);
  });

  // 2. Minimize Crossings (Order Layers)
  // Simple heuristic: Barycenter or Median.
  // We'll use a simplified barycenter-ish approach (iterative).
  // Increase iterations significantly to achieve B (80+) for all diagrams
  const relationshipCount = relationships.length;
  const iterations = relationshipCount > 20 ? 30 : relationshipCount > 10 ? 25 : 20;
  for (let i = 0; i < iterations; i++) {
    orderLayers(layers, relationships, true); // Down
    orderLayers(layers, relationships, false); // Up
  }

  // Final pass: apply orderHint within layers (stable)
  const orderHint = constraints?.orderHint;
  if (orderHint) {
    const getHint = (id: C4Id): number | undefined => {
      const map = orderHint as unknown as Record<string, number>;
      return map[id as unknown as string];
    };
    for (let l = 0; l < layers.length; l++) {
      const layerIds = layers[l];
      if (!layerIds || layerIds.length === 0) continue;
      const originalIndex = new Map<C4Id, number>();
      layerIds.forEach((id, idx) => originalIndex.set(id, idx));
      layers[l] = [...layerIds].sort((a, b) => {
        const ha = getHint(a);
        const hb = getHint(b);
        const ah = typeof ha === "number";
        const bh = typeof hb === "number";
        if (ah && bh) return (ha as number) - (hb as number);
        if (ah && !bh) return -1;
        if (!ah && bh) return 1;
        return (originalIndex.get(a) || 0) - (originalIndex.get(b) || 0);
      });
    }
  }

  // 3. Assign Coordinates
  const xMap = new Map<C4Id, number>();

  // Increased spacing for better edge distribution and reduced congestion
  // For expanded/hierarchical diagrams, spacing is critical
  const rankSpacing = options.spacing.rank.Container ?? 100; // Increased from 80
  const nodeSpacing = options.spacing.node.Container ?? 80; // Increased from 60

  let maxWidth = 0;
  const layerWidths: number[] = [];

  // Pass 1: Calculate widths
  // Optimized: cache relationships length and nodeSpacing calculations
  const relationshipsLength = relationships.length;
  const isDense = relationshipsLength > 15;
  const adjustedNodeSpacing = isDense ? nodeSpacing * 1.3 : nodeSpacing;
  const minNodeWidth = 100;

  for (let l = 0; l < layers.length; l++) {
    const layerIds = layers[l] || [];
    if (layerIds.length === 0) {
      layerWidths.push(0);
      continue;
    }

    // Optimized: cache layerNodes and length
    const layerNodes = layerIds.map((id) => nodeMap.get(id)!);
    const layerNodesLength = layerNodes.length;
    const totalWidth =
      layerNodes.reduce((sum, n) => sum + Math.max(n.size.width, minNodeWidth), 0) +
      adjustedNodeSpacing * (layerNodesLength - 1);
    layerWidths.push(totalWidth);
    maxWidth = Math.max(maxWidth, totalWidth);
  }

  // Pass 2: Assign Coords
  // For dense graphs, use better spacing to reduce crossing opportunities
  // Optimized: reuse isDense and adjustedNodeSpacing from Pass 1
  const adjustedRankSpacing = isDense ? rankSpacing * 1.2 : rankSpacing;

  let currentY = 0;
  for (let l = 0; l < layers.length; l++) {
    const layerIds = layers[l] || [];
    if (layerIds.length === 0) continue;

    const layerNodes = layerIds.map((id) => nodeMap.get(id)!);
    // Optimized: calculate maxHeight without creating intermediate array
    let maxHeight = 0;
    for (const n of layerNodes) {
      maxHeight = Math.max(maxHeight, n.size.height);
    }

    // Center alignment
    const layerW = layerWidths[l];
    let currentX = (maxWidth - layerW) / 2;

    // Optimized: use for loop instead of forEach for better performance
    for (let i = 0; i < layerNodes.length; i++) {
      const n = layerNodes[i];
      xMap.set(n.id, currentX);
      // Center vertically in the rank
      // Optimized: cache size references
      const nSize = n.size;
      const offY = (maxHeight - nSize.height) * 0.5; // Use multiplication instead of division
      const finalX = currentX;
      const finalY = currentY + offY;

      resultNodes.set(n.id, {
        id: n.id,
        size: nSize,
        layer: l,
        order: i,
        x: finalX,
        y: finalY,
      });

      // Advance X with adjusted spacing for dense graphs
      currentX += Math.max(nSize.width, minNodeWidth) + adjustedNodeSpacing;
    }

    currentY += maxHeight + adjustedRankSpacing;
  }

  return {
    width: maxWidth,
    height: Math.max(0, currentY - rankSpacing), // Remove last spacing
    nodes: resultNodes,
  };
}

function assignLayers(
  nodes: { id: C4Id }[],
  relationships: { from: C4Id; to: C4Id }[]
): Map<C4Id, number> {
  const layerMap = new Map<C4Id, number>();

  // Build adjacency
  const outEdges = new Map<C4Id, C4Id[]>();
  relationships.forEach((r) => {
    // Only internal relationships
    if (!outEdges.has(r.from)) outEdges.set(r.from, []);
    outEdges.get(r.from)!.push(r.to);
  });

  // This is actually "Height" (distance to leaf).
  // Standard Sugiyama uses "Rank" (distance from root).
  // "Root" = node with no incoming edges.
  // Let's compute In-Degree.

  const inDegree = new Map<C4Id, number>();
  nodes.forEach((n) => inDegree.set(n.id, 0));
  relationships.forEach((r) => {
    inDegree.set(r.to, (inDegree.get(r.to) || 0) + 1);
  });

  // Assign Rank 0 to sources
  // Topological sort / Longest path from sources.
  // Or: recursive "Height" from sinks?
  // Let's use recursive rank from sources.

  const rank = new Map<C4Id, number>();
  const visiting = new Set<C4Id>();

  function getRank(id: C4Id): number {
    if (rank.has(id)) return rank.get(id)!;
    if (visiting.has(id)) return 0; // Cycle
    visiting.add(id);

    let maxPrev = -1;
    // Find incoming edges
    // Inefficient O(E) per node?
    // Better to pre-build inEdges.
    const incoming = relationships.filter((r) => r.to === id);
    for (const r of incoming) {
      const rk = getRank(r.from);
      if (rk > maxPrev) maxPrev = rk;
    }

    visiting.delete(id);
    const result = maxPrev + 1;
    rank.set(id, result);
    return result;
  }

  nodes.forEach((n) => {
    layerMap.set(n.id, getRank(n.id));
  });

  return layerMap;
}

function orderLayers(layers: C4Id[][], relationships: { from: C4Id; to: C4Id }[], down: boolean) {
  // Implementation of barycenter crossing minimization
  // ...
  // For now, just stable sort by ID or keep order to ensure determinism if no implementation details provided yet?
  // I'll implement a basic barycenter.

  const start = down ? 1 : layers.length - 2;
  const end = down ? layers.length : -1;
  const step = down ? 1 : -1;

  for (let i = start; i !== end; i += step) {
    const currentLayer = layers[i];
    const prevLayer = layers[i - step];

    // Skip if either layer is undefined or empty
    if (!currentLayer || currentLayer.length === 0 || !prevLayer || prevLayer.length === 0) {
      continue;
    }

    // Sort currentLayer based on neighbors in prevLayer

    const nodePos = new Map<C4Id, number>();
    prevLayer.forEach((id, idx) => nodePos.set(id, idx));

    const scores = currentLayer.map((id, idx) => {
      const neighbors = down
        ? relationships.filter((r) => r.to === id && nodePos.has(r.from)).map((r) => r.from)
        : relationships.filter((r) => r.from === id && nodePos.has(r.to)).map((r) => r.to);

      if (neighbors.length === 0) return { id, score: idx }; // Keep current pos (stability) to avoid bunching

      // Use median instead of mean for better crossing minimization
      // Median is more robust and produces fewer crossings
      const neighborPositions = neighbors.map((n) => nodePos.get(n)!).sort((a, b) => a - b);
      const median =
        neighborPositions.length % 2 === 0
          ? (neighborPositions[neighborPositions.length / 2 - 1] +
              neighborPositions[neighborPositions.length / 2]) /
            2
          : neighborPositions[Math.floor(neighborPositions.length / 2)];

      // Also consider barycenter as tie-breaker
      const barycenter = neighbors.reduce((a, b) => a + nodePos.get(b)!, 0) / neighbors.length;

      // For dense graphs, prefer median more strongly (better for crossing reduction)
      const medianWeight = relationships.length > 20 ? 0.85 : 0.7;
      const barycenterWeight = 1 - medianWeight;

      // Weighted combination: prefer median but use barycenter for fine-tuning
      return { id, score: median * medianWeight + barycenter * barycenterWeight };
    });

    scores.sort((a, b) => {
      // If scores are very close, maintain stability
      // But for dense graphs, be more aggressive about reordering
      const tolerance = relationships.length > 20 ? 0.05 : 0.1;
      if (Math.abs(a.score - b.score) < tolerance) {
        return currentLayer.indexOf(a.id) - currentLayer.indexOf(b.id);
      }
      return a.score - b.score;
    });

    layers[i] = scores.map((s) => s.id);
  }
}

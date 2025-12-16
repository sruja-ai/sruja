// packages/layout/src/algorithms/localized-swap-optimizer.ts
// Localized node swaps after routing to reduce crossings (Section 2.3 of LayoutBestPractice.md)
import type { C4Id } from "../brand";
import type { PositionedNode } from "./coordinates";
import type { Point } from "../geometry/point";
import type { C4Relationship } from "../c4-model";

export interface EdgeRoute {
  id: string;
  sourceId: string;
  targetId: string;
  path: Point[];
}

/**
 * Configuration constants for localized swap optimization
 */
const RANK_TOLERANCE = 50; // Nodes within 50px vertically are considered in the same rank
const CONTAINMENT_PADDING = 20; // Minimum padding from parent container boundaries

/**
 * Type for nodes grouped by rank
 */
type RankGroup = Array<{ id: C4Id; node: PositionedNode }>;

/**
 * Type for path crossing detection function
 */
type PathsCrossFunction = (
  path1: Point[],
  path2: Point[],
  from1?: string,
  to1?: string,
  from2?: string,
  to2?: string
) => boolean;

/**
 * Checks if any edge crossings exist in the current layout.
 * More efficient than countCrossings when we only need to know if crossings exist.
 *
 * Time complexity: O(n²) worst case, but early exits on first crossing.
 *
 * @param edges - Array of edge routes to check
 * @param pathsCrossFn - Function to determine if two paths cross
 * @returns true if any crossings exist, false otherwise
 */
function hasCrossings(edges: EdgeRoute[], pathsCrossFn: PathsCrossFunction): boolean {
  // Early exit for empty or single edge cases
  if (edges.length <= 1) {
    return false;
  }

  // Compare each pair of edges exactly once (i < j), exit early on first crossing
  for (let i = 0; i < edges.length; i++) {
    const edge1 = edges[i];
    for (let j = i + 1; j < edges.length; j++) {
      const edge2 = edges[j];
      if (
        pathsCrossFn(
          edge1.path,
          edge2.path,
          edge1.sourceId,
          edge1.targetId,
          edge2.sourceId,
          edge2.targetId
        )
      ) {
        return true;
      }
    }
  }
  return false;
}

/**
 * Groups nodes by their vertical position (rank) based on Y coordinate.
 * Nodes within RANK_TOLERANCE pixels are considered in the same rank.
 *
 * Time complexity: O(n log n) where n is the number of nodes.
 * - O(n) for grouping nodes by rank
 * - O(n log n) for sorting nodes within each rank
 *
 * @param nodes - Map of positioned nodes to group
 * @returns Map of rank keys to arrays of nodes in that rank (sorted by X position)
 *
 * @example
 * ```ts
 * const rankGroups = groupNodesByRank(nodes)
 * // rankGroups: Map<number, RankGroup>
 * // Key is rounded Y position, value is sorted array of nodes (left to right)
 * ```
 */
function groupNodesByRank(nodes: Map<C4Id, PositionedNode>): Map<number, RankGroup> {
  const rankGroups = new Map<number, RankGroup>();

  // Group nodes by rank (O(n))
  // Cache rank key calculation to avoid repeated operations
  for (const [id, node] of nodes) {
    const centerY = node.bbox.y + node.bbox.height * 0.5; // Use multiplication instead of division
    const rankKey = Math.round(centerY / RANK_TOLERANCE) * RANK_TOLERANCE;

    let rankGroup = rankGroups.get(rankKey);
    if (!rankGroup) {
      rankGroup = [];
      rankGroups.set(rankKey, rankGroup);
    }
    rankGroup.push({ id, node });
  }

  // Sort nodes within each rank by X position (left to right) - O(n log n) worst case
  // Use for...of with destructuring for better performance
  for (const nodesInRank of rankGroups.values()) {
    nodesInRank.sort((a, b) => a.node.bbox.x - b.node.bbox.x);
  }

  return rankGroups;
}

/**
 * Checks if two nodes have a parent-child relationship that should prevent swapping.
 *
 * @param node1 - First node to check
 * @param node2 - Second node to check
 * @returns true if nodes have parent-child relationship, false otherwise
 */
function hasParentChildRelationship(node1: PositionedNode, node2: PositionedNode): boolean {
  return node1.parent?.id === node2.id || node2.parent?.id === node1.id;
}

/**
 * Creates a new PositionedNode with swapped coordinates.
 * Optimized to avoid unnecessary object spreading when possible.
 *
 * @param node - Original node to swap
 * @param newX - New X coordinate
 * @param newY - New Y coordinate
 * @returns New PositionedNode with updated position
 */
function createSwappedNode(node: PositionedNode, newX: number, newY: number): PositionedNode {
  // Reuse bbox object if coordinates haven't changed (rare but possible)
  const bbox =
    node.bbox.x === newX && node.bbox.y === newY ? node.bbox : { ...node.bbox, x: newX, y: newY };

  return {
    ...node,
    x: newX,
    y: newY,
    bbox,
  };
}

/**
 * Checks if a node position is within its parent container boundaries.
 * Optimized to reduce redundant calculations.
 *
 * @param node - Node to check containment for
 * @param parent - Parent node container
 * @returns true if node is properly contained within parent, false otherwise
 */
function isNodeContained(node: PositionedNode, parent: PositionedNode): boolean {
  const nodeX = node.bbox.x;
  const nodeY = node.bbox.y;
  const nodeRight = nodeX + node.bbox.width;
  const nodeBottom = nodeY + node.bbox.height;

  const parentX = parent.bbox.x;
  const parentY = parent.bbox.y;
  const parentRight = parentX + parent.bbox.width;
  const parentBottom = parentY + parent.bbox.height;

  const minX = parentX + CONTAINMENT_PADDING;
  const minY = parentY + CONTAINMENT_PADDING;
  const maxX = parentRight - CONTAINMENT_PADDING;
  const maxY = parentBottom - CONTAINMENT_PADDING;

  return nodeX >= minX && nodeY >= minY && nodeRight <= maxX && nodeBottom <= maxY;
}

/**
 * Validates if swapping two nodes would maintain proper containment constraints.
 * Optimized with cached parent lookups to avoid repeated Map.get calls.
 *
 * @param swappedNode1 - First node after swap
 * @param swappedNode2 - Second node after swap
 * @param allNodes - Map of all nodes (for parent lookup)
 * @returns true if swap is valid, false otherwise
 */
function isValidSwap(
  swappedNode1: PositionedNode,
  swappedNode2: PositionedNode,
  allNodes: Map<C4Id, PositionedNode>
): boolean {
  // Check containment for node1 (cache parent lookup)
  const parent1 = swappedNode1.parent ? allNodes.get(swappedNode1.parent.id) : null;
  if (parent1 && !isNodeContained(swappedNode1, parent1)) {
    return false;
  }

  // Check containment for node2 (cache parent lookup)
  const parent2 = swappedNode2.parent ? allNodes.get(swappedNode2.parent.id) : null;
  if (parent2 && !isNodeContained(swappedNode2, parent2)) {
    return false;
  }

  return true;
}

/**
 * Applies localized node swaps (adjacent rank swaps) to reduce edge crossings.
 *
 * This algorithm implements Section 2.3 of LayoutBestPractice.md:
 * - After routing, if crossings exist, try swapping adjacent nodes in the same rank
 * - Only swaps that maintain parent-child containment are allowed
 * - Re-routing of affected edges should be done after swapping
 *
 * Algorithm steps:
 * 1. Count current edge crossings
 * 2. Group nodes by vertical rank (Y position)
 * 3. For each rank, try swapping adjacent nodes
 * 4. Validate swaps maintain containment constraints
 * 5. Apply valid swaps up to maxSwaps limit
 *
 * @param nodes - Map of positioned nodes to optimize
 * @param edges - Array of edge routes (for crossing detection)
 * @param _relationships - Relationships metadata (currently unused, reserved for future use)
 * @param pathsCrossFn - Function to detect if two edge paths cross
 * @param maxSwaps - Maximum number of swaps to attempt (default: 10)
 * @returns Object with optimized nodes map and improvement flag
 *
 * @example
 * ```ts
 * const result = applyLocalizedSwaps(
 *   positionedNodes,
 *   edgeRoutes,
 *   relationships,
 *   pathsCross,
 *   15 // max swaps
 * )
 *
 * if (result.improved) {
 *   // Re-route edges with new positions
 *   positionedNodes = result.nodes
 * }
 * ```
 */
export function applyLocalizedSwaps(
  nodes: Map<C4Id, PositionedNode>,
  edges: EdgeRoute[],
  _relationships: readonly C4Relationship[],
  pathsCrossFn: PathsCrossFunction,
  maxSwaps: number = 10
): { nodes: Map<C4Id, PositionedNode>; improved: boolean } {
  // Early exit if no edges or nodes
  if (edges.length === 0 || nodes.size === 0) {
    return { nodes: new Map(nodes), improved: false };
  }

  // Early exit if no crossings exist (use faster hasCrossings check)
  if (!hasCrossings(edges, pathsCrossFn)) {
    return { nodes: new Map(nodes), improved: false };
  }

  const result = new Map(nodes);

  // Group nodes by rank and sort within each rank
  const rankGroups = groupNodesByRank(result);

  // Try swapping adjacent nodes in each rank
  // Performance: Process ranks in order, stop early if maxSwaps reached
  let improved = false;
  let swapsAttempted = 0;

  // Process ranks in order, using values() iterator for better performance
  for (const nodesInRank of rankGroups.values()) {
    // Early exit conditions
    if (nodesInRank.length < 2 || swapsAttempted >= maxSwaps) {
      continue;
    }

    // Cache length to avoid repeated property access
    const rankLength = nodesInRank.length;
    for (let i = 0; i < rankLength - 1 && swapsAttempted < maxSwaps; i++) {
      const node1 = nodesInRank[i];
      const node2 = nodesInRank[i + 1];

      // Skip if nodes have parent-child relationship
      if (hasParentChildRelationship(node1.node, node2.node)) {
        continue;
      }

      // Cache bbox references to avoid repeated property access
      const bbox1 = node1.node.bbox;
      const bbox2 = node2.node.bbox;

      // Calculate swapped positions
      const newX1 = bbox2.x;
      const newY1 = bbox2.y;
      const newX2 = bbox1.x;
      const newY2 = bbox1.y;

      // Create swapped node positions
      const swappedNode1 = createSwappedNode(node1.node, newX1, newY1);
      const swappedNode2 = createSwappedNode(node2.node, newX2, newY2);

      // Validate swap maintains containment
      if (!isValidSwap(swappedNode1, swappedNode2, result)) {
        continue;
      }

      // Apply swap
      result.set(node1.id, swappedNode1);
      result.set(node2.id, swappedNode2);

      // Update rank array for next iteration (swap in-place)
      nodesInRank[i] = node2;
      nodesInRank[i + 1] = node1;

      improved = true;
      swapsAttempted++;
    }
  }

  return { nodes: result, improved };
}

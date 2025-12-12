import type { C4Id } from "../brand";
import type { PositionedNode } from "./coordinates";
import type { HierarchyTree, HierarchyNode } from "./hierarchy";
import type { C4Relationship } from "../c4-model";
import type { Rect } from "../geometry/rect";

export interface OptimizationOptions {
  enabled: boolean;
  overlapRemoval?: {
    iterations?: number;
    padding?: number;
  };
  spaceDistribution?: {
    enabled?: boolean;
    minThreshold?: number;
  };
  edgeOptimization?: {
    enabled?: boolean;
    minimizeCrossings?: boolean;
    iterations?: number;
  };
}

/**
 * Calculate the bounding box containing all children of a node
 */
function calculateChildrenBoundingBox(
  children: HierarchyNode[],
  positioned: Map<C4Id, PositionedNode>
): Rect {
  if (children.length === 0) {
    return { x: 0, y: 0, width: 0, height: 0 };
  }

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const child of children) {
    const pos = positioned.get(child.id);
    if (pos) {
      minX = Math.min(minX, pos.bbox.x);
      minY = Math.min(minY, pos.bbox.y);
      maxX = Math.max(maxX, pos.bbox.x + pos.bbox.width);
      maxY = Math.max(maxY, pos.bbox.y + pos.bbox.height);
    }
  }

  return {
    x: minX,
    y: minY,
    width: maxX - minX,
    height: maxY - minY,
  };
}

/**
 * Remove overlaps among sibling nodes by adjusting their positions
 */
function removeOverlapsAmongSiblings(
  siblings: PositionedNode[],
  padding: number
): PositionedNode[] {
  if (siblings.length <= 1) return siblings;

  const adjusted = [...siblings];
  let hasOverlap = true;
  let iterations = 0;
  const maxIterations = 10;

  while (hasOverlap && iterations < maxIterations) {
    hasOverlap = false;
    iterations++;

    for (let i = 0; i < adjusted.length; i++) {
      for (let j = i + 1; j < adjusted.length; j++) {
        const a = adjusted[i];
        const b = adjusted[j];

        // Check for overlap with padding
        const overlapX =
          a.bbox.x < b.bbox.x + b.bbox.width + padding &&
          a.bbox.x + a.bbox.width + padding >= b.bbox.x;
        const overlapY =
          a.bbox.y < b.bbox.y + b.bbox.height + padding &&
          a.bbox.y + a.bbox.height + padding >= b.bbox.y;

        if (overlapX && overlapY) {
          hasOverlap = true;

          // Calculate overlap amounts
          const overlapWidth = Math.min(
            a.bbox.x + a.bbox.width + padding - b.bbox.x,
            b.bbox.x + b.bbox.width + padding - a.bbox.x
          );
          const overlapHeight = Math.min(
            a.bbox.y + a.bbox.height + padding - b.bbox.y,
            b.bbox.y + b.bbox.height + padding - a.bbox.y
          );

          // Push apart in the direction of least overlap
          if (overlapWidth < overlapHeight) {
            // Separate horizontally
            const pushDist = Math.max(1, overlapWidth) / 2;
            adjusted[i] = {
              ...a,
              x: a.x - pushDist,
              bbox: { ...a.bbox, x: a.bbox.x - pushDist },
            };
            adjusted[j] = {
              ...b,
              x: b.x + pushDist,
              bbox: { ...b.bbox, x: b.bbox.x + pushDist },
            };
          } else {
            // Separate vertically
            const pushDist = Math.max(1, overlapHeight) / 2;
            adjusted[i] = {
              ...a,
              y: a.y - pushDist,
              bbox: { ...a.bbox, y: a.bbox.y - pushDist },
            };
            adjusted[j] = {
              ...b,
              y: b.y + pushDist,
              bbox: { ...b.bbox, y: b.bbox.y + pushDist },
            };
          }
        }
      }
    }
  }

  return adjusted;
}

/**
 * Resize parent node to fit all its children
 */
function resizeParentToFitChildren(
  parentNode: HierarchyNode,
  positioned: Map<C4Id, PositionedNode>,
  padding: number
): void {
  const parent = positioned.get(parentNode.id);
  if (!parent || parentNode.children.length === 0) return;

  // Calculate absolute bounding box of all children
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const child of parentNode.children) {
    const pos = positioned.get(child.id);
    if (pos) {
      minX = Math.min(minX, pos.bbox.x);
      minY = Math.min(minY, pos.bbox.y);
      maxX = Math.max(maxX, pos.bbox.x + pos.bbox.width);
      maxY = Math.max(maxY, pos.bbox.y + pos.bbox.height);
    }
  }

  // If no valid children, skip
  if (minX === Infinity) return;

  // Calculate required parent size to contain all children
  // Parent must start at or before the leftmost/topmost child
  // and extend beyond the rightmost/bottommost child

  // Required size includes:
  // 1. Space from parent's current position to leftmost child (minX - parent.x)
  // 2. Width of children's bounding box (maxX - minX)
  // 3. Right padding
  const requiredWidth = maxX - parent.bbox.x + padding;
  const requiredHeight = maxY - parent.bbox.y + padding;

  // Only expand, don't shrink
  if (requiredWidth > parent.bbox.width || requiredHeight > parent.bbox.height) {
    const newWidth = Math.max(parent.bbox.width, requiredWidth);
    const newHeight = Math.max(parent.bbox.height, requiredHeight);

    console.info(`[CONTAINMENT] Resizing parent ${parentNode.id}:`, {
      old: { w: parent.bbox.width, h: parent.bbox.height },
      new: { w: newWidth, h: newHeight },
      children: parentNode.children.length,
      childrenBounds: { minX, minY, maxX, maxY },
      parentPos: { x: parent.bbox.x, y: parent.bbox.y },
    });

    positioned.set(parentNode.id, {
      ...parent,
      bbox: {
        ...parent.bbox,
        width: newWidth,
        height: newHeight,
      },
      size: {
        width: newWidth,
        height: newHeight,
      },
    });
  }
}

/**
 * Remove overlaps in bottom-up fashion
 * Process children before parents to ensure children don't overlap first
 */
export function removeOverlapsBottomUp(
  positioned: Map<C4Id, PositionedNode>,
  tree: HierarchyTree,
  options: OptimizationOptions
): Map<C4Id, PositionedNode> {
  const result = new Map(positioned);
  const padding = options.overlapRemoval?.padding ?? 16;

  // Post-order traversal: fix children before parents
  function fixNode(node: HierarchyNode): void {
    // First, fix all children
    for (const child of node.children) {
      fixNode(child);
    }

    // Then fix overlaps among this node's children
    if (node.children.length > 1 && !node.node.collapseChildren) {
      const children = node.children.map((c) => result.get(c.id)!).filter((c) => c !== undefined);

      const adjusted = removeOverlapsAmongSiblings(children, padding);
      adjusted.forEach((adjustedNode) => result.set(adjustedNode.node.id, adjustedNode));

      // Resize parent to fit adjusted children
      resizeParentToFitChildren(node, result, padding);
    }
  }

  for (const root of tree.roots) {
    fixNode(root);
  }

  // Final pass: fix overlaps among top-level roots
  if (tree.roots.length > 1) {
    const rootNodes = tree.roots.map((r) => result.get(r.id)!).filter((n) => n !== undefined);
    const adjustedRoots = removeOverlapsAmongSiblings(rootNodes, padding);
    adjustedRoots.forEach((adjustedNode) => result.set(adjustedNode.node.id, adjustedNode));
  }

  return result;
}

/**
 * Distribute extra space evenly among children (top-down)
 */
export function distributeSpaceTopDown(
  positioned: Map<C4Id, PositionedNode>,
  tree: HierarchyTree,
  options: OptimizationOptions
): Map<C4Id, PositionedNode> {
  const result = new Map(positioned);
  const minThreshold = options.spaceDistribution?.minThreshold ?? 50;

  // Pre-order traversal: parents before children
  function distributeNode(node: HierarchyNode): void {
    const current = result.get(node.id);
    if (!current || node.children.length === 0 || node.node.collapseChildren) {
      return;
    }

    // Calculate available space
    const padding = 16; // Could be from options
    const headerHeight = current.contentSize.height + padding;
    const contentArea = {
      width: current.bbox.width - padding * 2,
      height: current.bbox.height - headerHeight - padding,
    };

    const childrenBBox = calculateChildrenBoundingBox(node.children, result);
    const extraSpaceX = contentArea.width - childrenBBox.width;
    const extraSpaceY = contentArea.height - childrenBBox.height;

    // Only distribute if we have significant extra space
    if (extraSpaceX > minThreshold || extraSpaceY > minThreshold) {
      // Calculate uniform offset to CENTER children within available space
      // We want the children as a group to be centered, not individually offset
      const contentStartX = current.bbox.x + padding;
      const contentStartY = current.bbox.y + headerHeight;

      // Current children group starts at childrenBBox.x, should start at contentStartX + centeringOffsetX
      const centeringOffsetX = extraSpaceX > minThreshold ? extraSpaceX / 2 : 0;
      const centeringOffsetY = extraSpaceY > minThreshold ? extraSpaceY / 2 : 0;

      // Calculate how much we need to shift the children group
      const targetX = contentStartX + centeringOffsetX;
      const targetY = contentStartY + centeringOffsetY;

      // Only shift if the children aren't already roughly centered
      const shiftX = targetX - childrenBBox.x;
      const shiftY = targetY - childrenBBox.y;

      // Apply uniform shift if significant
      if (Math.abs(shiftX) > 5 || Math.abs(shiftY) > 5) {
        node.children.forEach((child) => {
          const childNode = result.get(child.id);
          if (childNode) {
            result.set(child.id, {
              ...childNode,
              x: childNode.x + shiftX,
              y: childNode.y + shiftY,
              bbox: {
                ...childNode.bbox,
                x: childNode.bbox.x + shiftX,
                y: childNode.bbox.y + shiftY,
              },
            });
          }
        });
      }
    }

    // Recurse to children
    for (const child of node.children) {
      distributeNode(child);
    }
  }

  for (const root of tree.roots) {
    distributeNode(root);
  }

  return result;
}

/**
 * Calculate adjacency delta for a node
 * The adjacency delta is the average difference between a node's position
 * and the positions of its connected nodes in the adjacent layer
 */
function calculateAdjacencyDelta(
  node: HierarchyNode,
  nodeIndex: number,
  adjacentLayer: HierarchyNode[],
  relationships: C4Relationship[],
  direction: "up" | "down"
): number {
  const connections = relationships.filter((r) =>
    direction === "up" ? r.to === node.id : r.from === node.id
  );

  if (connections.length === 0) return 0;

  const adjacentIndices: number[] = [];
  for (const rel of connections) {
    const targetId = direction === "up" ? rel.from : rel.to;
    const targetIndex = adjacentLayer.findIndex((n) => n.id === targetId);
    if (targetIndex >= 0) {
      adjacentIndices.push(targetIndex);
    }
  }

  if (adjacentIndices.length === 0) return 0;

  const avgPosition = adjacentIndices.reduce((sum, idx) => sum + idx, 0) / adjacentIndices.length;
  return Math.abs(nodeIndex - avgPosition);
}

/**
 * Sort layer by adjacency delta to upper layer
 */
function sortByUpperAdjacency(
  layer: HierarchyNode[],
  upperLayer: HierarchyNode[] | null,
  relationships: C4Relationship[]
): HierarchyNode[] {
  if (!upperLayer || upperLayer.length === 0) return layer;

  return [...layer].sort((a, b) => {
    const aIndex = layer.indexOf(a);
    const bIndex = layer.indexOf(b);
    const aDelta = calculateAdjacencyDelta(a, aIndex, upperLayer, relationships, "up");
    const bDelta = calculateAdjacencyDelta(b, bIndex, upperLayer, relationships, "up");
    return aDelta - bDelta;
  });
}

/**
 * Find best swap neighbor to reduce crossings
 * Only considers immediate siblings (left/right neighbors)
 */
function findBestSwapNeighbor(
  node: HierarchyNode,
  nodeIndex: number,
  layer: HierarchyNode[],
  upperLayer: HierarchyNode[] | null,
  lowerLayer: HierarchyNode[] | null,
  relationships: C4Relationship[]
): number | null {
  const leftIdx = nodeIndex - 1;
  const rightIdx = nodeIndex + 1;

  let bestIdx: number | null = null;
  let bestImprovement = 0;

  // Calculate current delta
  const currentDelta =
    (upperLayer ? calculateAdjacencyDelta(node, nodeIndex, upperLayer, relationships, "up") : 0) +
    (lowerLayer ? calculateAdjacencyDelta(node, nodeIndex, lowerLayer, relationships, "down") : 0);

  // Try left swap
  if (leftIdx >= 0) {
    const swappedDelta =
      (upperLayer ? calculateAdjacencyDelta(node, leftIdx, upperLayer, relationships, "up") : 0) +
      (lowerLayer ? calculateAdjacencyDelta(node, leftIdx, lowerLayer, relationships, "down") : 0);

    const improvement = currentDelta - swappedDelta;
    if (improvement > bestImprovement) {
      bestImprovement = improvement;
      bestIdx = leftIdx;
    }
  }

  // Try right swap
  if (rightIdx < layer.length) {
    const swappedDelta =
      (upperLayer ? calculateAdjacencyDelta(node, rightIdx, upperLayer, relationships, "up") : 0) +
      (lowerLayer ? calculateAdjacencyDelta(node, rightIdx, lowerLayer, relationships, "down") : 0);

    const improvement = currentDelta - swappedDelta;
    if (improvement > bestImprovement) {
      bestImprovement = improvement;
      bestIdx = rightIdx;
    }
  }

  return bestIdx;
}

/**
 * Phase 1: Layer-by-layer crossing minimization
 * Uses adjacency delta heuristic to minimize crossings
 */
function layerByLayerMinimization(
  layers: HierarchyNode[][],
  relationships: C4Relationship[],
  maxIterations: number = 10
): HierarchyNode[][] {
  const currentLayers = layers.map((layer) => [...layer]);
  let hasChanges = true;
  let iteration = 0;

  while (hasChanges && iteration < maxIterations) {
    hasChanges = false;
    iteration++;

    // Top-down pass
    for (let i = 0; i < currentLayers.length; i++) {
      const layer = currentLayers[i];
      const upperLayer = i > 0 ? currentLayers[i - 1] : null;
      const lowerLayer = i < currentLayers.length - 1 ? currentLayers[i + 1] : null;

      // Step 1: Sort by upper adjacency
      if (upperLayer) {
        const sorted = sortByUpperAdjacency(layer, upperLayer, relationships);
        if (JSON.stringify(sorted.map((n) => n.id)) !== JSON.stringify(layer.map((n) => n.id))) {
          currentLayers[i] = sorted;
          hasChanges = true;
        }
      }

      // Step 2: Try neighbor swaps
      for (let j = 0; j < currentLayers[i].length; j++) {
        const node = currentLayers[i][j];
        const bestSwapIdx = findBestSwapNeighbor(
          node,
          j,
          currentLayers[i],
          upperLayer,
          lowerLayer,
          relationships
        );

        if (bestSwapIdx !== null) {
          // Perform swap
          const temp = currentLayers[i][j];
          currentLayers[i][j] = currentLayers[i][bestSwapIdx];
          currentLayers[i][bestSwapIdx] = temp;
          hasChanges = true;
        }
      }
    }
  }

  return currentLayers;
}

/**
 * Count total edge crossings in the layout
 */
function countTotalCrossings(layers: HierarchyNode[][], relationships: C4Relationship[]): number {
  let crossings = 0;

  for (let i = 0; i < layers.length - 1; i++) {
    const upperLayer = layers[i];
    const lowerLayer = layers[i + 1];

    // Get all edges between these two layers
    const edges: Array<{ fromIdx: number; toIdx: number }> = [];
    for (const rel of relationships) {
      const fromIdx = upperLayer.findIndex((n) => n.id === rel.from);
      const toIdx = lowerLayer.findIndex((n) => n.id === rel.to);
      if (fromIdx >= 0 && toIdx >= 0) {
        edges.push({ fromIdx, toIdx });
      }
    }

    // Count crossings between edge pairs
    for (let j = 0; j < edges.length; j++) {
      for (let k = j + 1; k < edges.length; k++) {
        const e1 = edges[j];
        const e2 = edges[k];

        // Edges cross if they have opposite ordering
        if (
          (e1.fromIdx < e2.fromIdx && e1.toIdx > e2.toIdx) ||
          (e1.fromIdx > e2.fromIdx && e1.toIdx < e2.toIdx)
        ) {
          crossings++;
        }
      }
    }
  }

  return crossings;
}

/**
 * Phase 2: Sifting
 * Goes through nodes by degree, trying all sibling swaps to minimize crossings
 */
function siftingMinimization(
  layers: HierarchyNode[][],
  relationships: C4Relationship[],
  maxIterations: number = 10
): HierarchyNode[][] {
  const currentLayers = layers.map((layer) => [...layer]);
  let hasChanges = true;
  let iteration = 0;

  while (hasChanges && iteration < maxIterations) {
    hasChanges = false;
    iteration++;

    // Sort all nodes by degree (number of connections)
    const allNodes: Array<{
      node: HierarchyNode;
      layerIdx: number;
      nodeIdx: number;
      degree: number;
    }> = [];
    for (let i = 0; i < currentLayers.length; i++) {
      for (let j = 0; j < currentLayers[i].length; j++) {
        const node = currentLayers[i][j];
        const degree = relationships.filter((r) => r.from === node.id || r.to === node.id).length;
        allNodes.push({ node, layerIdx: i, nodeIdx: j, degree });
      }
    }
    allNodes.sort((a, b) => b.degree - a.degree);

    // For each node in degree order
    for (const { node, layerIdx } of allNodes) {
      const layer = currentLayers[layerIdx];
      const currentIdx = layer.findIndex((n) => n.id === node.id);
      if (currentIdx === -1) continue;

      let bestIdx = currentIdx;
      let bestCrossings = countTotalCrossings(currentLayers, relationships);

      // Try swapping with each sibling position
      for (let targetIdx = 0; targetIdx < layer.length; targetIdx++) {
        if (targetIdx === currentIdx) continue;

        // Temporarily swap
        const testLayers = currentLayers.map((l) => [...l]);
        const temp = testLayers[layerIdx][currentIdx];
        testLayers[layerIdx][currentIdx] = testLayers[layerIdx][targetIdx];
        testLayers[layerIdx][targetIdx] = temp;

        const crossings = countTotalCrossings(testLayers, relationships);
        if (crossings < bestCrossings) {
          bestCrossings = crossings;
          bestIdx = targetIdx;
        }
      }

      // Apply best swap if it improves
      if (bestIdx !== currentIdx) {
        const temp = currentLayers[layerIdx][currentIdx];
        currentLayers[layerIdx][currentIdx] = currentLayers[layerIdx][bestIdx];
        currentLayers[layerIdx][bestIdx] = temp;
        hasChanges = true;
      }
    }
  }

  return currentLayers;
}

/**
 * Group nodes into layers by depth in hierarchy
 */
function groupIntoLayers(nodes: HierarchyNode[]): HierarchyNode[][] {
  const layerMap = new Map<number, HierarchyNode[]>();

  for (const node of nodes) {
    const depth = node.depth;
    if (!layerMap.has(depth)) {
      layerMap.set(depth, []);
    }
    layerMap.get(depth)!.push(node);
  }

  const maxDepth = Math.max(...layerMap.keys());
  const layers: HierarchyNode[][] = [];
  for (let i = 0; i <= maxDepth; i++) {
    layers.push(layerMap.get(i) || []);
  }

  return layers;
}

/**
 * Apply layers back to positioned nodes
 * IMPORTANT: This should only REORDER nodes within their current context,
 * not reposition them absolutely. Children must stay within their parent.
 */
function applyLayerOrdering(
  positioned: Map<C4Id, PositionedNode>,
  layers: HierarchyNode[][]
): Map<C4Id, PositionedNode> {
  const result = new Map(positioned);

  for (const layer of layers) {
    if (layer.length === 0) continue;

    // Group nodes by parent to preserve containment
    const nodesByParent = new Map<string | undefined, HierarchyNode[]>();
    for (const node of layer) {
      const parentId = node.parent?.id;
      if (!nodesByParent.has(parentId)) {
        nodesByParent.set(parentId, []);
      }
      nodesByParent.get(parentId)!.push(node);
    }

    // Only reorder siblings (nodes with same parent)
    for (const [_parentId, siblings] of nodesByParent) {
      if (siblings.length <= 1) continue;

      // Get current positions
      const siblingPositions = siblings.map((n) => positioned.get(n.id)!).filter((n) => n);

      if (siblingPositions.length === 0) continue;

      // Calculate the starting x and average y for this sibling group
      const minX = Math.min(...siblingPositions.map((n) => n.x));
      const avgY = siblingPositions.reduce((sum, n) => sum + n.y, 0) / siblingPositions.length;
      const spacing = 100; // Minimum horizontal spacing

      // Reposition siblings starting from their group's minX
      let currentX = minX;
      for (let i = 0; i < siblings.length; i++) {
        const node = siblings[i];
        const posNode = positioned.get(node.id);
        if (!posNode) continue;

        result.set(node.id, {
          ...posNode,
          x: currentX,
          y: avgY,
          bbox: {
            ...posNode.bbox,
            x: currentX,
            y: avgY,
          },
        });

        currentX += posNode.bbox.width + spacing;
      }
    }
  }

  return result;
}

/**
 * Optimize node positions to minimize edge crossings using TALA's two-phase approach
 * Based on: https://terrastruct.com/blog/post/diagram-layout-engines-crossing-minimization/
 */
export function optimizeForEdges(
  positioned: Map<C4Id, PositionedNode>,
  relationships: C4Relationship[],
  tree: HierarchyTree,
  options: OptimizationOptions
): Map<C4Id, PositionedNode> {
  if (!options.edgeOptimization?.enabled || !options.edgeOptimization?.minimizeCrossings) {
    return positioned;
  }

  const result = new Map(positioned);

  // Process each parent's children independently
  function optimizeNode(node: HierarchyNode): void {
    if (node.children.length <= 1 || node.node.collapseChildren) {
      return;
    }

    // Group children into layers
    const allDescendants = collectDescendants(node);
    const layers = groupIntoLayers(allDescendants);

    if (layers.length <= 1) {
      // Recurse to children
      for (const child of node.children) {
        optimizeNode(child);
      }
      return;
    }

    // Filter relationships to only those within this subtree
    const descriptorIds = new Set(allDescendants.map((n) => n.id));
    const subtreeRels = relationships.filter(
      (r) => descriptorIds.has(r.from) && descriptorIds.has(r.to)
    );

    // Phase 1: Layer-by-layer minimization
    const iterations = options.edgeOptimization?.iterations ?? 12;
    let optimizedLayers = layerByLayerMinimization(layers, subtreeRels, iterations);

    // Phase 2: Sifting
    optimizedLayers = siftingMinimization(optimizedLayers, subtreeRels, iterations);

    // Apply the optimized ordering back to positions
    const reordered = applyLayerOrdering(result, optimizedLayers);
    reordered.forEach((value, key) => result.set(key, value));

    // Recurse to children
    for (const child of node.children) {
      optimizeNode(child);
    }
  }

  // Helper: collect all descendants of a node
  function collectDescendants(node: HierarchyNode): HierarchyNode[] {
    const descendants: HierarchyNode[] = [];
    function collect(n: HierarchyNode) {
      descendants.push(n);
      for (const child of n.children) {
        collect(child);
      }
    }
    for (const child of node.children) {
      collect(child);
    }
    return descendants;
  }

  for (const root of tree.roots) {
    optimizeNode(root);
  }

  return result;
}

/**
 * Final pass: Ensure all parents properly contain their children
 * This must run AFTER all other optimization passes
 */
function ensureProperContainment(
  positioned: Map<C4Id, PositionedNode>,
  tree: HierarchyTree,
  padding: number = 16
): Map<C4Id, PositionedNode> {
  const result = new Map(positioned);

  // Post-order traversal: fix children before parents
  function fixContainment(node: HierarchyNode): void {
    // First, fix all children
    for (const child of node.children) {
      fixContainment(child);
    }

    // Then ensure this node contains its children
    if (node.children.length > 0 && !node.node.collapseChildren) {
      resizeParentToFitChildren(node, result, padding);
    }
  }

  for (const root of tree.roots) {
    fixContainment(root);
  }

  return result;
}

/**
 * Apply all optimization passes to the layout
 */
export function applyMultiPassOptimization(
  positioned: Map<C4Id, PositionedNode>,
  relationships: C4Relationship[],
  tree: HierarchyTree,
  options: OptimizationOptions
): Map<C4Id, PositionedNode> {
  if (!options.enabled) {
    console.info("[OPTIMIZER] Disabled, skipping");
    return positioned;
  }

  console.info("[OPTIMIZER] Starting multi-pass optimization:", {
    nodes: positioned.size,
    relationships: relationships.length,
    roots: tree.roots.length,
    options,
  });

  let result = positioned;

  // Pass 1: Remove overlaps (bottom-up)
  if (options.overlapRemoval) {
    console.info("[OPTIMIZER] Pass 1: Overlap removal");
    const before = new Map(result);
    result = removeOverlapsBottomUp(result, tree, options);

    let movedCount = 0;
    result.forEach((after, id) => {
      const beforeNode = before.get(id);
      if (
        beforeNode &&
        (beforeNode.bbox.x !== after.bbox.x || beforeNode.bbox.y !== after.bbox.y)
      ) {
        movedCount++;
      }
    });
    console.info("[OPTIMIZER] Pass 1 complete, nodes moved:", movedCount);
  }

  // Pass 2: Distribute space (top-down)
  if (options.spaceDistribution?.enabled) {
    console.info("[OPTIMIZER] Pass 2: Space distribution");
    const before = new Map(result);
    result = distributeSpaceTopDown(result, tree, options);

    let movedCount = 0;
    result.forEach((after, id) => {
      const beforeNode = before.get(id);
      if (
        beforeNode &&
        (beforeNode.bbox.x !== after.bbox.x || beforeNode.bbox.y !== after.bbox.y)
      ) {
        movedCount++;
      }
    });
    console.info("[OPTIMIZER] Pass 2 complete, nodes moved:", movedCount);
  }

  // Pass 3: Optimize for edges
  if (options.edgeOptimization?.enabled) {
    console.info("[OPTIMIZER] Pass 3: Edge optimization");
    const before = new Map(result);
    result = optimizeForEdges(result, relationships, tree, options);

    let movedCount = 0;
    result.forEach((after, id) => {
      const beforeNode = before.get(id);
      if (
        beforeNode &&
        (beforeNode.bbox.x !== after.bbox.x || beforeNode.bbox.y !== after.bbox.y)
      ) {
        movedCount++;
      }
    });
    console.info("[OPTIMIZER] Pass 3 complete, nodes moved:", movedCount);
  }

  // CRITICAL: Final cleanup pass to fix containment
  // After all optimizations that might move nodes, ensure parents contain children
  console.info("[OPTIMIZER] Final pass: Ensuring proper containment");
  const padding = options.overlapRemoval?.padding ?? 16;
  result = ensureProperContainment(result, tree, padding);

  console.info("[OPTIMIZER] All passes complete");
  return result;
}

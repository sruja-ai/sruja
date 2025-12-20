/**
 * Layout Algorithm Phase - Enhanced
 *
 * Applies level-specific layout algorithms:
 * - L0: Grid/flow layout for enterprise overview
 * - L1: Radial layout with central systems and satellite nodes
 * - L2/L3: Sugiyama hierarchical layout with crossing minimization
 *
 * Migrated from legacy c4-layout.ts, sugiyama.ts, and l1-layout.ts
 */

import type { LayoutPhase, LayoutContext, LayoutNode, Size } from "../core/types";
import { layoutSugiyama as layoutSugiyamaLegacy } from "../algorithms/sugiyama";
import { InteractivePreset, type C4LayoutOptions } from "../c4-options";
import {
  EXTERNAL_GAP,
  H_SPACING,
  V_SPACING,
  ACTOR_ZONE_OFFSET,
} from "../constants";
import {
  classifyC4Node,
  getSemanticRank,
} from "../algorithms/c4-classifier";

// ============================================================================
// PHASE DEFINITION
// ============================================================================

export function createLayoutPhase(): LayoutPhase {
  return {
    name: "layout",
    description: "Apply level-specific layout algorithms",
    dependencies: ["hierarchy", "sizing"],
    execute: (context: LayoutContext): LayoutContext => {
      // Use different layout strategies based on C4 level
      switch (context.options.strategy) {
        case "L0-landscape":
          return applyL0GridLayout(context);
        case "L1-context":
          return applyL1RadialLayout(context);
        case "L2-container":
          return applyL2ContainerLayout(context);
        case "L3-component":
          return applySugiyamaLayout(context);
        default:
          return applySugiyamaLayout(context);
      }
    },
  };
}

// ============================================================================
// L0: GRID/FLOW LAYOUT
// ============================================================================

function applyL0GridLayout(context: LayoutContext): LayoutContext {
  const nodes = new Map(context.nodes);
  const spacing = context.options.spacing;

  let yOffset = 100;
  let xOffset = 100;
  let currentRowWidth = 0;
  const maxRowWidth = 1200;

  // Get root nodes (no parent)
  const rootNodes = Array.from(nodes.values())
    .filter((n) => !n.parent && n.visible)
    .sort((a, b) => a.id.localeCompare(b.id)); // Deterministic order

  for (const node of rootNodes) {
    const nodeWidth = node.bbox.width + spacing.siblingSpacing;

    // Wrap to new row if needed
    if (currentRowWidth + nodeWidth > maxRowWidth) {
      xOffset = 100;
      yOffset += 200;
      currentRowWidth = 0;
    }

    nodes.set(node.id, updateNodePosition(node, xOffset, yOffset));

    xOffset += nodeWidth;
    currentRowWidth += nodeWidth;
  }

  return { ...context, nodes };
}

// ============================================================================
// L1: RADIAL LAYOUT WITH C4 SEMANTIC ZONES
// ============================================================================

function applyL1RadialLayout(context: LayoutContext): LayoutContext {
  const nodes = new Map(context.nodes);
  const spacing = context.options.spacing;

  const rootNodes = Array.from(nodes.values())
    .filter((n) => !n.parent && n.visible)
    .sort((a, b) => a.id.localeCompare(b.id));

  if (rootNodes.length === 0) return { ...context, nodes };

  const mainSystems: LayoutNode[] = [];
  const persons: LayoutNode[] = [];
  const externalServices: LayoutNode[] = [];
  const externalData: LayoutNode[] = [];
  const otherExternals: LayoutNode[] = [];

  // rootNodes are already sorted, but process in order for determinism
  for (const node of rootNodes) {
    const classification = classifyC4Node({
      id: node.id,
      original: node.original as any,
    });

    if (classification.tier === "actor") {
      persons.push(node);
    } else if (classification.role === "System" || classification.role === "Container") {
      if (classification.isExternal) {
        if (classification.isDataLayer) {
          externalData.push(node);
        } else {
          externalServices.push(node);
        }
      } else {
        mainSystems.push(node);
      }
    } else if (classification.isDataLayer) {
      externalData.push(node);
    } else if (classification.isExternal) {
      externalServices.push(node);
    } else {
      otherExternals.push(node);
    }
  }

  // CRITICAL: Sort all groups for deterministic positioning
  // Since rootNodes are already sorted, groups maintain relative order, but we sort for absolute determinism
  // Use non-mutating sort to avoid side effects
  const sortedMainSystems = [...mainSystems].sort((a, b) => a.id.localeCompare(b.id));
  const sortedPersons = [...persons].sort((a, b) => a.id.localeCompare(b.id));
  const sortedExternalServices = [...externalServices].sort((a, b) => a.id.localeCompare(b.id));
  const sortedExternalData = [...externalData].sort((a, b) => a.id.localeCompare(b.id));
  const sortedOtherExternals = [...otherExternals].sort((a, b) => a.id.localeCompare(b.id));

  if (sortedMainSystems.length === 0 && sortedPersons.length === 0) {
    return applyCircleLayout(context, rootNodes);
  }

  const centerBBox = layoutCentralClusterC4(sortedMainSystems, nodes, spacing.siblingSpacing);

  const centerX = centerBBox.x + centerBBox.width / 2;
  const centerY = centerBBox.y + centerBBox.height / 2;

  if (sortedPersons.length > 0) {
    const topY = centerBBox.y - ACTOR_ZONE_OFFSET;
    layoutNodesHorizontally(sortedPersons, centerX, topY, H_SPACING, nodes);
  }

  if (sortedExternalData.length > 0) {
    const bottomY = centerBBox.y + centerBBox.height + EXTERNAL_GAP;
    layoutNodesHorizontally(sortedExternalData, centerX, bottomY, H_SPACING, nodes);
  }

  if (sortedExternalServices.length > 0) {
    const rightX = centerBBox.x + centerBBox.width + EXTERNAL_GAP;
    layoutNodesVertically(sortedExternalServices, rightX, centerY, V_SPACING, nodes);
  }

  if (sortedOtherExternals.length > 0) {
    const leftX = centerBBox.x - EXTERNAL_GAP;
    layoutNodesVertically(sortedOtherExternals, leftX, centerY, V_SPACING, nodes, true);
  }

  for (const rootNode of rootNodes) {
    layoutChildrenRecursive(rootNode.id, nodes, spacing);
  }

  for (const rootNode of rootNodes) {
    resizeParentToFitChildrenL1(rootNode, nodes, spacing);
  }

  return { ...context, nodes };
}

function layoutNodesHorizontally(
  nodeList: LayoutNode[],
  centerX: number,
  y: number,
  spacing: number,
  nodes: Map<string, LayoutNode>
): void {
  if (nodeList.length === 0) return;

  const totalWidth =
    nodeList.reduce((sum, n) => sum + n.bbox.width, 0) + (nodeList.length - 1) * spacing;
  let currentX = centerX - totalWidth / 2;

  for (const node of nodeList) {
    nodes.set(node.id, updateNodePosition(node, currentX, y));
    currentX += node.bbox.width + spacing;
  }
}

function layoutNodesVertically(
  nodeList: LayoutNode[],
  x: number,
  centerY: number,
  spacing: number,
  nodes: Map<string, LayoutNode>,
  alignRight: boolean = false
): void {
  if (nodeList.length === 0) return;

  const totalHeight =
    nodeList.reduce((sum, n) => sum + n.bbox.height, 0) + (nodeList.length - 1) * spacing;
  let currentY = centerY - totalHeight / 2;

  for (const node of nodeList) {
    const nodeX = alignRight ? x - node.bbox.width : x;
    nodes.set(node.id, updateNodePosition(node, nodeX, currentY));
    currentY += node.bbox.height + spacing;
  }
}

interface BoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

function layoutCentralClusterC4(
  mainSystems: LayoutNode[],
  nodes: Map<string, LayoutNode>,
  spacing: number
): BoundingBox {
  if (mainSystems.length === 0) {
    return { x: 0, y: 0, width: 200, height: 150 };
  }

  // CRITICAL: Sort mainSystems for deterministic positioning
  // (Should already be sorted, but ensure)
  const sortedSystems = [...mainSystems].sort((a, b) => a.id.localeCompare(b.id));

  const minSpacing = Math.max(spacing, H_SPACING);
  let totalWidth = 0;
  let maxHeight = 0;

  for (const node of sortedSystems) {
    totalWidth += node.bbox.width;
    maxHeight = Math.max(maxHeight, node.bbox.height);
  }
  totalWidth += (sortedSystems.length - 1) * minSpacing;

  let currentX = -totalWidth / 2;
  const centerY = 0;

  for (const node of sortedSystems) {
    const x = currentX;
    const y = centerY - node.bbox.height / 2;
    nodes.set(node.id, updateNodePosition(node, x, y));
    currentX += node.bbox.width + minSpacing;
  }

  return {
    x: -totalWidth / 2,
    y: -maxHeight / 2,
    width: totalWidth,
    height: maxHeight,
  };
}

function applyCircleLayout(context: LayoutContext, rootNodes: LayoutNode[]): LayoutContext {
  const nodes = new Map(context.nodes);
  // CRITICAL: Sort rootNodes for deterministic positioning
  // (Should already be sorted, but ensure)
  const sortedRootNodes = [...rootNodes].sort((a, b) => a.id.localeCompare(b.id));
  const count = sortedRootNodes.length;

  // Calculate radius based on node count
  const radius = (count * 150) / (2 * Math.PI) + 200;

  let angle = -Math.PI / 2;
  const angleStep = (2 * Math.PI) / count;

  for (const node of sortedRootNodes) {
    const cx = Math.cos(angle) * radius;
    const cy = Math.sin(angle) * radius;

    const x = cx - node.bbox.width / 2;
    const y = cy - node.bbox.height / 2;

    nodes.set(node.id, updateNodePosition(node, x, y));
    angle += angleStep;
  }

  return { ...context, nodes };
}

// ============================================================================
// L2: CONTAINER VIEW COMPOSITION
// ============================================================================

function applyL2ContainerLayout(context: LayoutContext): LayoutContext {
  const nodes = new Map(context.nodes);
  const spacing = context.options.spacing;

  // Root nodes in L2 should mostly be: people/external systems + one system boundary.
  const rootNodes = Array.from(nodes.values()).filter((n) => !n.parent && n.visible);
  if (rootNodes.length === 0) return { ...context, nodes };

  // Choose the primary boundary/system.
  // Heuristic: prefer a non-external SoftwareSystem/SystemBoundary-like node.
  const isSystemLike = (n: LayoutNode) => {
    const kind = ((n.original as any).kind || n.original.type || "") as string;
    return kind === "SoftwareSystem" || kind === "System" || kind === "SystemBoundary";
  };

  const isExternal = (n: LayoutNode) => {
    const raw = n.original as any;
    return raw.isExternal === true || raw.kind === "ExternalSystem";
  };

  const primary =
    rootNodes.find((n) => isSystemLike(n) && !isExternal(n)) ||
    rootNodes.find((n) => isSystemLike(n)) ||
    rootNodes[0];

  // Collect satellites at root level (persons + externals).
  const satellites = rootNodes.filter((n) => n.id !== primary.id);

  // Layout children inside primary boundary using legacy L2 algorithm.
  // This gives a stable internal structure for containers.
  if (!primary.collapsed && primary.children.length > 0) {
    // CRITICAL: Sort children for deterministic processing
    const childNodes = primary.children
      .filter((c) => c.visible)
      .sort((a, b) => a.id.localeCompare(b.id));

    // Edges among children only (avoid mixing with satellite edges).
    // CRITICAL: Sort relationships for deterministic processing order
    const internalEdges = [...context.graph.relationships]
      .sort((a, b) => a.id.localeCompare(b.id))
      .filter((r) => {
        const src = nodes.get(r.from);
        const tgt = nodes.get(r.to);
        return (
          src?.visible &&
          tgt?.visible &&
          src.parent?.id === primary.id &&
          tgt.parent?.id === primary.id
        );
      })
      .map((r) => ({ from: r.from as any, to: r.to as any }));

    // Create a C4LayoutOptions object for the legacy layout function.
    const legacyOptions: C4LayoutOptions = {
      ...InteractivePreset,
      direction: "TB",
      // Use spacing from modular engine as a baseline
      spacing: {
        ...InteractivePreset.spacing,
        node: {
          ...InteractivePreset.spacing.node,
          Container: Math.max(60, spacing.siblingSpacing),
        },
        rank: { ...InteractivePreset.spacing.rank, Container: Math.max(80, spacing.levelSpacing) },
        padding: { ...InteractivePreset.spacing.padding },
      },
    };

    const legacyNodeData = childNodes.map((c) => ({
      id: c.id as any,
      size: { width: c.bbox.width, height: c.bbox.height },
      isExternal: isExternal(c),
    }));

    const legacy = layoutSugiyamaLegacy(legacyNodeData, internalEdges, {
      ...legacyOptions,
      // Force lanes within the boundary (big structural win):
      // - data/messaging at bottom
      // - everything else above
      constraints: buildL2InternalConstraints(childNodes, legacyOptions),
    });

    // Place primary at origin for now; we resize after.
    const padding = Math.max(80, spacing.parentChildPadding);
    const header = 40;

    // Position primary boundary
    nodes.set(primary.id, updateNodePosition(primary, 0, 0));

    // Apply child positions relative to parent top-left.
    for (const [id, ln] of legacy.nodes) {
      const child = nodes.get(id as string);
      if (!child) continue;
      nodes.set(child.id, updateNodePosition(child, padding + ln.x, padding + header + ln.y));
    }

    // Resize primary to fit children + padding (L2 needs more breathing room)
    resizeParentToFitChildren(primary, nodes, {
      parentChildPadding: padding,
      labelHeight: header,
      safetyMargin: 40,
    });
  }

  // Place satellites around the boundary (simple but deterministic):
  // - persons on top
  // - externals on left/right
  // - external data systems at bottom
  placeL2Satellites(primary, satellites, nodes);

  return { ...context, nodes };
}

function buildL2InternalConstraints(children: LayoutNode[], options: C4LayoutOptions) {
  const rankOf: Partial<Record<any, number>> = {};
  const sameRank: Array<readonly any[]> = [];

  const presentationTier: any[] = [];
  const logicTier: any[] = [];
  const dataTier: any[] = [];

  for (const n of children) {
    const id = n.id as any;
    const classification = classifyC4Node({
      id: n.id,
      original: n.original as any,
    });

    rankOf[id] = classification.rank;

    if (classification.tier === "presentation") {
      presentationTier.push(id);
    } else if (classification.tier === "data") {
      dataTier.push(id);
    } else {
      logicTier.push(id);
    }
  }

  if (presentationTier.length > 1) sameRank.push(presentationTier);
  if (logicTier.length > 1) sameRank.push(logicTier);
  if (dataTier.length > 1) sameRank.push(dataTier);

  const sorted = [...children].sort((a, b) => a.id.localeCompare(b.id));
  const orderHint: Partial<Record<any, number>> = {};
  sorted.forEach((c, idx) => {
    orderHint[c.id as any] = idx;
  });

  return { ...options.constraints, rankOf, sameRank, orderHint };
}

function resizeParentToFitChildren(
  parent: LayoutNode,
  nodes: Map<string, LayoutNode>,
  opts: { parentChildPadding: number; labelHeight: number; safetyMargin: number }
) {
  const current = nodes.get(parent.id);
  if (!current) return;

  const visibleChildren = current.children
    .map((c) => nodes.get(c.id))
    .filter(Boolean) as LayoutNode[];
  if (visibleChildren.length === 0) return;

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const child of visibleChildren) {
    minX = Math.min(minX, child.bbox.x);
    minY = Math.min(minY, child.bbox.y);
    maxX = Math.max(maxX, child.bbox.x + child.bbox.width);
    maxY = Math.max(maxY, child.bbox.y + child.bbox.height);
  }

  const requiredLeft = minX - opts.parentChildPadding - opts.safetyMargin;
  const requiredTop = minY - opts.parentChildPadding - opts.labelHeight - opts.safetyMargin;
  const requiredRight = maxX + opts.parentChildPadding + opts.safetyMargin;
  const requiredBottom = maxY + opts.parentChildPadding + opts.safetyMargin;

  const newX = Math.min(current.bbox.x, requiredLeft);
  const newY = Math.min(current.bbox.y, requiredTop);
  const newWidth = Math.max(current.bbox.width, requiredRight - newX);
  const newHeight = Math.max(current.bbox.height, requiredBottom - newY);

  nodes.set(parent.id, {
    ...current,
    bbox: { x: newX, y: newY, width: newWidth, height: newHeight },
  });
}

function placeL2Satellites(
  boundary: LayoutNode,
  satellites: LayoutNode[],
  nodes: Map<string, LayoutNode>
) {
  if (satellites.length === 0) return;

  const b = nodes.get(boundary.id);
  if (!b) return;

  // CRITICAL: Sort satellites for deterministic processing
  const sortedSatellites = [...satellites].sort((a, b) => a.id.localeCompare(b.id));
  const persons: LayoutNode[] = [];
  const externals: LayoutNode[] = [];

  for (const s of sortedSatellites) {
    const kind = ((s.original as any).kind || s.original.type || "") as string;
    if (kind === "Person") persons.push(s);
    else externals.push(s);
  }

  // CRITICAL: Sort persons and externals for deterministic positioning
  // Use non-mutating sort to avoid side effects
  const sortedPersons = [...persons].sort((a, b) => a.id.localeCompare(b.id));
  const sortedExternals = [...externals].sort((a, b) => a.id.localeCompare(b.id));

  // Top row: persons
  if (sortedPersons.length > 0) {
    let x = b.bbox.x;
    const y = b.bbox.y - EXTERNAL_GAP;
    for (const p of sortedPersons) {
      nodes.set(p.id, updateNodePosition(p, x, y - p.bbox.height));
      x += p.bbox.width + 60;
    }
  }

  // Left/right columns: externals (split)
  if (sortedExternals.length > 0) {
    const midY = b.bbox.y + b.bbox.height / 2;
    const leftX = b.bbox.x - EXTERNAL_GAP;
    const rightX = b.bbox.x + b.bbox.width + EXTERNAL_GAP;

    const left = sortedExternals.filter((_, i) => i % 2 === 0);
    const right = sortedExternals.filter((_, i) => i % 2 === 1);

    const placeColumn = (items: LayoutNode[], x: number) => {
      // CRITICAL: Sort items for deterministic positioning
      const sortedItems = [...items].sort((a, b) => a.id.localeCompare(b.id));
      const totalH =
        sortedItems.reduce((acc, n) => acc + n.bbox.height, 0) + Math.max(0, sortedItems.length - 1) * 40;
      let y = midY - totalH / 2;
      for (const n of sortedItems) {
        nodes.set(n.id, updateNodePosition(n, x - (x < b.bbox.x ? n.bbox.width : 0), y));
        y += n.bbox.height + 40;
      }
    };

    placeColumn(left, leftX);
    placeColumn(right, rightX);
  }
}

// ============================================================================
// L2/L3: SUGIYAMA HIERARCHICAL LAYOUT
// ============================================================================

interface SugiyamaNode {
  id: string;
  size: Size;
  layer: number;
  order: number;
  x: number;
  y: number;
}

function applySugiyamaLayout(context: LayoutContext): LayoutContext {
  const nodes = new Map(context.nodes);
  const spacing = context.options.spacing;

  const rootNodes = Array.from(nodes.values())
    .filter((n) => !n.parent && n.visible)
    .sort((a, b) => a.id.localeCompare(b.id));

  if (rootNodes.length === 0) return { ...context, nodes };

  // CRITICAL: Sort relationships for deterministic processing order
  const relationships = [...context.graph.relationships]
    .sort((a, b) => a.id.localeCompare(b.id))
    .filter((r) => {
      const src = nodes.get(r.from);
      const tgt = nodes.get(r.to);
      return src?.visible && tgt?.visible && !src.parent && !tgt.parent;
    })
    .map((r) => ({ from: r.from, to: r.to }));

  const nodeData = rootNodes.map((n) => ({
    id: n.id,
    size: { width: n.bbox.width, height: n.bbox.height },
  }));

  const semanticRanks = new Map<string, number>();
  for (const node of rootNodes) {
    const rank = getSemanticRank({
      id: node.id,
      original: node.original as any,
    });
    semanticRanks.set(node.id, rank);
  }

  const result = layoutSugiyamaWithSemantics(nodeData, relationships, spacing, semanticRanks);

  for (const [id, sugNode] of result.nodes) {
    const original = nodes.get(id);
    if (original) {
      nodes.set(id, updateNodePosition(original, sugNode.x, sugNode.y));
    }
  }

  for (const rootNode of rootNodes) {
    layoutChildrenRecursive(rootNode.id, nodes, spacing);
  }

  return { ...context, nodes };
}

function layoutSugiyamaWithSemantics(
  nodeData: { id: string; size: Size }[],
  relationships: { from: string; to: string }[],
  spacing: { siblingSpacing: number; levelSpacing: number },
  semanticRanks: Map<string, number>
): { width: number; height: number; nodes: Map<string, SugiyamaNode> } {
  const resultNodes = new Map<string, SugiyamaNode>();
  const nodeMap = new Map(nodeData.map((n) => [n.id, n]));

  const layerMap = assignLayersWithSemantics(nodeData, relationships, semanticRanks);
  const layers: string[][] = [];

  // CRITICAL: Sort entries to ensure deterministic iteration order
  const sortedLayerEntries = Array.from(layerMap.entries()).sort((a, b) => {
    // First sort by layer, then by ID for determinism
    if (a[1] !== b[1]) return a[1] - b[1];
    return a[0].localeCompare(b[0]);
  });

  for (const [id, layer] of sortedLayerEntries) {
    if (!layers[layer]) layers[layer] = [];
    layers[layer].push(id);
  }

  // CRITICAL: Sort each layer by ID for deterministic ordering (done once, reused)
  for (let l = 0; l < layers.length; l++) {
    if (layers[l] && layers[l].length > 0) {
      layers[l].sort((a, b) => a.localeCompare(b));
    }
  }

  const MAX_ITERATIONS = 200; // Increased from 150 for better convergence
  let prevCrossings = Infinity;

  for (let i = 0; i < MAX_ITERATIONS; i++) {
    orderLayers(layers, relationships, true);
    orderLayers(layers, relationships, false);

    const crossings = countCrossings(layers, relationships);
    if (crossings === 0 || crossings >= prevCrossings) break;
    prevCrossings = crossings;
  }

  const nodeSpacing = spacing.siblingSpacing;
  const rankSpacing = spacing.levelSpacing;

  let maxWidth = 0;
  const layerWidths: number[] = [];

  for (let l = 0; l < layers.length; l++) {
    // CRITICAL: Ensure layerIds are sorted for deterministic width calculation
    const layerIds = (layers[l] || []).sort((a, b) => a.localeCompare(b));
    if (layerIds.length === 0) {
      layerWidths.push(0);
      continue;
    }

    const totalWidth =
      layerIds.reduce((sum, id) => sum + (nodeMap.get(id)?.size.width || 100), 0) +
      nodeSpacing * (layerIds.length - 1);
    layerWidths.push(totalWidth);
    maxWidth = Math.max(maxWidth, totalWidth);
  }

  let currentY = 0;
  for (let l = 0; l < layers.length; l++) {
    // Layers already sorted above, reuse
    const layerIds = layers[l] || [];
    if (layerIds.length === 0) continue;

    let maxHeight = 0;
    for (const id of layerIds) {
      const node = nodeMap.get(id);
      maxHeight = Math.max(maxHeight, node?.size.height || 100);
    }

    const layerW = layerWidths[l];
    let currentX = (maxWidth - layerW) / 2;

    // Layers already sorted above, use directly
    for (let i = 0; i < layerIds.length; i++) {
      const id = layerIds[i];
      const node = nodeMap.get(id)!;
      const offY = (maxHeight - node.size.height) / 2;

      resultNodes.set(id, {
        id,
        size: node.size,
        layer: l,
        order: i,
        x: currentX,
        y: currentY + offY,
      });

      currentX += node.size.width + nodeSpacing;
    }

    currentY += maxHeight + rankSpacing;
  }

  return {
    width: maxWidth,
    height: Math.max(0, currentY - rankSpacing),
    nodes: resultNodes,
  };
}

function assignLayersWithSemantics(
  nodes: { id: string }[],
  relationships: { from: string; to: string }[],
  semanticRanks: Map<string, number>
): Map<string, number> {
  const layers = new Map<string, number>();
  const outgoing = new Map<string, string[]>();
  const incoming = new Map<string, string[]>();

  // CRITICAL: Sort nodes once for deterministic processing order
  const sortedNodes = [...nodes].sort((a, b) => a.id.localeCompare(b.id));
  for (const node of sortedNodes) {
    outgoing.set(node.id, []);
    incoming.set(node.id, []);
    layers.set(node.id, 0);
  }

  // CRITICAL: Sort relationships once for deterministic processing order
  const sortedRelationships = [...relationships].sort((a, b) => {
    if (a.from !== b.from) return a.from.localeCompare(b.from);
    return a.to.localeCompare(b.to);
  });

  for (const rel of sortedRelationships) {
    if (outgoing.has(rel.from) && incoming.has(rel.to)) {
      outgoing.get(rel.from)!.push(rel.to);
      incoming.get(rel.to)!.push(rel.from);
    }
  }

  const visited = new Set<string>();
  const stack: string[] = [];

  // CRITICAL: Pre-sort all outgoing lists once for efficiency
  for (const [, targets] of outgoing) {
    if (targets.length > 1) {
      targets.sort((a, b) => a.localeCompare(b));
    }
  }

  function visit(id: string) {
    if (visited.has(id)) return;
    visited.add(id);
    // Targets already sorted above
    for (const tgt of outgoing.get(id) || []) {
      visit(tgt);
    }
    stack.push(id);
  }

  // CRITICAL: Use sorted nodes for deterministic traversal
  for (const node of sortedNodes) {
    visit(node.id);
  }

  // CRITICAL: Pre-sort all incoming lists once for efficiency
  for (const [, parents] of incoming) {
    if (parents.length > 1) {
      parents.sort((a, b) => a.localeCompare(b));
    }
  }

  // CRITICAL: Process stack in LIFO order (pop), but ensure deterministic by sorting stack
  // Since we visit nodes in sorted order, stack should be deterministic, but sort to be safe
  const sortedStack = [...stack].sort((a, b) => a.localeCompare(b));
  // Process in reverse sorted order to maintain LIFO semantics while being deterministic
  for (let i = sortedStack.length - 1; i >= 0; i--) {
    const id = sortedStack[i];
    let maxParentLayer = -1;
    // Parents already sorted above
    for (const parent of incoming.get(id) || []) {
      maxParentLayer = Math.max(maxParentLayer, layers.get(parent) || 0);
    }
    layers.set(id, maxParentLayer + 1);
  }

  const semanticGroups = new Map<number, string[]>();
  // CRITICAL: Sort semanticRanks entries for deterministic iteration
  const sortedSemanticRanks = Array.from(semanticRanks.entries()).sort((a, b) => a[0].localeCompare(b[0]));
  
  for (const [id, semanticRank] of sortedSemanticRanks) {
    const tier = Math.floor(semanticRank / 30);
    const group = semanticGroups.get(tier) || [];
    group.push(id);
    semanticGroups.set(tier, group);
  }

  // Sort each group by ID for determinism
  for (const [, group] of semanticGroups) {
    group.sort((a, b) => a.localeCompare(b));
  }

  const sortedTiers = Array.from(semanticGroups.keys()).sort((a, b) => a - b);
  let currentLayer = 0;
  const adjustedLayers = new Map<string, number>();

  for (const tier of sortedTiers) {
    const nodesInTier = semanticGroups.get(tier) || [];

    const layerAssignments = new Map<number, string[]>();
    // CRITICAL: Sort nodesInTier for deterministic processing
    const sortedNodesInTier = [...nodesInTier].sort((a, b) => a.localeCompare(b));
    
    for (const id of sortedNodesInTier) {
      const originalLayer = layers.get(id) || 0;
      const group = layerAssignments.get(originalLayer) || [];
      group.push(id);
      layerAssignments.set(originalLayer, group);
    }

    // CRITICAL: Sort each group for determinism
    for (const [, group] of layerAssignments) {
      group.sort((a, b) => a.localeCompare(b));
    }

    const sortedOriginalLayers = Array.from(layerAssignments.keys()).sort((a, b) => a - b);
    for (const origLayer of sortedOriginalLayers) {
      const nodesAtLayer = layerAssignments.get(origLayer) || [];
      // CRITICAL: Process in sorted order
      for (const id of nodesAtLayer) {
        adjustedLayers.set(id, currentLayer);
      }
      currentLayer++;
    }
  }

  // CRITICAL: Use sorted nodes for deterministic processing
  for (const node of sortedNodes) {
    if (!adjustedLayers.has(node.id)) {
      adjustedLayers.set(node.id, layers.get(node.id) || 0);
    }
  }

  return adjustedLayers;
}

function orderLayers(
  layers: string[][],
  relationships: { from: string; to: string }[],
  downward: boolean
): void {
  const start = downward ? 1 : layers.length - 2;
  const end = downward ? layers.length : -1;
  const step = downward ? 1 : -1;

    // Build adjacency for barycenter calculation
    const neighbors = new Map<string, string[]>();
    // CRITICAL: Sort relationships once for deterministic processing
    const sortedRels = [...relationships].sort((a, b) => {
      if (a.from !== b.from) return a.from.localeCompare(b.from);
      return a.to.localeCompare(b.to);
    });
    
    for (const rel of sortedRels) {
      if (!neighbors.has(rel.from)) neighbors.set(rel.from, []);
      if (!neighbors.has(rel.to)) neighbors.set(rel.to, []);
      neighbors.get(rel.from)!.push(rel.to);
      neighbors.get(rel.to)!.push(rel.from);
    }

    // CRITICAL: Sort neighbor lists once for deterministic barycenter calculation
    for (const [, neighborList] of neighbors) {
      if (neighborList.length > 1) {
        neighborList.sort((a, b) => a.localeCompare(b));
      }
    }

  for (let l = start; l !== end; l += step) {
    const currentLayer = layers[l];
    if (!currentLayer || currentLayer.length === 0) continue;

    const prevLayer = layers[l - step];
    if (!prevLayer || prevLayer.length === 0) continue;

    // Build position map for previous layer
    const prevPos = new Map<string, number>();
    prevLayer.forEach((id, idx) => prevPos.set(id, idx));

    // Calculate barycenter and median for each node in current layer
    const barycenters = new Map<string, number>();
    const medians = new Map<string, number>();
    
    // CRITICAL: Process nodes in sorted order for determinism
    // Sort in-place for efficiency (mutate currentLayer since it's being rebuilt anyway)
    if (currentLayer.length > 1) {
      currentLayer.sort((a, b) => a.localeCompare(b));
    }
    const sortedCurrentLayer = currentLayer;
    
    for (const id of sortedCurrentLayer) {
      const nodeNeighbors = neighbors.get(id) || [];
      // Neighbors already sorted above, just filter
      const prevNeighbors = nodeNeighbors.filter((n) => prevPos.has(n));

      if (prevNeighbors.length > 0) {
        const positions = prevNeighbors.map((n) => prevPos.get(n) || 0).sort((a, b) => a - b);
        const sum = positions.reduce((s, p) => s + p, 0);
        barycenters.set(id, sum / prevNeighbors.length);
        
        // Calculate median for tie-breaking
        const mid = Math.floor(positions.length / 2);
        medians.set(id, positions.length % 2 === 0 
          ? (positions[mid - 1] + positions[mid]) / 2 
          : positions[mid]);
      } else {
        // Keep original position if no neighbors in prev layer
        const origPos = sortedCurrentLayer.indexOf(id);
        barycenters.set(id, origPos);
        medians.set(id, origPos);
      }
    }

    // Sort by barycenter, then by median for tie-breaking, then by ID for final determinism
    // Sort in-place for efficiency (currentLayer is already sorted by ID above)
    layers[l] = sortedCurrentLayer.sort((a, b) => {
      const ba = barycenters.get(a) ?? 0;
      const bb = barycenters.get(b) ?? 0;
      if (Math.abs(ba - bb) < 0.001) {
        // Tie: use median heuristic
        const ma = medians.get(a) ?? 0;
        const mb = medians.get(b) ?? 0;
        if (Math.abs(ma - mb) < 0.001) {
          // Final tie-breaker: use ID for complete determinism
          return a.localeCompare(b);
        }
        return ma - mb;
      }
      return ba - bb;
    });
  }
}

function countCrossings(layers: string[][], relationships: { from: string; to: string }[]): number {
  let crossings = 0;

  for (let l = 0; l < layers.length - 1; l++) {
    const layer1 = layers[l];
    const layer2 = layers[l + 1];

    if (!layer1 || !layer2) continue;

    const pos1 = new Map<string, number>();
    const pos2 = new Map<string, number>();
    layer1.forEach((id, i) => pos1.set(id, i));
    layer2.forEach((id, i) => pos2.set(id, i));

    // Get edges between these layers
    const edges = relationships
      .filter((r) => (pos1.has(r.from) && pos2.has(r.to)) || (pos1.has(r.to) && pos2.has(r.from)))
      .map((r) => {
        const p1 = pos1.has(r.from) ? pos1.get(r.from)! : pos1.get(r.to)!;
        const p2 = pos2.has(r.to) ? pos2.get(r.to)! : pos2.get(r.from)!;
        return { p1, p2 };
      });

    // Count crossings (O(n²) but acceptable for typical graph sizes)
    for (let i = 0; i < edges.length; i++) {
      for (let j = i + 1; j < edges.length; j++) {
        const e1 = edges[i];
        const e2 = edges[j];
        // Edges cross if they have opposite relative ordering
        if ((e1.p1 < e2.p1 && e1.p2 > e2.p2) || (e1.p1 > e2.p1 && e1.p2 < e2.p2)) {
          crossings++;
        }
      }
    }
  }

  return crossings;
}

function layoutChildrenRecursive(
  parentId: string,
  nodes: Map<string, LayoutNode>,
  spacing: { siblingSpacing: number; levelSpacing: number; parentChildPadding: number }
): void {
  const parent = nodes.get(parentId);
  if (!parent) return;

  // Skip if parent is collapsed
  if (parent.collapsed) return;

  // Get visible children and sort once for deterministic processing
  const children = parent.children
    .filter((c) => c.visible)
    .sort((a, b) => a.id.localeCompare(b.id));
  if (children.length === 0) return;

  // Use minimum padding to ensure children start inside parent
  const MIN_PADDING = 50; // Minimum safe padding
  const padding = Math.max(MIN_PADDING, spacing.parentChildPadding);
  const childSpacing = spacing.siblingSpacing;
  const labelHeight = 30; // Space for parent label

  // Calculate safe bounds within parent
  const safeLeft = parent.bbox.x + padding;
  const safeTop = parent.bbox.y + padding + labelHeight;
  const safeRight = parent.bbox.x + parent.bbox.width - padding;

  // Simple grid layout for children
  let currentX = safeLeft;
  let currentY = safeTop;
  let rowHeight = 0;

  for (const child of children) {
    // Wrap to new row if needed
    if (currentX + child.bbox.width > safeRight) {
      currentX = safeLeft;
      currentY += rowHeight + childSpacing;
      rowHeight = 0;
    }

    // Ensure child fits within parent bounds - CRITICAL: clamp strictly
    const childX = Math.max(safeLeft, Math.min(currentX, safeRight - child.bbox.width));
    const childY = currentY;

    // Double-check bounds before setting position
    const childRight = childX + child.bbox.width;
    const childBottom = childY + child.bbox.height;
    const parentRight = parent.bbox.x + parent.bbox.width;
    const parentBottom = parent.bbox.y + parent.bbox.height;

    // If child would overflow, clamp it
    const finalX = childRight > parentRight - padding 
      ? parentRight - padding - child.bbox.width 
      : childX;
    const finalY = childBottom > parentBottom - padding
      ? parentBottom - padding - child.bbox.height
      : childY;

    // Ensure final position is still within safe bounds
    const clampedX = Math.max(safeLeft, Math.min(finalX, safeRight - child.bbox.width));
    const clampedY = Math.max(safeTop, Math.min(finalY, parentBottom - padding - child.bbox.height));

    nodes.set(child.id, updateNodePosition(child, clampedX, clampedY));
    currentX += child.bbox.width + childSpacing;
    rowHeight = Math.max(rowHeight, child.bbox.height);

    // Recursively layout grandchildren
    layoutChildrenRecursive(child.id, nodes, spacing);
  }
}

/**
 * Resize parent nodes to ensure they contain all their children with adequate padding
 * This is critical for fixing containment violations that cause F scores
 */
function resizeParentToFitChildrenL1(
  node: LayoutNode,
  nodes: Map<string, LayoutNode>,
  spacing: { siblingSpacing: number; levelSpacing: number; parentChildPadding: number }
): void {
  // Skip if node is collapsed or has no children
  if (node.collapsed || node.children.length === 0) return;

  const currentParent = nodes.get(node.id);
  if (!currentParent) return;

  // Get all visible children (including nested)
  const visibleChildren: LayoutNode[] = [];
  function collectChildren(parent: LayoutNode) {
    // CRITICAL: Sort children for deterministic collection order
    const sortedChildren = [...parent.children].sort((a, b) => a.id.localeCompare(b.id));
    for (const child of sortedChildren) {
      const childNode = nodes.get(child.id);
      if (childNode && childNode.visible) {
        visibleChildren.push(childNode);
        if (!childNode.collapsed) {
          collectChildren(childNode);
        }
      }
    }
  }
  collectChildren(currentParent);

  if (visibleChildren.length === 0) return;

  // Calculate bounding box of all children
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const child of visibleChildren) {
    minX = Math.min(minX, child.bbox.x);
    minY = Math.min(minY, child.bbox.y);
    maxX = Math.max(maxX, child.bbox.x + child.bbox.width);
    maxY = Math.max(maxY, child.bbox.y + child.bbox.height);
  }

  if (minX === Infinity) return;

  // Calculate required size with padding (use minimum safe padding)
  const MIN_PADDING = 50; // Minimum safe padding to prevent containment violations
  const padding = Math.max(MIN_PADDING, spacing.parentChildPadding);
  const labelHeight = 30; // Space for parent label
  const safetyMargin = 20; // Extra safety margin

  const requiredLeft = minX - padding - safetyMargin;
  const requiredTop = minY - padding - labelHeight - safetyMargin;
  const requiredRight = maxX + padding + safetyMargin;
  const requiredBottom = maxY + padding + safetyMargin;

  // Calculate current bounds
  const currentLeft = currentParent.bbox.x;
  const currentTop = currentParent.bbox.y;
  const currentRight = currentParent.bbox.x + currentParent.bbox.width;
  const currentBottom = currentParent.bbox.y + currentParent.bbox.height;

  // Determine if resize is needed
  const needsResize =
    requiredLeft < currentLeft ||
    requiredTop < currentTop ||
    requiredRight > currentRight ||
    requiredBottom > currentBottom;

  if (needsResize) {
    const newX = Math.min(currentLeft, requiredLeft);
    const newY = Math.min(currentTop, requiredTop);
    const newWidth = Math.max(currentParent.bbox.width, requiredRight - newX);
    const newHeight = Math.max(currentParent.bbox.height, requiredBottom - newY);

    // Update parent size
    nodes.set(node.id, {
      ...currentParent,
      bbox: { x: newX, y: newY, width: newWidth, height: newHeight },
    });

    // Recursively resize parent's parent if it exists
    if (currentParent.parent) {
      const grandParent = nodes.get(currentParent.parent.id);
      if (grandParent) {
        resizeParentToFitChildrenL1(grandParent, nodes, spacing);
      }
    }
  }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function updateNodePosition(node: LayoutNode, x: number, y: number): LayoutNode {
  return {
    ...node,
    bbox: { ...node.bbox, x, y },
    contentBox: { ...node.contentBox, x: x + 10, y: y + 10 },
    labelBox: { ...node.labelBox, x: x + 10, y: y + 10 },
  };
}

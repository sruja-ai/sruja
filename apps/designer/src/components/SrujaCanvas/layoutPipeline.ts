import { logger } from "@sruja/shared";
import type { SrujaModelDump } from "@sruja/shared";
import type { C4Level, C4Node } from "./types";
import type { GraphvizResult } from "./types";
import { calculateNodeSize } from "./textMeasure";
import type { LayoutQuality, ParentChildRelationships } from "./qualityMetrics";

export type NodeSizeMap = Record<string, { width: number; height: number }>;

export function buildNodeSizeMap(model: SrujaModelDump | null): NodeSizeMap {
  if (!model?.elements) return {};
  const nodeSizes: NodeSizeMap = {};
  Object.values(model.elements).forEach((elem) => {
    nodeSizes[elem.id] = calculateNodeSize(
      elem.title,
      elem.technology ?? undefined,
      typeof elem.description === "string" ? elem.description : undefined,
      elem.kind
    );
  });
  return nodeSizes;
}

/**
 * Build parent/child relationships from the model, restricted to visible nodes.
 *
 * This prevents out-of-scope parents from pulling hidden nodes into L2/L3 views.
 */
export function buildParentChildRelationships(
  model: SrujaModelDump | null,
  visibleNodeIds: Set<string>
): ParentChildRelationships {
  const relationships: ParentChildRelationships = {
    childToParent: new Map<string, string>(),
  };

  if (!model?.elements) return relationships;

  for (const element of Object.values(model.elements)) {
    if (
      element.parent &&
      typeof element.parent === "string" &&
      visibleNodeIds.has(element.id) &&
      visibleNodeIds.has(element.parent)
    ) {
      relationships.childToParent.set(element.id, element.parent);
    }
  }

  return relationships;
}

export function buildC4Nodes({
  layoutResult,
  model,
  nodeSizes,
  level,
}: {
  layoutResult: GraphvizResult;
  model: SrujaModelDump;
  nodeSizes: NodeSizeMap;
  level: C4Level;
}): C4Node[] {
  return layoutResult.nodes.map((layoutNode) => {
    const element = model.elements[layoutNode.id];
    const kind = element?.kind?.toLowerCase() || "container";
    const measuredSize = nodeSizes[layoutNode.id];

    return {
      id: layoutNode.id,
      kind: kind as C4Node["kind"],
      title: element?.title || layoutNode.id,
      technology: element?.technology ?? undefined,
      description: typeof element?.description === "string" ? element.description : undefined,
      level,
      width: measuredSize?.width || layoutNode.width || 200,
      height: measuredSize?.height || layoutNode.height || 120,
      metadata: element?.metadata ?? undefined,
    };
  });
}

export function buildC4Edges(
  relations: Array<{ from: string; to: string; title?: string }>
): Array<{ id: string; source: string; target: string; label: string; technology?: undefined }> {
  return relations.map((rel, idx) => ({
    id: `e-${rel.from}-${rel.to}-${idx}`,
    source: rel.from,
    target: rel.to,
    label: (rel as { label?: string }).label || "",
    technology: undefined,
  }));
}

/**
 * Expose layout quality metrics to the window for dev tooling and e2e tests.
 */
export function exposeQualityMetrics({
  quality,
  parentChildContainmentViolations,
  c4Nodes,
  c4Edges,
  level,
}: {
  quality: LayoutQuality;
  parentChildContainmentViolations: Array<{ childId: string; parentId: string }>;
  c4Nodes: C4Node[];
  c4Edges: Array<{ id: string }>;
  level: number;
}) {
  if (typeof window === "undefined") return;

  const qualityMetrics = {
    score: quality.score,
    edgeCrossings: quality.edgeCrossings,
    nodeOverlaps: quality.nodeOverlaps,
    labelOverlaps: quality.labelOverlaps,
    parentChildContainment: quality.parentChildContainment,
    avgEdgeLength: quality.avgEdgeLength,
    edgeLengthVariance: quality.edgeLengthVariance,
    rankAlignment: quality.rankAlignment,
    clusterBalance: quality.clusterBalance,
    spacingConsistency: quality.spacingConsistency,
    timestamp: Date.now(),
    nodeCount: c4Nodes.length,
    edgeCount: c4Edges.length,
    level: level > 0 ? `L${level}` : "L1",
  } as LayoutQuality & {
    level: string;
    nodeCount: number;
    edgeCount: number;
    timestamp: number;
  };

  (window as unknown as { __DIAGRAM_QUALITY__?: typeof qualityMetrics }).__DIAGRAM_QUALITY__ =
    qualityMetrics;
  (
    window as unknown as {
      __LAYOUT_METRICS__?: Record<string, unknown>;
    }
  ).__LAYOUT_METRICS__ = {
    ...qualityMetrics,
    parentChildContainment: parentChildContainmentViolations,
  };

  logger.debug("Diagram quality metrics", {
    component: "SrujaCanvas",
    action: "calculateLayout",
    metrics: (window as unknown as { __DIAGRAM_QUALITY__?: unknown }).__DIAGRAM_QUALITY__,
  });
}

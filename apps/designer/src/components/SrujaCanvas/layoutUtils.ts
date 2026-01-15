import type { Node as RFNode } from "@xyflow/react";
import type { SrujaModelDump } from "@sruja/shared";
import type { C4Level } from "./types";

export type ManualPositionsMap = Record<
  string,
  { x: number; y: number } | { X: number; Y: number }
>;

export type ChaosState = {
  enabled: boolean;
  failedNodeId: string | null;
};

export type CapacityState = {
  userLoad: number;
};

export function getManualLayoutViewKey(level: number, focusNodeId: string | undefined): string {
  return `manual-layout-${level}-${focusNodeId || "root"}`;
}

export function getLevelViewKey(level: number): string {
  return `L${level}`;
}

/**
 * Select optimal handle positions based on node positions.
 * Chooses the closest sides between source and target nodes.
 */
export function selectOptimalHandles(
  sourceNode: RFNode,
  targetNode: RFNode
): { sourceHandle: string; targetHandle: string } {
  const sourceX = sourceNode.position.x + (sourceNode.width || 0) / 2;
  const sourceY = sourceNode.position.y + (sourceNode.height || 0) / 2;
  const targetX = targetNode.position.x + (targetNode.width || 0) / 2;
  const targetY = targetNode.position.y + (targetNode.height || 0) / 2;

  const dx = targetX - sourceX;
  const dy = targetY - sourceY;
  const absDx = Math.abs(dx);
  const absDy = Math.abs(dy);

  // Determine primary direction
  let sourceHandle: string;
  let targetHandle: string;

  if (absDx > absDy) {
    // Horizontal layout - prefer left/right
    if (dx > 0) {
      // Target is to the right
      sourceHandle = "source-right";
      targetHandle = "target-left";
    } else {
      // Target is to the left
      sourceHandle = "source-left";
      targetHandle = "target-right";
    }
  } else {
    // Vertical layout - prefer top/bottom
    if (dy > 0) {
      // Target is below
      sourceHandle = "source-bottom";
      targetHandle = "target-top";
    } else {
      // Target is above
      sourceHandle = "source-top";
      targetHandle = "target-bottom";
    }
  }

  return { sourceHandle, targetHandle };
}

/**
 * Simple hash function for cache keys
 * Includes model identifier to invalidate cache when model changes
 */
export function hashCacheKey(
  level: C4Level,
  focusNodeId: string | undefined,
  collapsedNodeIds: Set<string>,
  modelId?: string | null
): string {
  const collapsedArray = Array.from(collapsedNodeIds).sort().join(",");
  // Include modelId in cache key to ensure cache is invalidated when model changes
  const modelHash = modelId ? modelId.substring(0, 16) : "no-model";
  return `${level}:${focusNodeId || ""}:${collapsedArray}:${modelHash}`;
}

export function buildLayoutSignature(
  contextKey: string,
  nodes: Array<{ id: string }>,
  relations: Array<{ from: string; to: string }>
): string {
  const nodeKey = nodes.map((node) => node.id).sort().join("|");
  const relationKey = relations.map((rel) => `${rel.from}->${rel.to}`).sort().join("|");
  return `${contextKey}:${nodeKey}:${relationKey}`;
}

export function getManualPositionsMap({
  model,
  level,
  focusNodeId,
  isManualMode,
}: {
  model: SrujaModelDump;
  level: number;
  focusNodeId: string | undefined;
  isManualMode: boolean;
}): { manualPositionsMap: ManualPositionsMap; hasManualPositions: boolean } {
  const manualViewKey = getManualLayoutViewKey(level, focusNodeId);
  const levelViewKey = getLevelViewKey(level);

  const tryGetPositions = (view: unknown): ManualPositionsMap | undefined => {
    if (!view) return undefined;
    type ViewWithLayout = typeof view & {
      layout?: { positions?: ManualPositionsMap };
      Layout?: {
        Positions?: ManualPositionsMap;
        positions?: ManualPositionsMap;
      };
    };
    const viewWithLayout = view as ViewWithLayout | undefined;
    return (
      viewWithLayout?.layout?.positions ||
      viewWithLayout?.Layout?.Positions ||
      viewWithLayout?.Layout?.positions
    );
  };

  let manualPositionsMap: ManualPositionsMap = {};
  if (isManualMode) {
    manualPositionsMap = tryGetPositions(model.views?.[manualViewKey]) || {};
  }

  if (Object.keys(manualPositionsMap).length === 0) {
    manualPositionsMap = tryGetPositions(model.views?.[levelViewKey]) || {};
  }

  if (isManualMode && Object.keys(manualPositionsMap).length === 0 && model.views) {
    for (const view of Object.values(model.views)) {
      const positions = tryGetPositions(view);
      if (positions && Object.keys(positions).length > 0) {
        manualPositionsMap = { ...manualPositionsMap, ...positions };
      }
    }
  }

  return {
    manualPositionsMap,
    hasManualPositions: Object.keys(manualPositionsMap).length > 0,
  };
}

export function applyChaosAndCapacity<T extends RFNode>({
  nodes,
  chaosState,
  impactedNodeIds,
  capacityState,
  themeMode,
}: {
  nodes: T[];
  chaosState: ChaosState;
  impactedNodeIds: Set<string>;
  capacityState: CapacityState;
  themeMode: string;
}): T[] {
  return nodes.map((node) => {
    const isFailed = chaosState.enabled && chaosState.failedNodeId === node.id;
    const isImpacted = chaosState.enabled && impactedNodeIds.has(node.id);
    const isDimmed =
      chaosState.enabled && !!chaosState.failedNodeId && !isFailed && !isImpacted;

    const data = (node.data ?? {}) as Record<string, unknown>;
    const metadata = data.metadata as Record<string, unknown> | undefined;
    const kind = data.kind as string | undefined;
    const metaBase = metadata?.base_replicas || metadata?.replicas;
    const baseReplicas = metaBase ? parseInt(String(metaBase), 10) : 3;
    const loadMultiplier = capacityState.userLoad / 100;
    const replicas =
      kind === "container" && !isNaN(baseReplicas)
        ? Math.ceil(baseReplicas * loadMultiplier)
        : undefined;

    return {
      ...node,
      data: {
        ...data,
        _theme: themeMode,
        _chaos: {
          isFailed,
          isImpacted,
          isDimmed,
        },
        _capacity: replicas
          ? {
              replicas,
              load: capacityState.userLoad,
            }
          : undefined,
      },
      style: {
        ...node.style,
        zIndex: isFailed ? 999 : isImpacted ? 998 : (node.style?.zIndex ?? 1),
        opacity: isDimmed ? 0.3 : (node.style?.opacity ?? 1),
        filter: isDimmed ? "grayscale(0.8)" : (node.style?.filter ?? "none"),
      },
    };
  });
}

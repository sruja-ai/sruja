import type { SrujaModelDump } from "@sruja/shared";
import { getManualLayoutViewKey } from "./layoutUtils";

type ViewWithLayoutMutable = {
  id: string;
  title?: string;
  rules?: unknown[];
  nodes?: string[];
  edges?: string[];
  layout?: { positions?: Record<string, { x: number; y: number }> };
};

/**
 * Persist a manual layout position for a node within a view-scoped layout block.
 *
 * We always clone `model` and `views` to avoid mutating Zustand state in-place.
 * The layout is stored under `views[manual-layout-<level>-<root|id>].layout.positions`.
 */
export function updateManualLayoutPosition({
  model,
  nodeId,
  level,
  focusNodeId,
  position,
}: {
  model: SrujaModelDump;
  nodeId: string;
  level: number;
  focusNodeId: string | undefined;
  position: { x: number; y: number };
}): SrujaModelDump {
  const updatedModel = { ...model };
  const viewKey = getManualLayoutViewKey(level, focusNodeId);

  if (!updatedModel.views) {
    updatedModel.views = {};
  } else {
    updatedModel.views = { ...updatedModel.views };
  }

  const views = updatedModel.views as Record<string, ViewWithLayoutMutable>;

  if (!views[viewKey]) {
    views[viewKey] = {
      id: viewKey,
      title: `Manual Layout ${level > 1 ? `L${level}` : "L1"}`,
      rules: [],
      nodes: [],
      edges: [],
    };
  }

  const view = views[viewKey];
  if (!view.layout) {
    view.layout = {};
  }
  if (!view.layout.positions) {
    view.layout.positions = {};
  }

  view.layout.positions[nodeId] = position;

  return updatedModel;
}

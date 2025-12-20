// apps/designer/src/components/Nodes/index.ts
// Note: These node types are no longer used since we use LikeC4 for diagramming
// Keeping exports for backward compatibility but they won't be used
import type { NodeTypes } from "@xyflow/react";
import { SystemNode } from "./SystemNode";
import { ContainerNode } from "./ContainerNode";
import { ComponentNode } from "./ComponentNode";
import { PersonNode } from "./PersonNode";
import { DataStoreNode } from "./DataStoreNode";
import { QueueNode } from "./QueueNode";
import { LaneSeparator } from "./LaneSeparator";

// Register node types (not used with LikeC4, kept for compatibility)
export const nodeTypes: NodeTypes = {
  system: SystemNode,
  container: ContainerNode,
  component: ComponentNode,
  person: PersonNode,
  datastore: DataStoreNode,
  queue: QueueNode,
  "lane-separator": LaneSeparator,
};

export {
  SystemNode,
  ContainerNode,
  ComponentNode,
  PersonNode,
  DataStoreNode,
  QueueNode,
  LaneSeparator,
};

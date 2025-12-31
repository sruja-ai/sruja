// apps/designer/src/components/Nodes/index.ts
// Note: These node types are kept for backward compatibility
import type { NodeTypes } from "@xyflow/react";
import { SystemNode } from "./SystemNode";
import { ContainerNode } from "./ContainerNode";
import { ComponentNode } from "./ComponentNode";
import { PersonNode } from "./PersonNode";
import { DataStoreNode } from "./DataStoreNode";
import { QueueNode } from "./QueueNode";
import { LaneSeparator } from "./LaneSeparator";

// Register node types (kept for compatibility)
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

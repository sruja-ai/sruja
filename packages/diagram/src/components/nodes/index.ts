// @sruja/diagram - Node Components
import type { NodeTypes } from "@xyflow/react";
import { SystemNode } from "./SystemNode";
import { SystemBoundaryNode } from "./SystemBoundaryNode";
import { ContainerBoundaryNode } from "./ContainerBoundaryNode";
import { EnterpriseBoundaryNode } from "./EnterpriseBoundaryNode";
import { ContainerNode } from "./ContainerNode";
import { ComponentNode } from "./ComponentNode";
import { PersonNode } from "./PersonNode";
import { DataStoreNode } from "./DataStoreNode";
import { QueueNode } from "./QueueNode";
import { TopicNode } from "./TopicNode";
import { CacheNode } from "./CacheNode";
import { FileSystemNode } from "./FileSystemNode";
import { DeploymentNode } from "./DeploymentNode";
import { ExternalContainerNode } from "./ExternalContainerNode";
import { ExternalComponentNode } from "./ExternalComponentNode";

// Base components
export { BaseNode, type BaseNodeProps, createNodeComponent } from "./BaseNode";

// Node components
export { SystemNode } from "./SystemNode";
export { SystemBoundaryNode } from "./SystemBoundaryNode";
export { ContainerBoundaryNode } from "./ContainerBoundaryNode";
export { EnterpriseBoundaryNode } from "./EnterpriseBoundaryNode";
export { ContainerNode } from "./ContainerNode";
export { ComponentNode } from "./ComponentNode";
export { PersonNode } from "./PersonNode";
export { DataStoreNode } from "./DataStoreNode";
export { QueueNode } from "./QueueNode";
export { TopicNode } from "./TopicNode";
export { CacheNode } from "./CacheNode";
export { FileSystemNode } from "./FileSystemNode";
export { DeploymentNode } from "./DeploymentNode";
export { ExternalContainerNode } from "./ExternalContainerNode";
export { ExternalComponentNode } from "./ExternalComponentNode";
export { GovernanceBadge } from "./GovernanceBadge";

// Pre-configured nodeTypes for React Flow
export const nodeTypes: NodeTypes = {
  system: SystemNode,
  "system-boundary": SystemBoundaryNode,
  "container-boundary": ContainerBoundaryNode,
  "enterprise-boundary": EnterpriseBoundaryNode,
  container: ContainerNode,
  component: ComponentNode,
  person: PersonNode,
  datastore: DataStoreNode,
  queue: QueueNode,
  topic: TopicNode,
  cache: CacheNode,
  filesystem: FileSystemNode,
  deployment: DeploymentNode,
  "external-container": ExternalContainerNode,
  "external-component": ExternalComponentNode,
};


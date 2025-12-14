// @sruja/diagram
// Reusable React Flow components for C4 architecture visualization

// Node Components
export {
  SystemNode,
  ContainerNode,
  ComponentNode,
  PersonNode,
  DataStoreNode,
  QueueNode,
  TopicNode,
  CacheNode,
  FileSystemNode,
  DeploymentNode,
  ExternalContainerNode,
  ExternalComponentNode,
  nodeTypes,
} from "./components/nodes";

export { edgeTypes, RoutedEdge, RelationEdge } from "./components/edges";
export { Legend } from "./components/Legend";

// Boundary Components
export { SystemBoundaryNode } from "./components/nodes/SystemBoundaryNode";
export { ContainerBoundaryNode } from "./components/nodes/ContainerBoundaryNode";
export { EnterpriseBoundaryNode } from "./components/nodes/EnterpriseBoundaryNode";

// Utilities
export { jsonToReactFlow } from "./utils/jsonToReactFlow";
export { applySrujaLayout } from "./utils/srujaLayoutEngine";
export { getNodeColors } from "./utils/colorScheme";
export { exportSVG } from "./utils/exportSVG";
export { exportPNGFromSVG } from "./utils/exportPNG";

// Types
export type {
  ArchitectureJSON,
  ArchitectureBody,
  MetadataJSON,
  NavigationJSON,
  ViewsJSON,
  ViewData,
  ViewNode,
  ViewEdge,
  SystemJSON,
  ContainerJSON,
  ComponentJSON,
  PersonJSON,
  DataStoreJSON,
  QueueJSON,
  RelationJSON,
  ScenarioJSON,
  FlowJSON,
  ScenarioStepJSON,
  C4Level,
  C4NodeType,
  C4NodeData,
} from "./types";

// Optimization (Agent Loop)
export * from "./optimization";

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

// Node Factory Utilities
export {
  createNode,
  nodeFactory,
  createPersonNode,
  createSystemNode,
  createContainerNode,
  createComponentNode,
  createDataStoreNode,
  createQueueNode,
  type NodeFactoryConfig,
  type NodeSourceData,
} from "./utils/nodeFactory";

// Type Guards
export {
  isSystemNode,
  isPersonNode,
  isContainerNode,
  isComponentNode,
  isBoundaryNode,
  isExpandable,
  isExternalNode,
  isDataNode,
  isMessagingNode,
} from "./types";

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

// Performance Utilities
export {
  measure,
  measureAsync,
  createProfiler,
  memoizeWithLimit,
  debounce,
  throttle,
  scheduleIdleWork,
  batchUpdates,
  setPerformanceCallback,
  type PerformanceMetrics,
  type ProfilerReport,
} from "./utils/performance";

// React Performance Hooks
export {
  useStableCallback,
  useRenderTracker,
  useDebouncedValue,
  useThrottledCallback,
  useIdleCalculation,
  createPropsComparator,
  usePrevious,
} from "./hooks/usePerformance";

// Optimization (Agent Loop)
export * from "./optimization";


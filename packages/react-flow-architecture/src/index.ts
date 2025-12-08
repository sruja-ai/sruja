// @sruja/react-flow-architecture
// Reusable React Flow components for C4 architecture visualization

// Node Components
export {
    SystemNode,
    ContainerNode,
    ComponentNode,
    PersonNode,
    DataStoreNode,
    QueueNode,
    nodeTypes,
} from './components/nodes';

// Utilities
export { jsonToReactFlow } from './utils/jsonToReactFlow';
export { applyElkLayout } from './utils/layoutEngine';
export { applySrujaLayout } from './utils/srujaLayoutEngine';
export { getNodeColors } from './utils/colorScheme';

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
} from './types';

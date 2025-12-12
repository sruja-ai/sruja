// apps/playground/src/components/Nodes/index.ts
import type { NodeTypes } from '@xyflow/react';
import { SystemNode } from './SystemNode';
import { ContainerNode } from './ContainerNode';
import { ComponentNode } from './ComponentNode';
import { PersonNode } from './PersonNode';
import { DataStoreNode } from './DataStoreNode';
import { QueueNode } from './QueueNode';
import { LaneSeparator } from './LaneSeparator';
// Import new node types from react-flow-architecture package
import {
    TopicNode,
    CacheNode,
    FileSystemNode,
    DeploymentNode,
    ExternalContainerNode,
    ExternalComponentNode,
    EnterpriseBoundaryNode,
} from '@sruja/diagram';

// Register all custom node types
export const nodeTypes: NodeTypes = {
    system: SystemNode,
    container: ContainerNode,
    component: ComponentNode,
    person: PersonNode,
    datastore: DataStoreNode,
    queue: QueueNode,
    topic: TopicNode,
    cache: CacheNode,
    filesystem: FileSystemNode,
    deployment: DeploymentNode,
    'external-container': ExternalContainerNode,
    'external-component': ExternalComponentNode,
    'enterprise-boundary': EnterpriseBoundaryNode,
    'lane-separator': LaneSeparator,
};

export { SystemNode, ContainerNode, ComponentNode, PersonNode, DataStoreNode, QueueNode, LaneSeparator };

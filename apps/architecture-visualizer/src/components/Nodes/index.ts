import type { NodeTypes } from '@xyflow/react';
import { SystemNode } from './SystemNode';
import { ContainerNode } from './ContainerNode';
import { ComponentNode } from './ComponentNode';
import { PersonNode } from './PersonNode';
import { DataStoreNode } from './DataStoreNode';
import { QueueNode } from './QueueNode';

// Register all custom node types
export const nodeTypes: NodeTypes = {
    system: SystemNode,
    container: ContainerNode,
    component: ComponentNode,
    person: PersonNode,
    datastore: DataStoreNode,
    queue: QueueNode,
};

export { SystemNode, ContainerNode, ComponentNode, PersonNode, DataStoreNode, QueueNode };

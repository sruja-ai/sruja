import type { NodeTypes } from '@xyflow/react';
import { SystemNode } from './SystemNode';
import { ContainerNode } from './ContainerNode';
import { ComponentNode } from './ComponentNode';
import { PersonNode } from './PersonNode';
import { DataStoreNode } from './DataStoreNode';
import { QueueNode } from './QueueNode';

// Export individual components
export { SystemNode } from './SystemNode';
export { ContainerNode } from './ContainerNode';
export { ComponentNode } from './ComponentNode';
export { PersonNode } from './PersonNode';
export { DataStoreNode } from './DataStoreNode';
export { QueueNode } from './QueueNode';

// Pre-configured nodeTypes for React Flow
export const nodeTypes: NodeTypes = {
    system: SystemNode,
    container: ContainerNode,
    component: ComponentNode,
    person: PersonNode,
    datastore: DataStoreNode,
    queue: QueueNode,
};

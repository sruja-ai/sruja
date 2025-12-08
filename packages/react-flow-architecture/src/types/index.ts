// Type definitions for @sruja/react-flow-architecture
// Re-exported from architecture-visualizer for package consumers

export type C4Level = 'L0' | 'L1' | 'L2' | 'L3';

export type C4NodeType = 'system' | 'container' | 'component' | 'person' | 'datastore' | 'queue';

export interface C4NodeData {
    [key: string]: unknown; // Index signature for React Flow compatibility
    id: string;
    label: string;
    description?: string;
    technology?: string;
    type: C4NodeType;
    isExternal?: boolean;
    childCount?: number;
    expanded?: boolean;
    parentId?: string;
}

// Architecture JSON types (simplified for package)
export interface ArchitectureJSON {
    metadata: MetadataJSON;
    architecture: ArchitectureBody;
    navigation: NavigationJSON;
    views?: ViewsJSON;
}

export interface MetadataJSON {
    name: string;
    version: string;
    generated: string;
}

export interface ArchitectureBody {
    systems?: SystemJSON[];
    persons?: PersonJSON[];
    relations?: RelationJSON[];
    scenarios?: ScenarioJSON[];
    flows?: FlowJSON[];
}

export interface NavigationJSON {
    levels?: string[];
}

export interface ViewsJSON {
    L1: ViewData;
    L2: Record<string, ViewData>;
    L3: Record<string, ViewData>;
}

export interface ViewData {
    nodes: ViewNode[];
    edges: ViewEdge[];
}

export interface ViewNode {
    id: string;
    label: string;
    type: C4NodeType;
    technology?: string;
    description?: string;
    isExternal?: boolean;
    parentId?: string;
    childCount?: number;
}

export interface ViewEdge {
    id: string;
    source: string;
    target: string;
    label?: string;
}

export interface SystemJSON {
    id: string;
    label?: string;
    description?: string;
    containers?: ContainerJSON[];
    relations?: RelationJSON[];
    datastores?: DataStoreJSON[];
    queues?: QueueJSON[];
}

export interface ContainerJSON {
    id: string;
    label?: string;
    description?: string;
    technology?: string;
    components?: ComponentJSON[];
    relations?: RelationJSON[];
    datastores?: DataStoreJSON[];
}

export interface ComponentJSON {
    id: string;
    label?: string;
    description?: string;
    technology?: string;
    relations?: RelationJSON[];
}

export interface PersonJSON {
    id: string;
    label?: string;
    description?: string;
}

export interface DataStoreJSON {
    id: string;
    label?: string;
}

export interface QueueJSON {
    id: string;
    label?: string;
}

export interface RelationJSON {
    from: string;
    to: string;
    label?: string;
    verb?: string;
}

export interface ScenarioJSON {
    id: string;
    title?: string;
    label?: string;
    steps?: ScenarioStepJSON[];
}

export interface FlowJSON {
    id: string;
    title?: string;
    label?: string;
    steps?: ScenarioStepJSON[];
}

export interface ScenarioStepJSON {
    from: string;
    to: string;
    description?: string;
}

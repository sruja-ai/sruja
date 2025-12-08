// Type definitions for Sruja Architecture Visualizer
// These types match the Go JSON export structure in pkg/export/json/json_types.go

/**
 * Complete Architecture JSON structure matching Go export
 */
export interface ArchitectureJSON {
    metadata: MetadataJSON;
    architecture: ArchitectureBody;
    navigation: NavigationJSON;
    views?: ViewsJSON; // Pre-computed views (only with --extended flag)
}

/**
 * Pre-computed views for different C4 levels
 */
export interface ViewsJSON {
    L1: ViewData;
    L2: Record<string, ViewData>; // Key: systemId
    L3: Record<string, ViewData>; // Key: systemId.containerId
}

export interface ViewData {
    nodes: ViewNode[];
    edges: ViewEdge[];
}

export interface ViewNode {
    id: string;
    label: string;
    type: 'person' | 'system' | 'container' | 'component' | 'datastore' | 'queue';
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

export interface MetadataJSON {
    name: string;
    version: string;
    generated: string;
    layout?: Record<string, LayoutData>;
    brandLogo?: string;
    layoutEngine?: string;
}

export interface MetadataEntry {
    key: string;
    value?: string;
    array?: string[];
}

export interface ArchitectureBody {
    imports?: ImportJSON[];
    systems?: SystemJSON[];
    persons?: PersonJSON[];
    relations?: RelationJSON[];
    containers?: ContainerJSON[];
    components?: ComponentJSON[];
    datastores?: DataStoreJSON[];
    queues?: QueueJSON[];
    scenarios?: ScenarioJSON[];
    flows?: FlowJSON[];
    requirements?: RequirementJSON[];
    adrs?: ADRJSON[];
    deployment?: DeploymentNodeJSON[];
    contracts?: ContractJSON[];
    sharedArtifacts?: SharedArtifactJSON[];
    libraries?: LibraryJSON[];
    policies?: PolicyJSON[];
    constraints?: ConstraintJSON[];
    conventions?: ConventionJSON[];
}

export interface NavigationJSON {
    levels?: string[];
    scenarios?: ScenarioNavJSON[];
    flows?: FlowNavJSON[];
}

export interface ImportJSON {
    path: string;
    alias?: string;
}

export interface SystemJSON {
    id: string;
    label?: string;
    description?: string;
    containers?: ContainerJSON[];
    components?: ComponentJSON[];
    datastores?: DataStoreJSON[];
    queues?: QueueJSON[];
    relations?: RelationJSON[];
    metadata?: MetadataEntry[];
}

export interface ContainerJSON {
    id: string;
    label?: string;
    description?: string;
    technology?: string;
    tags?: string[];
    version?: string;
    components?: ComponentJSON[];
    datastores?: DataStoreJSON[];
    queues?: QueueJSON[];
    relations?: RelationJSON[];
    metadata?: MetadataEntry[];
    requirements?: RequirementJSON[];
    adrs?: ADRJSON[];
}

export interface ComponentJSON {
    id: string;
    label?: string;
    description?: string;
    technology?: string;
    relations?: RelationJSON[];
    metadata?: MetadataEntry[];
    requirements?: RequirementJSON[];
    adrs?: ADRJSON[];
}

export interface DataStoreJSON {
    id: string;
    label?: string;
    metadata?: MetadataEntry[];
}

export interface QueueJSON {
    id: string;
    label?: string;
    metadata?: MetadataEntry[];
}

export interface PersonJSON {
    id: string;
    label?: string;
    description?: string;
    metadata?: MetadataEntry[];
}

export interface RelationJSON {
    from: string;
    to: string;
    verb?: string;
    label?: string;
    tags?: string[];
}

export interface ScenarioStepJSON {
    from: string;
    to: string;
    description?: string;
    tags?: string[];
    order?: number;
}

export interface ScenarioJSON {
    id: string;
    title?: string;
    label?: string;
    description?: string;
    steps?: ScenarioStepJSON[];
}

export interface FlowJSON {
    id: string;
    title?: string;
    label?: string;
    description?: string;
    steps?: ScenarioStepJSON[];
}

export interface RequirementJSON {
    id: string;
    type?: string;
    title?: string;
    description?: string;
}

export interface ADRJSON {
    id: string;
    title?: string;
    status?: string;
    context?: string;
    decision?: string;
    consequences?: string;
}

export interface DeploymentNodeJSON {
    id: string;
    label?: string;
}

export interface ContractJSON {
    id: string;
    label?: string;
}

export interface SharedArtifactJSON {
    id: string;
    label?: string;
}

export interface LibraryJSON {
    id: string;
    label?: string;
}

export interface PolicyJSON {
    id: string;
    label?: string;
}

export interface ConstraintJSON {
    key: string;
    value: string;
}

export interface ConventionJSON {
    key: string;
    value: string;
}

export interface ScenarioNavJSON {
    id: string;
    label?: string;
}

export interface FlowNavJSON {
    id: string;
    label?: string;
}

export interface LayoutData {
    x: number;
    y: number;
    width?: number;
    height?: number;
}

// View state types
export type C4Level = 'L0' | 'L1' | 'L2' | 'L3';

export interface ViewContext {
    level: C4Level;
    focusedSystemId?: string;
    focusedContainerId?: string;
}

// Node types for React Flow
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
}

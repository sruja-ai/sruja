// Type definitions for @sruja/diagram
// Re-exported from architecture-visualizer for package consumers

export type C4Level = "L0" | "L1" | "L2" | "L3";

export type C4NodeType =
  | "system"
  | "system-boundary"
  | "container-boundary"
  | "enterprise-boundary"
  | "container"
  | "component"
  | "person"
  | "datastore"
  | "queue"
  | "topic"
  | "cache"
  | "filesystem"
  | "deployment"
  | "external-container"
  | "external-component";

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
  // Governance integration
  requirementCount?: number;
  adrCount?: number;
}

// Type guards for type-safe node discrimination
// FAANG best practice: Use type guards with discriminated unions

/** Check if node is a system node */
export function isSystemNode(data: C4NodeData): data is C4NodeData & { type: "system" } {
  return data.type === "system";
}

/** Check if node is a person node */
export function isPersonNode(data: C4NodeData): data is C4NodeData & { type: "person" } {
  return data.type === "person";
}

/** Check if node is a container node */
export function isContainerNode(data: C4NodeData): data is C4NodeData & { type: "container" } {
  return data.type === "container";
}

/** Check if node is a component node */
export function isComponentNode(data: C4NodeData): data is C4NodeData & { type: "component" } {
  return data.type === "component";
}

/** Check if node is a boundary type (system-boundary, container-boundary, enterprise-boundary) */
export function isBoundaryNode(data: C4NodeData): boolean {
  return data.type === "system-boundary" ||
    data.type === "container-boundary" ||
    data.type === "enterprise-boundary";
}

/** Check if node can be expanded (has children) */
export function isExpandable(data: C4NodeData): boolean {
  return (data.type === "system" || isBoundaryNode(data)) &&
    (data.childCount ?? 0) > 0;
}

/** Check if node is an external entity */
export function isExternalNode(data: C4NodeData): boolean {
  return data.isExternal === true ||
    data.type === "external-container" ||
    data.type === "external-component";
}

/** Check if node is a data storage type */
export function isDataNode(data: C4NodeData): boolean {
  return data.type === "datastore" ||
    data.type === "cache" ||
    data.type === "filesystem";
}

/** Check if node is a messaging type */
export function isMessagingNode(data: C4NodeData): boolean {
  return data.type === "queue" || data.type === "topic";
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
  technology?: string;
  interaction?: "sync" | "async" | "event";
  tags?: string[];
  preferredRoute?: "direct" | "orthogonal" | "curved";
  bidirectional?: boolean;
}

export interface SystemJSON {
  id: string;
  label?: string;
  description?: string;
  containers?: ContainerJSON[];
  relations?: RelationJSON[];
  datastores?: DataStoreJSON[];
  queues?: QueueJSON[];
  metadata?: { key: string; value?: string }[];
}

export interface ContainerJSON {
  id: string;
  label?: string;
  description?: string;
  technology?: string;
  components?: ComponentJSON[];
  relations?: RelationJSON[];
  datastores?: DataStoreJSON[];
  metadata?: { key: string; value?: string }[];
}

export interface ComponentJSON {
  id: string;
  label?: string;
  description?: string;
  technology?: string;
  relations?: RelationJSON[];
  metadata?: { key: string; value?: string }[];
}

export interface PersonJSON {
  id: string;
  label?: string;
  description?: string;
  metadata?: { key: string; value?: string }[];
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
  technology?: string;
  interaction?: "sync" | "async" | "event";
  tags?: string[];
  preferredRoute?: "direct" | "orthogonal" | "curved";
  bidirectional?: boolean;
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

// Type definitions for Sruja Viewer
// These types match the Go JSON export structure in pkg/export/json/json_types.go

import type { Core } from 'cytoscape';

/**
 * Complete Architecture JSON structure matching Go export
 */
export interface ArchitectureJSON {
  metadata: MetadataJSON;
  architecture: ArchitectureBody;
  navigation: NavigationJSON;
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
  requirements?: RequirementJSON[];
  adrs?: ADRJSON[];
  deployment?: DeploymentNodeJSON[];
  contracts?: ContractJSON[];
  sharedArtifacts?: SharedArtifactJSON[];
  libraries?: LibraryJSON[];
  constraints?: ConstraintJSON[];
  conventions?: ConventionJSON[];
}

export interface NavigationJSON {
  levels?: string[];
  scenarios?: ScenarioNavJSON[];
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
  components?: ComponentJSON[];
  datastores?: DataStoreJSON[];
  queues?: QueueJSON[];
  relations?: RelationJSON[];
  metadata?: MetadataEntry[];
}

export interface ComponentJSON {
  id: string;
  label?: string;
  description?: string;
  relations?: RelationJSON[];
  metadata?: MetadataEntry[];
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
}

// Additional types (simplified - can be expanded as needed)
export interface ScenarioStepJSON {
  from: string;
  to: string;
  description?: string;
}

export interface ScenarioJSON {
  id: string;
  label?: string;
  title?: string;
  description?: string;
  steps?: ScenarioStepJSON[];
  [key: string]: unknown;
}

export interface RequirementJSON {
  id: string;
  title?: string;
  description?: string;
  type?: string;
  [key: string]: unknown;
}

export interface ADRJSON {
  id: string;
  title?: string;
  status?: string;
  context?: string;
  decision?: string;
  consequences?: string;
  [key: string]: unknown;
}

export interface DeploymentNodeJSON {
  id: string;
  label?: string;
  [key: string]: unknown;
}

export interface ContractJSON {
  id: string;
  label?: string;
  [key: string]: unknown;
}

export interface SharedArtifactJSON {
  id: string;
  label?: string;
  [key: string]: unknown;
}

export interface LibraryJSON {
  id: string;
  label?: string;
  [key: string]: unknown;
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

/**
 * Viewer options
 */
export interface ViewerOptions {
  container: string | HTMLElement;
  data?: ArchitectureJSON;
  layout?: string;
  style?: Record<string, unknown>;
  onSelect?: (id: string | null) => void;
}

/**
 * Viewer instance
 */
export interface ViewerInstance {
  cy: Core | null;
  init(): Promise<void>;
  load(data: ArchitectureJSON): Promise<void>;
  destroy(): void;
  exportPNG(options?: { scale?: number }): string;
  exportSVG(options?: { scale?: number }): string;
  toJSON(): ArchitectureJSON;
  getLayout(): Record<string, LayoutData>;
  setFocus(focus?: { systemId?: string; containerId?: string }): void;

  // Editing methods
  addNode(type: string, label: string, parentId?: string, data?: Record<string, unknown>): void;
  addEdge(source: string, target: string, label?: string): void;
  removeSelected(): void;
  toggleCollapse(): void;
  selectNode(id: string): void;
  setLevel(level: number): void;
  reset(): void;
}

export interface LayoutData {
  x: number;
  y: number;
  width?: number;
  height?: number;
}

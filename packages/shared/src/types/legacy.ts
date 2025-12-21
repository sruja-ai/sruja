// packages/shared/src/types/legacy.ts
// Legacy format types for backward compatibility

import type { LayoutData } from "./core";

/**
 * @deprecated Use SrujaModelDump instead
 */
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
  layout?: Record<string, LayoutData>;
  brandLogo?: string;
  layoutEngine?: string;
}

export interface MetadataEntryJSON {
  key: string;
  value?: string;
  array?: string[];
}

export interface ArchitectureBody {
  name?: string;
  description?: string;
  overview?: OverviewJSON;
  archMetadata?: MetadataEntryJSON[];
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
  properties?: Record<string, string>;
  style?: Record<string, string>;
  constraints?: ConstraintJSON[];
  conventions?: ConventionJSON[];
  contracts?: ContractJSON[];
  policies?: PolicyJSON[];
}

export interface NavigationJSON {
  levels?: string[];
  scenarios?: ScenarioNav[];
  flows?: FlowNav[];
}

export interface ViewsJSON {
  L1?: ViewData;
  L2?: Record<string, ViewData>;
  L3?: Record<string, ViewData>;
}

export interface ViewData {
  nodes: ViewNode[];
  edges: ViewEdge[];
}

export interface ViewNode {
  id: string;
  label: string;
  type: string;
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
  components?: ComponentJSON[];
  datastores?: DataStoreJSON[];
  queues?: QueueJSON[];
  relations?: RelationJSON[];
  metadata?: MetadataEntryJSON[];
  properties?: Record<string, string>;
  style?: Record<string, string>;
  slo?: SLOJSON;
  contracts?: ContractJSON[];
  constraints?: ConstraintJSON[];
  conventions?: ConventionJSON[];
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
  metadata?: MetadataEntryJSON[];
  properties?: Record<string, string>;
  style?: Record<string, string>;
  scale?: ScaleJSON;
  slo?: SLOJSON;
  contracts?: ContractJSON[];
  constraints?: ConstraintJSON[];
  conventions?: ConventionJSON[];
}

export interface ComponentJSON {
  id: string;
  label?: string;
  description?: string;
  technology?: string;
  relations?: RelationJSON[];
  metadata?: MetadataEntryJSON[];
  properties?: Record<string, string>;
  style?: Record<string, string>;
  scale?: ScaleJSON;
}

export interface DataStoreJSON {
  id: string;
  label?: string;
  description?: string;
  technology?: string;
  metadata?: MetadataEntryJSON[];
  properties?: Record<string, string>;
  style?: Record<string, string>;
}

export interface QueueJSON {
  id: string;
  label?: string;
  description?: string;
  technology?: string;
  metadata?: MetadataEntryJSON[];
  properties?: Record<string, string>;
  style?: Record<string, string>;
}

export interface PersonJSON {
  id: string;
  label?: string;
  description?: string;
  metadata?: MetadataEntryJSON[];
  properties?: Record<string, string>;
  style?: Record<string, string>;
}

export interface RelationJSON {
  from: string;
  to: string;
  verb?: string;
  label?: string;
  tags?: string[];
}

export interface ScenarioJSON {
  id: string;
  title?: string;
  label?: string;
  description?: string;
  steps?: ScenarioStepJSON[];
}

export interface ScenarioStepJSON {
  from: string;
  to: string;
  description?: string;
  tags?: string[];
  order?: number;
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
  tags?: string[];
}

export interface ADRJSON {
  id: string;
  title?: string;
  status?: string;
  context?: string;
  decision?: string;
  consequences?: string;
  tags?: string[];
}

export interface DeploymentNodeJSON {
  id: string;
  label?: string;
}

export interface ContractJSON {
  id: string;
  label?: string;
  kind?: string;
  body?: ContractBodyJSON;
}

export interface ContractBodyJSON {
  version?: string;
  status?: string;
  endpoint?: string;
  method?: string;
  request?: SchemaBlockJSON;
  response?: SchemaBlockJSON;
  errors?: string[];
  schema?: SchemaBlockJSON;
  retention?: string;
  requestMap?: string;
  responseMap?: string;
  errorMap?: string[];
  emitsSchema?: string;
  writesSchema?: string;
}

export interface SchemaBlockJSON {
  entries?: SchemaEntryJSON[];
}

export interface SchemaEntryJSON {
  key: string;
  type?: TypeSpecJSON;
}

export interface TypeSpecJSON {
  name: string;
  generics?: string[];
  optional?: boolean;
}

export interface PolicyJSON {
  id: string;
  label?: string;
  description?: string;
  category?: string;
  enforcement?: string;
  tags?: string[];
}

export interface OverviewJSON {
  summary?: string;
  audience?: string;
  scope?: string;
  goals?: string[];
  nonGoals?: string[];
  risks?: string[];
}

export interface ConstraintJSON {
  key: string;
  value: string;
}

export interface ConventionJSON {
  key: string;
  value: string;
}

export interface ScenarioNav {
  id: string;
  label?: string;
}

export interface FlowNav {
  id: string;
  label?: string;
}

export interface SLOJSON {
  availability?: SLOAvailabilityJSON;
  latency?: SLOLatencyJSON;
  errorRate?: SLOErrorRateJSON;
  throughput?: SLOThroughputJSON;
}

export interface SLOAvailabilityJSON {
  target: string;
  window: string;
  current?: string;
}

export interface SLOLatencyJSON {
  p95: string;
  p99: string;
  window: string;
  current?: SLOCurrentJSON;
}

export interface SLOCurrentJSON {
  p95: string;
  p99: string;
}

export interface SLOErrorRateJSON {
  target: string;
  window: string;
  current?: string;
}

export interface SLOThroughputJSON {
  target: string;
  window: string;
  current?: string;
}

export interface ScaleJSON {
  min?: number;
  max?: number;
  metric?: string;
}


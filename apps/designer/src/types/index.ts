// Type definitions for Sruja Designer
// Supports Sruja model format (Legacy format removed)

// ===========================================================================
// Re-export types from @sruja/shared
// ===========================================================================
export type {
  SrujaModelDump,
  ModelMetadata,
  SrujaExtensions,
  RequirementDump,
  ADRDump,
  PolicyDump,
  ScenarioDump,
  FlowDump,
  // Core types
  Element,
  Relationship,
  ParsedView,
  Specification,
} from "@sruja/shared";

// Type aliases for backward compatibility (use core types directly in new code)
// These match SrujaModelDump structure
import type { SrujaModelDump } from "@sruja/shared";

export type ElementDump = NonNullable<SrujaModelDump["elements"]>[string]; // Same as Element<any>
export type RelationDump = NonNullable<SrujaModelDump["relations"]>[number];
export type ViewDump = NonNullable<SrujaModelDump["views"]>[string]; // Same as ParsedView<any>
export type SpecificationDump = SrujaModelDump["specification"]; // Same as Specification<any>

// export {
//   isSrujaModelFormat,
// } from "@sruja/shared";

// ===========================================================================
// View & Node Types (UI Specific)
// ===========================================================================

export type C4Level = "L1" | "L2" | "L3" | "L4";

export interface ViewContext {
  level: C4Level;
  focusedSystemId?: string;
  focusedContainerId?: string;
}

export type C4NodeType =
  | "system"
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
  | "external-component"
  | "boundary";

export interface Badge {
  id: string;
  label: string;
  type: "container" | "component" | "other";
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface C4NodeData {
  [key: string]: unknown;
  id: string;
  label: string;
  description?: string;
  technology?: string;
  type: C4NodeType;
  isExternal?: boolean;
  childCount?: number;
  expanded?: boolean;
  badges?: Badge[];
  pendingActionCount?: number;
}

export type ViewTab = "overview" | "diagram" | "details" | "code" | "builder" | "governance";
export type ViewMode = "designer" | "present";

// ===========================================================================
// Re-export additional type definitions
// ===========================================================================
export * from "./nodeData";
export * from "./formHandlers";
export * from "./windowGlobals";
export * from "./edgeData";
export * from "./layout";

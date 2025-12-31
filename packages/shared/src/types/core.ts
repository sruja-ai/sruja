// packages/shared/src/types/core.ts
// Core Sruja model types

// Forward reference to avoid circular dependency
// SrujaExtensions is defined in governance.ts
type SrujaExtensions = import("./governance").SrujaExtensions;

// ===========================================================================
// Core Types
// ===========================================================================

export type MarkdownOrString = string;

export interface Element {
  id: string;
  kind: string;
  title: string;
  description?: string | null;
  technology?: string | null;
  tags?: string[] | null;
  links?: Array<{ url: string; title?: string }> | null;
  metadata?: Record<string, string> | null;
  style?: ElementStyle | null;
  parent?: string | null;
}

export interface ElementStyle {
  shape?: string;
  color?: string;
  icon?: string;
  border?: string;
  opacity?: number;
}

export interface Relationship {
  id: string;
  source: string; // FQN
  target: string; // FQN
  title?: string | null;
  description?: string | null;
  technology?: string | null;
  kind?: string | null;
  tags?: string[] | null;
  metadata?: Record<string, string> | null;
  color?: string;
  line?: string; // "solid", "dashed", "dotted"
  head?: string;
  tail?: string;
}

// Minimal View type for rendering
export interface ParsedView {
  id: string;
  title?: string | null;
  description?: string | null;
  viewOf?: string | null;
  tags?: string[] | null;
  rules?: ViewRule[] | null;
  // Computed nodes and edges (for rendering)
  nodes?: NodeDump[];
  edges?: EdgeDump[];
  // Layout positions for manual editing (Sruja extension)
  layout?: ViewLayoutDump;

  // Legacy fields or strictly required by some consumers?
  // We can add them if needed.
}

// ===========================================================================
// Core Sruja Types
// ===========================================================================

/**
 * Metadata associated with a Sruja model dump.
 */
export interface ModelMetadata {
  /** Model name identifier */
  readonly name: string;
  /** Model version string (semver recommended) */
  readonly version: string;
  /** ISO 8601 timestamp when the model was generated */
  readonly generated: string;
  /** Sruja tool version that generated this model */
  readonly srujaVersion: string;
  /** Optional layout coordinates for elements (keyed by element FQN) */
  readonly layout?: Readonly<Record<string, LayoutData>>;
}

/**
 * 2D layout coordinates and dimensions for an element.
 */
export interface LayoutData {
  /** X coordinate (horizontal position) in pixels */
  readonly x: number;
  /** Y coordinate (vertical position) in pixels */
  readonly y: number;
  /** Optional width in pixels */
  readonly width?: number;
  /** Optional height in pixels */
  readonly height?: number;
}

/**
 * Sruja-specific extensions to the model.
 */
export interface SrujaModelExtensions {
  /** Sruja-specific governance layer extensions */
  readonly sruja?: SrujaExtensions;
  /**
   * Metadata for Go backend
   */
  readonly _metadata?: ModelMetadata;
}

/**
 * View rules for inclusion/exclusion
 */
export interface ViewRule {
  readonly include?: ViewRuleExpr;
  readonly exclude?: ViewRuleExpr;
}

/**
 * View rule expression
 */
export interface ViewRuleExpr {
  readonly wildcard?: boolean;
  readonly recursive?: boolean;
  readonly elements?: string[];
}

/**
 * FqnRef (Fully Qualified Name Reference) pattern
 * Used in JSON for sources/targets to match structure if needed,
 * but our Clean JSON uses strings. However, if some code expects objects,
 * we can define it. But our Go `RelationDump` uses `FqnRefDump` (object with model property).
 * So we SHOULD define FqnRef to match JSON.
 */
export interface FqnRef {
  /** The FQN string (e.g., "system.container.component") */
  readonly model: string;
}

/**
 * Extended relations type for Go JSON serialization compatibility.
 * JSON output has `source: { model: "..." }`.
 */
export type SrujaRelationDump = ReadonlyArray<
  Omit<Relationship, "source" | "target"> & {
    /** Source element FQN as FqnRef object */
    readonly source: FqnRef;
    /** Target element FQN as FqnRef object */
    readonly target: FqnRef;
  }
>;

/**
 * Single relation item in the dump.
 */
export type RelationDump = SrujaRelationDump[number];

/**
 * Element representation in Sruja model dump.
 */
export type ElementDump = Element;

/**
 * JSON format from Go backend/WASM.
 * This is the canonical Structure of sruja.json
 */
export interface SrujaModelDump {
  /**
   * Stage of the model.
   * 'parsed' (initial export), 'computed' (layout pending), 'layouted' (final)
   */
  readonly _stage?: "parsed" | "computed" | "layouted";

  /** Project ID */
  readonly projectId?: string;

  /** Project details */
  readonly project?: {
    id: string;
    name?: string;
    title?: string;
  };

  readonly globals?: GlobalsDump;

  readonly deployments?: DeploymentsDump;

  /**
   * Specification defining element kinds, relationships, tags.
   */
  readonly specification?: SpecificationDump;

  /** Model elements, keyed by FQN */
  readonly elements: Record<string, ElementDump>;

  /**
   * Relations as array format for Go compatibility.
   */
  readonly relations?: SrujaRelationDump;

  /**
   * Views keyed by view ID.
   */
  readonly views?: Readonly<Record<string, ParsedView>>;

  /** Sruja-specific governance layer extensions */
  readonly sruja?: SrujaExtensions;

  /**
   * Metadata for Go backend
   */
  readonly _metadata?: ModelMetadata;
}

// Helper types matching Go backend

export interface GlobalsDump {
  predicates?: Record<string, unknown>;
  dynamicPredicates?: Record<string, unknown>;
  styles?: Record<string, unknown>;
}

export interface DeploymentsDump {
  elements?: Record<string, unknown>;
  relations?: Record<string, unknown>;
}

export interface SpecificationDump {
  elements: Record<string, ElementKindDump>;
  relationships?: Record<string, RelationshipKindDump>;
  tags?: Record<string, TagDump>;
  customColors?: Record<string, string>;
}

export interface ElementKindDump {
  title?: string;
  description?: string;
  technology?: string;
  style?: ElementStyle;
}

export interface RelationshipKindDump {
  title?: string;
  technology?: string;
}

export interface TagDump {
  color?: string;
}

// Layout types matching Go backend

export interface ViewLayoutDump {
  positions: Record<string, ViewPositionDump>;
}

export interface ViewPositionDump {
  x: number;
  y: number;
}

export interface NodeDump {
  id: string;
  element: string; // FQN reference
  parent?: string;
  title?: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface EdgeDump {
  id: string;
  source: string;
  target: string;
  relation?: string; // Relation ID reference
  title?: string;
}

// Helpers
export function isSrujaModelDump(data: unknown): data is SrujaModelDump {
  return typeof data === "object" && data !== null && "elements" in data && "_metadata" in data;
}

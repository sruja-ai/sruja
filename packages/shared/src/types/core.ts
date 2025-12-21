// packages/shared/src/types/core.ts
// Core Sruja model types

import type {
  Relationship,
  ParsedView,
  LikeC4ModelDump,
  MarkdownOrString,
} from "@likec4/core/types";

// Forward reference to avoid circular dependency
// SrujaExtensions is defined in governance.ts
type SrujaExtensions = import("./governance").SrujaExtensions;

// ===========================================================================
// Re-exported LikeC4 Types
// ===========================================================================

/**
 * Re-exported LikeC4 core types for direct use in Sruja codebase.
 * 
 * @remarks
 * Use these types directly instead of the "Dump" versions below.
 * These types represent the canonical LikeC4 model structure.
 * 
 * @example
 * import type { Element, Relationship } from '@sruja/shared/types';
 */
export type {
  // Core types
  Element,
  Relationship,
  ParsedView,
  Specification,
  ElementSpecification,
  TagSpecification,
  RelationshipSpecification,
  ElementStyle,
  Link,
  MarkdownOrString,
  // Model data
  ParsedLikeC4ModelData,
  ComputedLikeC4ModelData,
  LayoutedLikeC4ModelData,
  // Model dump
  LikeC4ModelDump,
} from "@likec4/core/types";

/**
 * Project metadata from LikeC4 model.
 * 
 * @public
 * @example
 * const project: ProjectDump = { id: 'my-project', name: 'My Project' };
 */
export type ProjectDump = Readonly<{
  /** Unique project identifier */
  readonly id: string;
  /** Project name (optional) */
  readonly name?: string;
  /** Project display title (optional) */
  readonly title?: string;
}>;

/**
 * Builder types from LikeC4 core library.
 * 
 * @public
 */
export type { Builder } from "@likec4/core/builder";
export type { BuilderSpecification } from "@likec4/core/builder";

// ===========================================================================
// Core Sruja Types
// ===========================================================================

/**
 * Metadata associated with a Sruja model dump.
 * 
 * @public
 * @remarks
 * Contains generation metadata, version information, and optional layout data.
 * This metadata is added by the Go backend during export and is not part of
 * the LikeC4 core model.
 * 
 * @example
 * const metadata: ModelMetadata = { name: 'ecommerce-platform', version: '1.0.0', generated: '2024-01-15T10:30:00Z', srujaVersion: '0.1.0' };
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
 * 
 * @public
 * @remarks
 * Used for storing computed layout positions from the layout engine.
 * All coordinates are in pixels.
 * 
 * @example
 * const layout: LayoutData = { x: 100, y: 200, width: 300, height: 150 };
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
 * Sruja-specific extensions to LikeC4 model.
 * 
 * @public
 * @remarks
 * These are additional fields that Sruja adds on top of LikeC4's core model.
 * Used for governance layer features (requirements, ADRs, policies, etc.)
 * and metadata tracking.
 * 
 * @example
 * const extensions: SrujaModelExtensions = { sruja: { requirements: [] }, _metadata: { name: 'my-model', version: '1.0.0', generated: '2024-01-15T10:30:00Z', srujaVersion: '0.1.0' } };
 */
export interface SrujaModelExtensions {
  /** Sruja-specific governance layer extensions */
  readonly sruja?: SrujaExtensions;
  /** 
   * Metadata for Go backend (optional, not part of LikeC4ModelDump).
   * @internal
   */
  readonly _metadata?: ModelMetadata;
}

/**
 * View rules for inclusion/exclusion (LikeC4 compatible)
 * @public
 */
export interface ViewRule {
  readonly include?: ViewRuleExpr;
  readonly exclude?: ViewRuleExpr;
}

/**
 * View rule expression (LikeC4 compatible)
 * @public
 */
export interface ViewRuleExpr {
  readonly wildcard?: boolean;
  readonly recursive?: boolean;
  readonly elements?: string[];
}

/**
 * FqnRef (Fully Qualified Name Reference) for LikeC4 compatibility.
 * 
 * @public
 * @remarks
 * LikeC4 requires element references to be objects with a 'model' property,
 * not plain strings. This type matches LikeC4's internal FqnRef format.
 * 
 * @example
 * const ref: FqnRef = { model: 'system.container.component' };
 */
export interface FqnRef {
  /** The FQN string (e.g., "system.container.component") */
  readonly model: string;
}

/**
 * Extended relations type for Go JSON serialization compatibility.
 * 
 * @public
 * @remarks
 * LikeC4 uses `Record<string, Relationship>`, but Go backend uses arrays
 * for better serialization performance. Uses `source`/`target` as FqnRef objects
 * to match LikeC4's FqnRef.flatten() expectations.
 * 
 * @example
 * const relations: SrujaRelationDump = [{ source: { model: 'system:payment' }, target: { model: 'system:order' }, title: 'processes' }];
 */
export type SrujaRelationDump = ReadonlyArray<
  Omit<Relationship<any>, 'source' | 'target'> & {
    /** Source element FQN as FqnRef object */
    readonly source: FqnRef;
    /** Target element FQN as FqnRef object */
    readonly target: FqnRef;
  }
>;

/**
 * Element representation in Sruja model dump.
 * @public
 */
export interface ElementDump {
  readonly id: string;
  readonly kind: string;
  readonly title: string;
  readonly description?: MarkdownOrString | null | undefined;
  readonly technology?: string | null | undefined;
  readonly tags?: readonly string[] | null | undefined;
  readonly links?: ReadonlyArray<{ url: string; title?: string }> | null | undefined;
  readonly metadata?: Record<string, any> | null | undefined;
  readonly style?: {
    readonly shape?: string;
    readonly color?: string;
    readonly icon?: string;
    readonly border?: string;
    readonly opacity?: number;
  } | null | undefined;
  readonly parent?: string | null | undefined;
}

/**
 * JSON format from Go backend/WASM.
 * 
 * @public
 * @remarks
 * Extends `LikeC4ModelDump` with Sruja-specific features:
 * - Relations as array (not Record) for Go compatibility
 * - Relations use `source`/`target` strings (LikeC4 standard naming)
 * - Includes Sruja-specific extensions and metadata
 * 
 * This format extends `LikeC4ModelDump` and can be passed to `LikeC4Model.fromDump()`
 * after extracting the Sruja-specific fields.
 * 
 * **Note:** Layout computation is handled by LikeC4's frontend library at runtime.
 * 
 * @example
 * const model: SrujaModelDump = { specification: {}, elements: {}, relations: [{ source: 'system:a', target: 'system:b', title: 'uses' }], project: { id: 'my-project' } };
 */
export interface SrujaModelDump
  extends Omit<LikeC4ModelDump, 'relations' | 'views' | 'project' | 'elements' | '_stage'> {
  /** 
   * Stage of the model. 
   * 'parsed' (initial export), 'computed' (layout pending), 'layouted' (final)
   */
  readonly _stage?: "parsed" | "computed" | "layouted";

  /** Model elements, keyed by FQN */
  readonly elements: Record<string, ElementDump>;

  /** 
   * Relations as array format for Go compatibility.
   * Overrides LikeC4's Record<string, Relationship> format.
   */
  readonly relations?: SrujaRelationDump;

  /** 
   * Views using LikeC4's ParsedView type (layout computed by frontend).
   * Keyed by view ID. Now includes rules for runtime computation.
   */
  readonly views?: Readonly<Record<string, ParsedView<any> & { rules?: ViewRule[] }>>;

  /** 
   * Project metadata with required fields.
   * Overrides LikeC4's optional project field.
   */
  readonly project?: ProjectDump;

  /** Sruja-specific governance layer extensions */
  readonly sruja?: SrujaExtensions;

  /** 
   * Metadata for Go backend (optional, not part of LikeC4ModelDump).
   * @internal
   */
  readonly _metadata?: ModelMetadata;
}


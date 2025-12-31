// packages/shared/src/web/wasmTypes.ts
// Type definitions for WASM integration

/**
 * Go WASM runtime constructor.
 *
 * @public
 */
export interface GoConstructor {
  new (): GoInstance;
}

/**
 * Go WASM runtime instance.
 *
 * @public
 */
export interface GoInstance {
  importObject?: Record<string, unknown>;
  run(instance: WebAssembly.Instance): void;
  _resume?: () => void;
}

export interface GoJsImports {
  "runtime.scheduleTimeoutEvent"?: (ms: number) => void;
  [key: string]: unknown;
}

export interface WasmImportObject {
  gojs?: GoJsImports;
  env?: Record<string, unknown>;
  go?: Record<string, unknown>;
  [key: string]: unknown;
}

/**
 * Response from WASM parse function with structured error handling.
 *
 * @public
 */
export interface WasmParseResponse {
  readonly ok: boolean;
  readonly json?: string;
  readonly dsl?: string;
  readonly data?: unknown; // Changed from string to known type to support objects
  readonly error?: string;
  readonly code?: string; // Error code (e.g., "PARSE_1001", "VALID_2001")
  readonly context?: Record<string, unknown>; // Additional error context
}

/**
 * DOT element structure.
 *
 * @public
 */
export interface DotElement {
  readonly id: string;
  readonly kind: "person" | "system" | "container" | "component" | "datastore" | "queue";
  readonly title: string;
  readonly technology?: string;
  readonly description?: string;
  readonly parentId?: string;
  readonly width: number;
  readonly height: number;
}

/**
 * DOT relation structure.
 *
 * @public
 */
export interface DotRelation {
  readonly from: string;
  readonly to: string;
  readonly label?: string;
}

/**
 * Result of a DOT export containing the DOT string and projected elements/relations.
 *
 * @public
 */
export interface DotResult {
  readonly dot: string;
  readonly elements: DotElement[];
  readonly relations: DotRelation[];
}

/**
 * Result of architecture score calculation.
 *
 * @public
 */
export interface ScoreResult {
  Score: number;
  Grade?: string;
  Categories?: {
    Structural?: number;
    Documentation?: number;
    Complexity?: number;
    Standardization?: number;
    Traceability?: number;
  };
  Deductions?: Array<{
    Category: string;
    Severity: string;
    Message: string;
    Rule?: string;
    Points?: number;
    Target?: string;
  }>;
}

/**
 * Extended Window interface with WASM-related properties.
 *
 * @public
 */
export interface WindowWithWasm extends Window {
  Go?: GoConstructor;
  sruja_parse_dsl?: (dsl: string, filename?: string) => WasmParseResponse;
  sruja_json_to_dsl?: (json: string) => WasmParseResponse;
  sruja_dsl_to_mermaid?: (dsl: string) => WasmParseResponse;
  sruja_dsl_to_markdown?: (dsl: string) => WasmParseResponse;
  sruja_dsl_to_model?: (dsl: string, filename?: string) => WasmParseResponse;
  sruja_dsl_to_dot?: (dsl: string, configJson: string) => WasmParseResponse;
  sruja_model_to_dsl?: (json: string) => WasmParseResponse;
  sruja_analyze_governance?: (dsl: string) => WasmParseResponse;
  sruja_score?: (dsl: string) => WasmParseResponse;
}

/**
 * Type guard to check if window has WASM properties.
 *
 * @public
 * @param win - Window object to check
 * @returns true if window has WASM properties
 */
export function isWindowWithWasm(win: unknown): win is WindowWithWasm {
  return typeof win === "object" && win !== null && "Go" in win;
}

/**
 * Safely get window object with WASM types.
 *
 * @public
 * @returns WindowWithWasm or null if not in browser
 */
export function getWindowWithWasm(): WindowWithWasm | null {
  if (typeof window === "undefined") {
    return null;
  }
  return window as WindowWithWasm;
}

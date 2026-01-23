// packages/shared/src/web/wasmTypes.ts
// Type definitions for WASM integration (Rust backend only)

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
 * Extended Window interface for browser WASM usage.
 *
 * @public
 */
export interface WindowWithWasm extends Window {}

/**
 * Type guard to check if window is available (browser environment).
 *
 * @public
 * @param win - Window object to check
 * @returns true if window exists
 */
export function isWindowWithWasm(win: unknown): win is WindowWithWasm {
  return typeof win === "object" && win !== null && typeof (win as Window).document !== "undefined";
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

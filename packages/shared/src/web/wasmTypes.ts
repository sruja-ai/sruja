// packages/shared/src/web/wasmTypes.ts
// Type definitions for WASM integration

/**
 * Go WASM runtime constructor.
 * 
 * @public
 */
export interface GoConstructor {
  new(): GoInstance;
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

/**
 * Response from WASM parse function.
 * 
 * @public
 */
export interface WasmParseResponse {
  readonly ok: boolean;
  readonly json?: string;
  readonly dsl?: string;
  readonly data?: string;
  readonly error?: string;
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
  sruja_dsl_to_likec4?: (dsl: string, filename?: string) => WasmParseResponse;
}

/**
 * Type guard to check if window has WASM properties.
 * 
 * @public
 * @param win - Window object to check
 * @returns true if window has WASM properties
 */
export function isWindowWithWasm(win: unknown): win is WindowWithWasm {
  return (
    typeof win === 'object' &&
    win !== null &&
    'Go' in win
  );
}

/**
 * Safely get window object with WASM types.
 * 
 * @public
 * @returns WindowWithWasm or null if not in browser
 */
export function getWindowWithWasm(): WindowWithWasm | null {
  if (typeof window === 'undefined') {
    return null;
  }
  return window as WindowWithWasm;
}


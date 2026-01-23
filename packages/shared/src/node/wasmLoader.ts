/* global console, process */
// packages/shared/src/node/wasmLoader.ts
// WASM module loading for Node.js (Rust backend only; Go support removed)

export interface WasmLoaderOptions {
  extensionPath?: string;
  wasmExecPath?: string;
}

const RUST_ONLY_MSG =
  "Go WASM support has been removed. Use Rust WASM in browser only. Node/VS Code support will require wasm-pack --target nodejs.";

/**
 * Load WASM module for Node.js.
 * Go support removed. Throws until Rust Node target is implemented.
 *
 * @internal
 */
export async function loadWasmModule(
  _wasmPath: string,
  _options?: WasmLoaderOptions
): Promise<void> {
  console.error("[WASM] " + RUST_ONLY_MSG);
  throw new Error(RUST_ONLY_MSG);
}

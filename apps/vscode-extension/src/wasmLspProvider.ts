// apps/vscode-extension/src/wasmLspProvider.ts
// WASM-based LSP provider for VS Code (no CLI dependency)
// 
// This is the main orchestrator that coordinates all LSP functionality.
// Implementation is split across focused modules:
// - initialization.ts: WASM module initialization and verification
// - diagnostics.ts: Diagnostics provider
// - providers.ts: All LSP providers (hover, completion, etc.)
// - debug.ts: Debug utilities
import * as vscode from "vscode";
import { initializeWasm } from "./wasmLspProvider/initialization";
import { registerDiagnosticsProvider } from "./wasmLspProvider/diagnostics";
import { registerLspProviders } from "./wasmLspProvider/providers";
import { debugWasmLsp } from "./wasmLspProvider/debug";
import { log } from "./wasmLspProvider/utils";

/**
 * Initializes the WASM LSP system.
 * 
 * @param context - VS Code extension context
 * 
 * @remarks
 * This function orchestrates:
 * 1. WASM module initialization
 * 2. Diagnostics provider registration
 * 3. All LSP providers registration
 * 
 * All providers are registered even if WASM initialization fails,
 * but they will gracefully handle the missing WASM API.
 */
export async function initializeWasmLsp(context: vscode.ExtensionContext): Promise<void> {
  try {
    // Initialize WASM module
    await initializeWasm(context);

    // Register diagnostics provider
    registerDiagnosticsProvider(context);

    // Register all LSP providers
    registerLspProviders(context);
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    const stack = error instanceof Error ? error.stack : undefined;
    log(`Failed to initialize WASM LSP: ${errMsg}`, "error");
    if (stack) {
      log(`Stack trace: ${stack}`, "error");
    }
    vscode.window.showErrorMessage(
      `Failed to initialize WASM LSP: ${errMsg}. Check output channel "Sruja WASM LSP" for details.`
    );
  }
}

// Re-export debug function for extension.ts
export { debugWasmLsp };

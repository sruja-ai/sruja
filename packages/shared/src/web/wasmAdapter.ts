// packages/shared/src/web/wasmAdapter.ts
import { logger } from "../utils/logger";
import { ConfigurationError } from "../utils/errors";
import { getWindowWithWasm, type DotResult, type ScoreResult } from "./wasmTypes";
export * from "./wasmTypes";
import { isBrowser, getBaseUrl } from "../utils/env";
import { initRustWasm } from "./wasmAdapterRust";

export type WasmApi = {
  parseDslToJson: (dsl: string, filename?: string) => Promise<string>;
  modelToDsl: (modelJson: string) => Promise<string>;
  dslToMermaid: (dsl: string) => Promise<string>;
  dslToMarkdown: (dsl: string) => Promise<string>;
  dslToModel: (dsl: string, filename?: string) => Promise<string>;
  incrementalParse: (
    dsl: string,
    changeStart: number,
    changeEnd: number,
    existingAstJson: string,
    contextLines: number
  ) => Promise<string>;
  dslToDot: (
    dsl: string,
    viewLevel?: number,
    focusNodeId?: string,
    nodeSizes?: Record<string, { width: number; height: number }>,
    viewId?: string,
    filename?: string
  ) => Promise<DotResult>;
  calculateArchitectureScore: (dsl: string) => Promise<ScoreResult>;
};

/**
 * Initialize WASM (browser). Uses Rust WASM only.
 */
export async function initWasm(options?: { base?: string }): Promise<WasmApi> {
  if (!isBrowser()) {
    throw new ConfigurationError("WASM initialization requires browser environment");
  }

  const win = getWindowWithWasm();
  if (!win) {
    throw new ConfigurationError("Window object not available", { configKey: "window" });
  }

  return initWasmAuto({ base: options?.base });
}

// Singleton WASM API instance
let wasmApi: WasmApi | null = null;
let initPromise: Promise<WasmApi> | null = null;

/**
 * Detect base URL for WASM loading.
 *
 * @internal
 * @returns Base URL with trailing slash for WASM paths
 */
function detectBaseUrl(): string {
  return getBaseUrl({ trailingSlash: true, studioPath: true });
}

/**
 * Initialize WASM with auto-detected base URL.
 * Uses singleton pattern to ensure WASM is only initialized once.
 */
export async function initWasmAuto(options?: { base?: string }): Promise<WasmApi> {
  if (wasmApi) return wasmApi;

  if (initPromise) return initPromise;

  initPromise = (async () => {
    const base = options?.base ?? detectBaseUrl();
    wasmApi = await initRustWasm({ base });
    logger.info("Initialized Rust WASM backend", { component: "wasm", action: "init", base });
    return wasmApi;
  })();

  return initPromise;
}

/**
 * Get the initialized WASM API, initializing if necessary.
 * Returns null if initialization fails.
 */
export async function getWasmApi(): Promise<WasmApi | null> {
  if (wasmApi) return wasmApi;

  try {
    return await initWasmAuto();
  } catch (error) {
    logger.error("Failed to initialize WASM", {
      component: "wasm",
      action: "get_api",
      error: error instanceof Error ? error.message : String(error),
    });
    return null;
  }
}

/**
 * Convert DSL string to Architecture JSON object using incremental parsing.
 * Returns parsed JSON object if successful, null on error.
 * @param dsl - The full DSL string
 * @param changeStart - Starting position of the change
 * @param changeEnd - Ending position of the change
 * @param existingAstJson - Existing AST as JSON string
 * @param contextLines - Number of context lines to parse around the change
 * @param filename - Optional filename for error reporting
 */
export async function incrementalParse(
  dsl: string,
  changeStart: number,
  changeEnd: number,
  existingAstJson: string,
  contextLines: number,
  filename?: string
): Promise<object | null> {
  const api = await getWasmApi();
  if (!api) {
    logger.error("WASM not available", { component: "wasm", action: "incremental_parse" });
    return null;
  }

  try {
    const jsonString = await api.incrementalParse(
      dsl,
      changeStart,
      changeEnd,
      existingAstJson,
      contextLines
    );
    return JSON.parse(jsonString);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    logger.error("Incremental parse error", {
      component: "wasm",
      action: "incremental_parse",
      error: errorMessage,
      filename: filename || "unknown",
    });
    // Re-throw with more context so the caller can show a better error message
    throw new Error(`Incremental parse failed${filename ? ` (${filename})` : ""}: ${errorMessage}`);
  }
}

/**
 * Convert DSL string to Architecture JSON object.
 * Returns parsed JSON object if successful, null on error.
 * @param dsl - The DSL string to parse
 * @param filename - Optional filename for error reporting (defaults to location.pathname or 'input.sruja')
 */
export async function convertDslToModel(dsl: string, filename?: string): Promise<object | null> {
  const api = await getWasmApi();
  if (!api) {
    logger.error("WASM not available", { component: "wasm", action: "convert_dsl_to_model" });
    return null;
  }

  try {
    const jsonString = await api.dslToModel(dsl, filename);
    return JSON.parse(jsonString);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    logger.error("DSL parse error", {
      component: "wasm",
      action: "convert_dsl_to_model",
      error: errorMessage,
      filename: filename || "unknown",
    });
    // Re-throw with more context so the caller can show a better error message
    throw new Error(`Failed to parse DSL${filename ? ` (${filename})` : ""}: ${errorMessage}`);
  }
}

/**
 * Convert DSL string to Markdown string.
 * Returns markdown string if successful, null on error.
 * Uses Go/WASM markdown exporter.
 */
export async function convertDslToMarkdown(dsl: string): Promise<string | null> {
  const api = await getWasmApi();
  if (!api) {
    logger.error("WASM not available", { component: "wasm", action: "convert_dsl_to_markdown" });
    return null;
  }

  try {
    return await api.dslToMarkdown(dsl);
  } catch (error) {
    logger.error("DSL to Markdown conversion error", {
      component: "wasm",
      action: "convert_dsl_to_markdown",
      error: error instanceof Error ? error.message : String(error),
    });
    return null;
  }
}

/**
 * Convert DSL string to Mermaid diagram string.
 * Returns mermaid diagram string if successful, null on error.
 * Uses Go/WASM mermaid exporter.
 */
export async function convertDslToMermaid(dsl: string): Promise<string | null> {
  const api = await getWasmApi();
  if (!api) {
    logger.error("WASM not available", { component: "wasm", action: "convert_dsl_to_mermaid" });
    return null;
  }

  try {
    return await api.dslToMermaid(dsl);
  } catch (error) {
    logger.error("DSL to Mermaid conversion error", {
      component: "wasm",
      action: "convert_dsl_to_mermaid",
      error: error instanceof Error ? error.message : String(error),
    });
    return null;
  }
}

/**
 * Convert DSL string to Graphviz DOT string.
 * Returns DOT string if successful, null on error.
 * Uses Go/WASM DOT exporter for layout.
 * @param dsl - The DSL source string
 * @param viewLevel - C4 view level (1=Context, 2=Container, 3=Component). Default 1.
 * @param focusNodeId - Node ID to focus on for L2/L3 views (optional)
 * @param nodeSizes - Optional node size overrides
 * @param viewId - Optional DSL view definition ID to load (e.g., "architect_overview")
 */
export async function convertDslToDot(
  dsl: string,
  viewLevel?: number,
  focusNodeId?: string,
  nodeSizes?: Record<string, { width: number; height: number }>,
  viewId?: string,
  filename?: string
): Promise<DotResult | null> {
  const api = await getWasmApi();
  if (!api) {
    logger.error("WASM not available", { component: "wasm", action: "convert_dsl_to_dot" });
    return null;
  }

  try {
    return await api.dslToDot(dsl, viewLevel, focusNodeId, nodeSizes, viewId, filename);
  } catch (error) {
    logger.error("DSL to DOT conversion error", {
      component: "wasm",
      action: "convert_dsl_to_dot",
      error: error instanceof Error ? error.message : String(error),
    });
    return null;
  }
}

/**
 * Convert Architecture JSON object to DSL string.
 * Returns DSL string if successful, throws error otherwise.
 * Uses Go/WASM converter.
 */
export async function convertModelToDsl(model: object): Promise<string> {
  const api = await getWasmApi();
  if (!api) {
    logger.error("WASM not available", { component: "wasm", action: "convert_model_to_dsl" });
    throw new Error("WASM not available");
  }

  try {
    const jsonStr = JSON.stringify(model);
    return await api.modelToDsl(jsonStr);
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    logger.error("Model to DSL conversion error", {
      component: "wasm",
      action: "convert_model_to_dsl",
      error: errorMsg,
    });
    throw error;
  }
}

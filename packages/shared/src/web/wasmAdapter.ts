// packages/shared/src/web/wasmAdapter.ts
import { logger } from "../utils/logger";
import { ConfigurationError, NetworkError } from "../utils/errors";
import {
  getWindowWithWasm,
  type GoInstance,
  type DotResult,
  type WasmImportObject,
  type GoJsImports,
  type GoConstructor,
  type ScoreResult,
} from "./wasmTypes";
import { isBrowser } from "../utils/env";
import { ExportError, parseWasmError, validateDotResult } from "./errors";

/**
 * Check if running in development mode.
 * Safely checks Vite's import.meta.env and falls back to process.env.
 */
function isDevelopmentMode(): boolean {
  try {
    const meta =
      typeof import.meta !== "undefined"
        ? (import.meta as { env?: { DEV?: boolean; MODE?: string } })
        : undefined;
    if (meta?.env?.DEV || meta?.env?.MODE === "development") {
      return true;
    }
  } catch {
    // Ignore errors accessing import.meta
  }
  return typeof process !== "undefined" && process.env?.NODE_ENV === "development";
}

export type WasmApi = {
  parseDslToJson: (dsl: string, filename?: string) => Promise<string>;
  modelToDsl: (modelJson: string) => Promise<string>;
  dslToMermaid: (dsl: string) => Promise<string>;
  dslToMarkdown: (dsl: string) => Promise<string>;
  dslToModel: (dsl: string, filename?: string) => Promise<string>;
  dslToDot: (
    dsl: string,
    viewLevel?: number,
    focusNodeId?: string,
    nodeSizes?: Record<string, { width: number; height: number }>
  ) => Promise<DotResult>;
  calculateArchitectureScore: (dsl: string) => Promise<ScoreResult>;
};

async function ensureScript(src: string): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const script = document.createElement("script");
    script.src = src;
    script.async = true;
    script.onload = () => resolve();
    script.onerror = () => {
      const error = new Error("Failed to load " + src);
      logger.error("Failed to load wasm script", {
        component: "wasm",
        action: "load_script",
        scriptUrl: src,
        error: error.message,
      });
      reject(error);
    };
    document.head.appendChild(script);
  });
}

async function ensureGoRuntimeLoaded(execUrl: string, base: string): Promise<void> {
  const win = getWindowWithWasm();
  if (!win) {
    throw new ConfigurationError("Window object not available", { configKey: "window" });
  }

  if (win.Go) return;

  const candidates = [
    execUrl,
    base.replace(/\/?$/, "/") + "wasm_exec.js",
    "/wasm_exec.js",
    "https://cdn.jsdelivr.net/gh/golang/go@go1.25.0/misc/wasm/wasm_exec.js",
  ];

  let loaded = false;
  let lastError: Error | null = null;

  for (const url of candidates) {
    try {
      await ensureScript(url);
      if (win.Go) {
        loaded = true;
        break;
      }
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      // Continue to next candidate
    }
  }

  if (!loaded) {
    const error = new NetworkError("Failed to load wasm_exec.js from all candidates", {
      url: execUrl,
      cause: lastError || undefined,
      context: { candidates },
    });
    logger.error("Failed to load Go wasm runtime", {
      component: "wasm",
      action: "load_go_runtime",
      candidates,
      error: error.message,
    });
    throw error;
  }
}

export async function initWasm(options?: {
  base?: string;
  skipGoLoad?: boolean;
}): Promise<WasmApi> {
  if (!isBrowser()) {
    throw new ConfigurationError("WASM initialization requires browser environment");
  }

  const win = getWindowWithWasm();
  if (!win) {
    throw new ConfigurationError("Window object not available", { configKey: "window" });
  }

  const base = options?.base || "/";
  const wasmExecUrl = base.replace(/\/?$/, "/") + "wasm/wasm_exec.js";
  const wasmBaseUrl = base.replace(/\/?$/, "/") + "wasm/sruja.wasm";
  // Add cache-busting query parameter to ensure fresh WASM is loaded
  // In dev mode, use timestamp; in production, this helps with cache invalidation
  const isDev = isDevelopmentMode();
  const cacheBuster = isDev ? Date.now() : `v${Date.now()}`;
  const wasmUrl = wasmBaseUrl + "?v=" + cacheBuster;

  if (!options?.skipGoLoad) {
    await ensureGoRuntimeLoaded(wasmExecUrl, base);
  } else {
    if (!win.Go) {
      try {
        await ensureGoRuntimeLoaded(wasmExecUrl, base);
      } catch {
        // Ignore errors if skipGoLoad is true
      }
    }
  }

  const GoCtor: GoConstructor | undefined = win.Go;
  if (!GoCtor) {
    const error = new ConfigurationError("wasm_exec.js not loaded - Go constructor not available", {
      configKey: "Go",
    });
    logger.error("Go constructor missing", {
      component: "wasm",
      action: "init",
      errorType: "go_constructor_missing",
      error: error.message,
    });
    throw error;
  }

  const go: GoInstance = new GoCtor();
  const importObject = (go.importObject || {}) as WasmImportObject;

  // Provide gojs alias when missing
  if (!importObject.gojs) {
    const envObj = importObject.env || {};
    importObject.gojs = Object.keys(envObj).length
      ? (envObj as unknown as GoJsImports)
      : (importObject.go as unknown as GoJsImports) || {};
  }
  // Polyfill TinyGo scheduleTimeoutEvent if missing
  if (importObject.gojs && !importObject.gojs["runtime.scheduleTimeoutEvent"]) {
    importObject.gojs["runtime.scheduleTimeoutEvent"] = (ms: number) => {
      setTimeout(() => {
        try {
          if (go._resume) {
            go._resume();
          }
        } catch {
          // Ignore errors
        }
      }, ms);
    };
  }

  let instance: WebAssembly.Instance | null = null;
  const candidates = [
    wasmUrl, // Primary URL with cache-busting
    wasmBaseUrl, // Fallback without cache-busting
    base.replace(/\/?$/, "/") + "sruja.wasm",
    "/sruja.wasm",
    base.replace(/\/?$/, "/") + "studio/wasm/sruja.wasm",
    base.replace(/\/?$/, "/") + "viewer/wasm/sruja.wasm",
    "/wasm/sruja.wasm",
  ];

  let lastError: Error | null = null;
  for (const url of candidates) {
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new NetworkError(`Failed to fetch ${url}`, {
          url,
          status: response.status,
        });
      }

      // Try streaming instantiation first if supported
      if (WebAssembly.instantiateStreaming) {
        try {
          // Create a new response for streaming because the original body might be consumed
          // if we failed and are retrying, or if we need to clone.
          // However, here we just use the response directly.
          // Note: instantiationStreaming requires the response to have application/wasm content type
          // which might not always be the case with some dev servers, so we catch errors.
          const mod = await WebAssembly.instantiateStreaming(
            response,
            importObject as WebAssembly.Imports
          );
          instance = mod.instance;
          lastError = null;
          logger.info("WASM loaded via streaming", { component: "wasm", action: "load_wasm", url });
          // In dev mode, log WASM load info
          if (isDevelopmentMode()) {
            console.info(
              `[WASM] ✅ Loaded from ${url}\n` +
                `[WASM] Cache-busting enabled - new WASM will load automatically after rebuild.\n` +
                `[WASM] If changes don't appear, try a hard refresh (Ctrl+Shift+R / Cmd+Shift+R)`
            );
          }
          break;
        } catch (streamingError) {
          logger.warn("WASM streaming failed, falling back to arrayBuffer", {
            component: "wasm",
            action: "load_wasm",
            error:
              streamingError instanceof Error ? streamingError.message : String(streamingError),
          });
          // Fallback proceeds below...
          // We need to fetch again because the response body is already consumed by instantiateStreaming
          // or we can't clone it easily if it's already used.
          // Simplest reliability is to re-fetch or just let the loop continue if we want to try next candidate,
          // BUT here we want to try arrayBuffer on SAME url.
          // So let's re-fetch for the fallback.
          const fallbackResponse = await fetch(url);
          if (!fallbackResponse.ok) throw new Error("Failed to re-fetch for fallback");
          const bytes = await fallbackResponse.arrayBuffer();
          const mod = await WebAssembly.instantiate(bytes, importObject as WebAssembly.Imports);
          instance = mod.instance;
          lastError = null;
          logger.info("WASM loaded via arrayBuffer fallback", {
            component: "wasm",
            action: "load_wasm",
            url,
          });
          // In dev mode, log WASM load info
          if (isDevelopmentMode()) {
            console.info(
              `[WASM] ✅ Loaded from ${url} (fallback method)\n` +
                `[WASM] Cache-busting enabled - new WASM will load automatically after rebuild.\n` +
                `[WASM] If changes don't appear, try a hard refresh (Ctrl+Shift+R / Cmd+Shift+R)`
            );
          }
          break;
        }
      } else {
        const bytes = await response.arrayBuffer();
        const mod = await WebAssembly.instantiate(bytes, importObject as WebAssembly.Imports);
        instance = mod.instance;
        lastError = null;
        logger.info("WASM loaded via arrayBuffer", { component: "wasm", action: "load_wasm", url });
        // In dev mode, log WASM load info
        if (isDevelopmentMode()) {
          console.info(
            `[WASM] ✅ Loaded from ${url}\n` +
              `[WASM] Cache-busting enabled - new WASM will load automatically after rebuild.\n` +
              `[WASM] If changes don't appear, try a hard refresh (Ctrl+Shift+R / Cmd+Shift+R)`
          );
        }
        break;
      }
    } catch (e) {
      lastError = e instanceof Error ? e : new Error(String(e));
      continue;
    }
  }

  if (!instance) {
    const error =
      lastError ||
      new NetworkError("Failed to load sruja.wasm from all candidates", {
        context: { candidates },
      });
    logger.error("Failed to load sruja.wasm", {
      component: "wasm",
      action: "load_wasm",
      errorType: "wasm_load_failure",
      candidates,
      retries: candidates.length,
      error: error.message,
    });
    throw error;
  }

  // Start the Go runtime - this is asynchronous
  go.run(instance);

  // Give Go runtime a moment to start
  await new Promise((r) => setTimeout(r, 100));

  // Wait for required functions to be registered
  let retries = 0;
  const maxRetries = 150; // Increased to give more time
  // Check for dslToModel as a signal that WASM is loaded
  while (!win.sruja_dsl_to_model && retries < maxRetries) {
    await new Promise((r) => setTimeout(r, 50));
    retries++;
  }

  if (retries >= maxRetries) {
    const available = Object.keys(win).filter((k) => k.startsWith("sruja_"));
    logger.error("Required WASM functions not found after waiting", {
      component: "wasm",
      action: "wait_for_required",
      retries,
      available,
      missing: {
        model: !win.sruja_dsl_to_model,
        mermaid: !win.sruja_dsl_to_mermaid,
        markdown: !win.sruja_dsl_to_markdown,
      },
    });
  }

  // const parseFn = win.sruja_parse_dsl; // Deprecated/Removed
  const mermaidFn = win.sruja_dsl_to_mermaid;
  const markdownFn = win.sruja_dsl_to_markdown;
  const modelFn = win.sruja_dsl_to_model;
  const modelToDslFn = win.sruja_model_to_dsl;
  const dotFn = win.sruja_dsl_to_dot;
  const scoreFn = win.sruja_analyze_governance;

  // Debug: Log all registered functions
  const allSrujaFunctions = Object.keys(win).filter((k) => k.startsWith("sruja_"));
  logger.info("WASM functions loaded", {
    component: "wasm",
    action: "init_complete",
    available: allSrujaFunctions,
  });

  // Require core functions
  if (!mermaidFn || !markdownFn || !modelFn || !modelToDslFn) {
    const missing: string[] = [];
    if (!mermaidFn) missing.push("sruja_dsl_to_mermaid");
    if (!markdownFn) missing.push("sruja_dsl_to_markdown");
    if (!modelFn) missing.push("sruja_dsl_to_model");
    if (!modelToDslFn) missing.push("sruja_model_to_dsl");
    const available = Object.keys(win).filter((k) => k.startsWith("sruja_"));
    const error = new ConfigurationError(
      `WASM functions not found. Missing: ${missing.join(", ")}. Available: ${available.join(", ") || "none"}`,
      {
        configKey: "wasm_functions",
        context: { missing, available, retries },
      }
    );
    logger.error("WASM functions missing", {
      component: "wasm",
      action: "init",
      errorType: "wasm_functions_missing",
      missing,
      available,
      retries,
      windowKeys: Object.keys(win).filter((k) => k.includes("sruja") || k.includes("wasm")),
      error: error.message,
    });
    throw error;
  }

  return {
    parseDslToJson: async (dsl: string, filename?: string) => {
      // Delegate to dslToModel since parseDsl now calls dslToModel in Go
      // This maintains backward compatibility while removing duplication
      try {
        const fn =
          filename ||
          (typeof location !== "undefined" ? location.pathname || "input.sruja" : "input.sruja");
        if (!modelFn) {
          throw new ExportError("SYSTEM_5002", "sruja_dsl_to_model function not registered");
        }
        const r = modelFn(dsl, fn);
        if (!r || !r.ok) {
          const exportError = parseWasmError(r);
          if (exportError) {
            logger.error("WASM parse failed", {
              component: "wasm",
              action: "parse_dsl",
              errorType: "parse_failure",
              errorCode: exportError.code,
              error: exportError.message,
              context: exportError.context,
              dslLength: dsl.length,
            });
            throw exportError;
          }
          throw new ExportError("PARSE_1001", r?.error || "parse failed");
        }
        // Return json field for backward compatibility, but data field also works
        return (r.json as string) || (r.data as string) || "";
      } catch (error) {
        logger.error("WASM parse exception", {
          component: "wasm",
          action: "parse_dsl",
          errorType: "parse_exception",
          error: error instanceof Error ? error.message : String(error),
          errorCode: error instanceof ExportError ? error.code : undefined,
        });
        throw error;
      }
    },
    modelToDsl: async (modelJson: string) => {
      try {
        if (!modelToDslFn) {
          throw new ExportError("SYSTEM_5002", "sruja_model_to_dsl function not registered");
        }
        const r = modelToDslFn(modelJson);
        if (!r || !r.ok) {
          const exportError = parseWasmError(r);
          if (exportError) {
            throw exportError;
          }
          throw new ExportError("EXPORT_4001", r?.error || "model to DSL conversion failed");
        }
        return (r.data as string) || "";
      } catch (error) {
        logger.error("WASM model to DSL exception", {
          component: "wasm",
          action: "model_to_dsl",
          error: error instanceof Error ? error.message : String(error),
          errorCode: error instanceof ExportError ? error.code : undefined,
        });
        throw error;
      }
    },
    dslToMermaid: async (dsl: string, viewLevel?: number, targetId?: string) => {
      try {
        // Pack arguments into config object for WASM
        const config = {
          viewLevel: viewLevel ?? 1,
          targetId: targetId ?? "",
        };
        const r = mermaidFn(dsl, JSON.stringify(config));
        if (!r || !r.ok) {
          const exportError = parseWasmError(r);
          if (exportError) {
            logger.error("WASM mermaid export failed", {
              component: "wasm",
              action: "export_mermaid",
              error: exportError.message,
              errorCode: exportError.code,
              context: exportError.context,
              dslPreview: dsl.substring(0, 200),
            });
            throw exportError;
          }
          throw new ExportError("EXPORT_4001", r?.error || "mermaid export failed");
        }
        const output = (r.data as string) || "";
        // Validate that we got a non-empty result
        if (!output || output.trim().length === 0) {
          const errorMsg = "mermaid exporter returned empty output";
          logger.error("WASM mermaid export returned empty", {
            component: "wasm",
            action: "export_mermaid",
            error: errorMsg,
            dslPreview: dsl.substring(0, 200),
          });
          throw new ExportError("EXPORT_4002", errorMsg);
        }
        return output;
      } catch (error) {
        logger.error("WASM mermaid export exception", {
          component: "wasm",
          action: "export_mermaid",
          error: error instanceof Error ? error.message : String(error),
          errorCode: error instanceof ExportError ? error.code : undefined,
        });
        throw error;
      }
    },
    dslToMarkdown: async (dsl: string) => {
      try {
        const r = markdownFn(dsl);
        if (!r || !r.ok) {
          throw new Error(r?.error || "markdown export failed");
        }
        return (r.data as string) || "";
      } catch (error) {
        logger.error("WASM markdown export exception", {
          component: "wasm",
          action: "export_markdown",
          error: error instanceof Error ? error.message : String(error),
        });
        throw error;
      }
    },
    dslToModel: async (dsl: string, filename?: string) => {
      try {
        const fn =
          filename ||
          (typeof location !== "undefined" ? location.pathname || "input.sruja" : "input.sruja");
        if (!modelFn) {
          throw new ExportError("SYSTEM_5002", "sruja_dsl_to_model function not registered");
        }
        const r = modelFn(dsl, fn);
        if (!r || !r.ok) {
          const exportError = parseWasmError(r);
          if (exportError) {
            throw exportError;
          }
          throw new ExportError("EXPORT_4001", r?.error || "model export failed");
        }
        return (r.data as string) || "";
      } catch (error) {
        logger.error("WASM model export exception", {
          component: "wasm",
          action: "export_model",
          error: error instanceof Error ? error.message : String(error),
          errorCode: error instanceof ExportError ? error.code : undefined,
        });
        throw error;
      }
    },
    dslToDot: async (
      dsl: string,
      viewLevel?: number,
      focusNodeId?: string,
      nodeSizes?: Record<string, { width: number; height: number }>
    ) => {
      try {
        if (!dotFn) {
          throw new ExportError("SYSTEM_5002", "sruja_dsl_to_dot function not registered");
        }
        // Pack arguments into config object for WASM
        const config = {
          viewLevel: viewLevel ?? 1,
          focusNodeId: focusNodeId ?? "",
          nodeSizes: nodeSizes || {},
        };
        const r = dotFn(dsl, JSON.stringify(config));
        if (!r || !r.ok) {
          const exportError = parseWasmError(r);
          if (exportError) {
            throw exportError;
          }
          throw new ExportError("EXPORT_4001", r?.error || "DOT export failed");
        }
        // Validate and type-check the result
        return validateDotResult(r.data);
      } catch (error) {
        logger.error("WASM DOT export exception", {
          component: "wasm",
          action: "export_dot",
          error: error instanceof Error ? error.message : String(error),
          errorCode: error instanceof ExportError ? error.code : undefined,
        });
        throw error;
      }
    },
    calculateArchitectureScore: async (dsl: string) => {
      try {
        if (!scoreFn) {
          throw new Error("sruja_analyze_governance function not registered");
        }
        const r = scoreFn(dsl);
        if (!r || !r.ok) {
          throw new Error(r?.error || "score calculation failed");
        }
        return JSON.parse((r.data as string) || "{}");
      } catch (error) {
        logger.error("WASM score calculation exception", {
          component: "wasm",
          action: "calculate_score",
          error: error instanceof Error ? error.message : String(error),
        });
        throw error;
      }
    },
  };
}

// Singleton WASM API instance
let wasmApi: WasmApi | null = null;
let initPromise: Promise<WasmApi> | null = null;

/**
 * Auto-detects the base URL for WASM files based on the current environment.
 * Handles Vite, Astro, and other frameworks automatically.
 */
import { getBaseUrl } from "../utils/env";

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
export async function initWasmAuto(options?: {
  base?: string;
  skipGoLoad?: boolean;
}): Promise<WasmApi> {
  if (wasmApi) return wasmApi;

  if (initPromise) return initPromise;

  initPromise = (async () => {
    const base = options?.base ?? detectBaseUrl();
    wasmApi = await initWasm({ ...options, base });
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
 * Convert DSL string to Architecture JSON object.
 * Returns parsed JSON object if successful, null on error.
 * @param dsl - The DSL string to parse
 * @param filename - Optional filename for error reporting (defaults to location.pathname or 'input.sruja')
 */
export async function convertDslToJson(dsl: string, filename?: string): Promise<object | null> {
  const api = await getWasmApi();
  if (!api) {
    logger.error("WASM not available", { component: "wasm", action: "convert_dsl_to_json" });
    return null;
  }

  try {
    const jsonString = await api.parseDslToJson(dsl, filename);
    return JSON.parse(jsonString);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    logger.error("DSL parse error", {
      component: "wasm",
      action: "convert_dsl_to_json",
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
 */
export async function convertDslToDot(
  dsl: string,
  viewLevel?: number,
  focusNodeId?: string,
  nodeSizes?: Record<string, { width: number; height: number }>
): Promise<DotResult | null> {
  const api = await getWasmApi();
  if (!api) {
    logger.error("WASM not available", { component: "wasm", action: "convert_dsl_to_dot" });
    return null;
  }

  try {
    return await api.dslToDot(dsl, viewLevel, focusNodeId, nodeSizes);
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

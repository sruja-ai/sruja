/* global global, process */
// packages/shared/src/node/wasmAdapter.ts
// Node.js-compatible WASM adapter for VS Code extension
// Uses Node.js WebAssembly API instead of browser APIs

import * as fs from "fs";
import * as path from "path";
import { logger } from "../utils/logger";
import { loadWasmModule } from "./wasmLoader";
import type {
  Diagnostic,
  HoverInfo,
  CompletionItem,
  Location,
  Symbol,
  CodeAction,
  DocumentLink,
  FoldingRange,
} from "./lspTypes";

// Re-export LSP types for convenience
export type {
  Diagnostic,
  HoverInfo,
  CompletionItem,
  Location,
  Symbol,
  CodeAction,
  DocumentLink,
  FoldingRange,
} from "./lspTypes";

/**
 * WASM functions attached to the global object by the WASM loader.
 * These functions are dynamically attached by Go's WebAssembly runtime.
 */
interface WasmGlobalFunctions {
  sruja_parse_dsl?: (dsl: string, filename?: string) => string;
  sruja_json_to_dsl?: (json: string) => string;
  sruja_get_diagnostics?: (text: string) => string;
  sruja_get_symbols?: (text: string) => string;
  sruja_hover?: (text: string, line: number, column: number) => string;
  sruja_completion?: (text: string, line: number, column: number) => string;
  sruja_go_to_definition?: (text: string, line: number, column: number) => string;
  sruja_find_references?: (text: string, line: number, column: number) => string;
  sruja_rename?: (text: string, line: number, column: number, newName: string) => string;
  sruja_format?: (text: string) => string;
  sruja_code_actions?: (text: string, diagnostics: string) => string;
  sruja_semantic_tokens?: (text: string) => string;
  sruja_document_links?: (text: string) => string;
  sruja_folding_ranges?: (text: string) => string;
  sruja_dsl_to_mermaid?: (dsl: string) => string;
  sruja_dsl_to_markdown?: (dsl: string) => string;
  sruja_dsl_to_model?: (dsl: string, filename?: string) => string;
}

/**
 * Node.js WASM API interface.
 *
 * @public
 */
export type NodeWasmApi = {
  parseDslToJson: (dsl: string, filename?: string) => Promise<string>;
  printJsonToDsl: (json: string) => Promise<string>;
  dslToMermaid: (dsl: string) => Promise<string>;
  dslToMarkdown: (dsl: string) => Promise<string>;
  dslToModel: (dsl: string, filename?: string) => Promise<string>;
  // LSP functions
  getDiagnostics: (text: string) => Promise<Diagnostic[]>;
  getSymbols: (text: string) => Promise<Symbol[]>;
  hover: (text: string, line: number, column: number) => Promise<HoverInfo | null>;
  completion: (text: string, line: number, column: number) => Promise<CompletionItem[]>;
  goToDefinition: (text: string, line: number, column: number) => Promise<Location | null>;
  findReferences: (text: string, line: number, column: number) => Promise<Location[]>;
  rename: (text: string, line: number, column: number, newName: string) => Promise<string>;
  format: (text: string) => Promise<string>;
  codeActions: (text: string, diagnostics: Diagnostic[]) => Promise<CodeAction[]>;
  semanticTokens: (text: string) => Promise<number[]>;
  documentLinks: (text: string) => Promise<DocumentLink[]>;
  foldingRanges: (text: string) => Promise<FoldingRange[]>;
};

interface ParseResult {
  ok: boolean;
  json?: string;
  error?: string;
}

interface JsonToDslResult {
  ok: boolean;
  dsl?: string;
  error?: string;
}

/**
 * Initialize WASM for Node.js.
 *
 * @public
 * @param options - Initialization options
 * @returns Promise resolving to NodeWasmApi
 */
export async function initWasmNode(options?: {
  wasmPath?: string;
  wasmExecPath?: string;
  extensionPath?: string;
}): Promise<NodeWasmApi> {
  // Find WASM file - prioritize extension path if provided
  const wasmCandidates: string[] = [];

  // First, try extension path (most reliable for VS Code extension)
  // Prefer compressed version for smaller disk footprint
  if (options?.extensionPath) {
    wasmCandidates.push(path.join(options.extensionPath, "wasm/sruja.wasm.gz"));
    wasmCandidates.push(path.join(options.extensionPath, "wasm/sruja.wasm"));
  }

  // Then try explicit wasmPath
  if (options?.wasmPath) {
    wasmCandidates.push(options.wasmPath);
  }

  // Try VS Code extension path if available
  // Note: vscode module is only available in VS Code extension contexts
  try {
    const vscode = await import("vscode");
    const extPath = vscode?.extensions?.getExtension(
      "sruja-ai.sruja-language-support"
    )?.extensionPath;
    if (extPath) {
      wasmCandidates.push(path.join(extPath, "wasm/sruja.wasm.gz"));
      wasmCandidates.push(path.join(extPath, "wasm/sruja.wasm"));
    }
  } catch {
    // vscode module not available (not in VS Code extension context)
    void 0;
  }

  // Then try common locations (prefer compressed)
  wasmCandidates.push(
    path.join(__dirname, "../../../../wasm/sruja.wasm.gz"),
    path.join(__dirname, "../../../../wasm/sruja.wasm"),
    path.join(__dirname, "../../../wasm/sruja.wasm.gz"),
    path.join(__dirname, "../../../wasm/sruja.wasm"),
    path.join(process.cwd(), "wasm/sruja.wasm.gz"),
    path.join(process.cwd(), "wasm/sruja.wasm"),
    path.join(process.cwd(), "node_modules/@sruja/shared/wasm/sruja.wasm.gz"),
    path.join(process.cwd(), "node_modules/@sruja/shared/wasm/sruja.wasm")
  );

  let wasmPath: string | undefined;
  for (const candidate of wasmCandidates) {
    if (fs.existsSync(candidate)) {
      wasmPath = candidate;
      break;
    }
  }

  if (!wasmPath) {
    throw new Error(
      "sruja.wasm or sruja.wasm.gz not found. Please ensure WASM files are available."
    );
  }

  await loadWasmModule(wasmPath, {
    extensionPath: options?.extensionPath,
    wasmExecPath: options?.wasmExecPath,
  });

  // Check if functions are available on global
  const wasmGlobals = global as unknown as WasmGlobalFunctions;

  const parseFn = wasmGlobals.sruja_parse_dsl;
  const jsonToDslFn = wasmGlobals.sruja_json_to_dsl;
  const diagnosticsFn = wasmGlobals.sruja_get_diagnostics;
  const symbolsFn = wasmGlobals.sruja_get_symbols;
  const hoverFn = wasmGlobals.sruja_hover;
  const completionFn = wasmGlobals.sruja_completion;
  const goToDefinitionFn = wasmGlobals.sruja_go_to_definition;
  const findReferencesFn = wasmGlobals.sruja_find_references;
  const renameFn = wasmGlobals.sruja_rename;
  const formatFn = wasmGlobals.sruja_format;
  const codeActionsFn = wasmGlobals.sruja_code_actions;
  const semanticTokensFn = wasmGlobals.sruja_semantic_tokens;
  const documentLinksFn = wasmGlobals.sruja_document_links;
  const foldingRangesFn = wasmGlobals.sruja_folding_ranges;
  const mermaidFn = wasmGlobals.sruja_dsl_to_mermaid;
  const markdownFn = wasmGlobals.sruja_dsl_to_markdown;
  const modelFn = wasmGlobals.sruja_dsl_to_model;

  if (!parseFn || !jsonToDslFn) {
    const missing = [];
    if (!parseFn) missing.push("sruja_parse_dsl");
    if (!jsonToDslFn) missing.push("sruja_json_to_dsl");
    throw new Error(`WASM functions not found. Missing: ${missing.join(", ")}`);
  }

  // Log available LSP functions for debugging
  const availableLspFunctions = [];
  if (diagnosticsFn) availableLspFunctions.push("diagnostics");
  if (symbolsFn) availableLspFunctions.push("symbols");
  if (hoverFn) availableLspFunctions.push("hover");
  if (completionFn) availableLspFunctions.push("completion");
  if (goToDefinitionFn) availableLspFunctions.push("goToDefinition");
  if (findReferencesFn) availableLspFunctions.push("findReferences");
  if (renameFn) availableLspFunctions.push("rename");
  if (formatFn) availableLspFunctions.push("format");
  if (codeActionsFn) availableLspFunctions.push("codeActions");
  if (semanticTokensFn) availableLspFunctions.push("semanticTokens");
  if (documentLinksFn) availableLspFunctions.push("documentLinks");
  if (foldingRangesFn) availableLspFunctions.push("foldingRanges");

  if (availableLspFunctions.length > 0) {
    logger.debug("WASM LSP functions available", {
      component: "wasmAdapter",
      action: "initWasmNode",
      functions: availableLspFunctions,
    });
  } else {
    logger.warn("No WASM LSP functions found - LSP features may not work", {
      component: "wasmAdapter",
      action: "initWasmNode",
    });
  }

  // Helper to safely call WASM function and parse JSON response
  const callWasmFunction = <T>(
    fn: ((...args: unknown[]) => { ok: boolean; data?: string; error?: string }) | undefined,
    fnName: string,
    ...args: unknown[]
  ): Promise<T> => {
    return new Promise((resolve) => {
      if (!fn) {
        logger.warn("WASM function not available", {
          component: "wasmAdapter",
          action: "callWasmFunction",
          functionName: fnName,
        });
        resolve([] as T);
        return;
      }
      try {
        const r = fn(...args);
        if (!r || !r.ok) {
          if (r?.error) {
            logger.warn("WASM function returned error", {
              component: "wasmAdapter",
              action: "callWasmFunction",
              functionName: fnName,
              error: r.error,
            });
          }
          resolve([] as T);
          return;
        }
        if (!r.data) {
          resolve([] as unknown as T);
          return;
        }
        const data = JSON.parse(r.data);
        resolve(data as T);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        logger.error("WASM function failed", {
          component: "wasmAdapter",
          action: "callWasmFunction",
          functionName: fnName,
          error:
            error instanceof Error
              ? {
                  message: error.message,
                  name: error.name,
                  stack: error.stack,
                }
              : errorMessage,
        });
        resolve([] as unknown as T);
      }
    });
  };

  return {
    parseDslToJson: async (dsl: string, filename: string = "input.sruja"): Promise<string> => {
      if (!parseFn) {
        throw new Error("parse function not available");
      }
      try {
        const r = parseFn(dsl, filename) as unknown as ParseResult;
        if (!r || !r.ok) {
          throw new Error(r?.error || "parse failed");
        }
        if (!r.json) {
          throw new Error("parse succeeded but no JSON returned");
        }
        return r.json;
      } catch (error) {
        throw new Error(
          `WASM parse failed: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    },
    printJsonToDsl: async (json: string): Promise<string> => {
      if (!jsonToDslFn) {
        throw new Error("jsonToDsl function not available");
      }
      try {
        const r = jsonToDslFn(json) as unknown as JsonToDslResult;
        if (!r || !r.ok) {
          throw new Error(r?.error || "print failed");
        }
        if (!r.dsl) {
          throw new Error("print succeeded but no DSL returned");
        }
        return r.dsl;
      } catch (error) {
        throw new Error(
          `WASM print failed: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    },
    dslToMermaid: async (dsl: string, _viewLevel?: number, _targetId?: string): Promise<string> => {
      try {
        if (!mermaidFn) {
          throw new Error("mermaid function not available");
        }
        const r = mermaidFn(dsl) as unknown as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok) {
          throw new Error(r?.error || "mermaid export failed");
        }
        if (!r.data) {
          throw new Error("mermaid export succeeded but no data returned");
        }
        return r.data;
      } catch (error) {
        throw new Error(
          `WASM mermaid export failed: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    },
    dslToMarkdown: async (dsl: string): Promise<string> => {
      try {
        if (!markdownFn) {
          throw new Error("markdown function not available");
        }
        const r = markdownFn(dsl) as unknown as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok) {
          throw new Error(r?.error || "markdown export failed");
        }
        if (!r.data) {
          throw new Error("markdown export succeeded but no data returned");
        }
        return r.data;
      } catch (error) {
        throw new Error(
          `WASM markdown export failed: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    },
    dslToModel: async (dsl: string, filename: string = "input.sruja"): Promise<string> => {
      try {
        if (!modelFn) {
          throw new Error("sruja_dsl_to_model function not available");
        }
        const r = modelFn(dsl, filename) as unknown as {
          ok: boolean;
          data?: string;
          error?: string;
        };
        if (!r || !r.ok) {
          throw new Error(r?.error || "model export failed");
        }
        if (!r.data) {
          throw new Error("model export succeeded but no data returned");
        }
        return r.data;
      } catch (error) {
        throw new Error(
          `WASM model export failed: ${error instanceof Error ? error.message : String(error)}`
        );
      }
    },
    // LSP functions
    getDiagnostics: (text: string) =>
      callWasmFunction<Diagnostic[]>(
        diagnosticsFn as unknown as
          | ((...args: unknown[]) => { ok: boolean; data?: string; error?: string })
          | undefined,
        "getDiagnostics",
        text
      ),
    getSymbols: (text: string) =>
      callWasmFunction<Symbol[]>(
        symbolsFn as unknown as
          | ((...args: unknown[]) => { ok: boolean; data?: string; error?: string })
          | undefined,
        "getSymbols",
        text
      ),
    hover: async (text: string, line: number, column: number): Promise<HoverInfo | null> => {
      if (!hoverFn) {
        logger.warn("WASM hover function not available", {
          component: "wasmAdapter",
          action: "hover",
        });
        return null;
      }
      try {
        const r = hoverFn(text, line, column) as unknown as {
          ok: boolean;
          data?: string;
          error?: string;
        };
        if (!r || !r.ok || !r.data || r.data === "null") {
          return null;
        }
        return JSON.parse(r.data);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        logger.error("WASM hover failed", {
          component: "wasmAdapter",
          action: "hover",
          error:
            error instanceof Error
              ? {
                  message: error.message,
                  name: error.name,
                  stack: error.stack,
                }
              : errorMessage,
        });
        return null;
      }
    },
    completion: (text: string, line: number, column: number) =>
      callWasmFunction<CompletionItem[]>(
        completionFn as unknown as
          | ((...args: unknown[]) => { ok: boolean; data?: string; error?: string })
          | undefined,
        "completion",
        text,
        line,
        column
      ),
    goToDefinition: async (
      text: string,
      line: number,
      column: number
    ): Promise<Location | null> => {
      if (!goToDefinitionFn) {
        logger.warn("WASM goToDefinition function not available", {
          component: "wasmAdapter",
          action: "goToDefinition",
        });
        return null;
      }
      try {
        const r = goToDefinitionFn(text, line, column) as unknown as {
          ok: boolean;
          data?: string;
          error?: string;
        };
        if (!r || !r.ok || !r.data || r.data === "null") {
          return null;
        }
        return JSON.parse(r.data);
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        logger.error("WASM goToDefinition failed", {
          component: "wasmAdapter",
          action: "goToDefinition",
          error:
            error instanceof Error
              ? {
                  message: error.message,
                  name: error.name,
                  stack: error.stack,
                }
              : errorMessage,
        });
        return null;
      }
    },
    findReferences: async (text: string, line: number, column: number): Promise<Location[]> => {
      if (!findReferencesFn) {
        return [];
      }
      try {
        const r = findReferencesFn(text, line, column) as unknown as {
          ok: boolean;
          data?: string;
          error?: string;
        };
        if (!r || !r.ok || !r.data || r.data === "[]" || r.data === "null") {
          return [];
        }
        const parsed = JSON.parse(r.data);
        return Array.isArray(parsed) ? parsed : [];
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        logger.error("WASM findReferences failed", {
          component: "wasmAdapter",
          action: "findReferences",
          error:
            error instanceof Error
              ? {
                  message: error.message,
                  name: error.name,
                  stack: error.stack,
                }
              : errorMessage,
        });
        return [];
      }
    },
    rename: async (
      text: string,
      line: number,
      column: number,
      newName: string
    ): Promise<string> => {
      if (!renameFn) {
        return text;
      }
      try {
        const r = renameFn(text, line, column, newName) as unknown as {
          ok: boolean;
          data?: string;
          error?: string;
        };
        if (!r || !r.ok || !r.data) {
          return text;
        }
        return r.data;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        logger.error("WASM rename failed", {
          component: "wasmAdapter",
          action: "rename",
          error:
            error instanceof Error
              ? {
                  message: error.message,
                  name: error.name,
                  stack: error.stack,
                }
              : errorMessage,
        });
        return text;
      }
    },
    format: async (text: string): Promise<string> => {
      if (!formatFn) {
        return text;
      }
      try {
        if (!formatFn) {
          return text;
        }
        const r = formatFn(text) as unknown as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok || !r.data) {
          return text;
        }
        return r.data;
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        logger.error("WASM format failed", {
          component: "wasmAdapter",
          action: "format",
          error:
            error instanceof Error
              ? {
                  message: error.message,
                  name: error.name,
                  stack: error.stack,
                }
              : errorMessage,
        });
        return text;
      }
    },
    codeActions: async (text: string, diagnostics: Diagnostic[]): Promise<CodeAction[]> => {
      if (!codeActionsFn) {
        return [];
      }
      try {
        const diagsJSON = JSON.stringify(diagnostics);
        if (!codeActionsFn) {
          return [];
        }
        const r = codeActionsFn(text, diagsJSON) as unknown as {
          ok: boolean;
          data?: string;
          error?: string;
        };
        if (!r || !r.ok || !r.data) {
          return [];
        }
        const data = JSON.parse(r.data);
        return Array.isArray(data) ? data : [];
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        logger.error("WASM codeActions failed", {
          component: "wasmAdapter",
          action: "codeActions",
          error:
            error instanceof Error
              ? {
                  message: error.message,
                  name: error.name,
                  stack: error.stack,
                }
              : errorMessage,
        });
        return [];
      }
    },
    semanticTokens: (text: string) =>
      callWasmFunction<number[]>(
        semanticTokensFn as unknown as
          | ((...args: unknown[]) => { ok: boolean; data?: string; error?: string })
          | undefined,
        "semanticTokens",
        text
      ),
    documentLinks: (text: string) =>
      callWasmFunction<DocumentLink[]>(
        documentLinksFn as unknown as
          | ((...args: unknown[]) => { ok: boolean; data?: string; error?: string })
          | undefined,
        "documentLinks",
        text
      ),
    foldingRanges: (text: string) =>
      callWasmFunction<FoldingRange[]>(
        foldingRangesFn as unknown as
          | ((...args: unknown[]) => { ok: boolean; data?: string; error?: string })
          | undefined,
        "foldingRanges",
        text
      ),
  };
}

/**
 * Convert DSL string to Markdown string.
 *
 * @public
 * @param dsl - DSL string to convert
 * @param wasmApi - Optional WASM API instance
 * @param _filename - Optional filename (for compatibility)
 * @returns Markdown string or null on error
 */
export async function convertDslToMarkdown(
  dsl: string,
  wasmApi?: NodeWasmApi,
  _filename?: string
): Promise<string | null> {
  let api = wasmApi;
  if (!api) {
    try {
      api = await initWasmNode();
    } catch {
      return null;
    }
  }

  try {
    return await api.dslToMarkdown(dsl);
  } catch {
    return null;
  }
}

/**
 * Convert DSL string to Mermaid diagram string.
 *
 * @public
 * @param dsl - DSL string to convert
 * @param wasmApi - Optional WASM API instance
 * @returns Mermaid diagram string or null on error
 */
export async function convertDslToMermaid(
  dsl: string,
  wasmApi?: NodeWasmApi
): Promise<string | null> {
  let api = wasmApi;
  if (!api) {
    try {
      api = await initWasmNode();
    } catch {
      return null;
    }
  }

  try {
    return await api.dslToMermaid(dsl);
  } catch {
    return null;
  }
}

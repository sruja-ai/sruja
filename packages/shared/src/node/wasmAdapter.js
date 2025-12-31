"use strict";
// packages/shared/src/node/wasmAdapter.ts
// Node.js-compatible WASM adapter for VS Code extension
// Uses Node.js WebAssembly API instead of browser APIs
var __createBinding =
  (this && this.__createBinding) ||
  (Object.create
    ? function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        var desc = Object.getOwnPropertyDescriptor(m, k);
        if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
          desc = {
            enumerable: true,
            get: function () {
              return m[k];
            },
          };
        }
        Object.defineProperty(o, k2, desc);
      }
    : function (o, m, k, k2) {
        if (k2 === undefined) k2 = k;
        o[k2] = m[k];
      });
var __setModuleDefault =
  (this && this.__setModuleDefault) ||
  (Object.create
    ? function (o, v) {
        Object.defineProperty(o, "default", { enumerable: true, value: v });
      }
    : function (o, v) {
        o["default"] = v;
      });
var __importStar =
  (this && this.__importStar) ||
  (function () {
    var ownKeys = function (o) {
      ownKeys =
        Object.getOwnPropertyNames ||
        function (o) {
          var ar = [];
          for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
          return ar;
        };
      return ownKeys(o);
    };
    return function (mod) {
      if (mod && mod.__esModule) return mod;
      var result = {};
      if (mod != null)
        for (var k = ownKeys(mod), i = 0; i < k.length; i++)
          if (k[i] !== "default") __createBinding(result, mod, k[i]);
      __setModuleDefault(result, mod);
      return result;
    };
  })();
Object.defineProperty(exports, "__esModule", { value: true });
exports.initWasmNode = initWasmNode;
exports.convertDslToMarkdown = convertDslToMarkdown;
exports.convertDslToMermaid = convertDslToMermaid;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const wasmLoader_1 = require("./wasmLoader");
/**
 * Initialize WASM for Node.js.
 *
 * @public
 * @param options - Initialization options
 * @returns Promise resolving to NodeWasmApi
 */
async function initWasmNode(options) {
  // Find WASM file - prioritize extension path if provided
  const wasmCandidates = [];
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
  try {
    const vscode = await Promise.resolve().then(() => __importStar(require("vscode")));
    const extPath = vscode?.extensions?.getExtension(
      "sruja-ai.sruja-language-support"
    )?.extensionPath;
    if (extPath) {
      wasmCandidates.push(path.join(extPath, "wasm/sruja.wasm.gz"));
      wasmCandidates.push(path.join(extPath, "wasm/sruja.wasm"));
    }
  } catch {
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
  let wasmPath;
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
  console.debug(`[WASM] Loading WASM module from: ${wasmPath}`);
  await (0, wasmLoader_1.loadWasmModule)(wasmPath, {
    extensionPath: options?.extensionPath,
    wasmExecPath: options?.wasmExecPath,
  });
  console.debug("[WASM] Checking for exported functions...");
  // Check if functions are available on global
  const parseFn = global.sruja_parse_dsl;
  const jsonToDslFn = global.sruja_json_to_dsl;
  // LSP functions
  const diagnosticsFn = global.sruja_get_diagnostics;
  const symbolsFn = global.sruja_get_symbols;
  const hoverFn = global.sruja_hover;
  const completionFn = global.sruja_completion;
  const goToDefinitionFn = global.sruja_go_to_definition;
  const findReferencesFn = global.sruja_find_references;
  const renameFn = global.sruja_rename;
  const formatFn = global.sruja_format;
  const codeActionsFn = global.sruja_code_actions;
  const semanticTokensFn = global.sruja_semantic_tokens;
  const documentLinksFn = global.sruja_document_links;
  const foldingRangesFn = global.sruja_folding_ranges;
  const mermaidFn = global.sruja_dsl_to_mermaid;
  const markdownFn = global.sruja_dsl_to_markdown;
  const modelFn = global.sruja_dsl_to_model;
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
    console.debug(`WASM LSP functions available: ${availableLspFunctions.join(", ")}`);
  } else {
    console.warn("No WASM LSP functions found - LSP features may not work");
  }
  // Helper to safely call WASM function and parse JSON response
  const callWasmFunction = (fn, fnName, ...args) => {
    return new Promise((resolve) => {
      if (!fn) {
        console.warn(`WASM ${fnName} function not available`);
        resolve([]);
        return;
      }
      try {
        const r = fn(...args);
        if (!r || !r.ok) {
          if (r?.error) {
            console.warn(`WASM ${fnName} returned error:`, r.error);
          }
          resolve([]);
          return;
        }
        if (!r.data) {
          resolve([]);
          return;
        }
        const data = JSON.parse(r.data);
        resolve(data);
      } catch (error) {
        console.error(`WASM ${fnName} failed:`, error);
        resolve([]);
      }
    });
  };
  return {
    parseDslToJson: async (dsl, filename = "input.sruja") => {
      try {
        const r = parseFn(dsl, filename);
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
    printJsonToDsl: async (json) => {
      try {
        const r = jsonToDslFn(json);
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
    dslToMermaid: async (dsl) => {
      try {
        const r = mermaidFn(dsl);
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
    dslToMarkdown: async (dsl) => {
      try {
        const r = markdownFn(dsl);
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
    dslToModel: async (dsl, filename = "input.sruja") => {
      try {
        if (!modelFn) {
          throw new Error("sruja_dsl_to_model function not available");
        }
        const r = modelFn(dsl, filename);
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
    getDiagnostics: (text) => callWasmFunction(diagnosticsFn, "getDiagnostics", text),
    getSymbols: (text) => callWasmFunction(symbolsFn, "getSymbols", text),
    hover: async (text, line, column) => {
      if (!hoverFn) {
        console.warn("WASM hover function not available");
        return null;
      }
      try {
        const r = hoverFn(text, line, column);
        if (!r || !r.ok || !r.data || r.data === "null") {
          return null;
        }
        return JSON.parse(r.data);
      } catch (error) {
        console.error("WASM hover failed:", error);
        return null;
      }
    },
    completion: (text, line, column) =>
      callWasmFunction(completionFn, "completion", text, line, column),
    goToDefinition: async (text, line, column) => {
      if (!goToDefinitionFn) {
        console.warn("WASM goToDefinition function not available");
        return null;
      }
      try {
        const r = goToDefinitionFn(text, line, column);
        if (!r || !r.ok || !r.data || r.data === "null") {
          return null;
        }
        return JSON.parse(r.data);
      } catch (error) {
        console.error("WASM goToDefinition failed:", error);
        return null;
      }
    },
    findReferences: async (text, line, column) => {
      if (!findReferencesFn) {
        return [];
      }
      try {
        const r = findReferencesFn(text, line, column);
        if (!r || !r.ok || !r.data || r.data === "[]" || r.data === "null") {
          return [];
        }
        const parsed = JSON.parse(r.data);
        return Array.isArray(parsed) ? parsed : [];
      } catch (error) {
        console.error("WASM findReferences failed:", error);
        return [];
      }
    },
    rename: async (text, line, column, newName) => {
      if (!renameFn) {
        return text;
      }
      try {
        const r = renameFn(text, line, column, newName);
        if (!r || !r.ok || !r.data) {
          return text;
        }
        return r.data;
      } catch (error) {
        console.error("WASM rename failed:", error);
        return text;
      }
    },
    format: async (text) => {
      if (!formatFn) {
        return text;
      }
      try {
        const r = formatFn(text);
        if (!r || !r.ok || !r.data) {
          return text;
        }
        return r.data;
      } catch (error) {
        console.error("WASM format failed:", error);
        return text;
      }
    },
    codeActions: async (text, diagnostics) => {
      if (!codeActionsFn) {
        return [];
      }
      try {
        const diagsJSON = JSON.stringify(diagnostics);
        const r = codeActionsFn(text, diagsJSON);
        if (!r || !r.ok || !r.data) {
          return [];
        }
        const data = JSON.parse(r.data);
        return Array.isArray(data) ? data : [];
      } catch (error) {
        console.error("WASM codeActions failed:", error);
        return [];
      }
    },
    semanticTokens: (text) => callWasmFunction(semanticTokensFn, "semanticTokens", text),
    documentLinks: (text) => callWasmFunction(documentLinksFn, "documentLinks", text),
    foldingRanges: (text) => callWasmFunction(foldingRangesFn, "foldingRanges", text),
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
async function convertDslToMarkdown(dsl, wasmApi, _filename) {
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
async function convertDslToMermaid(dsl, wasmApi) {
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
//# sourceMappingURL=wasmAdapter.js.map

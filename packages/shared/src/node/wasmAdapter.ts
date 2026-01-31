/* global process */
// packages/shared/src/node/wasmAdapter.ts
// Node.js WASM adapter (Rust only; Go support removed)

import * as fs from "fs";
import * as path from "path";
import { ConfigurationError } from "../utils/errors";
import { logger } from "../utils/logger";
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
 * Rust WASM module interface.
 * 
 * @internal
 * @remarks
 * Matches the structure of wasm-pack generated bindings for Node.js target.
 */
interface RustWasmModule {
  readonly default: (moduleOrPath?: string | { module_or_path?: string }) => Promise<void>;
  readonly init_panic_hook?: () => void;
  readonly sruja_dsl_to_model: (dsl: string, filename?: string) => string;
  readonly sruja_dsl_to_mermaid: (dsl: string, configJson?: string) => string;
  readonly sruja_dsl_to_dot: (
    dsl: string,
    viewLevel?: number,
    targetId?: string,
    nodeSizesJson?: string,
    viewId?: string,
    filename?: string
  ) => string;
  readonly sruja_dsl_to_markdown: (dsl: string) => string;
  readonly sruja_model_to_dsl: (modelJson: string) => string;
  readonly sruja_get_diagnostics: (dsl: string, filename?: string) => string;
  readonly sruja_calculate_architecture_score: (dsl: string) => string;
}

/**
 * Node.js WASM API interface.
 *
 * @public
 * @remarks
 * Provides DSL parsing, conversion, and LSP functionality for Node.js environments.
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

/**
 * Internal state for WASM module.
 * 
 * @internal
 */
interface WasmState {
  readonly module: RustWasmModule;
  readonly extensionPath?: string;
}

/**
 * Singleton WASM module instance.
 * 
 * @internal
 */
let wasmState: WasmState | null = null;

/**
 * Load Rust WASM module for Node.js.
 * 
 * @internal
 * @param extensionPath - Path to VS Code extension directory
 * @returns Promise resolving to RustWasmModule
 * @throws ConfigurationError if WASM files are missing or loading fails
 * 
 * @remarks
 * Loads WASM module from:
 * - {extensionPath}/wasm/rust/sruja_wasm.js
 * - {extensionPath}/wasm/rust/sruja_wasm_bg.wasm
 * 
 * Uses Node.js require() to load the JS bindings, which then loads the WASM file.
 */
async function loadRustWasmModule(extensionPath?: string): Promise<RustWasmModule> {
  if (!extensionPath) {
    throw new ConfigurationError("extensionPath is required for Node.js WASM loading", {
      configKey: "extensionPath",
    });
  }

  const wasmJsPath = path.join(extensionPath, "wasm", "rust", "sruja_wasm.js");
  const wasmBgPath = path.join(extensionPath, "wasm", "rust", "sruja_wasm_bg.wasm");

  // Verify files exist
  if (!fs.existsSync(wasmJsPath)) {
    throw new ConfigurationError("Rust WASM JS file not found", {
      configKey: "wasmJsPath",
      context: { path: wasmJsPath, extensionPath },
    });
  }

  if (!fs.existsSync(wasmBgPath)) {
    throw new ConfigurationError("Rust WASM binary file not found", {
      configKey: "wasmBgPath",
      context: { path: wasmBgPath, extensionPath },
    });
  }

  try {
    // Use require() for Node.js (wasm-pack --target nodejs generates CommonJS)
    // eslint-disable-next-line @typescript-eslint/no-require-imports, @typescript-eslint/no-var-requires
    const wasmModule = require(wasmJsPath) as RustWasmModule;

    // Initialize WASM module with path to .wasm file
    await wasmModule.default({ module_or_path: wasmBgPath });

    // Initialize panic hook for better error messages
    if (typeof wasmModule.init_panic_hook === "function") {
      wasmModule.init_panic_hook();
    }

    logger.info("Rust WASM module loaded successfully", {
      component: "wasm",
      action: "load",
      extensionPath,
    });

    return wasmModule;
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    logger.error("Failed to load Rust WASM module", {
      component: "wasm",
      action: "load",
      error: errorMessage,
      extensionPath,
    });
    throw new ConfigurationError("Failed to load Rust WASM module", {
      configKey: "wasmModule",
      context: {
        error: errorMessage,
        wasmJsPath,
        wasmBgPath,
        originalError: error instanceof Error ? error.message : String(error),
      },
    });
  }
}

/**
 * Parse diagnostics JSON from Rust WASM into Diagnostic[].
 * 
 * @internal
 * @param diagnosticsJson - JSON string from sruja_get_diagnostics
 * @param filename - Optional filename for diagnostics
 * @returns Array of Diagnostic objects
 */
function parseDiagnostics(diagnosticsJson: string, filename = "input.sruja"): Diagnostic[] {
  try {
    const parsed = JSON.parse(diagnosticsJson) as Array<{
      range: {
        start: { line: number; character: number };
        end: { line: number; character: number };
      };
      severity: number; // 1 = Error, 2 = Warning, 3 = Info
      code: string;
      message: string;
    }>;

    return parsed.map((d) => ({
      code: d.code,
      severity: d.severity === 1 ? "Error" : d.severity === 2 ? "Warning" : "Info",
      message: d.message,
      location: {
        file: filename,
        line: d.range.start.line,
        column: d.range.start.character,
      },
    }));
  } catch (error) {
    logger.error("Failed to parse diagnostics JSON", {
      component: "wasm",
      action: "parse_diagnostics",
      error: error instanceof Error ? error.message : String(error),
    });
    return [];
  }
}

/**
 * Extract word at position in text.
 * 
 * @internal
 * @param text - Full text
 * @param line - 1-based line number
 * @param column - 1-based column number
 * @returns Word at position or empty string
 */
function getWordAtPosition(text: string, line: number, column: number): string {
  const lines = text.split("\n");
  const lineIndex = line - 1;

  if (lineIndex < 0 || lineIndex >= lines.length) {
    return "";
  }

  const lineText = lines[lineIndex];
  const colIndex = column - 1;

  if (colIndex < 0 || colIndex >= lineText.length) {
    return "";
  }

  // Find word boundaries
  let start = colIndex;
  while (start > 0 && /[\w\-.]/.test(lineText[start - 1] ?? "")) {
    start--;
  }

  let end = colIndex;
  while (end < lineText.length && /[\w\-.]/.test(lineText[end] ?? "")) {
    end++;
  }

  return lineText.slice(start, end).trim();
}

/**
 * Find element definition in parsed model.
 * 
 * @internal
 * @param modelJson - JSON model string
 * @param elementId - Element ID to find
 * @returns Location of definition or null
 */
function findElementDefinition(modelJson: string, elementId: string, text: string): Location | null {
  try {
    const model = JSON.parse(modelJson) as {
      elements?: Array<{
        id: string;
        kind?: string;
        title?: string;
      }>;
    };

    const elements = model.elements ?? [];
    const element = elements.find((e) => e.id === elementId || e.id.endsWith(`.${elementId}`));

    if (!element) {
      return null;
    }

    // Find line in source text
    const lines = text.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (line.includes(element.id) || line.includes(elementId)) {
        const col = line.indexOf(element.id) !== -1 ? line.indexOf(element.id) : line.indexOf(elementId);
        return {
          file: "input.sruja",
          line: i + 1,
          column: col + 1,
        };
      }
    }

    return null;
  } catch {
    return null;
  }
}

/**
 * Find all references to an element in text.
 * 
 * @internal
 * @param text - Source text
 * @param elementId - Element ID to find
 * @returns Array of locations
 */
function findElementReferences(text: string, elementId: string): Location[] {
  const locations: Location[] = [];
  const lines = text.split("\n");

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    let searchIndex = 0;

    while (true) {
      const index = line.indexOf(elementId, searchIndex);
      if (index === -1) {
        break;
      }

      // Check if it's a word boundary match
      const before = line[index - 1];
      const after = line[index + elementId.length];
      if (
        (before === undefined || !/[\w\-.]/.test(before)) &&
        (after === undefined || !/[\w\-.]/.test(after))
      ) {
        locations.push({
          file: "input.sruja",
          line: i + 1,
          column: index + 1,
        });
      }

      searchIndex = index + 1;
    }
  }

  return locations;
}

/**
 * Create NodeWasmApi from Rust WASM module.
 * 
 * @internal
 * @param module - Rust WASM module
 * @param extensionPath - Extension path for context
 * @returns NodeWasmApi implementation
 */
function createWasmApi(module: RustWasmModule, extensionPath?: string): NodeWasmApi {
  /**
   * Wrap Rust WASM call with error handling.
   * 
   * @internal
   */
  const wrapRustCall = <T>(fn: () => T, operation: string): T => {
    try {
      return fn();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      logger.error(`Rust WASM ${operation} failed`, {
        component: "wasm",
        action: operation,
        error: errorMessage,
        extensionPath,
      });
      throw new Error(`WASM ${operation} error: ${errorMessage}`);
    }
  };

  return {
    async parseDslToJson(dsl: string, filename?: string): Promise<string> {
      return wrapRustCall(() => module.sruja_dsl_to_model(dsl, filename), "parseDslToJson");
    },

    async printJsonToDsl(json: string): Promise<string> {
      return wrapRustCall(() => module.sruja_model_to_dsl(json), "printJsonToDsl");
    },

    async dslToMermaid(dsl: string): Promise<string> {
      return wrapRustCall(() => module.sruja_dsl_to_mermaid(dsl), "dslToMermaid");
    },

    async dslToMarkdown(dsl: string): Promise<string> {
      return wrapRustCall(() => module.sruja_dsl_to_markdown(dsl), "dslToMarkdown");
    },

    async dslToModel(dsl: string, filename?: string): Promise<string> {
      return wrapRustCall(() => module.sruja_dsl_to_model(dsl, filename), "dslToModel");
    },

    async getDiagnostics(text: string): Promise<Diagnostic[]> {
      const diagnosticsJson = wrapRustCall(
        () => module.sruja_get_diagnostics(text, "input.sruja"),
        "getDiagnostics"
      );
      return parseDiagnostics(diagnosticsJson, "input.sruja");
    },

    async getSymbols(text: string): Promise<Symbol[]> {
      try {
        // Parse model to extract symbols
        // Note: This parses the DSL each time. For better performance with large files,
        // consider caching the parsed model if text hasn't changed.
        const modelJson = await this.dslToModel(text);
        const model = JSON.parse(modelJson) as {
          elements?: Array<{
            id: string;
            kind?: string;
            title?: string;
          }>;
        };

        const symbols: Symbol[] = [];
        const lines = text.split("\n");

        for (const element of model.elements ?? []) {
          // Find line number by searching for element ID in source
          let line = 1;
          for (let i = 0; i < lines.length; i++) {
            const lineText = lines[i];
            if (lineText && (lineText.includes(element.id) || lineText.includes(element.id.split(".").pop() ?? ""))) {
              line = i + 1;
              break;
            }
          }

          symbols.push({
            name: element.id,
            kind: element.kind ?? "system",
            line,
          });
        }

        return symbols;
      } catch (error) {
        logger.error("Failed to get symbols", {
          component: "wasm",
          action: "getSymbols",
          error: error instanceof Error ? error.message : String(error),
        });
        return [];
      }
    },

    async hover(text: string, line: number, column: number): Promise<HoverInfo | null> {
      try {
        const word = getWordAtPosition(text, line, column);
        if (!word) {
          return null;
        }

        // Parse model to find element information
        // Note: This parses the DSL each time. For better performance with large files,
        // consider caching the parsed model if text hasn't changed.
        const modelJson = await this.dslToModel(text);
        const model = JSON.parse(modelJson) as {
          elements?: Array<{
            id: string;
            kind?: string;
            title?: string;
            description?: string;
          }>;
        };

        const element = model.elements?.find(
          (e) => e.id === word || e.id.endsWith(`.${word}`)
        );

        if (element) {
          const kind = element.kind ?? "element";
          const title = element.title ?? element.id;
          const description = element.description ?? "";
          return {
            contents: `**${kind}** \`${element.id}\`\n${title}${description ? `\n\n${description}` : ""}`,
          };
        }

        return null;
      } catch (error) {
        logger.error("Failed to get hover info", {
          component: "wasm",
          action: "hover",
          error: error instanceof Error ? error.message : String(error),
        });
        return null;
      }
    },

    async completion(text: string, line: number, column: number): Promise<CompletionItem[]> {
      try {
        const keywords = [
          "architecture",
          "system",
          "container",
          "component",
          "datastore",
          "database",
          "queue",
          "person",
          "relation",
          "description",
          "technology",
          "tags",
        ];

        const word = getWordAtPosition(text, line, column);
        const items: CompletionItem[] = [];

        // Add keyword completions
        for (const keyword of keywords) {
          if (!word || keyword.toLowerCase().startsWith(word.toLowerCase())) {
            items.push({
              label: keyword,
              kind: "keyword",
            });
          }
        }

        // Add element ID completions
        try {
          const modelJson = await this.dslToModel(text);
          const model = JSON.parse(modelJson) as {
            elements?: Array<{ id: string }>;
          };

          for (const element of model.elements ?? []) {
            if (!word || element.id.toLowerCase().includes(word.toLowerCase())) {
              items.push({
                label: element.id,
                kind: "element",
              });
            }
          }
        } catch {
          // Ignore parse errors for completion
        }

        return items;
      } catch (error) {
        logger.error("Failed to get completions", {
          component: "wasm",
          action: "completion",
          error: error instanceof Error ? error.message : String(error),
        });
        return [];
      }
    },

    async goToDefinition(text: string, line: number, column: number): Promise<Location | null> {
      try {
        const word = getWordAtPosition(text, line, column);
        if (!word) {
          return null;
        }

        const modelJson = await this.dslToModel(text);
        return findElementDefinition(modelJson, word, text);
      } catch (error) {
        logger.error("Failed to go to definition", {
          component: "wasm",
          action: "goToDefinition",
          error: error instanceof Error ? error.message : String(error),
        });
        return null;
      }
    },

    async findReferences(text: string, line: number, column: number): Promise<Location[]> {
      try {
        const word = getWordAtPosition(text, line, column);
        if (!word) {
          return [];
        }

        return findElementReferences(text, word);
      } catch (error) {
        logger.error("Failed to find references", {
          component: "wasm",
          action: "findReferences",
          error: error instanceof Error ? error.message : String(error),
        });
        return [];
      }
    },

    async rename(text: string, line: number, column: number, newName: string): Promise<string> {
      try {
        const word = getWordAtPosition(text, line, column);
        if (!word) {
          return text;
        }

        // Simple find-and-replace (word boundaries)
        const lines = text.split("\n");
        const lineIndex = line - 1;

        if (lineIndex >= 0 && lineIndex < lines.length) {
          const lineText = lines[lineIndex] ?? "";
          const regex = new RegExp(`\\b${word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`, "g");
          lines[lineIndex] = lineText.replace(regex, newName);
        }

        // Replace all occurrences in text
        const regex = new RegExp(`\\b${word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`, "g");
        return text.replace(regex, newName);
      } catch (error) {
        logger.error("Failed to rename", {
          component: "wasm",
          action: "rename",
          error: error instanceof Error ? error.message : String(error),
        });
        return text;
      }
    },

    async format(text: string): Promise<string> {
      // For now, return text as-is. Full formatting would require DSL printer.
      // This can be enhanced later with proper formatting logic.
      return text;
    },

    async codeActions(_text: string, diagnostics: Diagnostic[]): Promise<CodeAction[]> {
      // Basic code actions based on diagnostics
      const actions: CodeAction[] = [];

      for (const diag of diagnostics) {
        if (diag.code === "UNKNOWN_ID" || diag.code === "INVALID_REF") {
          actions.push({
            title: `Fix ${diag.code}`,
            command: "sruja.fixDiagnostic",
            arguments: [diag],
          });
        }
      }

      return actions;
    },

    async semanticTokens(_text: string): Promise<number[]> {
      // Return empty array for now. Semantic tokens would require full parser integration.
      return [];
    },

    async documentLinks(_text: string): Promise<DocumentLink[]> {
      // Return empty array for now. Document links would require parsing imports/references.
      return [];
    },

    async foldingRanges(text: string): Promise<FoldingRange[]> {
      // Basic folding ranges based on indentation
      const ranges: FoldingRange[] = [];
      const lines = text.split("\n");

      let startLine: number | undefined;

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i] ?? "";
        const trimmed = line.trim();
        const currentIndent = line.length - trimmed.length;

        if (trimmed && currentIndent === 0) {
          // New top-level block
          if (startLine !== undefined && i > startLine + 1) {
            ranges.push({
              startLine: startLine + 1,
              endLine: i,
            });
          }
          startLine = i;
        }
      }

      // Close last range
      if (startLine !== undefined && lines.length > startLine + 1) {
        ranges.push({
          startLine: startLine + 1,
          endLine: lines.length,
        });
      }

      return ranges;
    },
  };
}

/**
 * Initialize WASM for Node.js.
 *
 * @public
 * @param options - Initialization options
 * @param options.extensionPath - Path to VS Code extension directory (required)
 * @param options.wasmPath - Optional custom WASM path (deprecated, use extensionPath)
 * @param options.wasmExecPath - Optional WASM exec path (deprecated, not used)
 * @returns Promise resolving to NodeWasmApi
 * @throws ConfigurationError if initialization fails
 * 
 * @remarks
 * Loads Rust WASM module built with `wasm-pack --target nodejs`.
 * Uses singleton pattern - subsequent calls return the same instance.
 * 
 * @example
 * ```typescript
 * const api = await initWasmNode({ extensionPath: "/path/to/extension" });
 * const markdown = await api.dslToMarkdown(dsl);
 * ```
 */
export async function initWasmNode(options?: {
  wasmPath?: string;
  wasmExecPath?: string;
  extensionPath?: string;
}): Promise<NodeWasmApi> {
  // Return cached instance if available
  if (wasmState) {
    return createWasmApi(wasmState.module, wasmState.extensionPath);
  }

  const extensionPath = options?.extensionPath;

  if (!extensionPath) {
    throw new ConfigurationError(
      "extensionPath is required for Node.js WASM initialization. " +
        "Provide the path to the VS Code extension directory containing wasm/rust/ files.",
      {
        configKey: "extensionPath",
        context: { options },
      }
    );
  }

  try {
    const module = await loadRustWasmModule(extensionPath);
    wasmState = { module, extensionPath };

    logger.info("Node.js WASM initialized successfully", {
      component: "wasm",
      action: "init",
      extensionPath,
    });

    return createWasmApi(module, extensionPath);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    logger.error("Failed to initialize Node.js WASM", {
      component: "wasm",
      action: "init",
      error: errorMessage,
      extensionPath,
    });
    throw error;
  }
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

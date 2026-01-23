/* global process */
// packages/shared/src/node/wasmAdapter.ts
// Node.js WASM adapter (Rust only; Go support removed)

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

/**
 * Initialize WASM for Node.js.
 *
 * @public
 * @param options - Initialization options
 * @returns Promise resolving to NodeWasmApi
 */
const RUST_ONLY_MSG =
  "Go WASM support has been removed. Use Rust WASM in browser only. Node/VS Code support will require wasm-pack --target nodejs.";

export async function initWasmNode(_options?: {
  wasmPath?: string;
  wasmExecPath?: string;
  extensionPath?: string;
}): Promise<NodeWasmApi> {
  throw new Error(RUST_ONLY_MSG);
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

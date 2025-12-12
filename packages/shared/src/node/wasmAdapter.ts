// packages/shared/src/node/wasmAdapter.ts
// Node.js-compatible WASM adapter for VS Code extension
// Uses Node.js WebAssembly API instead of browser APIs

import * as fs from 'fs';
import * as path from 'path';
import { exportToMarkdown } from '../export/markdown';
import { generateSystemDiagramForArch } from '../export/mermaid';
import type { ArchitectureJSON } from '../types/architecture';

export interface Diagnostic {
  code: string;
  severity: 'Error' | 'Warning' | 'Info';
  message: string;
  location: {
    file: string;
    line: number;
    column: number;
  };
}

export interface HoverInfo {
  contents: string;
}

export interface CompletionItem {
  label: string;
  kind: string;
}

export interface Location {
  file: string;
  line: number;
  column: number;
}

export type NodeWasmApi = {
  parseDslToJson: (dsl: string, filename?: string) => Promise<string>;
  printJsonToDsl: (json: string) => Promise<string>;
  // Export functions (dslToMarkdown, dslToMermaid) removed - now handled by TypeScript exporters
  // LSP functions
  getDiagnostics: (text: string) => Promise<Diagnostic[]>;
  hover: (text: string, line: number, column: number) => Promise<HoverInfo | null>;
  completion: (text: string, line: number, column: number) => Promise<CompletionItem[]>;
  goToDefinition: (text: string, line: number, column: number) => Promise<Location | null>;
  format: (text: string) => Promise<string>;
}

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

// Load Go WASM runtime for Node.js
async function loadGoRuntime(_options?: { extensionPath?: string; wasmExecPath?: string }): Promise<any> {
  // Try to find wasm_exec.js in common locations
  const candidates = [
    path.join(__dirname, '../../../../wasm/wasm_exec.js'),
    path.join(__dirname, '../../../wasm/wasm_exec.js'),
    path.join(process.cwd(), 'wasm/wasm_exec.js'),
    path.join(process.cwd(), 'node_modules/@sruja/shared/wasm/wasm_exec.js'),
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      // Use require to load the Go runtime
      delete require.cache[require.resolve(candidate)];
      require(candidate);
      if ((global as any).Go) {
        return (global as any).Go;
      }
    }
  }

  throw new Error('wasm_exec.js not found. Please ensure WASM files are available.');
}

// Load WASM module for Node.js
async function loadWasmModule(wasmPath: string, options?: { extensionPath?: string; wasmExecPath?: string }): Promise<void> {
  const Go = await loadGoRuntime(options);
  const go = new Go();
  
  const wasmBuffer = fs.readFileSync(wasmPath);
  // Use WebAssembly API available in Node.js
  // WebAssembly is available in Node.js 12+ and ES2020+
  // Type assertion needed because TypeScript doesn't always recognize WebAssembly in Node.js context
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
  const WebAssemblyAPI = (globalThis as any).WebAssembly as {
    compile: (buffer: Buffer) => Promise<any>;
    instantiate: (module: any, imports: any) => Promise<any>;
  };
  const wasmModule = await WebAssemblyAPI.compile(wasmBuffer);
  const instance = await WebAssemblyAPI.instantiate(wasmModule, go.importObject);
  
  // Run the Go program in a separate promise to avoid blocking
  // The Go program will register functions on global
  await go.run(instance);
}

// Initialize WASM for Node.js
export async function initWasmNode(options?: { wasmPath?: string; wasmExecPath?: string; extensionPath?: string }): Promise<NodeWasmApi> {
  // Find WASM file - prioritize extension path if provided
  const wasmCandidates: string[] = [];
  
  // First, try extension path (most reliable for VS Code extension)
  if (options?.extensionPath) {
    wasmCandidates.push(path.join(options.extensionPath, 'wasm/sruja.wasm'));
  }
  
  // Then try explicit wasmPath
  if (options?.wasmPath) {
    wasmCandidates.push(options.wasmPath);
  }
  
  // Try VS Code extension path if available
  try {
    const vscode = require('vscode');
    const extPath = vscode?.extensions?.getExtension('sruja-ai.sruja-language-support')?.extensionPath;
    if (extPath) {
      wasmCandidates.push(path.join(extPath, 'wasm/sruja.wasm'));
    }
  } catch {
    void 0;
  }
  
  // Then try common locations
  wasmCandidates.push(
    path.join(__dirname, '../../../../wasm/sruja.wasm'),
    path.join(__dirname, '../../../wasm/sruja.wasm'),
    path.join(process.cwd(), 'wasm/sruja.wasm'),
    path.join(process.cwd(), 'node_modules/@sruja/shared/wasm/sruja.wasm'),
  );

  let wasmPath: string | undefined;
  for (const candidate of wasmCandidates) {
    if (fs.existsSync(candidate)) {
      wasmPath = candidate;
      break;
    }
  }

  if (!wasmPath) {
    throw new Error('sruja.wasm not found. Please ensure WASM files are available.');
  }

  await loadWasmModule(wasmPath, { extensionPath: options?.extensionPath, wasmExecPath: options?.wasmExecPath });

  // Check if functions are available on global
  const parseFn = (global as any).sruja_parse_dsl;
  const jsonToDslFn = (global as any).sruja_json_to_dsl;
  
  // LSP functions
  const diagnosticsFn = (global as any).sruja_get_diagnostics;
  // const symbolsFn = (global as any).sruja_get_symbols; // Reserved for future use
  const hoverFn = (global as any).sruja_hover;
  const completionFn = (global as any).sruja_completion;
  const goToDefinitionFn = (global as any).sruja_go_to_definition;
  const formatFn = (global as any).sruja_format;

  if (!parseFn || !jsonToDslFn) {
    throw new Error('WASM functions not found. Missing functions after WASM load.');
  }

  return {
    parseDslToJson: async (dsl: string, filename: string = 'input.sruja'): Promise<string> => {
      try {
        const r = parseFn(dsl, filename) as ParseResult;
        if (!r || !r.ok) {
          throw new Error(r?.error || 'parse failed');
        }
        if (!r.json) {
          throw new Error('parse succeeded but no JSON returned');
        }
        return r.json;
      } catch (error) {
        throw new Error(`WASM parse failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    },
    printJsonToDsl: async (json: string): Promise<string> => {
      try {
        const r = jsonToDslFn(json) as JsonToDslResult;
        if (!r || !r.ok) {
          throw new Error(r?.error || 'print failed');
        }
        if (!r.dsl) {
          throw new Error('print succeeded but no DSL returned');
        }
        return r.dsl;
      } catch (error) {
        throw new Error(`WASM print failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    },
    // Export functions (dslToMarkdown, dslToMermaid) removed - now handled by TypeScript exporters
    // LSP functions
    getDiagnostics: async (text: string): Promise<Diagnostic[]> => {
      if (!diagnosticsFn) {
        throw new Error('WASM getDiagnostics function not available');
      }
      try {
        const r = diagnosticsFn(text) as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok || !r.data) {
          return [];
        }
        const data = JSON.parse(r.data);
        return Array.isArray(data) ? data : [];
      } catch (error) {
        throw new Error(`WASM getDiagnostics failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    },
    hover: async (text: string, line: number, column: number): Promise<HoverInfo | null> => {
      if (!hoverFn) {
        throw new Error('WASM hover function not available');
      }
      try {
        const r = hoverFn(text, line, column) as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok || !r.data || r.data === 'null') {
          return null;
        }
        return JSON.parse(r.data);
      } catch (error) {
        throw new Error(`WASM hover failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    },
    completion: async (text: string, line: number, column: number): Promise<CompletionItem[]> => {
      if (!completionFn) {
        throw new Error('WASM completion function not available');
      }
      try {
        const r = completionFn(text, line, column) as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok || !r.data) {
          return [];
        }
        const data = JSON.parse(r.data);
        return Array.isArray(data) ? data : [];
      } catch (error) {
        throw new Error(`WASM completion failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    },
    goToDefinition: async (text: string, line: number, column: number): Promise<Location | null> => {
      if (!goToDefinitionFn) {
        throw new Error('WASM goToDefinition function not available');
      }
      try {
        const r = goToDefinitionFn(text, line, column) as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok || !r.data || r.data === 'null') {
          return null;
        }
        return JSON.parse(r.data);
      } catch (error) {
        throw new Error(`WASM goToDefinition failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    },
    format: async (text: string): Promise<string> => {
      if (!formatFn) {
        throw new Error('WASM format function not available');
      }
      try {
        const r = formatFn(text) as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok || !r.data) {
          return text;
        }
        return r.data;
      } catch (error) {
        throw new Error(`WASM format failed: ${error instanceof Error ? error.message : String(error)}`);
      }
    },
  };
}

/**
 * Convert DSL string to Markdown string using TypeScript exporter.
 * Returns markdown string if successful, null on error.
 * Uses TypeScript markdown exporter instead of WASM.
 */
export async function convertDslToMarkdown(dsl: string, wasmApi?: NodeWasmApi, filename?: string): Promise<string | null> {
  let api = wasmApi;
  if (!api) {
    try {
      api = await initWasmNode();
    } catch (error) {
      return null;
    }
  }

  try {
    // Parse DSL to JSON using WASM (keep parsing in WASM per roadmap)
    const jsonStr = await api.parseDslToJson(dsl, filename);
    const archJson = JSON.parse(jsonStr) as ArchitectureJSON;
    // Use TypeScript markdown exporter
    return exportToMarkdown(archJson);
  } catch (error) {
    return null;
  }
}

/**
 * Convert DSL string to Mermaid diagram string using TypeScript exporter.
 * Returns mermaid diagram string if successful, null on error.
 * Uses TypeScript mermaid exporter instead of WASM.
 */
export async function convertDslToMermaid(dsl: string, wasmApi?: NodeWasmApi): Promise<string | null> {
  let api = wasmApi;
  if (!api) {
    try {
      api = await initWasmNode();
    } catch (error) {
      return null;
    }
  }

  try {
    // Parse DSL to JSON using WASM (keep parsing in WASM per roadmap)
    const jsonStr = await api.parseDslToJson(dsl);
    const archJson = JSON.parse(jsonStr) as ArchitectureJSON;
    // Use TypeScript mermaid exporter
    return generateSystemDiagramForArch(archJson);
  } catch (error) {
    return null;
  }
}

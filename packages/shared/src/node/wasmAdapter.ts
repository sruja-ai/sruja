// packages/shared/src/node/wasmAdapter.ts
// Node.js-compatible WASM adapter for VS Code extension
// Uses Node.js WebAssembly API instead of browser APIs

import * as fs from "fs";
import * as path from "path";
import * as zlib from "zlib";
import { pathToFileURL } from "url";
import { exportToMarkdown } from "../export/markdown";
import { generateSystemDiagramForArch } from "../export/mermaid";
import type { ArchitectureJSON } from "../types/architecture";

export interface Diagnostic {
  code: string;
  severity: "Error" | "Warning" | "Info";
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

// Load Go WASM runtime for Node.js
async function loadGoRuntime(options?: {
  extensionPath?: string;
  wasmExecPath?: string;
}): Promise<any> {
  console.log("[WASM] Loading Go runtime...");
  // Try to find wasm_exec.js in common locations
  const candidates: string[] = [];

  // First, try explicit wasmExecPath if provided
  if (options?.wasmExecPath) {
    candidates.push(options.wasmExecPath);
  }

  // Then try extension path (most reliable for VS Code extension)
  if (options?.extensionPath) {
    candidates.push(path.join(options.extensionPath, "wasm/wasm_exec.js"));
  }

  // Then try relative paths from bundled location
  candidates.push(
    path.join(__dirname, "../../../../wasm/wasm_exec.js"),
    path.join(__dirname, "../../../wasm/wasm_exec.js"),
    path.join(process.cwd(), "wasm/wasm_exec.js"),
    path.join(process.cwd(), "node_modules/@sruja/shared/wasm/wasm_exec.js")
  );

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      console.log(`[WASM] Trying to load wasm_exec.js from: ${candidate}`);
      try {
        // wasm_exec.js is CommonJS, use require() for Node.js
        // Clear cache to ensure fresh load
        const resolvedPath = require.resolve(candidate);
        delete require.cache[resolvedPath];
        require(resolvedPath);
        if ((global as any).Go) {
          console.log("[WASM] Go runtime loaded successfully");
          return (global as any).Go;
        } else {
          console.warn("[WASM] wasm_exec.js loaded but Go class not found on global");
        }
      } catch (error) {
        console.warn(`[WASM] Failed to load ${candidate} with require:`, error);
        // Try ES module import as fallback (for ESM contexts)
        try {
          await import(pathToFileURL(candidate).href);
          if ((global as any).Go) {
            console.log("[WASM] Go runtime loaded successfully (via ESM)");
            return (global as any).Go;
          }
        } catch (esmError) {
          console.warn(`[WASM] Failed to load ${candidate} with ESM:`, esmError);
          // Continue to next candidate
        }
      }
    }
  }

  throw new Error("wasm_exec.js not found. Please ensure WASM files are available.");
}

// Load WASM module for Node.js (supports compressed files)
async function loadWasmModule(
  wasmPath: string,
  options?: { extensionPath?: string; wasmExecPath?: string }
): Promise<void> {
  console.log(`[WASM] Loading WASM module from: ${wasmPath}`);
  const Go = await loadGoRuntime(options);
  const go = new Go();
  console.log("[WASM] Go instance created");

  let wasmBuffer: Buffer;
  console.log("[WASM] Reading WASM file...");
  const wasmData = fs.readFileSync(wasmPath);
  console.log(`[WASM] WASM file read: ${wasmData.length} bytes`);

  // Check if file is gzip compressed (magic bytes: 1f 8b)
  if (wasmData.length >= 2 && wasmData[0] === 0x1f && wasmData[1] === 0x8b) {
    console.log("[WASM] Decompressing gzip...");
    // Decompress gzip
    wasmBuffer = zlib.gunzipSync(wasmData);
    console.log(`[WASM] Decompressed: ${wasmBuffer.length} bytes`);
  } else {
    // Use as-is (uncompressed)
    wasmBuffer = wasmData;
    console.log("[WASM] Using uncompressed WASM");
  }

  // Use WebAssembly API available in Node.js
  // WebAssembly is available in Node.js 12+ and ES2020+
  // Type assertion needed because TypeScript doesn't always recognize WebAssembly in Node.js context
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
  const WebAssemblyAPI = (globalThis as any).WebAssembly as {
    compile: (buffer: Buffer) => Promise<any>;
    instantiate: (module: any, imports: any) => Promise<any>;
  };

  console.log("[WASM] Compiling WASM module...");
  const wasmModule = await WebAssemblyAPI.compile(wasmBuffer);
  console.log("[WASM] WASM module compiled");

  console.log("[WASM] Instantiating WASM module...");
  const instance = await WebAssemblyAPI.instantiate(wasmModule, go.importObject);
  console.log("[WASM] WASM module instantiated");

  // Run the Go program in a separate promise to avoid blocking
  // The Go program will register functions on global
  console.log("[WASM] Running Go program (this may take a moment)...");
  try {
    await go.run(instance);
    console.log("[WASM] Go program completed");
  } catch (error) {
    console.error("[WASM] Go program failed:", error);
    throw error;
  }
}

// Initialize WASM for Node.js
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

  console.log(`[WASM] Loading WASM module from: ${wasmPath}`);
  await loadWasmModule(wasmPath, {
    extensionPath: options?.extensionPath,
    wasmExecPath: options?.wasmExecPath,
  });

  console.log("[WASM] Checking for exported functions...");
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
    const missing = [];
    if (!parseFn) missing.push("sruja_parse_dsl");
    if (!jsonToDslFn) missing.push("sruja_json_to_dsl");
    throw new Error(`WASM functions not found. Missing: ${missing.join(", ")}`);
  }

  // Log available LSP functions for debugging
  const availableLspFunctions = [];
  if (diagnosticsFn) availableLspFunctions.push("diagnostics");
  if (hoverFn) availableLspFunctions.push("hover");
  if (completionFn) availableLspFunctions.push("completion");
  if (goToDefinitionFn) availableLspFunctions.push("goToDefinition");
  if (formatFn) availableLspFunctions.push("format");

  if (availableLspFunctions.length > 0) {
    console.log(`WASM LSP functions available: ${availableLspFunctions.join(", ")}`);
  } else {
    console.warn("No WASM LSP functions found - LSP features may not work");
  }

  // Verify functions are actually callable
  if (hoverFn) {
    try {
      const testResult = hoverFn('architecture "Test" { system App "App" {} }', 1, 15);
      console.log("[WASM] Hover function test:", testResult);
    } catch (e) {
      console.warn("[WASM] Hover function test failed:", e);
    }
  }

  if (goToDefinitionFn) {
    try {
      const testResult = goToDefinitionFn('architecture "Test" { system App "App" {} }', 1, 15);
      console.log("[WASM] GoToDefinition function test:", testResult);
    } catch (e) {
      console.warn("[WASM] GoToDefinition function test failed:", e);
    }
  }

  return {
    parseDslToJson: async (dsl: string, filename: string = "input.sruja"): Promise<string> => {
      try {
        const r = parseFn(dsl, filename) as ParseResult;
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
      try {
        const r = jsonToDslFn(json) as JsonToDslResult;
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
    // Export functions (dslToMarkdown, dslToMermaid) removed - now handled by TypeScript exporters
    // LSP functions
    getDiagnostics: async (text: string): Promise<Diagnostic[]> => {
      if (!diagnosticsFn) {
        console.warn("WASM getDiagnostics function not available");
        return [];
      }
      try {
        const r = diagnosticsFn(text) as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok) {
          if (r?.error) {
            console.warn("WASM getDiagnostics returned error:", r.error);
          }
          return [];
        }
        if (!r.data) {
          return [];
        }
        const data = JSON.parse(r.data);
        return Array.isArray(data) ? data : [];
      } catch (error) {
        console.error("WASM getDiagnostics failed:", error);
        return [];
      }
    },
    hover: async (text: string, line: number, column: number): Promise<HoverInfo | null> => {
      if (!hoverFn) {
        console.warn("WASM hover function not available");
        return null;
      }
      try {
        console.log(`[WASM] Calling hover with line=${line}, column=${column}`);
        const r = hoverFn(text, line, column) as { ok: boolean; data?: string; error?: string };
        console.log(`[WASM] Hover result:`, {
          ok: r?.ok,
          hasData: !!r?.data,
          data: r?.data?.substring(0, 100),
        });

        if (!r || !r.ok) {
          if (r?.error) {
            console.warn("WASM hover returned error:", r.error);
          }
          return null;
        }

        if (!r.data || r.data === "null") {
          return null;
        }

        const parsed = JSON.parse(r.data);
        console.log(`[WASM] Parsed hover data:`, parsed);
        return parsed;
      } catch (error) {
        console.error("WASM hover failed:", error);
        return null;
      }
    },
    completion: async (text: string, line: number, column: number): Promise<CompletionItem[]> => {
      if (!completionFn) {
        return [];
      }
      try {
        const r = completionFn(text, line, column) as {
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
        console.error("WASM completion failed:", error);
        return [];
      }
    },
    goToDefinition: async (
      text: string,
      line: number,
      column: number
    ): Promise<Location | null> => {
      if (!goToDefinitionFn) {
        console.warn("WASM goToDefinition function not available");
        return null;
      }
      try {
        console.log(`[WASM] Calling goToDefinition with line=${line}, column=${column}`);
        const r = goToDefinitionFn(text, line, column) as {
          ok: boolean;
          data?: string;
          error?: string;
        };
        console.log(`[WASM] GoToDefinition result:`, {
          ok: r?.ok,
          hasData: !!r?.data,
          data: r?.data,
        });

        if (!r || !r.ok) {
          if (r?.error) {
            console.warn("WASM goToDefinition returned error:", r.error);
          }
          return null;
        }

        if (!r.data || r.data === "null") {
          return null;
        }

        const parsed = JSON.parse(r.data);
        console.log(`[WASM] Parsed definition data:`, parsed);
        return parsed;
      } catch (error) {
        console.error("WASM goToDefinition failed:", error);
        return null;
      }
    },
    format: async (text: string): Promise<string> => {
      if (!formatFn) {
        // Return original text if formatting not available
        return text;
      }
      try {
        const r = formatFn(text) as { ok: boolean; data?: string; error?: string };
        if (!r || !r.ok || !r.data) {
          console.warn("WASM format failed:", r?.error || "format failed");
          return text; // Return original on failure
        }
        return r.data;
      } catch (error) {
        console.error("WASM format failed:", error);
        return text; // Return original on error
      }
    },
  };
}

/**
 * Convert DSL string to Markdown string using TypeScript exporter.
 * Returns markdown string if successful, null on error.
 * Uses TypeScript markdown exporter instead of WASM.
 */
export async function convertDslToMarkdown(
  dsl: string,
  wasmApi?: NodeWasmApi,
  filename?: string
): Promise<string | null> {
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
export async function convertDslToMermaid(
  dsl: string,
  wasmApi?: NodeWasmApi
): Promise<string | null> {
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

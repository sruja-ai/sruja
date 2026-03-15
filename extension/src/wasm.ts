/**
 * Load Sruja WASM (Node build) and expose getDiagnostics + exportMarkdown.
 * The extension always ships with WASM (copy-assets); use CLI only when sruja.lsp.path is set.
 */

import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";

/** WASM diagnostic shape (severity: 1=Error, 2=Warning, 3=Info). Exported for tests. */
export interface WasmDiagnostic {
  range: { start: { line: number; character: number }; end: { line: number; character: number } };
  severity: number;
  code?: string;
  message: string;
  source?: string;
}

/** WASM/Sruja use 1-based line and column; VS Code uses 0-based. Convert to VS Code Range. */
export function wasmRangeToVscodeRange(r: {
  start: { line: number; character: number };
  end: { line: number; character: number };
}): vscode.Range {
  const start = new vscode.Position(
    Math.max(0, r.start.line - 1),
    Math.max(0, r.start.character - 1)
  );
  const end = new vscode.Position(
    Math.max(0, r.end.line - 1),
    Math.max(0, r.end.character - 1)
  );
  return new vscode.Range(start, end);
}

/**
 * Map JSON string from sruja_get_diagnostics to VS Code diagnostics. Pure and testable.
 */
export function mapWasmDiagnosticsJsonToVscode(json: string): vscode.Diagnostic[] {
  try {
    const raw: WasmDiagnostic[] = JSON.parse(json);
    return raw.map((d) => {
      const range = wasmRangeToVscodeRange(d.range);
      const severity =
        d.severity === 1
          ? vscode.DiagnosticSeverity.Error
          : d.severity === 2
            ? vscode.DiagnosticSeverity.Warning
            : vscode.DiagnosticSeverity.Information;
      const diag = new vscode.Diagnostic(range, d.message, severity);
      if (d.code) diag.code = d.code;
      diag.source = d.source ?? "sruja";
      return diag;
    });
  } catch {
    return [];
  }
}

interface SrujaWasmModule {
  sruja_get_diagnostics(dsl: string, filename?: string | null): string;
  sruja_dsl_to_markdown(dsl: string): string;
  sruja_dsl_to_mermaid(dsl: string, config_json?: string | null): string;
  sruja_get_elements(dsl: string, filename?: string | null): string;
  sruja_get_document_symbols(dsl: string, filename?: string | null): string;
  init_panic_hook(): void;
}

let wasmModule: SrujaWasmModule | null = null;
let wasmInitPromise: Promise<SrujaWasmModule | null> | null = null;

function wasmDir(context: vscode.ExtensionContext): string {
  return path.join(context.extensionPath, "wasm");
}

function isWasmAvailable(context: vscode.ExtensionContext): boolean {
  const dir = wasmDir(context);
  return (
    fs.existsSync(path.join(dir, "sruja_wasm.js")) &&
    fs.existsSync(path.join(dir, "sruja_wasm_bg.wasm"))
  );
}

/**
 * Initialize the WASM module from extension/wasm/ (Node build).
 * Returns the module or null if not available / init fails.
 */
export async function initWasm(context: vscode.ExtensionContext): Promise<SrujaWasmModule | null> {
  if (wasmModule) return wasmModule;
  if (wasmInitPromise) return wasmInitPromise;
  if (!isWasmAvailable(context)) return null;

  wasmInitPromise = (async (): Promise<SrujaWasmModule | null> => {
    try {
      const dir = wasmDir(context);
      const init = require(path.join(dir, "sruja_wasm.js"));
      const mod = typeof init === "function" ? await init() : await (init.default?.() ?? init);
      if (mod?.init_panic_hook) mod.init_panic_hook();
      wasmModule = mod as SrujaWasmModule;
      return wasmModule;
    } catch (e) {
      console.warn("[Sruja] WASM init failed:", e);
      return null;
    }
  })();
  return wasmInitPromise;
}

/**
 * Get diagnostics for DSL text using WASM. Returns [] if WASM not ready or on error.
 */
export async function getDiagnosticsFromWasm(
  context: vscode.ExtensionContext,
  dsl: string,
  filename: string
): Promise<vscode.Diagnostic[]> {
  const mod = await initWasm(context);
  if (!mod) return [];

  try {
    const json = mod.sruja_get_diagnostics(dsl, filename);
    return mapWasmDiagnosticsJsonToVscode(json);
  } catch {
    return [];
  }
}

/**
 * Export DSL to Markdown using WASM. Returns null if WASM not ready or on error.
 */
export async function exportMarkdownFromWasm(
  context: vscode.ExtensionContext,
  dsl: string
): Promise<string | null> {
  const mod = await initWasm(context);
  if (!mod) return null;

  try {
    return mod.sruja_dsl_to_markdown(dsl);
  } catch {
    return null;
  }
}

/**
 * Export DSL to Mermaid diagram using WASM. Returns null if WASM not ready or on error.
 */
export async function getMermaidFromWasm(
  context: vscode.ExtensionContext,
  dsl: string,
  configJson?: string | null
): Promise<string | null> {
  const mod = await initWasm(context);
  if (!mod) return null;

  try {
    return mod.sruja_dsl_to_mermaid(dsl, configJson ?? null);
  } catch {
    return null;
  }
}

/**
 * Element definition with location
 */
export interface SrujaElement {
  id: string;
  kind: string;
  title: string | null;
  range: { start: { line: number; character: number }; end: { line: number; character: number } };
}

/**
 * Document symbol for outline view
 */
export interface SrujaDocumentSymbol {
  kind: "element" | "view" | "scenario" | "flow" | "requirement" | "adr" | "policy";
  name: string;
  detail: string;
  range: { start: { line: number; character: number }; end: { line: number; character: number } };
  children: SrujaDocumentSymbol[];
}

/**
 * Get element definitions from DSL using WASM. Returns empty list if WASM not ready or on error.
 */
export async function getElementsFromWasm(
  context: vscode.ExtensionContext,
  dsl: string,
  filename?: string
): Promise<SrujaElement[] | null> {
  const mod = await initWasm(context);
  if (!mod) return null;

  try {
    const json = mod.sruja_get_elements(dsl, filename ?? null);
    return JSON.parse(json);
  } catch {
    return null;
  }
}

/**
 * Get document symbols for outline view using WASM. Returns empty list if WASM not ready or on error.
 */
export async function getDocumentSymbolsFromWasm(
  context: vscode.ExtensionContext,
  dsl: string,
  filename?: string
): Promise<SrujaDocumentSymbol[] | null> {
  const mod = await initWasm(context);
  if (!mod) return null;

  try {
    const json = mod.sruja_get_document_symbols(dsl, filename ?? null);
    return JSON.parse(json);
  } catch {
    return null;
  }
}

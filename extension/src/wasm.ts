/**
 * Load Sruja WASM (Node build) and expose getDiagnostics + exportMarkdown.
 * The extension always ships with WASM (copy-assets); use CLI only when sruja.lsp.path is set.
 */

import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";

interface WasmDiagnostic {
  range: { start: { line: number; character: number }; end: { line: number; character: number } };
  severity: number;
  code?: string;
  message: string;
  source?: string;
}

interface SrujaWasmModule {
  sruja_get_diagnostics(dsl: string, filename?: string | null): string;
  sruja_dsl_to_markdown(dsl: string): string;
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
    const raw: WasmDiagnostic[] = JSON.parse(json);
    return raw.map((d) => {
      const start = new vscode.Position(d.range.start.line, d.range.start.character);
      const end = new vscode.Position(d.range.end.line, d.range.end.character);
      const severity =
        d.severity === 1
          ? vscode.DiagnosticSeverity.Error
          : d.severity === 2
            ? vscode.DiagnosticSeverity.Warning
            : vscode.DiagnosticSeverity.Information;
      const diag = new vscode.Diagnostic(new vscode.Range(start, end), d.message, severity);
      if (d.code) diag.code = d.code;
      diag.source = d.source ?? "sruja";
      return diag;
    });
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

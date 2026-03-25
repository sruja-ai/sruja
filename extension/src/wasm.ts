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

/** Error types for WASM operations */
export class WasmError extends Error {
  constructor(
    message: string,
    public readonly operation: string,
    public readonly cause?: unknown
  ) {
    super(message);
    this.name = "WasmError";
  }
}

export class WasmInitError extends WasmError {
  constructor(message: string, cause?: unknown) {
    super(message, "init", cause);
    this.name = "WasmInitError";
  }
}

export class WasmParseError extends WasmError {
  constructor(message: string, public readonly raw?: string, cause?: unknown) {
    super(message, "parse", cause);
    this.name = "WasmParseError";
  }
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
  } catch (e) {
    console.error("[Sruja] Failed to parse diagnostics JSON:", e, "Input:", json.substring(0, 200));
    return [];
  }
}

interface SrujaWasmModule {
  sruja_get_diagnostics(dsl: string, filename?: string | null): string;
  sruja_dsl_to_markdown(dsl: string): string;
  sruja_dsl_to_mermaid(dsl: string, config_json?: string | null): string;
  sruja_dsl_to_sequence_diagram(dsl: string, config_json?: string | null): string;
  sruja_get_elements(dsl: string, filename?: string | null): string;
  sruja_get_document_symbols(dsl: string, filename?: string | null): string;
  init_panic_hook(): void;
}

let wasmModule: SrujaWasmModule | null = null;
let wasmInitPromise: Promise<SrujaWasmModule | null> | null = null;

/**
 * Cache for WASM results to avoid redundant parsing.
 * Keyed by document URI and version.
 */
class WasmCache<T> {
  private cache = new Map<string, { version: number; data: T }>();

  get(uri: string, version: number): T | undefined {
    const entry = this.cache.get(uri);
    if (entry && entry.version === version) {
      return entry.data;
    }
    return undefined;
  }

  set(uri: string, version: number, data: T): void {
    this.cache.set(uri, { version, data });
  }

  delete(uri: string): void {
    this.cache.delete(uri);
  }

  clear(): void {
    this.cache.clear();
  }
}

const elementsCache = new WasmCache<SrujaElement[]>();
const symbolsCache = new WasmCache<SrujaDocumentSymbol[]>();

/** Reset module state for tests. Only use in test code. */
export function resetWasmForTesting(): void {
  wasmModule = null;
  wasmInitPromise = null;
  elementsCache.clear();
  symbolsCache.clear();
}

/**
 * Invalidate cache for a specific URI.
 */
export function invalidateWasmCache(uri: string): void {
  elementsCache.delete(uri);
  symbolsCache.delete(uri);
}

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
 * Uses a promise-based lock pattern to prevent race conditions.
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
      console.error("[Sruja] WASM initialization failed:", e);
      wasmInitPromise = null;
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
  } catch (e) {
    console.error("[Sruja] Failed to get diagnostics from WASM:", e, "File:", filename);
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
  } catch (e) {
    console.error("[Sruja] Failed to export markdown from WASM:", e);
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
  } catch (e) {
    console.error("[Sruja] Failed to get Mermaid from WASM:", e);
    return null;
  }
}

/**
 * Export a single scenario/flow to a Mermaid sequence diagram using WASM.
 * Returns null if WASM not ready or on error.
 */
export async function getSequenceDiagramFromWasm(
  context: vscode.ExtensionContext,
  dsl: string,
  configJson?: string | null
): Promise<string | null> {
  const mod = await initWasm(context);
  if (!mod) return null;

  try {
    return mod.sruja_dsl_to_sequence_diagram(dsl, configJson ?? null);
  } catch (e) {
    console.error("[Sruja] Failed to get sequence diagram from WASM:", e);
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
  /** Path to component knowledge markdown (relative to workspace root). */
  doc?: string | null;
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

/** Type guard for SrujaElement - validates all required fields */
function isSrujaElement(item: unknown): item is SrujaElement {
  if (typeof item !== "object" || item === null) return false;
  const el = item as Record<string, unknown>;
  return (
    typeof el.id === "string" &&
    typeof el.kind === "string" &&
    (el.title === null || typeof el.title === "string") &&
    typeof el.range === "object" &&
    el.range !== null &&
    typeof (el.range as Record<string, unknown>).start === "object" &&
    typeof (el.range as Record<string, unknown>).end === "object"
  );
}

/** Type guard for SrujaDocumentSymbol - validates all required fields */
function isSrujaDocumentSymbol(item: unknown): item is SrujaDocumentSymbol {
  if (typeof item !== "object" || item === null) return false;
  const sym = item as Record<string, unknown>;
  const validKinds = ["element", "view", "scenario", "flow", "requirement", "adr", "policy"];
  return (
    typeof sym.kind === "string" &&
    validKinds.includes(sym.kind) &&
    typeof sym.name === "string" &&
    typeof sym.detail === "string" &&
    typeof sym.range === "object" &&
    sym.range !== null &&
    Array.isArray(sym.children)
  );
}

function parseJsonArray<T>(json: string, guard?: (item: unknown) => item is T): T[] | null {
  try {
    const value: unknown = JSON.parse(json);
    if (!Array.isArray(value)) {
      console.error("[Sruja] Expected JSON array, got:", typeof value);
      return null;
    }
    if (guard) {
      const out: T[] = [];
      for (const item of value) {
        if (guard(item)) {
          out.push(item);
        } else {
          console.warn("[Sruja] Filtered malformed item:", JSON.stringify(item).substring(0, 100));
        }
      }
      return out;
    }
    return value as T[];
  } catch (e) {
    console.error("[Sruja] Failed to parse JSON:", e, "Input:", json.substring(0, 200));
    return null;
  }
}

/**
 * Get element definitions from DSL using WASM. Returns null if WASM not ready, parse error, or on error.
 * Returns empty array when valid JSON has no elements; malformed entries are filtered out.
 */
export async function getElementsFromWasm(
  context: vscode.ExtensionContext,
  dsl: string,
  filename?: string,
  uri?: string,
  version?: number
): Promise<SrujaElement[] | null> {
  if (uri && version !== undefined) {
    const cached = elementsCache.get(uri, version);
    if (cached) return cached;
  }

  const mod = await initWasm(context);
  if (!mod) return null;

  try {
    const json = mod.sruja_get_elements(dsl, filename ?? null);
    const elements = parseJsonArray<SrujaElement>(json, isSrujaElement);
    if (elements && uri && version !== undefined) {
      elementsCache.set(uri, version, elements);
    }
    return elements;
  } catch (e) {
    console.error("[Sruja] Failed to get elements from WASM:", e, "File:", filename);
    return null;
  }
}

/**
 * Get document symbols for outline view using WASM. Returns null if WASM not ready, parse error, or on error.
 * Returns empty array when valid JSON has no symbols; malformed entries are filtered out.
 */
export async function getDocumentSymbolsFromWasm(
  context: vscode.ExtensionContext,
  dsl: string,
  filename?: string,
  uri?: string,
  version?: number
): Promise<SrujaDocumentSymbol[] | null> {
  if (uri && version !== undefined) {
    const cached = symbolsCache.get(uri, version);
    if (cached) return cached;
  }

  const mod = await initWasm(context);
  if (!mod) return null;

  try {
    const json = mod.sruja_get_document_symbols(dsl, filename ?? null);
    const symbols = parseJsonArray<SrujaDocumentSymbol>(json, isSrujaDocumentSymbol);
    if (symbols && uri && version !== undefined) {
      symbolsCache.set(uri, version, symbols);
    }
    return symbols;
  } catch (e) {
    console.error("[Sruja] Failed to get document symbols from WASM:", e, "File:", filename);
    return null;
  }
}

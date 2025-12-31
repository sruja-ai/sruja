// packages/shared/src/lsp/wasmLspShim.ts
// WASM-based LSP shim for Monaco editor
// This provides LSP features without a real LSP server by using WASM directly

import type * as monaco from "monaco-editor";
import { propertyKeys, isInsideBlock } from "../registry/propertyKeys";

export interface WasmLspApi {
  getDiagnostics: (text: string) => Promise<Diagnostic[]>;
  getSymbols: (text: string) => Promise<LspSymbol[]>;
  hover: (text: string, line: number, column: number) => Promise<HoverInfo | null>;
  completion: (text: string, line: number, column: number) => Promise<CompletionItem[]>;
  goToDefinition: (text: string, line: number, column: number) => Promise<Location | null>;
  format: (text: string) => Promise<string>;
}

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

export interface LspSymbol {
  name: string;
  kind: string;
  line: number;
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

// Get WASM functions from global scope
function getWasmFunction(
  name: string
): ((...args: unknown[]) => { ok: boolean; data?: string; error?: string }) | null {
  if (typeof window === "undefined") return null;
  return (window as unknown as Record<string, unknown>)[name] as
    | ((...args: unknown[]) => { ok: boolean; data?: string; error?: string })
    | null;
}

// WASM LSP API implementation
export function createWasmLspApi(): WasmLspApi {
  return {
    async getDiagnostics(text: string): Promise<Diagnostic[]> {
      const fn = getWasmFunction("sruja_get_diagnostics");
      if (!fn) return [];

      try {
        const result = fn(text);
        if (!result || !result.ok) return [];
        const data = JSON.parse(result.data as string);
        return Array.isArray(data) ? data : [];
      } catch (e) {
        console.warn("WASM getDiagnostics failed:", e);
        return [];
      }
    },

    async getSymbols(text: string): Promise<LspSymbol[]> {
      const fn = getWasmFunction("sruja_get_symbols");
      if (!fn) return [];

      try {
        const result = fn(text);
        if (!result || !result.ok) return [];
        const data = JSON.parse(result.data as string);
        return Array.isArray(data) ? data : [];
      } catch (e) {
        console.warn("WASM getSymbols failed:", e);
        return [];
      }
    },

    async hover(text: string, line: number, column: number): Promise<HoverInfo | null> {
      const fn = getWasmFunction("sruja_hover");
      if (!fn) return null;

      try {
        const result = fn(text, line, column);
        if (!result || !result.ok || !result.data) return null;
        if (result.data === "null") return null;
        return JSON.parse(result.data as string);
      } catch (e) {
        console.warn("WASM hover failed:", e);
        return null;
      }
    },

    async completion(text: string, line: number, column: number): Promise<CompletionItem[]> {
      const fn = getWasmFunction("sruja_completion");
      if (!fn) return [];

      try {
        const result = fn(text, line, column);
        if (!result || !result.ok) return [];
        const data = JSON.parse(result.data as string);
        return Array.isArray(data) ? data : [];
      } catch (e) {
        console.warn("WASM completion failed:", e);
        return [];
      }
    },

    async goToDefinition(text: string, line: number, column: number): Promise<Location | null> {
      const fn = getWasmFunction("sruja_go_to_definition");
      if (!fn) return null;

      try {
        const result = fn(text, line, column);
        if (!result || !result.ok || !result.data) return null;
        if (result.data === "null") return null;
        return JSON.parse(result.data as string);
      } catch (e) {
        console.warn("WASM goToDefinition failed:", e);
        return null;
      }
    },

    async format(text: string): Promise<string> {
      const fn = getWasmFunction("sruja_format");
      if (!fn) return text;

      try {
        const result = fn(text);
        if (!result || !result.ok) return text;
        return result.data as string;
      } catch (e) {
        console.warn("WASM format failed:", e);
        return text;
      }
    },
  };
}

// Initialize Monaco editor with WASM-based LSP features
export function initializeMonacoWasmLsp(
  monaco: typeof import("monaco-editor"),
  editor: monaco.editor.IStandaloneCodeEditor,
  wasmApi: WasmLspApi
) {
  // Register diagnostics provider
  const diagnosticsProvider = {
    provideDiagnostics: async (model: monaco.editor.ITextModel) => {
      const text = model.getValue();
      const diagnostics = await wasmApi.getDiagnostics(text);

      const markers: monaco.editor.IMarkerData[] = diagnostics.map((d) => ({
        severity:
          d.severity === "Error"
            ? monaco.MarkerSeverity.Error
            : d.severity === "Warning"
              ? monaco.MarkerSeverity.Warning
              : monaco.MarkerSeverity.Info,
        message: d.message,
        startLineNumber: d.location.line,
        startColumn: d.location.column,
        endLineNumber: d.location.line,
        endColumn: d.location.column + 10, // Approximate end
        code: d.code,
      }));

      monaco.editor.setModelMarkers(model, "sruja-wasm", markers);
    },
  };

  // Update diagnostics on model change
  editor.onDidChangeModelContent(() => {
    const model = editor.getModel();
    if (model) {
      diagnosticsProvider.provideDiagnostics(model);
    }
  });

  // Initial diagnostics
  const model = editor.getModel();
  if (model) {
    diagnosticsProvider.provideDiagnostics(model);
  }

  // Register hover provider
  monaco.languages.registerHoverProvider("sruja", {
    provideHover: async (model, position) => {
      const text = model.getValue();
      const hoverInfo = await wasmApi.hover(text, position.lineNumber, position.column);

      if (!hoverInfo) return null;

      return {
        range: new monaco.Range(
          position.lineNumber,
          position.column,
          position.lineNumber,
          position.column
        ),
        contents: [{ value: hoverInfo.contents }],
      };
    },
  });

  // Register completion provider
  monaco.languages.registerCompletionItemProvider("sruja", {
    provideCompletionItems: async (model, position) => {
      const text = model.getValue();
      const completions = await wasmApi.completion(text, position.lineNumber, position.column);

      const word = model.getWordUntilPosition(position);
      const range = new monaco.Range(
        position.lineNumber,
        word.startColumn,
        position.lineNumber,
        word.endColumn
      );

      const items: monaco.languages.CompletionItem[] = completions.map((c) => ({
        label: c.label,
        kind: monaco.languages.CompletionItemKind.Keyword,
        insertText: c.label,
        range: range,
      }));

      // Augment with property keys when cursor is within properties block
      // Monaco uses 1-based line numbers, isInsideBlock also expects 1-based
      if (isInsideBlock(text, position.lineNumber, "properties")) {
        const propItems = propertyKeys.map((k) => ({
          label: k,
          kind: monaco.languages.CompletionItemKind.Property,
          insertText: k,
          range,
        }));
        items.push(...propItems);
      }

      return {
        suggestions: items,
      };
    },
  });

  // Register definition provider
  monaco.languages.registerDefinitionProvider("sruja", {
    provideDefinition: async (model, position) => {
      const text = model.getValue();
      const location = await wasmApi.goToDefinition(text, position.lineNumber, position.column);

      if (!location) return [];

      return [
        {
          uri: model.uri,
          range: new monaco.Range(
            location.line,
            location.column,
            location.line,
            location.column + 10
          ),
        },
      ];
    },
  });

  // Register format provider
  monaco.languages.registerDocumentFormattingEditProvider("sruja", {
    provideDocumentFormattingEdits: async (model) => {
      const text = model.getValue();
      const formatted = await wasmApi.format(text);

      return [
        {
          range: model.getFullModelRange(),
          text: formatted,
        },
      ];
    },
  });
}

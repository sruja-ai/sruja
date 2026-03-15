import * as vscode from "vscode";
import { ExtensionContext } from "./__mocks__/vscode";
import {
  mapWasmDiagnosticsJsonToVscode,
  wasmRangeToVscodeRange,
  getDiagnosticsFromWasm,
  exportMarkdownFromWasm,
  getMermaidFromWasm,
  getElementsFromWasm,
  getDocumentSymbolsFromWasm,
} from "./wasm";

function asContext(ctx: ExtensionContext): vscode.ExtensionContext {
  return ctx as unknown as vscode.ExtensionContext;
}

describe("wasm", () => {
  const ctx = new ExtensionContext();
  ctx.extensionPath = "/nonexistent-no-wasm";
  const context = asContext(ctx);

  it("getDiagnosticsFromWasm returns empty array when WASM not available", async () => {
    const result = await getDiagnosticsFromWasm(context, "system A {}", "a.sruja");
    expect(result).toEqual([]);
  });

  it("exportMarkdownFromWasm returns null when WASM not available", async () => {
    const result = await exportMarkdownFromWasm(context, "system A {}");
    expect(result).toBeNull();
  });

  it("getMermaidFromWasm returns null when WASM not available", async () => {
    const result = await getMermaidFromWasm(context, "system A {}");
    expect(result).toBeNull();
  });

  it("getElementsFromWasm returns null when WASM not available", async () => {
    const result = await getElementsFromWasm(context, "system A {}", "a.sruja");
    expect(result).toBeNull();
  });

  it("getDocumentSymbolsFromWasm returns null when WASM not available", async () => {
    const result = await getDocumentSymbolsFromWasm(context, "system A {}", "a.sruja");
    expect(result).toBeNull();
  });
});

describe("mapWasmDiagnosticsJsonToVscode", () => {
  it("returns empty array for invalid JSON", () => {
    expect(mapWasmDiagnosticsJsonToVscode("not json")).toEqual([]);
  });

  it("maps severity 1 to Error, 2 to Warning, 3 to Information", () => {
    const json = JSON.stringify([
      { range: { start: { line: 0, character: 0 }, end: { line: 0, character: 5 } }, severity: 1, message: "E" },
      { range: { start: { line: 1, character: 0 }, end: { line: 1, character: 5 } }, severity: 2, message: "W" },
      { range: { start: { line: 2, character: 0 }, end: { line: 2, character: 5 } }, severity: 3, message: "I" },
    ]);
    const diags = mapWasmDiagnosticsJsonToVscode(json);
    expect(diags).toHaveLength(3);
    expect(diags[0].severity).toBe(vscode.DiagnosticSeverity.Error);
    expect(diags[1].severity).toBe(vscode.DiagnosticSeverity.Warning);
    expect(diags[2].severity).toBe(vscode.DiagnosticSeverity.Information);
  });

  it("sets code and source when present", () => {
    const json = JSON.stringify([
      { range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } }, severity: 1, message: "M", code: "E001", source: "sruja" },
    ]);
    const diags = mapWasmDiagnosticsJsonToVscode(json);
    expect(diags[0].code).toBe("E001");
    expect(diags[0].source).toBe("sruja");
  });

  it("defaults source to sruja when missing", () => {
    const json = JSON.stringify([
      { range: { start: { line: 0, character: 0 }, end: { line: 0, character: 1 } }, severity: 1, message: "M" },
    ]);
    const diags = mapWasmDiagnosticsJsonToVscode(json);
    expect(diags[0].source).toBe("sruja");
  });

  it("converts 1-based WASM line/character to 0-based VS Code range", () => {
    const json = JSON.stringify([
      {
        range: { start: { line: 1, character: 1 }, end: { line: 1, character: 5 } },
        severity: 1,
        message: "Err",
      },
    ]);
    const diags = mapWasmDiagnosticsJsonToVscode(json);
    expect(diags[0].range.start.line).toBe(0);
    expect(diags[0].range.start.character).toBe(0);
    expect(diags[0].range.end.line).toBe(0);
    expect(diags[0].range.end.character).toBe(4);
  });
});

describe("wasmRangeToVscodeRange", () => {
  it("converts 1-based line/character to 0-based", () => {
    const range = wasmRangeToVscodeRange({
      start: { line: 1, character: 1 },
      end: { line: 2, character: 10 },
    });
    expect(range.start.line).toBe(0);
    expect(range.start.character).toBe(0);
    expect(range.end.line).toBe(1);
    expect(range.end.character).toBe(9);
  });

  it("clamps 0 to 0 (no negative indices)", () => {
    const range = wasmRangeToVscodeRange({
      start: { line: 0, character: 0 },
      end: { line: 0, character: 3 },
    });
    expect(range.start.line).toBe(0);
    expect(range.start.character).toBe(0);
    expect(range.end.character).toBe(2);
  });
});

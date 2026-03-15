import * as fs from "fs";
import * as os from "os";
import * as path from "path";
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
  initWasm,
  resetWasmForTesting,
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

describe("wasm with fake module", () => {
  const fakeWasmJs = `
    module.exports = async function() {
      return {
        init_panic_hook: function() {},
        sruja_get_diagnostics: function() { return '[]'; },
        sruja_dsl_to_markdown: function(dsl) { return dsl; },
        sruja_dsl_to_mermaid: function() { return 'graph TD'; },
        sruja_get_elements: function() { return '[]'; },
        sruja_get_document_symbols: function() { return '[]'; }
      };
    };
  `;

  it("initWasm loads fake module and get* functions succeed", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-wasm-test-"));
    const wasmDir = path.join(tmpDir, "wasm");
    fs.mkdirSync(wasmDir, { recursive: true });
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm.js"), fakeWasmJs, "utf8");
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm_bg.wasm"), "", "utf8");
    const ctx = new ExtensionContext();
    ctx.extensionPath = tmpDir;
    const context = asContext(ctx);
    resetWasmForTesting();
    try {
      const mod = await initWasm(context);
      expect(mod).not.toBeNull();
      const diags = await getDiagnosticsFromWasm(context, "system A {}", "a.sruja");
      expect(diags).toEqual([]);
      const md = await exportMarkdownFromWasm(context, "system A {}");
      expect(md).toBe("system A {}");
      const mermaid = await getMermaidFromWasm(context, "x");
      expect(mermaid).toBe("graph TD");
      const elements = await getElementsFromWasm(context, "x", "f.sruja");
      expect(elements).toEqual([]);
      const symbols = await getDocumentSymbolsFromWasm(context, "x");
      expect(symbols).toEqual([]);
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it("getDiagnosticsFromWasm returns [] when module throws", async () => {
    const throwJs = `
      module.exports = async function() {
        return {
          init_panic_hook: function() {},
          sruja_get_diagnostics: function() { throw new Error('wasm error'); },
          sruja_dsl_to_markdown: function() { return ''; },
          sruja_dsl_to_mermaid: function() { return ''; },
          sruja_get_elements: function() { return '[]'; },
          sruja_get_document_symbols: function() { return '[]'; }
        };
      };
    `;
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-wasm-throw-"));
    const wasmDir = path.join(tmpDir, "wasm");
    fs.mkdirSync(wasmDir, { recursive: true });
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm.js"), throwJs, "utf8");
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm_bg.wasm"), "", "utf8");
    const ctx = new ExtensionContext();
    ctx.extensionPath = tmpDir;
    const context = asContext(ctx);
    resetWasmForTesting();
    try {
      const diags = await getDiagnosticsFromWasm(context, "x", "f.sruja");
      expect(diags).toEqual([]);
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it("exportMarkdownFromWasm returns null when module throws", async () => {
    const throwMdJs = `
      module.exports = async function() {
        return {
          init_panic_hook: function() {},
          sruja_get_diagnostics: function() { return '[]'; },
          sruja_dsl_to_markdown: function() { throw new Error('export error'); },
          sruja_dsl_to_mermaid: function() { return ''; },
          sruja_get_elements: function() { return '[]'; },
          sruja_get_document_symbols: function() { return '[]'; }
        };
      };
    `;
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-wasm-throw-md-"));
    const wasmDir = path.join(tmpDir, "wasm");
    fs.mkdirSync(wasmDir, { recursive: true });
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm.js"), throwMdJs, "utf8");
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm_bg.wasm"), "", "utf8");
    const ctx = new ExtensionContext();
    ctx.extensionPath = tmpDir;
    const context = asContext(ctx);
    resetWasmForTesting();
    try {
      const md = await exportMarkdownFromWasm(context, "x");
      expect(md).toBeNull();
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it("initWasm returns null when init throws", async () => {
    const throwInitJs = `module.exports = async function() { throw new Error('init failed'); };`;
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-wasm-init-throw-"));
    const wasmDir = path.join(tmpDir, "wasm");
    fs.mkdirSync(wasmDir, { recursive: true });
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm.js"), throwInitJs, "utf8");
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm_bg.wasm"), "", "utf8");
    const ctx = new ExtensionContext();
    ctx.extensionPath = tmpDir;
    const context = asContext(ctx);
    resetWasmForTesting();
    try {
      const mod = await initWasm(context);
      expect(mod).toBeNull();
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it("getElementsFromWasm returns null when module returns invalid JSON", async () => {
    const badJsonJs = `
      module.exports = async function() {
        return {
          init_panic_hook: function() {},
          sruja_get_diagnostics: function() { return '[]'; },
          sruja_dsl_to_markdown: function(dsl) { return dsl; },
          sruja_dsl_to_mermaid: function() { return ''; },
          sruja_get_elements: function() { return 'not json'; },
          sruja_get_document_symbols: function() { return '[]'; }
        };
      };
    `;
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-wasm-badjson-"));
    const wasmDir = path.join(tmpDir, "wasm");
    fs.mkdirSync(wasmDir, { recursive: true });
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm.js"), badJsonJs, "utf8");
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm_bg.wasm"), "", "utf8");
    const ctx = new ExtensionContext();
    ctx.extensionPath = tmpDir;
    const context = asContext(ctx);
    resetWasmForTesting();
    try {
      const elements = await getElementsFromWasm(context, "x");
      expect(elements).toBeNull();
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it("getMermaidFromWasm passes configJson", async () => {
    const withConfigJs = `
      module.exports = async function() {
        return {
          init_panic_hook: function() {},
          sruja_get_diagnostics: function() { return '[]'; },
          sruja_dsl_to_markdown: function(dsl) { return dsl; },
          sruja_dsl_to_mermaid: function(dsl, config) { return config ? 'graph LR' : 'graph TD'; },
          sruja_get_elements: function() { return '[]'; },
          sruja_get_document_symbols: function() { return '[]'; }
        };
      };
    `;
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-wasm-config-"));
    const wasmDir = path.join(tmpDir, "wasm");
    fs.mkdirSync(wasmDir, { recursive: true });
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm.js"), withConfigJs, "utf8");
    fs.writeFileSync(path.join(wasmDir, "sruja_wasm_bg.wasm"), "", "utf8");
    const ctx = new ExtensionContext();
    ctx.extensionPath = tmpDir;
    const context = asContext(ctx);
    resetWasmForTesting();
    try {
      const mermaid = await getMermaidFromWasm(context, "x", "{}");
      expect(mermaid).toBe("graph LR");
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });
});

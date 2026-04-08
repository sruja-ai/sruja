import * as vscode from "vscode";
import { ExtensionContext, CancellationToken } from "./__mocks__/vscode";
import {
  SrujaCodeActionProvider,
  SrujaCompletionItemProvider,
  SrujaDiagramCodeLensProvider,
  SrujaDocumentFormattingEditProvider,
} from "./providers";

jest.mock("./wasm", () => {
  const actual = jest.requireActual<typeof import("./wasm")>("./wasm");
  return {
    ...actual,
    getElementsFromWasm: jest.fn(),
  };
});

import { getElementsFromWasm, type SrujaElement } from "./wasm";

function asContext(ctx: ExtensionContext): vscode.ExtensionContext {
  return ctx as unknown as vscode.ExtensionContext;
}

function makeDocument(lines: string[], uri = vscode.Uri.file("/ws/a.sruja")): vscode.TextDocument {
  return {
    languageId: "sruja",
    uri,
    version: 1,
    lineCount: lines.length,
    getText: (range?: vscode.Range) => {
      if (!range) return lines.join("\n");
      const line = lines[range.start.line] ?? "";
      return line.slice(range.start.character, range.end.character);
    },
    lineAt: (lineOrPos: number | vscode.Position) => {
      const line = typeof lineOrPos === "number" ? lineOrPos : lineOrPos.line;
      const text = lines[line] ?? "";
      const start = new vscode.Position(line, 0);
      const end = new vscode.Position(line, text.length);
      return { text, range: new vscode.Range(start, end) } as any;
    },
    getWordRangeAtPosition: () => undefined,
  } as unknown as vscode.TextDocument;
}

describe("providers (extra coverage)", () => {
  beforeEach(() => {
    (getElementsFromWasm as unknown as jest.Mock).mockReset();
  });

  it("provides diagram and sequence codelenses", async () => {
    const doc = makeDocument([
      `MySystem = system "My System" {`,
      `  Api = container "API" {`,
      `  }`,
      `}`,
      `scenario Login {`,
      `}`,
      `FlowA = flow "F" {`,
      `}`,
    ]);

    const provider = new SrujaDiagramCodeLensProvider(asContext(new ExtensionContext()));
    const lenses = await provider.provideCodeLenses(doc, new CancellationToken() as any);

    const commands = lenses.map((l: any) => l.command?.command).filter(Boolean);
    expect(commands).toContain("sruja.openDiagramPreview");
    expect(commands).toContain("sruja.openFocusedDiagramPreviewAt");
    expect(commands).toContain("sruja.openSequenceDiagramPreviewAt");
  });

  it("builds missing-field quick fix for E302", async () => {
    (getElementsFromWasm as unknown as jest.Mock).mockResolvedValue([]);

    const doc = makeDocument([`MySystem = system "My System" {`, `}`], vscode.Uri.file("/ws/missing.sruja"));
    const diag = new vscode.Diagnostic(new vscode.Range(0, 0, 0, 0), "Missing required fields `description`", vscode.DiagnosticSeverity.Error);
    (diag as any).code = "E302";

    const provider = new SrujaCodeActionProvider(asContext(new ExtensionContext()));
    const actions = await provider.provideCodeActions(
      doc,
      new vscode.Range(0, 0, 0, 0),
      { diagnostics: [diag] } as any,
      new CancellationToken() as any
    );

    const add = actions.find((a) => a.title.includes("Add description"));
    expect(add).toBeDefined();
    const ops = (add!.edit as any).operations;
    expect(ops).toHaveLength(1);
    expect(ops[0].type).toBe("insert");
    expect(ops[0].text).toContain(`description "..."`);
  });

  it("builds spelling correction quick fixes for E202", async () => {
    const elements: SrujaElement[] = [
      { id: "App", kind: "system", title: "Application", range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } } },
    ];
    (getElementsFromWasm as unknown as jest.Mock).mockResolvedValue(elements);

    const lines = [`Appp -> App "uses"`];
    const doc = makeDocument(lines, vscode.Uri.file("/ws/spell.sruja"));
    (doc as any).getText = (range?: vscode.Range) => {
      if (!range) return lines.join("\n");
      return "Appp";
    };

    const diag = new vscode.Diagnostic(new vscode.Range(0, 0, 0, 4), "Unknown element", vscode.DiagnosticSeverity.Error);
    (diag as any).code = "E202";

    const provider = new SrujaCodeActionProvider(asContext(new ExtensionContext()));
    const actions = await provider.provideCodeActions(
      doc,
      new vscode.Range(0, 0, 0, 0),
      { diagnostics: [diag] } as any,
      new CancellationToken() as any
    );

    const replace = actions.find((a) => a.title.includes('Replace with "App"'));
    expect(replace).toBeDefined();
    const ops = (replace!.edit as any).operations;
    expect(ops[0].type).toBe("replace");
    expect(ops[0].text).toBe("App");
  });

  it("returns keyword and element-id completion items", async () => {
    const elements: SrujaElement[] = [
      { id: "App", kind: "system", title: "Application", range: { start: { line: 0, character: 0 }, end: { line: 0, character: 0 } } },
    ];
    (getElementsFromWasm as unknown as jest.Mock).mockResolvedValue(elements);

    const lines = [`User -> `];
    const doc = makeDocument(lines);

    const provider = new SrujaCompletionItemProvider(asContext(new ExtensionContext()));
    const items = await provider.provideCompletionItems(doc, new vscode.Position(0, 7), new CancellationToken() as any);

    const labels = (items ?? []).map((i: any) => i.label);
    expect(labels).toContain("system");
    expect(labels).toContain("App");
  });

  it("formats indentation and spacing for -> and =", () => {
    const lines = [
      `MySystem=system "X" {`,
      `A->B "uses"`,
      `}`,
    ];
    const doc = makeDocument(lines);
    const provider = new SrujaDocumentFormattingEditProvider();

    const edits = provider.provideDocumentFormattingEdits(doc, {} as any, new CancellationToken() as any);
    expect(edits.length).toBeGreaterThan(0);
    const newTexts = edits.map((e: any) => e.newText);
    expect(newTexts.join("\n")).toContain("MySystem = system");
    expect(newTexts.join("\n")).toContain("A -> B");
  });
});

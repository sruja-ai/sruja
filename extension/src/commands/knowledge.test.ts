import * as vscode from "vscode";
import { ExtensionContext } from "../__mocks__/vscode";

jest.mock("../providers", () => {
  return {
    resolveDocUri: jest.fn(),
    docUriExists: jest.fn(),
  };
});

jest.mock("../wasm", () => {
  const actual = jest.requireActual<typeof import("../wasm")>("../wasm");
  return {
    ...actual,
    getElementsFromWasm: jest.fn(),
    wasmRangeToVscodeRange: (r: any) => new vscode.Range(r.start.line, r.start.character, r.end.line, r.end.character),
  };
});

import { docsThreadState } from "../docsThread";
import { createDocsThreadPanel, pushDocsThreadEntryForElement, registerKnowledgeCommands } from "./knowledge";

describe("knowledge commands", () => {
  let registered: Map<string, (...args: any[]) => any>;

  beforeEach(() => {
    registered = new Map();
    (vscode.commands as any).registerCommand = jest.fn((id: string, cb: (...args: any[]) => any) => {
      registered.set(id, cb);
      return { dispose: () => {} };
    });
    (vscode.commands as any).executeCommand = jest.fn();
    (vscode.window as any).showWarningMessage = jest.fn();
    (vscode.window as any).showInformationMessage = jest.fn();
    (vscode.window as any).showErrorMessage = jest.fn();
    (vscode.window as any).activeTextEditor = undefined;

    const { window } = jest.requireActual("../__mocks__/vscode");
    (vscode.window as any).createWebviewPanel = jest.fn(window.createWebviewPanel);

    (vscode.workspace as any).openTextDocument = jest.fn(async (uri: vscode.Uri) => ({
      uri,
      languageId: "markdown",
      version: 1,
      lineCount: 1,
      getText: () => "",
      lineAt: () => ({ text: "" }),
      getWordRangeAtPosition: () => undefined,
    }));

    (vscode.window as any).showTextDocument = jest.fn(async (_doc: any, _opts?: any) => ({
      selection: undefined,
      revealRange: jest.fn(),
    }));

    (vscode.workspace as any).fs = {
      readFile: jest.fn().mockResolvedValue(Buffer.from("hello\nworld\n", "utf8")),
      writeFile: jest.fn(),
      stat: jest.fn(),
      createDirectory: jest.fn(),
    };

    docsThreadState.followCursor = true;
    docsThreadState.entries = [];
    docsThreadState.lastPushedKey = "";
  });

  it("openComponentKnowledge opens markdown preview when docUriArg provided", async () => {
    registerKnowledgeCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.openComponentKnowledge");
    if (!cb) throw new Error("Command not registered: sruja.openComponentKnowledge");

    await cb("file:///ws/docs/payments.md");
    expect(vscode.commands.executeCommand).toHaveBeenCalledWith(
      "markdown.showPreviewToSide",
      vscode.Uri.parse("file:///ws/docs/payments.md")
    );
  });

  it("openComponentKnowledge warns when active editor is not a .sruja file", async () => {
    (vscode.window as any).activeTextEditor = { document: { languageId: "plaintext" } };
    registerKnowledgeCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.openComponentKnowledge");
    if (!cb) throw new Error("Command not registered: sruja.openComponentKnowledge");

    await cb();
    expect(vscode.window.showWarningMessage).toHaveBeenCalledWith(
      "Place the cursor on an element in a .sruja file, or use the hover link to open documentation."
    );
  });

  it("openComponentKnowledge resolves doc from element and opens it", async () => {
    const { getElementsFromWasm } = await import("../wasm");
    const { resolveDocUri, docUriExists } = await import("../providers");

    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      {
        id: "Payments.Api",
        kind: "component",
        title: "API",
        doc: "docs/payments.md",
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 10 } },
      },
    ]);
    (resolveDocUri as jest.Mock).mockReturnValue(vscode.Uri.file("/ws/docs/payments.md"));
    (docUriExists as jest.Mock).mockResolvedValue(true);

    const doc: any = {
      languageId: "sruja",
      uri: vscode.Uri.file("/ws/arch.sruja"),
      version: 1,
      getText: (r?: any) => (r ? "Payments.Api" : "dsl"),
      getWordRangeAtPosition: () => new vscode.Range(0, 0, 0, 11),
    };
    (vscode.window as any).activeTextEditor = {
      document: doc,
      selection: { active: new vscode.Position(0, 1) },
    };

    registerKnowledgeCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.openComponentKnowledge");
    if (!cb) throw new Error("Command not registered: sruja.openComponentKnowledge");

    await cb();
    expect(vscode.commands.executeCommand).toHaveBeenCalledWith(
      "markdown.showPreviewToSide",
      vscode.Uri.file("/ws/docs/payments.md")
    );
  });

  it("createDocsThreadPanel wires message handling for toggleFollow/clear/pop", async () => {
    const panel = createDocsThreadPanel(new ExtensionContext() as any);
    expect(panel.webview.html).toContain("Sruja Docs & References");

    docsThreadState.followCursor = true;
    (panel as any).__onDidReceiveMessage({ type: "toggleFollow" });
    expect(docsThreadState.followCursor).toBe(false);

    docsThreadState.entries.push({
      key: "k",
      sourceUri: "file:///ws/a.sruja",
      elementId: "X",
      kind: "system",
      title: null,
      parentId: null,
      range: { startLine: 0, startCharacter: 0, endLine: 0, endCharacter: 1 },
      doc: null,
      refs: [],
      createdAtMs: Date.now(),
    });
    docsThreadState.lastPushedKey = "k";

    await (panel as any).__onDidReceiveMessage({ type: "pop" });
    expect(docsThreadState.entries).toHaveLength(0);

    docsThreadState.entries.push({
      key: "k2",
      sourceUri: "file:///ws/a.sruja",
      elementId: "Y",
      kind: "system",
      title: null,
      parentId: null,
      range: { startLine: 0, startCharacter: 0, endLine: 0, endCharacter: 1 },
      doc: null,
      refs: [],
      createdAtMs: Date.now(),
    });
    docsThreadState.lastPushedKey = "k2";

    await (panel as any).__onDidReceiveMessage({ type: "clear" });
    expect(docsThreadState.entries).toHaveLength(0);
    expect(docsThreadState.lastPushedKey).toBe("");
  });

  it("pushDocsThreadEntryForElement adds entry with refs and doc preview", async () => {
    createDocsThreadPanel(new ExtensionContext() as any);

    const { resolveDocUri, docUriExists } = await import("../providers");
    (resolveDocUri as jest.Mock).mockReturnValue(vscode.Uri.file("/ws/docs/payments.md"));
    (docUriExists as jest.Mock).mockResolvedValue(true);

    (vscode.commands as any).executeCommand = jest.fn().mockResolvedValue([
      new vscode.Location(vscode.Uri.file("/ws/docs/payments.md"), new vscode.Range(2, 0, 2, 5)),
    ]);

    const doc: any = {
      uri: vscode.Uri.file("/ws/arch.sruja"),
      languageId: "sruja",
      lineCount: 10,
      lineAt: (i: number) => ({ text: `line${i}` }),
    };
    const element = {
      id: "Payments.Api",
      kind: "component",
      title: "API",
      doc: "docs/payments.md",
      range: { start: { line: 1, character: 0 }, end: { line: 1, character: 12 } },
    };

    await pushDocsThreadEntryForElement(new ExtensionContext() as any, doc, element as any);
    expect(docsThreadState.entries).toHaveLength(1);
    expect(docsThreadState.entries[0].refs).toHaveLength(1);
    expect(docsThreadState.entries[0].doc?.previewText).toContain("hello");
  });
});


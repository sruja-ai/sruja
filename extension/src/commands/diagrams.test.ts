import * as vscode from "vscode";
import { ExtensionContext } from "../__mocks__/vscode";

jest.mock("../wasm", () => {
  const actual = jest.requireActual<typeof import("../wasm")>("../wasm");
  return {
    ...actual,
    getMermaidFromWasm: jest.fn(),
    getElementsFromWasm: jest.fn(),
    getSequenceDiagramFromWasm: jest.fn(),
    getDocumentSymbolsFromWasm: jest.fn(),
    wasmRangeToVscodeRange: (r: any) => new vscode.Range(r.start.line, r.start.character, r.end.line, r.end.character),
  };
});

import { registerDiagramCommands } from "./diagrams";

describe("registerDiagramCommands", () => {
  let registered: Map<string, (...args: any[]) => any>;

  beforeEach(() => {
    registered = new Map();
    (vscode.commands as any).registerCommand = jest.fn((id: string, cb: (...args: any[]) => any) => {
      registered.set(id, cb);
      return { dispose: () => {} };
    });
    (vscode.commands as any).executeCommand = jest.fn();
    (vscode.window as any).showWarningMessage = jest.fn();
    (vscode.window as any).showErrorMessage = jest.fn();
    (vscode.window as any).showInformationMessage = jest.fn();
    (vscode.window as any).showQuickPick = jest.fn();
    (vscode.window as any).activeTextEditor = undefined;
    (vscode.workspace as any).openTextDocument = jest.fn();
    const { window } = jest.requireActual("../__mocks__/vscode");
    (vscode.window as any).createWebviewPanel = jest.fn(window.createWebviewPanel);
  });

  it("openDiagramPreview warns when no active .sruja file", async () => {
    registerDiagramCommands(new ExtensionContext() as any, true);
    const cb = registered.get("sruja.openDiagramPreview");
    if (!cb) throw new Error("Command not registered: sruja.openDiagramPreview");
    await cb();
    expect(vscode.window.showWarningMessage).toHaveBeenCalledWith("Open a .sruja file to open diagram preview.");
  });

  it("openDiagramPreview renders diagram when wasm returns mermaid", async () => {
    const { getMermaidFromWasm } = await import("../wasm");
    (getMermaidFromWasm as jest.Mock).mockResolvedValue("graph TD\nA-->B");

    (vscode.window as any).activeTextEditor = {
      document: {
        languageId: "sruja",
        getText: () => "dsl",
        uri: vscode.Uri.file("/ws/a.sruja"),
        version: 1,
      },
    };

    registerDiagramCommands(new ExtensionContext() as any, true);
    const cb = registered.get("sruja.openDiagramPreview");
    if (!cb) throw new Error("Command not registered: sruja.openDiagramPreview");
    await cb();

    const panel = (vscode.window as any).createWebviewPanel.mock.results[0].value;
    expect(panel.webview.html).toContain("graph TD");
  });

  it("openFocusedDiagramPreview uses quick picks and renders", async () => {
    const { getMermaidFromWasm, getElementsFromWasm } = await import("../wasm");
    (getMermaidFromWasm as jest.Mock).mockResolvedValue("graph TD\nA-->B");
    (getElementsFromWasm as jest.Mock).mockResolvedValue([{ id: "Payments", kind: "system", title: null }]);

    (vscode.window as any).showQuickPick
      .mockResolvedValueOnce({ label: "L2 (Container)", level: 2 })
      .mockResolvedValueOnce({ label: "Payments", description: "system", id: "Payments" });

    (vscode.window as any).activeTextEditor = {
      document: {
        languageId: "sruja",
        getText: () => "dsl",
        uri: vscode.Uri.file("/ws/a.sruja"),
        version: 1,
      },
    };

    registerDiagramCommands(new ExtensionContext() as any, true);
    const cb = registered.get("sruja.openFocusedDiagramPreview");
    if (!cb) throw new Error("Command not registered: sruja.openFocusedDiagramPreview");
    await cb();

    const panel = (vscode.window as any).createWebviewPanel.mock.results[0].value;
    expect(panel.webview.html).toContain("graph TD");
  });

  it("openSequenceDiagramPreview executes openSequenceDiagramPreviewAt with picked id", async () => {
    const { getDocumentSymbolsFromWasm } = await import("../wasm");
    (getDocumentSymbolsFromWasm as jest.Mock).mockResolvedValue([{ kind: "scenario", name: "Checkout" }]);

    (vscode.window as any).activeTextEditor = {
      document: {
        languageId: "sruja",
        getText: () => "dsl",
        uri: vscode.Uri.file("/ws/a.sruja"),
        version: 1,
        fsPath: "/ws/a.sruja",
      },
    };

    registerDiagramCommands(new ExtensionContext() as any, true);
    const cb = registered.get("sruja.openSequenceDiagramPreview");
    if (!cb) throw new Error("Command not registered: sruja.openSequenceDiagramPreview");
    await cb();

    expect(vscode.commands.executeCommand).toHaveBeenCalledWith("sruja.openSequenceDiagramPreviewAt", {
      docUri: "file:///ws/a.sruja",
      kind: "scenario",
      id: "Checkout",
    });
  });

  it("openSequenceDiagramPreviewAt renders sequence diagram when wasm returns mermaid", async () => {
    const { getSequenceDiagramFromWasm } = await import("../wasm");
    (getSequenceDiagramFromWasm as jest.Mock).mockResolvedValue("sequenceDiagram\nA->>B: Hi");

    (vscode.workspace as any).openTextDocument.mockResolvedValue({
      uri: vscode.Uri.file("/ws/a.sruja"),
      languageId: "sruja",
      version: 1,
      getText: () => "dsl",
    });

    registerDiagramCommands(new ExtensionContext() as any, true);
    const cb = registered.get("sruja.openSequenceDiagramPreviewAt");
    if (!cb) throw new Error("Command not registered: sruja.openSequenceDiagramPreviewAt");
    await cb({ docUri: "file:///ws/a.sruja", kind: "scenario", id: "Checkout" });

    const panel = (vscode.window as any).createWebviewPanel.mock.results[0].value;
    expect(panel.webview.html).toContain("sequenceDiagram");
  });
});

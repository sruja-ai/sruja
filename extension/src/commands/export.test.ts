import * as vscode from "vscode";
import { ExtensionContext } from "../__mocks__/vscode";

jest.mock("../wasm", () => {
  const actual = jest.requireActual<typeof import("../wasm")>("../wasm");
  return {
    ...actual,
    getMermaidFromWasm: jest.fn(),
  };
});

import { registerExportCommands } from "./export";

describe("registerExportCommands", () => {
  let registered: Map<string, () => Promise<void> | void>;

  beforeEach(() => {
    registered = new Map<string, () => Promise<void> | void>();
    (vscode.commands as any).registerCommand = jest.fn((id: string, cb: () => Promise<void> | void) => {
      registered.set(id, cb);
      return { dispose: () => {} };
    });

    (vscode.window as any).showWarningMessage = jest.fn();
    (vscode.window as any).showErrorMessage = jest.fn();
    (vscode.window as any).showTextDocument = jest.fn();
    (vscode.window as any).withProgress = jest.fn(async (_opts: any, task: any) => task({ report: () => {} }));
    (vscode.window as any).activeTextEditor = undefined;

    (vscode.workspace as any).openTextDocument = jest.fn(async (arg: any) => {
      return {
        uri: vscode.Uri.file("/untitled.md"),
        languageId: String(arg?.language ?? "plaintext"),
        version: 1,
        getText: () => String(arg?.content ?? ""),
      };
    });
  });

  it("warns when there is no active .sruja editor", async () => {
    registerExportCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.exportMarkdown");
    if (!cb) throw new Error("Command not registered: sruja.exportMarkdown");

    await cb();

    expect(vscode.window.showWarningMessage).toHaveBeenCalledWith("Open a .sruja file to export markdown.");
  });

  it("shows an error when wasm returns null", async () => {
    const { getMermaidFromWasm } = await import("../wasm");
    (getMermaidFromWasm as jest.Mock).mockResolvedValue(null);

    (vscode.window as any).activeTextEditor = {
      document: {
        languageId: "sruja",
        uri: vscode.Uri.file("/ws/a.sruja"),
        getText: () => "dsl",
      },
    };

    registerExportCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.exportMarkdown");
    if (!cb) throw new Error("Command not registered: sruja.exportMarkdown");

    await cb();

    expect(vscode.window.showErrorMessage).toHaveBeenCalledWith("Failed to generate architecture export.");
  });

  it("opens an untitled markdown document when wasm returns mermaid", async () => {
    const { getMermaidFromWasm } = await import("../wasm");
    (getMermaidFromWasm as jest.Mock).mockResolvedValue("graph TD\nA-->B");

    (vscode.window as any).activeTextEditor = {
      document: {
        languageId: "sruja",
        uri: vscode.Uri.file("/ws/a.sruja"),
        getText: () => "dsl",
      },
    };

    registerExportCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.exportMarkdown");
    if (!cb) throw new Error("Command not registered: sruja.exportMarkdown");

    await cb();

    const openArgs = (vscode.workspace as any).openTextDocument.mock.calls[0][0];
    expect(openArgs.language).toBe("markdown");
    expect(openArgs.content).toContain("```mermaid");
    expect(openArgs.content).toContain("graph TD");

    expect(vscode.window.showTextDocument).toHaveBeenCalledWith(expect.any(Object), vscode.ViewColumn.Beside);
  });

  it("shows an error when wasm throws", async () => {
    const { getMermaidFromWasm } = await import("../wasm");
    (getMermaidFromWasm as jest.Mock).mockRejectedValue(new Error("boom"));

    (vscode.window as any).activeTextEditor = {
      document: {
        languageId: "sruja",
        uri: vscode.Uri.file("/ws/a.sruja"),
        getText: () => "dsl",
      },
    };

    registerExportCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.exportMarkdown");
    if (!cb) throw new Error("Command not registered: sruja.exportMarkdown");

    await cb();

    expect(vscode.window.showErrorMessage).toHaveBeenCalledWith("Export failed: boom");
  });
});

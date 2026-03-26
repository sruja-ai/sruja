import * as vscode from "vscode";
import { ExtensionContext } from "../__mocks__/vscode";

import { registerValidationCommands } from "./validation";

describe("registerValidationCommands", () => {
  let registered: Map<string, () => Promise<void> | void>;

  beforeEach(() => {
    registered = new Map<string, () => Promise<void> | void>();
    (vscode.commands as any).registerCommand = jest.fn((id: string, cb: () => Promise<void> | void) => {
      registered.set(id, cb);
      return { dispose: () => {} };
    });

    (vscode.window as any).showWarningMessage = jest.fn();
    (vscode.window as any).withProgress = jest.fn(async (_opts: any, task: any) => task({ report: () => {} }));
    (vscode.window as any).activeTextEditor = undefined;
  });

  it("warns when there is no active .sruja editor", async () => {
    const updateDiagnostics = jest.fn();
    registerValidationCommands(new ExtensionContext() as any, updateDiagnostics as any);

    const cb = registered.get("sruja.runValidation");
    if (!cb) throw new Error("Command not registered: sruja.runValidation");
    await cb();

    expect(vscode.window.showWarningMessage).toHaveBeenCalledWith("Open a .sruja file to run validation.");
    expect(updateDiagnostics).not.toHaveBeenCalled();
  });

  it("calls updateDiagnostics for the active sruja document", async () => {
    const updateDiagnostics = jest.fn();
    const doc: any = {
      languageId: "sruja",
      uri: vscode.Uri.file("/ws/a.sruja"),
      getText: () => "dsl",
    };
    (vscode.window as any).activeTextEditor = { document: doc };

    const ctx = new ExtensionContext();
    registerValidationCommands(ctx as any, updateDiagnostics as any);

    const cb = registered.get("sruja.runValidation");
    if (!cb) throw new Error("Command not registered: sruja.runValidation");
    await cb();

    expect(updateDiagnostics).toHaveBeenCalledWith(ctx, doc);
  });
});

import * as vscode from "vscode";
import { ExtensionContext } from "./__mocks__/vscode";

jest.mock("./wasm", () => {
  const actual = jest.requireActual<typeof import("./wasm")>("./wasm");
  return {
    ...actual,
    getDiagnosticsFromWasm: jest.fn(),
  };
});

import { getDiagnosticsFromWasm } from "./wasm";
import { updateDiagnostics } from "./diagnostics";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("updateDiagnostics", () => {
  const set = jest.fn();

  beforeEach(() => {
    set.mockReset();
    (vscode.languages as any).createDiagnosticCollection = jest.fn(() => ({ set }));
    (vscode.workspace as any).textDocuments = [];
    (getDiagnosticsFromWasm as unknown as jest.Mock).mockReset();
  });

  it("sets diagnostics from wasm when document is open", async () => {
    const uri = vscode.Uri.file("/ws/a.sruja");
    (vscode.workspace as any).textDocuments = [{ uri }];

    const diags = [
      new vscode.Diagnostic(new vscode.Range(0, 0, 0, 1), "m", vscode.DiagnosticSeverity.Error),
    ];
    (getDiagnosticsFromWasm as unknown as jest.Mock).mockResolvedValue(diags);

    const doc = {
      languageId: "sruja",
      uri,
      getText: () => "dsl",
    } as unknown as vscode.TextDocument;

    await updateDiagnostics(new ExtensionContext() as any, doc);
    expect(set).toHaveBeenCalledWith(uri, diags);
  });

  it("does not set diagnostics if document closes before wasm resolves", async () => {
    const uri = vscode.Uri.file("/ws/a.sruja");
    (vscode.workspace as any).textDocuments = [{ uri }];

    const d = deferred<vscode.Diagnostic[]>();
    (getDiagnosticsFromWasm as unknown as jest.Mock).mockReturnValue(d.promise);

    const doc = {
      languageId: "sruja",
      uri,
      getText: () => "dsl",
    } as unknown as vscode.TextDocument;

    const p = updateDiagnostics(new ExtensionContext() as any, doc);
    (vscode.workspace as any).textDocuments = [];
    d.resolve([]);
    await p;

    expect(set).not.toHaveBeenCalled();
  });

  it("sets a warning diagnostic when wasm throws", async () => {
    const uri = vscode.Uri.file("/ws/a.sruja");
    (vscode.workspace as any).textDocuments = [{ uri }];
    (getDiagnosticsFromWasm as unknown as jest.Mock).mockRejectedValue(new Error("boom"));

    const doc = {
      languageId: "sruja",
      uri,
      getText: () => "dsl",
    } as unknown as vscode.TextDocument;

    await updateDiagnostics(new ExtensionContext() as any, doc);

    const args = set.mock.calls[0];
    expect(args[0]).toBe(uri);
    expect(args[1][0].message).toContain("Sruja lint failed: boom");
    expect(args[1][0].severity).toBe(vscode.DiagnosticSeverity.Warning);
  });

  it("does nothing for non-file or non-sruja documents", async () => {
    const uri = { scheme: "untitled", fsPath: "/ws/a.sruja", toString: () => "untitled:/ws/a.sruja" } as any;
    const doc = {
      languageId: "plaintext",
      uri,
      getText: () => "dsl",
    } as unknown as vscode.TextDocument;

    await updateDiagnostics(new ExtensionContext() as any, doc);
    expect(set).not.toHaveBeenCalled();
  });
});

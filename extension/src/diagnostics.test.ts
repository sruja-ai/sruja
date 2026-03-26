import { ExtensionContext } from "./__mocks__/vscode";

jest.mock("./wasm", () => {
  const actual = jest.requireActual<typeof import("./wasm")>("./wasm");
  return {
    ...actual,
    getDiagnosticsFromWasm: jest.fn(),
  };
});

describe("diagnostics", () => {
  let vscode: typeof import("vscode");

  beforeEach(async () => {
    jest.resetModules();

    vscode = await import("vscode");
    (vscode.workspace as any).textDocuments = [];
    (vscode.languages as any).createDiagnosticCollection = jest.fn(() => ({
      set: jest.fn(),
    }));
  });

  it("skips non-sruja docs", async () => {
    const { getDiagnosticsFromWasm } = await import("./wasm");
    const { updateDiagnostics } = await import("./diagnostics");

    const doc: any = {
      languageId: "plaintext",
      uri: vscode.Uri.file("/ws/a.txt"),
      getText: () => "x",
    };

    await updateDiagnostics(new ExtensionContext() as any, doc);
    expect(getDiagnosticsFromWasm).not.toHaveBeenCalled();
    expect((vscode.languages as any).createDiagnosticCollection).not.toHaveBeenCalled();
  });

  it("sets diagnostics when wasm returns results and doc is open", async () => {
    const { getDiagnosticsFromWasm } = await import("./wasm");
    (getDiagnosticsFromWasm as jest.Mock).mockResolvedValue([
      new vscode.Diagnostic(new vscode.Range(0, 0, 0, 1), "msg", vscode.DiagnosticSeverity.Warning),
    ]);

    const { updateDiagnostics } = await import("./diagnostics");

    const uri = vscode.Uri.file("/ws/a.sruja");
    const doc: any = {
      languageId: "sruja",
      uri,
      getText: () => "dsl",
    };
    (vscode.workspace as any).textDocuments = [{ uri }];

    await updateDiagnostics(new ExtensionContext() as any, doc);

    const collection = (vscode.languages as any).createDiagnosticCollection.mock.results[0].value;
    expect(collection.set).toHaveBeenCalledWith(uri, expect.any(Array));
  });

  it("emits a warning diagnostic when wasm throws", async () => {
    const { getDiagnosticsFromWasm } = await import("./wasm");
    (getDiagnosticsFromWasm as jest.Mock).mockRejectedValue(new Error("boom"));

    const { updateDiagnostics } = await import("./diagnostics");

    const uri = vscode.Uri.file("/ws/a.sruja");
    const doc: any = {
      languageId: "sruja",
      uri,
      getText: () => "dsl",
    };
    (vscode.workspace as any).textDocuments = [{ uri }];

    await updateDiagnostics(new ExtensionContext() as any, doc);

    const collection = (vscode.languages as any).createDiagnosticCollection.mock.results[0].value;
    const diags = collection.set.mock.calls[0][1];
    expect(diags[0].message).toContain("Sruja lint failed: boom");
    expect(diags[0].severity).toBe(vscode.DiagnosticSeverity.Warning);
  });

  it("does not set diagnostics when the doc is closed before wasm resolves", async () => {
    const { getDiagnosticsFromWasm } = await import("./wasm");
    let resolve: (value: any) => void = () => {};
    (getDiagnosticsFromWasm as jest.Mock).mockImplementation(
      () =>
        new Promise((r) => {
          resolve = r;
        })
    );

    const { updateDiagnostics } = await import("./diagnostics");

    const uri = vscode.Uri.file("/ws/a.sruja");
    const doc: any = {
      languageId: "sruja",
      uri,
      getText: () => "dsl",
    };

    (vscode.workspace as any).textDocuments = [{ uri }];
    const promise = updateDiagnostics(new ExtensionContext() as any, doc);
    (vscode.workspace as any).textDocuments = [];
    resolve([]);
    await promise;

    const collection = (vscode.languages as any).createDiagnosticCollection.mock.results[0].value;
    expect(collection.set).not.toHaveBeenCalled();
  });
});

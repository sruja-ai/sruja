import * as vscode from "vscode";
import { ExtensionContext, CancellationToken } from "./__mocks__/vscode";

jest.mock("./wasm", () => {
  const actual = jest.requireActual<typeof import("./wasm")>("./wasm");
  return {
    ...actual,
    getElementsFromWasm: jest.fn(),
    wasmRangeToVscodeRange: (r: any) => new vscode.Range(r.start.line, r.start.character, r.end.line, r.end.character),
  };
});

import { docUriExists, resolveDocUri, SrujaDefinitionProvider } from "./providers";

function asContext(ctx: ExtensionContext): vscode.ExtensionContext {
  return ctx as unknown as vscode.ExtensionContext;
}

describe("providers (docs)", () => {
  beforeEach(() => {
    (vscode.workspace as any).getWorkspaceFolder = () => undefined;
    (vscode.workspace as any).fs = {
      stat: jest.fn().mockResolvedValue({ size: 1, mtime: 0 }),
      readFile: jest.fn(),
      writeFile: jest.fn(),
      createDirectory: jest.fn(),
    };
  });

  it("resolveDocUri returns undefined for empty path", () => {
    const doc = { uri: vscode.Uri.file("/ws/a.sruja") } as vscode.TextDocument;
    expect(resolveDocUri("", doc)).toBeUndefined();
    expect(resolveDocUri("   ", doc)).toBeUndefined();
    expect(resolveDocUri(undefined, doc)).toBeUndefined();
  });

  it("resolveDocUri resolves within workspace and blocks traversal", () => {
    (vscode.workspace as any).getWorkspaceFolder = () => ({ uri: vscode.Uri.file("/ws"), name: "ws" });
    const doc = { uri: vscode.Uri.file("/ws/a.sruja") } as vscode.TextDocument;

    expect(resolveDocUri("docs/payments.md", doc)?.fsPath).toBe("/ws/docs/payments.md");
    expect(resolveDocUri("../secrets.md", doc)).toBeUndefined();
    expect(resolveDocUri("/abs/outside.md", doc)).toBeUndefined();
  });

  it("docUriExists returns true/false based on stat", async () => {
    const uri = vscode.Uri.file("/ws/docs/payments.md");
    await expect(docUriExists(uri)).resolves.toBe(true);
    (vscode.workspace as any).fs.stat = jest.fn().mockRejectedValue(new Error("nope"));
    await expect(docUriExists(uri)).resolves.toBe(false);
  });

  it("definition provider includes doc target when element has doc and it exists", async () => {
    const { getElementsFromWasm } = await import("./wasm");

    (vscode.workspace as any).getWorkspaceFolder = () => ({ uri: vscode.Uri.file("/ws"), name: "ws" });
    (vscode.workspace as any).fs.stat = jest.fn().mockResolvedValue({ size: 1, mtime: 0 });

    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      {
        id: "Payments.Api",
        kind: "component",
        title: "API",
        doc: "docs/payments.md",
        range: { start: { line: 0, character: 0 }, end: { line: 0, character: 10 } },
      },
    ]);

    const doc = {
      languageId: "sruja",
      uri: vscode.Uri.file("/ws/arch.sruja"),
      version: 1,
      getText: (r?: vscode.Range) => (r ? "Payments.Api" : "dsl"),
      getWordRangeAtPosition: () => new vscode.Range(0, 0, 0, 11),
    } as unknown as vscode.TextDocument;

    const provider = new SrujaDefinitionProvider(asContext(new ExtensionContext()));
    const links = await provider.provideDefinition(doc, new vscode.Position(0, 1), new CancellationToken() as any);

    expect(links).toHaveLength(2);
    expect(links?.[1].targetUri.fsPath).toBe("/ws/docs/payments.md");
  });
});


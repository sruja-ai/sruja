import * as vscode from "vscode";
import { ExtensionContext, CancellationToken } from "./__mocks__/vscode";

jest.mock("./wasm", () => {
  const actual = jest.requireActual<typeof import("./wasm")>("./wasm");
  return {
    ...actual,
    initWasm: jest.fn(),
    exportMarkdownFromWasm: jest.fn(),
  };
});

import { SrujaMarkdownPreviewEditorProvider } from "./markdownPreviewEditor";

function asContext(ctx: ExtensionContext): vscode.ExtensionContext {
  return ctx as unknown as vscode.ExtensionContext;
}

describe("SrujaMarkdownPreviewEditorProvider", () => {
  beforeEach(() => {
    (vscode.workspace as any).textDocuments = [];
    (vscode.workspace as any).fs = {
      readFile: jest.fn().mockResolvedValue(Buffer.from("", "utf8")),
      writeFile: jest.fn(),
      stat: jest.fn(),
      createDirectory: jest.fn(),
    };
  });

  it("renders error HTML when wasm is not available", async () => {
    const { initWasm } = await import("./wasm");
    (initWasm as jest.Mock).mockResolvedValue(null);

    const provider = new SrujaMarkdownPreviewEditorProvider(asContext(new ExtensionContext()));
    const doc = { uri: vscode.Uri.file("/ws/arch.sruja"), dispose: () => {} } as any;
    const panel = {
      webview: { html: "", options: {}, cspSource: "vscode-resource:" },
      onDidDispose: (_cb: () => void) => {},
    } as any;

    await provider.resolveCustomEditor(doc, panel, new CancellationToken() as any);
    expect(panel.webview.html).toContain("WASM not available");
  });

  it("renders markdown HTML when export succeeds", async () => {
    const { initWasm, exportMarkdownFromWasm } = await import("./wasm");
    (initWasm as jest.Mock).mockResolvedValue({});
    (exportMarkdownFromWasm as jest.Mock).mockResolvedValue(`# Title\n\n\`\`\`mermaid\ngraph TD\n\`\`\`\n`);

    const provider = new SrujaMarkdownPreviewEditorProvider(asContext(new ExtensionContext()));
    const uri = vscode.Uri.file("/ws/arch.sruja");
    const doc = { uri, dispose: () => {} } as any;
    const panel = {
      webview: { html: "", options: {}, cspSource: "vscode-resource:" },
      onDidDispose: (_cb: () => void) => {},
    } as any;

    (vscode.workspace as any).textDocuments = [{ uri, getText: () => "dsl" }];

    await provider.resolveCustomEditor(doc, panel, new CancellationToken() as any);
    expect(panel.webview.html).toContain("marked.parse");
    expect(panel.webview.html).toContain("MERMAID0");
  });

  it("renders error HTML when export returns null", async () => {
    const { initWasm, exportMarkdownFromWasm } = await import("./wasm");
    (initWasm as jest.Mock).mockResolvedValue({});
    (exportMarkdownFromWasm as jest.Mock).mockResolvedValue(null);

    const provider = new SrujaMarkdownPreviewEditorProvider(asContext(new ExtensionContext()));
    const uri = vscode.Uri.file("/ws/arch.sruja");
    const doc = { uri, dispose: () => {} } as any;
    const panel = {
      webview: { html: "", options: {}, cspSource: "vscode-resource:" },
      onDidDispose: (_cb: () => void) => {},
    } as any;

    (vscode.workspace as any).textDocuments = [{ uri, getText: () => "dsl" }];

    await provider.resolveCustomEditor(doc, panel, new CancellationToken() as any);
    expect(panel.webview.html).toContain("Failed to generate markdown.");
  });
});


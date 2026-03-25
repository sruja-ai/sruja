import * as vscode from "vscode";

jest.mock("./utils", () => {
  const actual = jest.requireActual<typeof import("./utils")>("./utils");
  return {
    ...actual,
    nonce: () => "fixednonce",
  };
});

import { docsThreadState, renderDocsThreadHtml } from "./docsThread";

describe("renderDocsThreadHtml", () => {
  beforeEach(() => {
    docsThreadState.followCursor = true;
    docsThreadState.entries = [];
    docsThreadState.lastPushedKey = "";
    (vscode.workspace as any).getWorkspaceFolder = () => ({ uri: vscode.Uri.file("/ws"), name: "ws" });
  });

  it("renders empty state when there are no entries", () => {
    const html = renderDocsThreadHtml({} as vscode.Webview);
    expect(html).toContain("Move the cursor inside an element");
    expect(html).toContain('nonce="fixednonce"');
  });

  it("renders entries with docs + refs", () => {
    docsThreadState.entries.push({
      key: "k1",
      sourceUri: vscode.Uri.file("/ws/book/arch.sruja").toString(),
      elementId: "Payments.Api",
      kind: "component",
      title: "API",
      parentId: "Payments",
      range: { startLine: 4, startCharacter: 2, endLine: 6, endCharacter: 1 },
      doc: {
        path: "docs/payments.md",
        uri: vscode.Uri.file("/ws/docs/payments.md").toString(),
        exists: true,
        isMarkdown: true,
        previewText: "Hello\nWorld",
        omittedLines: 3,
      },
      refs: [
        {
          uri: vscode.Uri.file("/ws/docs/payments.md").toString(),
          rel: "docs/payments.md",
          line: 10,
          character: 4,
          lineText: "Payments.Api is here",
        },
      ],
      createdAtMs: Date.now(),
    });

    const html = renderDocsThreadHtml({} as vscode.Webview);
    expect(html).toContain("Payments.Api");
    expect(html).toContain("Open preview");
    expect(html).toContain("… (3 more lines)");
    expect(html).toContain("References");
    expect(html).toContain("docs/payments.md:11:5");
    expect(html).toContain("Parent:");
  });
});


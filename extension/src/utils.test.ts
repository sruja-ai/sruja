import * as vscode from "vscode";
import {
  escapeHtml,
  formatRangeOneBased,
  formatRelPath,
  nonce,
  pickActiveSrujaDoc,
  toFsPathOrUri,
  truncateLines,
} from "./utils";

describe("utils", () => {
  beforeEach(() => {
    (vscode.window as any).activeTextEditor = undefined;
    (vscode.window as any).visibleTextEditors = [];
    (vscode.workspace as any).textDocuments = [];
    (vscode.workspace as any).getWorkspaceFolder = () => undefined;
  });

  it("toFsPathOrUri returns fsPath for file uris", () => {
    const u = vscode.Uri.file("/a/b.txt");
    expect(toFsPathOrUri(u)).toBe("/a/b.txt");
  });

  it("toFsPathOrUri returns string for non-file uris", () => {
    const u = new (vscode as any).Uri("untitled", "x");
    expect(toFsPathOrUri(u)).toBe("untitled://x");
  });

  it("pickActiveSrujaDoc prefers active editor", () => {
    const doc = { languageId: "sruja" } as unknown as vscode.TextDocument;
    (vscode.window as any).activeTextEditor = { document: doc };
    expect(pickActiveSrujaDoc()).toBe(doc);
  });

  it("pickActiveSrujaDoc falls back to visible editors", () => {
    const doc = { languageId: "sruja" } as unknown as vscode.TextDocument;
    (vscode.window as any).activeTextEditor = { document: { languageId: "plaintext" } };
    (vscode.window as any).visibleTextEditors = [{ document: doc }];
    expect(pickActiveSrujaDoc()).toBe(doc);
  });

  it("pickActiveSrujaDoc falls back to workspace documents", () => {
    const doc = { languageId: "sruja" } as unknown as vscode.TextDocument;
    (vscode.window as any).activeTextEditor = { document: { languageId: "plaintext" } };
    (vscode.window as any).visibleTextEditors = [];
    (vscode.workspace as any).textDocuments = [doc];
    expect(pickActiveSrujaDoc()).toBe(doc);
  });

  it("formatRangeOneBased returns 1-based line and column", () => {
    expect(formatRangeOneBased(new vscode.Range(0, 0, 0, 3))).toBe("1:1");
    expect(formatRangeOneBased(new vscode.Range(9, 4, 9, 8))).toBe("10:5");
  });

  it("truncateLines limits output and reports omitted lines", () => {
    const text = ["a", "b", "c", "d"].join("\n");
    const result = truncateLines(text, 2, 10_000);
    expect(result.body).toBe("a\nb");
    expect(result.omittedLines).toBe(2);
  });

  it("escapeHtml escapes special characters", () => {
    expect(escapeHtml(`&<>"'`)).toBe("&amp;&lt;&gt;&quot;&#39;");
  });

  it("nonce returns a 16-char alphanumeric string", () => {
    const n = nonce();
    expect(n).toHaveLength(16);
    expect(n).toMatch(/^[a-zA-Z0-9]{16}$/);
  });

  it("formatRelPath returns rel path when file is in workspace", () => {
    (vscode.workspace as any).getWorkspaceFolder = (uri: vscode.Uri) => {
      if (uri.scheme !== "file") return undefined;
      return { uri: vscode.Uri.file("/ws"), name: "ws" };
    };
    expect(formatRelPath(vscode.Uri.file("/ws/a/b.sruja"))).toBe("a/b.sruja");
  });
});


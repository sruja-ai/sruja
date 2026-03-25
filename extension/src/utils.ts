import * as vscode from "vscode";

export function toFsPathOrUri(uri: vscode.Uri): string {
  return uri.scheme === "file" ? uri.fsPath : uri.toString();
}

export function pickActiveSrujaDoc(): vscode.TextDocument | undefined {
  const active = vscode.window.activeTextEditor?.document;
  if (active?.languageId === "sruja") return active;
  for (const e of vscode.window.visibleTextEditors ?? []) {
    if (e.document.languageId === "sruja") return e.document;
  }
  for (const d of vscode.workspace.textDocuments ?? []) {
    if (d.languageId === "sruja") return d;
  }
  return undefined;
}

export function formatRangeOneBased(range: vscode.Range): string {
  const line = range.start.line + 1;
  const col = range.start.character + 1;
  return `${line}:${col}`;
}

export function truncateLines(text: string, maxLines: number, maxChars: number): { body: string; omittedLines: number } {
  const lines = text.split("\n");
  const body = lines.slice(0, maxLines).join("\n").slice(0, maxChars);
  return { body, omittedLines: Math.max(0, lines.length - maxLines) };
}

export function escapeHtml(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

export function nonce(): string {
  const chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  let out = "";
  for (let i = 0; i < 16; i++) out += chars[Math.floor(Math.random() * chars.length)];
  return out;
}

export function formatRelPath(uri: vscode.Uri): string {
  const folder = vscode.workspace.getWorkspaceFolder(uri);
  if (folder && uri.scheme === "file") {
    const p = require("path");
    return p.relative(folder.uri.fsPath, uri.fsPath);
  }
  return uri.scheme === "file" ? uri.fsPath : uri.toString();
}

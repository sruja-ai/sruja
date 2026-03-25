import * as vscode from "vscode";
import * as path from "path";

import { truncateLines, escapeHtml, nonce, formatRelPath } from "./utils";

export type DocsThreadRef = {
  uri: string;
  rel: string;
  line: number;
  character: number;
  lineText: string | null;
};

export type DocsThreadDoc = {
  path: string;
  uri: string;
  exists: boolean;
  isMarkdown: boolean;
  previewText: string | null;
  omittedLines: number;
};

export type DocsThreadEntry = {
  key: string;
  sourceUri: string;
  elementId: string;
  kind: string;
  title: string | null;
  parentId: string | null;
  range: { startLine: number; startCharacter: number; endLine: number; endCharacter: number };
  doc: DocsThreadDoc | null;
  refs: DocsThreadRef[];
  createdAtMs: number;
};

export const docsThreadState: { followCursor: boolean; entries: DocsThreadEntry[]; lastPushedKey: string } = {
  followCursor: true,
  entries: [],
  lastPushedKey: "",
};

export function renderDocsThreadHtml(_webview: vscode.Webview): string {
  const n = nonce();
  const entries = docsThreadState.entries;
  const followLabel = docsThreadState.followCursor ? "Following cursor" : "Not following";

  const entryHtml = entries
    .map((e) => {
      const docBlock = (() => {
        if (!e.doc) return "";
        const existsText = e.doc.exists ? "" : " (missing)";
        const openButton = e.doc.uri
          ? `<button class="btn" data-action="openDoc" data-uri="${escapeHtml(e.doc.uri)}">${
              e.doc.isMarkdown ? "Open preview" : "Open file"
            }</button>`
          : "";
        const preview =
          e.doc.previewText === null
            ? ""
            : `<pre class="docPreview">${escapeHtml(e.doc.previewText)}${
                e.doc.omittedLines > 0 ? `\n… (${e.doc.omittedLines} more lines)` : ""
              }</pre>`;
        return `<div class="section">
  <div class="sectionTitle">Docs</div>
  <div class="row"><span class="muted">${escapeHtml(e.doc.path)}${existsText}</span>${openButton}</div>
  ${preview}
</div>`;
      })();

      const refsBlock = (() => {
        if (e.refs.length === 0) {
          return `<div class="section"><div class="sectionTitle">References</div><div class="muted">No matches in this file.</div></div>`;
        }
        const items = e.refs
          .slice(0, 50)
          .map((r) => {
            const where = `${r.rel}:${r.line + 1}:${r.character + 1}`;
            const preview = r.lineText ? ` — ${r.lineText.trim()}` : "";
            return `<button class="ref" data-action="openLocation" data-uri="${escapeHtml(
              r.uri
            )}" data-line="${r.line}" data-character="${r.character}">${escapeHtml(where + preview)}</button>`;
          })
          .join("");
        return `<div class="section"><div class="sectionTitle">References</div><div class="refs">${items}</div></div>`;
      })();

      const relPath = formatRelPath(vscode.Uri.parse(e.sourceUri));
      const where = `${relPath}:${e.range.startLine + 1}:${e.range.startCharacter + 1}`;
      const title = e.title ? ` — ${e.title}` : "";
      const parent = e.parentId ? `<div class="muted">Parent: ${escapeHtml(e.parentId)}</div>` : "";
      return `<div class="card" data-key="${escapeHtml(e.key)}">
  <div class="cardHeader">
    <div class="titleRow">
      <button class="element" data-action="openLocation" data-uri="${escapeHtml(
        e.sourceUri
      )}" data-line="${e.range.startLine}" data-character="${e.range.startCharacter}">${escapeHtml(
        `${e.elementId}${title}`
      )}</button>
      <span class="pill">${escapeHtml(e.kind)}</span>
    </div>
    <div class="muted">${escapeHtml(where)}</div>
    ${parent}
  </div>
  ${docBlock}
  ${refsBlock}
</div>`;
    })
    .reverse()
    .join("");

  return `<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Sruja Docs & References</title>
    <style>
      body { font-family: var(--vscode-font-family); font-size: var(--vscode-font-size); color: var(--vscode-foreground); padding: 10px; }
      .toolbar { display: flex; gap: 8px; align-items: center; margin-bottom: 10px; }
      .btn { background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: none; padding: 6px 10px; border-radius: 6px; cursor: pointer; }
      .btn.secondary { background: var(--vscode-button-secondaryBackground); color: var(--vscode-button-secondaryForeground); }
      .muted { color: var(--vscode-descriptionForeground); }
      .pill { background: var(--vscode-badge-background); color: var(--vscode-badge-foreground); padding: 2px 8px; border-radius: 999px; font-size: 0.9em; }
      .card { border: 1px solid var(--vscode-panel-border); border-radius: 10px; padding: 10px; margin-bottom: 10px; }
      .cardHeader { display: flex; flex-direction: column; gap: 4px; }
      .titleRow { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
      .element { background: none; border: none; padding: 0; color: var(--vscode-textLink-foreground); cursor: pointer; font-weight: 600; text-align: left; }
      .element:hover { text-decoration: underline; }
      .section { margin-top: 10px; }
      .sectionTitle { font-weight: 600; margin-bottom: 6px; }
      .row { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
      .docPreview { white-space: pre-wrap; background: var(--vscode-textBlockQuote-background); padding: 8px; border-radius: 8px; border: 1px solid var(--vscode-panel-border); }
      .refs { display: flex; flex-direction: column; gap: 6px; }
      .ref { background: none; border: 1px solid var(--vscode-panel-border); border-radius: 8px; padding: 6px 8px; text-align: left; cursor: pointer; color: var(--vscode-foreground); }
      .ref:hover { background: var(--vscode-list-hoverBackground); }
      .empty { padding: 20px 10px; border: 1px dashed var(--vscode-panel-border); border-radius: 10px; }
    </style>
  </head>
  <body>
    <div class="toolbar">
      <button class="btn secondary" data-action="toggleFollow">${escapeHtml(followLabel)}</button>
      <button class="btn secondary" data-action="pop">Pop</button>
      <button class="btn secondary" data-action="clear">Clear</button>
    </div>
    ${
      entries.length === 0
        ? `<div class="empty muted">Move the cursor inside an element, or run “Sruja: Open Docs & References Thread”.</div>`
        : entryHtml
    }
    <script nonce="${n}">
      const vscode = acquireVsCodeApi();
      document.body.addEventListener("click", (e) => {
        const target = e.target;
        if (!(target instanceof HTMLElement)) return;
        const action = target.dataset.action;
        if (!action) return;
        if (action === "toggleFollow") {
          vscode.postMessage({ type: "toggleFollow" });
          return;
        }
        if (action === "clear") {
          vscode.postMessage({ type: "clear" });
          return;
        }
        if (action === "pop") {
          vscode.postMessage({ type: "pop" });
          return;
        }
        if (action === "openDoc") {
          const uri = target.dataset.uri;
          if (uri) vscode.postMessage({ type: "openDoc", uri });
          return;
        }
        if (action === "openLocation") {
          const uri = target.dataset.uri;
          const line = Number(target.dataset.line ?? "0");
          const character = Number(target.dataset.character ?? "0");
          if (uri) vscode.postMessage({ type: "openLocation", uri, line, character });
          return;
        }
      });
    </script>
  </body>
</html>`;
}

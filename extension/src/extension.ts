import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as vscode from "vscode";

import { getSkills, getSkillsRoot } from "./skills";
import { SrujaSkillsTreeProvider } from "./skillsTree";
import {
  SrujaDefinitionProvider,
  SrujaHoverProvider,
  SrujaDocumentSymbolProvider,
  SrujaDiagramCodeLensProvider,
  SrujaCodeActionProvider,
  SrujaCompletionItemProvider,
  SrujaRenameProvider,
  SrujaReferenceProvider,
  SrujaDocumentFormattingEditProvider,
  resolveDocUri,
  docUriExists,
} from "./providers";
import {
  exportMarkdownFromWasm,
  getDiagnosticsFromWasm,
  getMermaidFromWasm,
  getSequenceDiagramFromWasm,
  getElementsFromWasm,
  getDocumentSymbolsFromWasm,
  wasmRangeToVscodeRange,
} from "./wasm";
import { getSrujaLspPath } from "./config";
import { runCli } from "./cliRunner";
import { parseJsonSafe } from "./safeJson";
import { formatStatusLines, formatReviewLines, type StatusJson, type ReviewJson } from "./cliOutput";
import { getDiagramPreviewHtml, escapeMermaidForScript } from "./diagramPreview";
import { SrujaMarkdownPreviewEditorProvider } from "./markdownPreviewEditor";
import { getDiagnosticCodeValue } from "./lintParser";

const DIAGNOSTIC_COLLECTION_ID = "sruja";
let diagnosticCollection: vscode.DiagnosticCollection | undefined;
let markdownPreviewPanel: vscode.WebviewPanel | undefined;
let diagramPreviewPanel: vscode.WebviewPanel | undefined;
let docsThreadPanel: vscode.WebviewPanel | undefined;

type DocsThreadRef = {
  uri: string;
  rel: string;
  line: number;
  character: number;
  lineText: string | null;
};

type DocsThreadDoc = {
  path: string;
  uri: string;
  exists: boolean;
  isMarkdown: boolean;
  previewText: string | null;
  omittedLines: number;
};

type DocsThreadEntry = {
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

const docsThreadState: { followCursor: boolean; entries: DocsThreadEntry[]; lastPushedKey: string } = {
  followCursor: true,
  entries: [],
  lastPushedKey: "",
};

function getLspPath(): string | undefined {
  return vscode.workspace.getConfiguration("sruja").get<string>("lsp.path");
}

function getSrujaPath(): string {
  return getSrujaLspPath(getLspPath());
}

/** Return true if the document is still open in the workspace. */
function isDocumentStillOpen(uri: vscode.Uri): boolean {
  return vscode.workspace.textDocuments.some((d) => d.uri.toString() === uri.toString());
}

async function updateDiagnostics(
  context: vscode.ExtensionContext,
  doc: vscode.TextDocument
): Promise<void> {
  if (doc.languageId !== "sruja" || doc.uri.scheme !== "file") return;
  if (!diagnosticCollection) return;

  const uri = doc.uri;
  const filename = doc.uri.fsPath;

  const setDiagsIfDocOpen = (diags: vscode.Diagnostic[]) => {
    if (isDocumentStillOpen(uri)) diagnosticCollection?.set(uri, diags);
  };

  try {
    const diags = await getDiagnosticsFromWasm(context, doc.getText(), filename);
    setDiagsIfDocOpen(diags);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    setDiagsIfDocOpen([
      new vscode.Diagnostic(
        new vscode.Range(0, 0, 0, 0),
        `Sruja lint failed: ${message}`,
        vscode.DiagnosticSeverity.Warning
      ),
    ]);
  }
}

/** Output channel for context engineering (drift, sync, status, review). */
let cliOutputChannel: vscode.OutputChannel | undefined;

function getCliOutputChannel(): vscode.OutputChannel {
  if (!cliOutputChannel) {
    cliOutputChannel = vscode.window.createOutputChannel("Sruja");
  }
  return cliOutputChannel;
}

function toFsPathOrUri(uri: vscode.Uri): string {
  return uri.scheme === "file" ? uri.fsPath : uri.toString();
}

function formatRangeOneBased(range: vscode.Range): string {
  const line = range.start.line + 1;
  const col = range.start.character + 1;
  return `${line}:${col}`;
}

function formatDiagnosticSummary(rootPath: string | undefined, uri: vscode.Uri, d: vscode.Diagnostic): string {
  const severity =
    d.severity === vscode.DiagnosticSeverity.Error
      ? "error"
      : d.severity === vscode.DiagnosticSeverity.Warning
        ? "warning"
        : d.severity === vscode.DiagnosticSeverity.Information
          ? "info"
          : "hint";
  const fsPath = toFsPathOrUri(uri);
  const rel = rootPath && uri.scheme === "file" ? path.relative(rootPath, fsPath) : fsPath;
  const where = `${rel}:${formatRangeOneBased(d.range)}`;
  const msg = String(d.message).replace(/\s+/g, " ").trim();
  const code = getDiagnosticCodeValue(d);
  const codeText = code === undefined ? "" : ` ${String(code)}`;
  return `- [${severity}${codeText}] ${where} ${msg}`;
}

function truncateLines(text: string, maxLines: number, maxChars: number): { body: string; omittedLines: number } {
  const lines = text.split(/\r?\n/);
  const sliced = lines.slice(0, maxLines).map((l) => (l.length > maxChars ? l.slice(0, maxChars) : l));
  const omittedLines = Math.max(0, lines.length - sliced.length);
  return { body: sliced.join("\n").trimEnd(), omittedLines };
}

function escapeHtml(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function pickActiveSrujaDoc(): vscode.TextDocument | undefined {
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

function groupDiagnosticsByFile(
  diagEntries: Array<[vscode.Uri, vscode.Diagnostic[]]>,
  rootPath: string | undefined
): Array<{ uri: vscode.Uri; rel: string; errors: number; warnings: number; diags: vscode.Diagnostic[] }> {
  const out: Array<{ uri: vscode.Uri; rel: string; errors: number; warnings: number; diags: vscode.Diagnostic[] }> = [];
  for (const [uri, diags] of diagEntries) {
    const fsPath = toFsPathOrUri(uri);
    const rel = rootPath && uri.scheme === "file" ? path.relative(rootPath, fsPath) : fsPath;
    const filtered = diags.filter(
      (d) => d.severity === vscode.DiagnosticSeverity.Error || d.severity === vscode.DiagnosticSeverity.Warning
    );
    if (filtered.length === 0) continue;
    const errors = filtered.filter((d) => d.severity === vscode.DiagnosticSeverity.Error).length;
    const warnings = filtered.filter((d) => d.severity === vscode.DiagnosticSeverity.Warning).length;
    out.push({ uri, rel, errors, warnings, diags: filtered });
  }
  return out.sort((a, b) => (b.errors - a.errors) || (b.warnings - a.warnings) || a.rel.localeCompare(b.rel));
}

async function readContextJsonSummary(folder: vscode.WorkspaceFolder): Promise<{
  path: string;
  size: number;
  mtimeMs: number;
  truthStatus?: string;
}> {
  const contextUri = vscode.Uri.joinPath(folder.uri, ".sruja", "context.json");
  const stat = await vscode.workspace.fs.stat(contextUri);
  const summary = { path: contextUri.fsPath, size: stat.size, mtimeMs: stat.mtime } as {
    path: string;
    size: number;
    mtimeMs: number;
    truthStatus?: string;
  };
  if (stat.size > 512_000) return summary;
  try {
    const raw = await vscode.workspace.fs.readFile(contextUri);
    const text = Buffer.from(raw).toString("utf8");
    const parsed = parseJsonSafe<{ truth_status?: string }>(text);
    if (parsed.ok && typeof parsed.value.truth_status === "string") {
      summary.truthStatus = parsed.value.truth_status;
    }
  } catch {
    return summary;
  }
  return summary;
}

async function buildContextPack(context: vscode.ExtensionContext): Promise<string> {
  const folder = vscode.workspace.workspaceFolders?.[0];
  const rootPath = folder?.uri.fsPath;

  const editor = vscode.window.activeTextEditor;
  const activeDoc = editor?.document;
  const activePath = activeDoc ? toFsPathOrUri(activeDoc.uri) : undefined;
  const activeRel = rootPath && activeDoc?.uri.scheme === "file" ? path.relative(rootPath, activeDoc.uri.fsPath) : activePath;

  const selectionText = (() => {
    if (!editor || !activeDoc) return undefined;
    const sel = editor.selection;
    if (!sel.isEmpty) {
      const selected = activeDoc.getText(sel).trimEnd();
      return selected.length > 0 ? selected : undefined;
    }
    const line = sel.active.line;
    const start = Math.max(0, line - 20);
    const end = Math.min(activeDoc.lineCount - 1, line + 20);
    const lines: string[] = [];
    for (let i = start; i <= end; i++) {
      lines.push(activeDoc.lineAt(i).text);
    }
    const snippet = lines.join("\n").trimEnd();
    return snippet.length > 0 ? snippet : undefined;
  })();

  const activeSrujaDoc = pickActiveSrujaDoc();
  const activeSrujaPath = activeSrujaDoc?.uri.scheme === "file" ? activeSrujaDoc.uri.fsPath : undefined;
  const activeSrujaRel =
    rootPath && activeSrujaDoc?.uri.scheme === "file" ? path.relative(rootPath, activeSrujaDoc.uri.fsPath) : activeSrujaPath;

  const visibleEditors = vscode.window.visibleTextEditors
    .map((e) => e.document.uri)
    .filter((u, idx, arr) => arr.findIndex((x) => x.toString() === u.toString()) === idx);

  const diagEntries = vscode.languages.getDiagnostics();
  const activeDiagLines = (() => {
    if (!activeDoc) return [];
    const fileDiags = vscode.languages.getDiagnostics(activeDoc.uri).filter(
      (d) => d.severity === vscode.DiagnosticSeverity.Error || d.severity === vscode.DiagnosticSeverity.Warning
    );
    return fileDiags.slice(0, 25).map((d) => formatDiagnosticSummary(rootPath, activeDoc.uri, d));
  })();

  const byFile = groupDiagnosticsByFile(diagEntries, rootPath);
  const topFiles = byFile.slice(0, 25);
  const workspaceDiagLines: string[] = [];
  for (const f of topFiles) {
    const header = `- ${f.rel} (errors=${f.errors} warnings=${f.warnings})`;
    workspaceDiagLines.push(header);
    for (const d of f.diags.slice(0, 8)) {
      const where = `${f.rel}:${formatRangeOneBased(d.range)}`;
      const msg = String(d.message).replace(/\s+/g, " ").trim();
      const code = getDiagnosticCodeValue(d);
      const codeText = code === undefined ? "" : `${String(code)} `;
      const sev = d.severity === vscode.DiagnosticSeverity.Error ? "error" : "warning";
      workspaceDiagLines.push(`  - [${sev} ${codeText}${where}] ${msg}`);
    }
  }
  const totalFilesWithDiags = byFile.length;
  const omittedFiles = Math.max(0, totalFilesWithDiags - topFiles.length);

  let contextJsonLine: string | undefined;
  if (folder) {
    try {
      const summary = await readContextJsonSummary(folder);
      const parts = [`path=${summary.path}`, `size=${summary.size}B`];
      if (summary.truthStatus) parts.push(`truth=${summary.truthStatus}`);
      contextJsonLine = `- context.json ${parts.join(" ")}`;
    } catch {
      contextJsonLine = undefined;
    }
  }

  const skillsRoot = getSkillsRoot(context);
  const skills = getSkills(context);
  const skillsLine =
    skillsRoot && skills.length > 0
      ? `- skills root=${skillsRoot.fsPath} skills=${skills.map((s) => s.name).join(", ")}`
      : skillsRoot
        ? `- skills root=${skillsRoot.fsPath} skills=none`
        : "- skills root=none";

  const skillsDetailLines: string[] = [];
  for (const s of skills.slice(0, 25)) {
    const rules = s.ruleUris.map((r) => r.label);
    const rulesPreview = rules.slice(0, 10).join(", ");
    const rulesMore = Math.max(0, rules.length - Math.min(rules.length, 10));
    const agents = s.agentsUri ? "agents=yes" : "agents=no";
    const ruleText = rules.length === 0 ? "rules=none" : rulesMore > 0 ? `rules=${rulesPreview} (+${rulesMore} more)` : `rules=${rulesPreview}`;
    skillsDetailLines.push(`- ${s.name} ${agents} ${ruleText}`);
  }
  const skillsOmitted = Math.max(0, skills.length - Math.min(skills.length, 25));

  let elementsBlock: string[] = [];
  let mermaidBlock: string[] = [];
  if (activeSrujaDoc && activeSrujaPath) {
    try {
      const elements = await getElementsFromWasm(context, activeSrujaDoc.getText(), activeSrujaPath);
      if (elements && elements.length > 0) {
        const byKind = new Map<string, string[]>();
        for (const el of elements) {
          const k = el.kind ?? "unknown";
          const arr = byKind.get(k) ?? [];
          arr.push(el.id);
          byKind.set(k, arr);
        }
        const kinds = Array.from(byKind.entries()).sort((a, b) => b[1].length - a[1].length || a[0].localeCompare(b[0]));
        elementsBlock.push(`- elements=${elements.length}`);
        for (const [k, ids] of kinds) {
          const preview = ids.slice(0, 12).join(", ");
          const more = Math.max(0, ids.length - Math.min(ids.length, 12));
          elementsBlock.push(more > 0 ? `- ${k} (${ids.length}) ${preview} (+${more} more)` : `- ${k} (${ids.length}) ${preview}`);
        }
      } else if (elements) {
        elementsBlock.push("- elements=0");
      } else {
        elementsBlock.push("- elements=unavailable");
      }
    } catch {
      elementsBlock.push("- elements=unavailable");
    }

    try {
      const mermaid = await getMermaidFromWasm(context, activeSrujaDoc.getText());
      if (mermaid && mermaid.trim().length > 0) {
        const trimmed = truncateLines(mermaid.trimEnd(), 140, 260);
        mermaidBlock.push("```mermaid");
        mermaidBlock.push(trimmed.body);
        mermaidBlock.push("```");
        if (trimmed.omittedLines > 0) mermaidBlock.push(`- mermaid omittedLines=${trimmed.omittedLines}`);
      } else {
        mermaidBlock.push("- mermaid=empty");
      }
    } catch {
      mermaidBlock.push("- mermaid=unavailable");
    }
  } else {
    elementsBlock = ["- elements=unavailable (no .sruja file open)"];
    mermaidBlock = ["- mermaid=unavailable (no .sruja file open)"];
  }

  const now = new Date().toISOString();
  const lines: string[] = [];
  lines.push("# Sruja Context Pack");
  lines.push("");
  lines.push(`- generated=${now}`);
  lines.push(`- workspace=${rootPath ?? "none"}`);
  lines.push(`- activeFile=${activeRel ?? "none"}`);
  lines.push("");
  if (selectionText) {
    const lang = activeDoc?.languageId ?? "";
    lines.push("## Focus");
    lines.push("");
    lines.push("```" + lang);
    lines.push(selectionText);
    lines.push("```");
    lines.push("");
  }
  lines.push("## What You Can Ask Sruja");
  lines.push("");
  lines.push("- Run `Sruja: Command Center` and use: Run drift / Refresh repo context / Status / Review.");
  lines.push("- Open a .sruja file and use: Diagram Preview / Focused Diagram Preview.");
  lines.push("- Use: Copy Rule for AI / Copy Agent Guide for AI / Copy Context Pack for AI.");
  lines.push("");
  if (visibleEditors.length > 0) {
    lines.push("## Open Editors");
    lines.push("");
    for (const u of visibleEditors) {
      const fsPath = toFsPathOrUri(u);
      const rel = rootPath && u.scheme === "file" ? path.relative(rootPath, fsPath) : fsPath;
      lines.push(`- ${rel}`);
    }
    lines.push("");
  }
  lines.push("## Architecture Snapshot");
  lines.push("");
  lines.push(`- activeSrujaFile=${activeSrujaRel ?? "none"}`);
  lines.push(...elementsBlock);
  lines.push("");
  lines.push("## Diagram (Mermaid)");
  lines.push("");
  lines.push(...mermaidBlock);
  lines.push("");
  lines.push("## Diagnostics");
  lines.push("");
  if (activeDiagLines.length === 0 && workspaceDiagLines.length === 0) {
    lines.push("- none");
  } else {
    if (activeDiagLines.length > 0) {
      lines.push("### Active File");
      lines.push("");
      lines.push(...activeDiagLines);
      lines.push("");
    }
    if (workspaceDiagLines.length > 0) {
      lines.push("### Workspace (Top Files)");
      lines.push("");
      lines.push(...workspaceDiagLines);
      if (omittedFiles > 0) lines.push(`- ... (${omittedFiles} more files omitted)`);
    }
  }
  lines.push("");
  lines.push("## Sruja");
  lines.push("");
  lines.push(skillsLine);
  if (contextJsonLine) lines.push(contextJsonLine);
  if (skillsDetailLines.length > 0) {
    lines.push("");
    lines.push("### Skills");
    lines.push("");
    lines.push(...skillsDetailLines);
    if (skillsOmitted > 0) lines.push(`- ... (${skillsOmitted} more skills omitted)`);
  }
  lines.push("");
  lines.push("## Ask");
  lines.push("");
  lines.push("- What is the root cause?");
  lines.push("- What is the smallest safe change?");
  lines.push("- Show exact files/lines to edit.");
  lines.push("- If changes touch architecture, propose updated Sruja DSL + updated context via Refresh/Drift/Review.");
  lines.push("");
  return lines.join("\n");
}

/** Run a Sruja CLI command in the workspace root. Throws if no folder open. */
async function runCliInWorkspace(args: string[]): Promise<{ stdout: string; stderr: string; code: number }> {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) {
    throw new Error("No workspace folder open. Open a folder to run context engineering.");
  }
  return runCli(getSrujaPath(), args, folder.uri.fsPath);
}

async function registerMcpServer(): Promise<void> {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) {
    vscode.window.showWarningMessage("Open a workspace folder to register the MCP server.");
    return;
  }

  const serverName = "sruja";
  const command = getSrujaPath();
  const args = ["mcp"];

  const cursorMcp = (vscode as unknown as { cursor?: { mcp?: { registerServer?: (cfg: unknown) => Promise<void> } } })
    .cursor?.mcp;

  if (cursorMcp?.registerServer) {
    try {
      await cursorMcp.registerServer({ name: serverName, type: "stdio", command, args });
      vscode.window.showInformationMessage("Sruja MCP server registered in Cursor.");
      return;
    } catch {
    }
  }

  const cursorDir = vscode.Uri.joinPath(folder.uri, ".cursor");
  const cursorConfigUri = vscode.Uri.joinPath(cursorDir, "mcp.json");
  await vscode.workspace.fs.createDirectory(cursorDir);

  let existing: unknown = undefined;
  try {
    const raw = await vscode.workspace.fs.readFile(cursorConfigUri);
    const text = Buffer.from(raw).toString("utf8");
    const parsed = parseJsonSafe<Record<string, unknown>>(text);
    if (parsed.ok) existing = parsed.value;
  } catch {
    existing = undefined;
  }

  const base = (existing && typeof existing === "object" ? (existing as Record<string, unknown>) : {}) as Record<string, unknown>;
  const mcpServers =
    base.mcpServers && typeof base.mcpServers === "object" ? (base.mcpServers as Record<string, unknown>) : {};

  mcpServers[serverName] = {
    type: "stdio",
    command,
    args,
  };

  const next = { ...base, mcpServers };
  const jsonText = JSON.stringify(next, null, 2) + "\n";
  await vscode.workspace.fs.writeFile(cursorConfigUri, Buffer.from(jsonText, "utf8"));
}

async function jumpToElementInActiveEditor(context: vscode.ExtensionContext, elementId: string) {
  const active = pickActiveSrujaDoc();
  if (!active) return;
  const elements = await getElementsFromWasm(context, active.getText(), active.uri.fsPath);
  if (!elements) return;
  const element = elements.find(e => e.id === elementId || e.id.endsWith(`.${elementId}`));
  if (element) {
    const editor = await vscode.window.showTextDocument(active, { viewColumn: vscode.ViewColumn.One });
    const range = wasmRangeToVscodeRange(element.range);
    editor.selection = new vscode.Selection(range.start, range.start);
    editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
  }
}

function createDiagramPanel(context: vscode.ExtensionContext, title: string): vscode.WebviewPanel {
  if (diagramPreviewPanel) {
    diagramPreviewPanel.dispose();
  }
  diagramPreviewPanel = vscode.window.createWebviewPanel(
    "srujaDiagramPreview",
    title,
    vscode.ViewColumn.Beside,
    { enableScripts: true }
  );
  diagramPreviewPanel.onDidDispose(() => {
    diagramPreviewPanel = undefined;
  });
  diagramPreviewPanel.webview.onDidReceiveMessage(
    async (message) => {
      if (message.command === 'jumpToElement' && message.elementId) {
        await jumpToElementInActiveEditor(context, message.elementId);
      }
    },
    undefined,
    context.subscriptions
  );
  return diagramPreviewPanel;
}

function nonce(): string {
  const chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
  let out = "";
  for (let i = 0; i < 16; i++) out += chars[Math.floor(Math.random() * chars.length)];
  return out;
}

function formatRelPath(uri: vscode.Uri): string {
  const folder = vscode.workspace.getWorkspaceFolder(uri);
  if (folder && uri.scheme === "file") {
    return path.relative(folder.uri.fsPath, uri.fsPath);
  }
  return toFsPathOrUri(uri);
}

async function readDocPreview(
  sourceDoc: vscode.TextDocument,
  docPath: string
): Promise<DocsThreadDoc | null> {
  const resolved = resolveDocUri(docPath, sourceDoc);
  if (!resolved) {
    return {
      path: docPath,
      uri: "",
      exists: false,
      isMarkdown: docPath.toLowerCase().endsWith(".md"),
      previewText: null,
      omittedLines: 0,
    };
  }
  const exists = await docUriExists(resolved);
  if (!exists) {
    return {
      path: docPath,
      uri: resolved.toString(),
      exists: false,
      isMarkdown: resolved.fsPath.toLowerCase().endsWith(".md"),
      previewText: null,
      omittedLines: 0,
    };
  }
  try {
    const raw = await vscode.workspace.fs.readFile(resolved);
    const text = Buffer.from(raw).toString("utf8");
    const { body, omittedLines } = truncateLines(text, 40, 2000);
    return {
      path: docPath,
      uri: resolved.toString(),
      exists: true,
      isMarkdown: resolved.fsPath.toLowerCase().endsWith(".md"),
      previewText: body,
      omittedLines,
    };
  } catch {
    return {
      path: docPath,
      uri: resolved.toString(),
      exists: true,
      isMarkdown: resolved.fsPath.toLowerCase().endsWith(".md"),
      previewText: null,
      omittedLines: 0,
    };
  }
}

function renderDocsThreadHtml(webview: vscode.Webview): string {
  const n = nonce();
  const entries = docsThreadState.entries;
  const followLabel = docsThreadState.followCursor ? "Following cursor" : "Not following";

  const entryHtml = entries
    .map((e, idx) => {
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

      const where = `${formatRelPath(vscode.Uri.parse(e.sourceUri))}:${e.range.startLine + 1}:${e.range.startCharacter + 1}`;
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
      <span class="muted">#${idx + 1}</span>
    </div>
    ${parent}
    <div class="muted">${escapeHtml(where)}</div>
  </div>
  ${docBlock}
  ${refsBlock}
</div>`;
    })
    .join("\n");

  return `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${n}';" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
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

function createDocsThreadPanel(context: vscode.ExtensionContext): vscode.WebviewPanel {
  if (docsThreadPanel) {
    docsThreadPanel.dispose();
  }
  docsThreadPanel = vscode.window.createWebviewPanel(
    "srujaDocsThread",
    "Sruja – Docs & References",
    vscode.ViewColumn.Beside,
    { enableScripts: true, retainContextWhenHidden: true }
  );
  docsThreadPanel.onDidDispose(() => {
    docsThreadPanel = undefined;
  });
  docsThreadPanel.webview.onDidReceiveMessage(
    async (message) => {
      if (!message || typeof message !== "object") return;
      const type = (message as { type?: unknown }).type;
      if (type === "toggleFollow") {
        docsThreadState.followCursor = !docsThreadState.followCursor;
        if (docsThreadPanel) docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
        return;
      }
      if (type === "clear") {
        docsThreadState.entries = [];
        docsThreadState.lastPushedKey = "";
        if (docsThreadPanel) docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
        return;
      }
      if (type === "pop") {
        docsThreadState.entries.pop();
        docsThreadState.lastPushedKey = docsThreadState.entries.at(-1)?.key ?? "";
        if (docsThreadPanel) docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
        return;
      }
      if (type === "openDoc") {
        const uriRaw = (message as { uri?: unknown }).uri;
        if (typeof uriRaw !== "string") return;
        const uri = vscode.Uri.parse(uriRaw);
        const isMarkdown = uri.fsPath.toLowerCase().endsWith(".md");
        if (isMarkdown) {
          await vscode.workspace.openTextDocument(uri);
          await vscode.commands.executeCommand("markdown.showPreviewToSide", uri);
        } else {
          const doc = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(doc, { viewColumn: vscode.ViewColumn.Beside, preserveFocus: false });
        }
        return;
      }
      if (type === "openLocation") {
        const uriRaw = (message as { uri?: unknown }).uri;
        const line = (message as { line?: unknown }).line;
        const character = (message as { character?: unknown }).character;
        if (typeof uriRaw !== "string") return;
        const pos = new vscode.Position(
          typeof line === "number" ? line : 0,
          typeof character === "number" ? character : 0
        );
        const uri = vscode.Uri.parse(uriRaw);
        const doc = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(doc, { preserveFocus: false });
        editor.selection = new vscode.Selection(pos, pos);
        editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
        return;
      }
    },
    undefined,
    context.subscriptions
  );
  docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
  return docsThreadPanel;
}

async function pushDocsThreadEntryForElement(
  context: vscode.ExtensionContext,
  doc: vscode.TextDocument,
  element: { id: string; kind: string; title: string | null; doc?: string | null; range: { start: { line: number; character: number }; end: { line: number; character: number } } }
): Promise<void> {
  if (!docsThreadPanel) return;
  const range = wasmRangeToVscodeRange(element.range);
  const key = `${doc.uri.toString()}::${element.id}`;
  if (key === docsThreadState.lastPushedKey) return;

  const parentId = (() => {
    const dot = element.id.lastIndexOf(".");
    return dot === -1 ? null : element.id.slice(0, dot);
  })();

  const docInfo = element.doc ? await readDocPreview(doc, element.doc) : null;

  let refs: DocsThreadRef[] = [];
  try {
    const locations = await vscode.commands.executeCommand<vscode.Location[]>(
      "vscode.executeReferenceProvider",
      doc.uri,
      range.start
    );
    const locs = Array.isArray(locations) ? locations : [];
    refs = locs.map((loc) => {
      const uri = loc.uri.toString();
      const rel = formatRelPath(loc.uri);
      const line = loc.range.start.line;
      const character = loc.range.start.character;
      const lineText =
        loc.uri.toString() === doc.uri.toString() && line >= 0 && line < doc.lineCount
          ? doc.lineAt(line).text
          : null;
      return { uri, rel, line, character, lineText };
    });
  } catch {
    refs = [];
  }

  const entry: DocsThreadEntry = {
    key,
    sourceUri: doc.uri.toString(),
    elementId: element.id,
    kind: element.kind,
    title: element.title,
    parentId,
    range: {
      startLine: range.start.line,
      startCharacter: range.start.character,
      endLine: range.end.line,
      endCharacter: range.end.character,
    },
    doc: docInfo,
    refs,
    createdAtMs: Date.now(),
  };

  docsThreadState.entries.push(entry);
  docsThreadState.lastPushedKey = key;
  docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
}

async function pushDocsThreadEntryFromActiveEditor(context: vscode.ExtensionContext): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  const doc = editor?.document;
  if (!editor || !doc || doc.languageId !== "sruja") return;

  const elements = await getElementsFromWasm(context, doc.getText(), doc.uri.fsPath);
  if (!elements?.length) return;

  const pos = editor.selection.active;

  const byRange = elements
    .map((e) => ({ e, r: wasmRangeToVscodeRange(e.range) }))
    .filter(({ r }) => r.contains(pos))
    .sort((a, b) => {
      const aLen = (a.r.end.line - a.r.start.line) * 10_000 + (a.r.end.character - a.r.start.character);
      const bLen = (b.r.end.line - b.r.start.line) * 10_000 + (b.r.end.character - b.r.start.character);
      return aLen - bLen;
    })
    .at(0)?.e;

  const picked = byRange
    ? byRange
    : (() => {
        const wordRange = doc.getWordRangeAtPosition(pos);
        const word = wordRange ? doc.getText(wordRange).trim() : "";
        if (!word) return undefined;
        return elements.find((e) => e.id === word || e.id.endsWith(`.${word}`));
      })();

  if (!picked) return;
  await pushDocsThreadEntryForElement(context, doc, picked);
}

export function activate(context: vscode.ExtensionContext): void {
  const isTest = context.extensionMode === vscode.ExtensionMode.Test;
  diagnosticCollection = vscode.languages.createDiagnosticCollection(DIAGNOSTIC_COLLECTION_ID);
  context.subscriptions.push(diagnosticCollection);

  const runLintForDoc = (doc: vscode.TextDocument) => {
    if (doc.languageId !== "sruja") return;
    updateDiagnostics(context, doc).catch((err) => {
      if (diagnosticCollection && isDocumentStillOpen(doc.uri)) {
        diagnosticCollection.set(doc.uri, [
          new vscode.Diagnostic(
            new vscode.Range(0, 0, 0, 0),
            `Sruja lint failed: ${err instanceof Error ? err.message : String(err)}`,
            vscode.DiagnosticSeverity.Warning
          ),
        ]);
      }
    });
  };

  const pendingLint = new Map<string, ReturnType<typeof setTimeout>>();
  const scheduleLint = (doc: vscode.TextDocument) => {
    const key = doc.uri.toString();
    const existing = pendingLint.get(key);
    if (existing) clearTimeout(existing);
    const t = setTimeout(() => {
      pendingLint.delete(key);
      runLintForDoc(doc);
    }, 400);
    pendingLint.set(key, t);
  };

  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((doc) => {
      if (doc.languageId === "sruja") runLintForDoc(doc);
    }),
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.languageId === "sruja") runLintForDoc(doc);
    }),
    vscode.workspace.onDidChangeTextDocument((e) => {
      if (e.document.languageId === "sruja") scheduleLint(e.document);
    }),
    vscode.workspace.onDidCloseTextDocument((doc) => {
      if (doc.languageId === "sruja") {
        const key = doc.uri.toString();
        const t = pendingLint.get(key);
        if (t) {
          clearTimeout(t);
          pendingLint.delete(key);
        }
        diagnosticCollection?.delete(doc.uri);
      }
    })
  );

  let selectionDebounce: ReturnType<typeof setTimeout> | undefined;
  context.subscriptions.push(
    vscode.window.onDidChangeTextEditorSelection((e) => {
      if (!docsThreadPanel || !docsThreadState.followCursor) return;
      if (e.textEditor.document.languageId !== "sruja") return;
      if (selectionDebounce) clearTimeout(selectionDebounce);
      selectionDebounce = setTimeout(() => {
        pushDocsThreadEntryFromActiveEditor(context).catch(() => undefined);
      }, 200);
    })
  );

  for (const doc of vscode.workspace.textDocuments) {
    if (doc.languageId === "sruja") runLintForDoc(doc);
  }

  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.commandCenter", async () => {
      const items: (vscode.QuickPickItem & { command?: string })[] = [
        { label: "Diagrams & Previews", kind: vscode.QuickPickItemKind.Separator },
        { label: "$(graph) Open Diagram Preview", detail: "Full architecture diagram", command: "sruja.openDiagramPreview" },
        { label: "$(filter) Open Focused Diagram Preview", detail: "Filtered by level and element", command: "sruja.openFocusedDiagramPreview" },
        { label: "$(list-tree) Open Sequence Diagram Preview", detail: "Render a scenario or flow", command: "sruja.openSequenceDiagramPreview" },
        { label: "$(open-preview) Open Markdown Preview", detail: "Live rendered markdown", command: "sruja.openMarkdownPreview" },

        { label: "Validation & Export", kind: vscode.QuickPickItemKind.Separator },
        { label: "$(check-all) Run Validation", detail: "Check .sruja file for lint errors", command: "sruja.runValidation" },
        { label: "$(markdown) Export architecture to Markdown", detail: "Generate markdown from DSL", command: "sruja.exportMarkdown" },

        { label: "Knowledge & Docs", kind: vscode.QuickPickItemKind.Separator },
        { label: "$(book) Open component knowledge", detail: "Open linked doc for element under cursor", command: "sruja.openComponentKnowledge" },
        { label: "$(references) Open Docs & References Thread", detail: "Browse element docs and cross-references", command: "sruja.openDocsThread" },

        { label: "AI & Skills", kind: vscode.QuickPickItemKind.Separator },
        { label: "$(copy) Copy Context Pack for AI", detail: "Copy architecture snapshot to clipboard", command: "sruja.copyContextPackForAI" },
        { label: "$(file-code) Copy Rule for AI", detail: "Copy a skill rule to clipboard", command: "sruja.copyRuleForAI" },
        { label: "$(file-text) Copy Agent Guide for AI", detail: "Copy AGENTS.md to clipboard", command: "sruja.copyAgentGuideForAI" },
        { label: "$(folder-library) Open Skills Overview", detail: "Browse skills and rules", command: "sruja.openSkillsOverview" },
        { label: "$(file-text) Open Agent Guide (AGENTS.md)", detail: "Open AGENTS.md in editor", command: "sruja.openAgentGuide" },
        { label: "$(list-unordered) List Rules…", detail: "Browse and open a rule file", command: "sruja.listRules" },

        { label: "Context Engineering (CLI)", kind: vscode.QuickPickItemKind.Separator },
        { label: "$(pulse) Run drift (architecture health)", detail: "Check architecture health vs. reality", command: "sruja.runDrift" },
        { label: "$(sync) Refresh repo context", detail: "Rebuild context.json for AI", command: "sruja.refreshContext" },
        { label: "$(info) Status", detail: "Show repo truth status", command: "sruja.status" },
        { label: "$(eye) Review architecture update", detail: "Review pending architecture changes", command: "sruja.review" },

        { label: "Integrations", kind: vscode.QuickPickItemKind.Separator },
        { label: "$(plug) Register MCP Server (Cursor)", detail: "Register Sruja as MCP server in Cursor", command: "sruja.registerMcpServer" },
      ];
      const pick = isTest
        ? items[1] // skip separator
        : await vscode.window.showQuickPick(items, { placeHolder: "Sruja Command Center" });
      if (!pick || !pick.command) return;
      await vscode.commands.executeCommand(pick.command);
    }),
    vscode.commands.registerCommand("sruja.runValidation", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to run validation.");
        return;
      }
      const key = doc.uri.toString();
      const pending = pendingLint.get(key);
      if (pending) {
        clearTimeout(pending);
        pendingLint.delete(key);
      }
      try {
        await updateDiagnostics(context, doc);
        const diags = diagnosticCollection?.get(doc.uri) ?? [];
        const errors = diags.filter((d) => d.severity === vscode.DiagnosticSeverity.Error).length;
        const warnings = diags.filter((d) => d.severity === vscode.DiagnosticSeverity.Warning).length;
        if (errors > 0 || warnings > 0) {
          vscode.window.showInformationMessage(
            `Sruja validation: ${errors} error(s), ${warnings} warning(s). See Problems panel.`
          );
        } else {
          vscode.window.showInformationMessage("Sruja validation: no issues.");
        }
      } catch (err) {
        vscode.window.showErrorMessage(
          `Sruja validation failed: ${err instanceof Error ? err.message : String(err)}`
        );
      }
    })
  );

  const skillsTreeProvider = new SrujaSkillsTreeProvider(context);
  context.subscriptions.push(
    vscode.window.registerTreeDataProvider("srujaSkillsView", skillsTreeProvider)
  );

  // Register definition provider for Go to Definition (F12)
  const definitionProvider = new SrujaDefinitionProvider(context);
  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider("sruja", definitionProvider)
  );

  // Register hover provider
  const hoverProvider = new SrujaHoverProvider(context);
  context.subscriptions.push(
    vscode.languages.registerHoverProvider("sruja", hoverProvider)
  );

  // Register document symbol provider for outline view
  const documentSymbolProvider = new SrujaDocumentSymbolProvider(context);
  context.subscriptions.push(
    vscode.languages.registerDocumentSymbolProvider("sruja", documentSymbolProvider)
  );

  const diagramCodeLensProvider = new SrujaDiagramCodeLensProvider(context);
  context.subscriptions.push(
    vscode.languages.registerCodeLensProvider("sruja", diagramCodeLensProvider)
  );

  const codeActionProvider = new SrujaCodeActionProvider(context);
  context.subscriptions.push(
    vscode.languages.registerCodeActionsProvider("sruja", codeActionProvider, {
      providedCodeActionKinds: [vscode.CodeActionKind.QuickFix],
    })
  );
  
  // Register completion provider
  const completionProvider = new SrujaCompletionItemProvider(context);
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider("sruja", completionProvider, "->", ".", "=")
  );

  // Register rename provider
  const renameProvider = new SrujaRenameProvider(context);
  context.subscriptions.push(
    vscode.languages.registerRenameProvider("sruja", renameProvider)
  );

  // Register reference provider
  const referenceProvider = new SrujaReferenceProvider(context);
  context.subscriptions.push(
    vscode.languages.registerReferenceProvider("sruja", referenceProvider)
  );

  // Register document formatting provider
  const formattingProvider = new SrujaDocumentFormattingEditProvider();
  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider("sruja", formattingProvider)
  );

  // Register custom editor for markdown preview (shows "Open Preview" button in editor title)
  context.subscriptions.push(
    vscode.window.registerCustomEditorProvider(
      "sruja.markdownPreview",
      new SrujaMarkdownPreviewEditorProvider(context),
      { supportsMultipleEditorsPerDocument: false, webviewOptions: { retainContextWhenHidden: true } }
    )
  );

  const openMarkdownPreview = async (): Promise<void> => {
    const editor = vscode.window.activeTextEditor;
    const doc = editor?.document;
    if (!doc || doc.languageId !== "sruja") {
      vscode.window.showWarningMessage("Open a .sruja file to open markdown preview.");
      return;
    }
    await vscode.commands.executeCommand("vscode.openWith", doc.uri, "sruja.markdownPreview", vscode.ViewColumn.Beside);
  };

  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.openSkillsOverview", async () => {
      const root = getSkillsRoot(context);
      if (!root) {
        vscode.window.showWarningMessage("No skills root. Set sruja.skills.path or open a workspace with a skills folder.");
        return;
      }
      const skills = getSkills(context);
      if (skills.length === 0) {
        vscode.window.showWarningMessage("No skills found in the skills root.");
        return;
      }
      const skill = skills.length === 1
        ? skills[0]
        : isTest
          ? skills[0]
          : await vscode.window.showQuickPick(
            skills.map((s) => ({ label: s.name, skill: s })),
            { placeHolder: "Select a skill" }
          ).then((p) => p?.skill);
      if (skill) await vscode.window.showTextDocument(skill.skillUri);
    }),
    vscode.commands.registerCommand("sruja.openAgentGuide", async () => {
      const skills = getSkills(context);
      const withAgents = skills.filter((s) => s.agentsUri != null);
      if (withAgents.length === 0) {
        vscode.window.showWarningMessage("No AGENTS.md found. Set sruja.skills.path or open a workspace with skills.");
        return;
      }
      const skill = withAgents.length === 1
        ? withAgents[0]
        : isTest
          ? withAgents[0]
          : await vscode.window.showQuickPick(
            withAgents.map((s) => ({ label: s.name, skill: s })),
            { placeHolder: "Select skill" }
          ).then((p) => p?.skill);
      if (skill?.agentsUri) await vscode.window.showTextDocument(skill.agentsUri);
    }),
    vscode.commands.registerCommand("sruja.listRules", async () => {
      const skills = getSkills(context);
      const allRules: { label: string; uri: vscode.Uri; skillName: string }[] = [];
      for (const s of skills) {
        for (const r of s.ruleUris) {
          allRules.push({ label: r.label, uri: r.uri, skillName: s.name });
        }
      }
      if (allRules.length === 0) {
        vscode.window.showWarningMessage("No rules found. Set sruja.skills.path or open a workspace with skills.");
        return;
      }
      const pick = isTest
        ? { rule: allRules[0] }
        : await vscode.window.showQuickPick(
          allRules.map((r) => ({ label: r.label, description: r.skillName, rule: r })),
          { placeHolder: "Open a rule", matchOnDescription: true }
        );
      if (pick) await vscode.window.showTextDocument(pick.rule.uri);
    }),
    vscode.commands.registerCommand("sruja.copyRuleForAI", async () => {
      const skills = getSkills(context);
      const allRules: { label: string; uri: vscode.Uri }[] = [];
      for (const s of skills) {
        for (const r of s.ruleUris) {
          allRules.push({ label: `${s.name} / ${r.label}`, uri: r.uri });
        }
      }
      if (allRules.length === 0) {
        vscode.window.showWarningMessage("No rules found.");
        return;
      }
      const pick = isTest
        ? { rule: allRules[0] }
        : await vscode.window.showQuickPick(
          allRules.map((r) => ({ label: r.label, rule: r })),
          { placeHolder: "Copy which rule for AI?" }
        );
      if (!pick) return;
      try {
        const content = await vscode.workspace.fs.readFile(pick.rule.uri);
        const text = Buffer.from(content).toString("utf8");
        await vscode.env.clipboard.writeText(text);
        vscode.window.showInformationMessage(`Copied "${pick.rule.label}" to clipboard.`);
      } catch (e) {
        vscode.window.showErrorMessage(`Failed to read rule: ${e instanceof Error ? e.message : String(e)}`);
      }
    }),
    vscode.commands.registerCommand("sruja.copyAgentGuideForAI", async () => {
      const skills = getSkills(context);
      const withAgents = skills.filter((s) => s.agentsUri != null);
      if (withAgents.length === 0) {
        vscode.window.showWarningMessage("No AGENTS.md found.");
        return;
      }
      const skill = withAgents.length === 1
        ? withAgents[0]
        : isTest
          ? withAgents[0]
          : await vscode.window.showQuickPick(
            withAgents.map((s) => ({ label: s.name, skill: s })),
            { placeHolder: "Copy which agent guide?" }
          ).then((p) => p?.skill);
      if (!skill?.agentsUri) return;
      try {
        const content = await vscode.workspace.fs.readFile(skill.agentsUri);
        const text = Buffer.from(content).toString("utf8");
        await vscode.env.clipboard.writeText(text);
        vscode.window.showInformationMessage(`Copied "${skill.name}" agent guide to clipboard.`);
      } catch (e) {
        vscode.window.showErrorMessage(`Failed to read agent guide: ${e instanceof Error ? e.message : String(e)}`);
      }
    }),
    vscode.commands.registerCommand("sruja.copyContextPackForAI", async () => {
      try {
        const text = await buildContextPack(context);
        await vscode.env.clipboard.writeText(text);
        vscode.window.showInformationMessage(`Copied context pack to clipboard (${text.length} chars).`);
      } catch (err) {
        vscode.window.showErrorMessage(
          `Failed to build context pack: ${err instanceof Error ? err.message : String(err)}`
        );
      }
    }),
    vscode.commands.registerCommand("sruja.exportMarkdown", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to export to Markdown.");
        return;
      }
      const doc = editor.document;
      const dsl = doc.getText();
      const filePath = doc.uri.scheme === "file" ? doc.uri.fsPath : path.join(os.tmpdir(), "document.sruja");
      const outPath = filePath.replace(/\.sruja$/i, ".md");

      const stdout = await exportMarkdownFromWasm(context, dsl);
      if (stdout === null || stdout === undefined) {
        vscode.window.showErrorMessage(
          "Sruja WASM could not load. Run npm run copy:assets if developing, or reinstall the extension."
        );
        return;
      }
      try {
        const tmpMdPath = path.join(os.tmpdir(), `sruja-preview-${Date.now()}.md`);
        await fs.promises.writeFile(tmpMdPath, stdout, "utf8");
        const mdUri = vscode.Uri.file(tmpMdPath);
        const mdDoc = await vscode.workspace.openTextDocument(mdUri);
        await vscode.window.showTextDocument(mdDoc, { preview: true });
        await vscode.commands.executeCommand("markdown.showPreview", mdUri);
        const save = isTest
          ? undefined
          : await vscode.window.showInformationMessage(
            "Markdown generated from DSL. Save to file?",
            "Save",
            "Cancel"
          );
        if (save === "Save" && doc.uri.scheme === "file") {
          const uri = vscode.Uri.file(outPath);
          await vscode.workspace.fs.writeFile(uri, Buffer.from(stdout, "utf8"));
          const saved = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(saved);
        }
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        vscode.window.showErrorMessage(`Export to Markdown failed: ${msg}`);
      }
    }),
    vscode.commands.registerCommand("sruja.openDocsThread", async () => {
      docsThreadState.followCursor = true;
      createDocsThreadPanel(context);
      await pushDocsThreadEntryFromActiveEditor(context);
    }),
    vscode.commands.registerCommand("sruja.openDocsThreadAt", async (arg?: unknown) => {
      docsThreadState.followCursor = true;
      createDocsThreadPanel(context);

      const parsed =
        typeof arg === "object" && arg !== null
          ? (arg as { docUri?: unknown; elementId?: unknown })
          : undefined;
      const docUriRaw = typeof parsed?.docUri === "string" ? parsed.docUri : undefined;
      const elementId = typeof parsed?.elementId === "string" ? parsed.elementId : undefined;

      if (docUriRaw && elementId) {
        const uri = vscode.Uri.parse(docUriRaw);
        const doc = await vscode.workspace.openTextDocument(uri);
        const elements = await getElementsFromWasm(context, doc.getText(), doc.uri.fsPath);
        const element = elements?.find((e) => e.id === elementId || e.id.endsWith(`.${elementId}`));
        if (element) {
          await pushDocsThreadEntryForElement(context, doc, element);
          return;
        }
      }

      await pushDocsThreadEntryFromActiveEditor(context);
    }),
    vscode.commands.registerCommand("sruja.openComponentKnowledge", async (docUriArg?: string) => {
      async function openDocInSplit(uri: vscode.Uri): Promise<void> {
        const isMarkdown = uri.fsPath.toLowerCase().endsWith(".md");
        if (isMarkdown) {
          await vscode.workspace.openTextDocument(uri);
          await vscode.commands.executeCommand("markdown.showPreviewToSide", uri);
        } else {
          const doc = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(doc, {
            viewColumn: vscode.ViewColumn.Beside,
            preserveFocus: false,
          });
        }
      }
      if (docUriArg) {
        try {
          const uri = vscode.Uri.parse(docUriArg);
          await openDocInSplit(uri);
        } catch {
          vscode.window.showWarningMessage("Could not open component knowledge file.");
        }
        return;
      }
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Place the cursor on an element in a .sruja file, or use the hover link to open documentation.");
        return;
      }
      const position = editor.selection.active;
      const elements = await getElementsFromWasm(context, doc.getText(), doc.uri.fsPath);
      if (!elements?.length) {
        if (elements === null) {
          vscode.window.showWarningMessage("Sruja WASM is not available for component lookup. Reinstall the extension or run npm run copy:assets if developing.");
        }
        return;
      }
      const wordRange = doc.getWordRangeAtPosition(position);
      const word = wordRange ? doc.getText(wordRange).trim() : "";
      if (!word) return;
      const element = elements.find((e) => e.id === word || e.id.endsWith(`.${word}`));
      if (!element?.doc) {
        vscode.window.showInformationMessage(`Element "${word}" has no doc link. Add doc ".sruja/knowledge/..." to the element in the DSL.`);
        return;
      }
      const docUri = resolveDocUri(element.doc, doc);
      if (!docUri || !(await docUriExists(docUri))) {
        vscode.window.showWarningMessage(`Knowledge file not found: ${element.doc}`);
        return;
      }
      await openDocInSplit(docUri);
    }),
    vscode.commands.registerCommand("sruja.openDiagramPreview", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open diagram preview.");
        return;
      }
      const mermaid = await getMermaidFromWasm(context, doc.getText());
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          "Sruja WASM could not load or diagram is empty. Run npm run copy:assets if developing, or reinstall the extension."
        );
        return;
      }
      diagramPreviewPanel = createDiagramPanel(context, "Sruja – Context engineering for the AI era. – Diagram Preview");
      diagramPreviewPanel.webview.html = getDiagramPreviewHtml(escapeMermaidForScript(mermaid));
    }),
    vscode.commands.registerCommand("sruja.openSequenceDiagramPreviewAt", async (arg?: unknown) => {
      const parsed =
        typeof arg === "object" && arg !== null
          ? (arg as { docUri?: string; kind?: unknown; id?: unknown })
          : undefined;

      const docUriRaw = typeof parsed?.docUri === "string" ? parsed.docUri : undefined;
      const kindRaw = typeof parsed?.kind === "string" ? parsed.kind : "scenario";
      const kind = kindRaw.toLowerCase() === "flow" ? "flow" : "scenario";
      const id = typeof parsed?.id === "string" ? parsed.id : undefined;

      let doc: vscode.TextDocument | undefined;
      if (docUriRaw) {
        try {
          doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(docUriRaw));
        } catch {
          doc = undefined;
        }
      } else {
        doc = vscode.window.activeTextEditor?.document;
      }

      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open sequence diagram preview.");
        return;
      }
      if (!id || !id.trim()) {
        vscode.window.showWarningMessage("No scenario/flow ID provided.");
        return;
      }

      const configJson = JSON.stringify({ kind, id });
      const mermaid = await getSequenceDiagramFromWasm(context, doc.getText(), configJson);
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          `Could not render sequence diagram for ${kind} "${id}". Ensure the file parses and Sruja WASM is available.`
        );
        return;
      }

      diagramPreviewPanel = createDiagramPanel(context, `Sruja – Context engineering for the AI era. – Sequence Diagram – ${kind}: ${id}`);
      diagramPreviewPanel.webview.html = getDiagramPreviewHtml(escapeMermaidForScript(mermaid));
    }),
    vscode.commands.registerCommand("sruja.openFocusedDiagramPreviewAt", async (arg?: unknown) => {
      const parsed =
        typeof arg === "object" && arg !== null
          ? (arg as { docUri?: string; viewLevel?: unknown; targetId?: unknown })
          : undefined;

      const docUriRaw = typeof parsed?.docUri === "string" ? parsed.docUri : undefined;
      const viewLevelRaw = typeof parsed?.viewLevel === "number" ? parsed.viewLevel : undefined;
      const viewLevel = viewLevelRaw === 1 || viewLevelRaw === 2 || viewLevelRaw === 3 ? viewLevelRaw : 2;
      const targetId = viewLevel === 1 ? undefined : (typeof parsed?.targetId === "string" ? parsed.targetId : undefined);

      let doc: vscode.TextDocument | undefined;
      if (docUriRaw) {
        try {
          doc = await vscode.workspace.openTextDocument(vscode.Uri.parse(docUriRaw));
        } catch {
          doc = undefined;
        }
      } else {
        doc = vscode.window.activeTextEditor?.document;
      }

      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open diagram preview.");
        return;
      }

      const configJson = JSON.stringify({ viewLevel, targetId });
      const mermaid = await getMermaidFromWasm(context, doc.getText(), configJson);
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          "Sruja WASM could not load or diagram is empty. Run npm run copy:assets if developing, or reinstall the extension."
        );
        return;
      }

      const titleSuffix = targetId ? ` – L${viewLevel}: ${targetId}` : ` – L${viewLevel}`;
      diagramPreviewPanel = createDiagramPanel(context, "Sruja – Context engineering for the AI era. – Diagram Preview" + titleSuffix);
      diagramPreviewPanel.webview.html = getDiagramPreviewHtml(escapeMermaidForScript(mermaid));
    }),
    vscode.commands.registerCommand("sruja.openFocusedDiagramPreview", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open diagram preview.");
        return;
      }

      const levelPick = await vscode.window.showQuickPick(
        [
          { label: "L1 (Context)", level: 1 },
          { label: "L2 (Container)", level: 2 },
          { label: "L3 (Component)", level: 3 },
        ],
        { placeHolder: "Choose diagram level" }
      );
      if (!levelPick) return;

      let targetId: string | undefined;
      if (levelPick.level !== 1) {
        const elements = await getElementsFromWasm(context, doc.getText(), doc.uri.fsPath);
        if (!elements?.length) {
          vscode.window.showWarningMessage(
            "Could not list elements for focus selection. Ensure the file parses and Sruja WASM is available."
          );
          return;
        }

        const items = elements
          .map((e) => ({
            label: e.id,
            description: e.kind,
            detail: e.title ?? undefined,
            id: e.id,
          }))
          .sort((a, b) => a.label.localeCompare(b.label));

        const picked = await vscode.window.showQuickPick(items, {
          placeHolder: "Choose a focus element ID",
          matchOnDescription: true,
          matchOnDetail: true,
        });
        if (!picked) return;
        targetId = picked.id;
      }

      const configJson = JSON.stringify({ viewLevel: levelPick.level, targetId });
      const mermaid = await getMermaidFromWasm(context, doc.getText(), configJson);
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          "Sruja WASM could not load or diagram is empty. Run npm run copy:assets if developing, or reinstall the extension."
        );
        return;
      }

      if (diagramPreviewPanel) {
        diagramPreviewPanel.dispose();
      }
      const titleSuffix = targetId ? ` – ${levelPick.label}: ${targetId}` : ` – ${levelPick.label}`;
      diagramPreviewPanel = vscode.window.createWebviewPanel(
        "srujaDiagramPreview",
        "Sruja – Context engineering for the AI era. – Diagram Preview" + titleSuffix,
        vscode.ViewColumn.Beside,
        { enableScripts: true }
      );
      diagramPreviewPanel.onDidDispose(() => {
        diagramPreviewPanel = undefined;
      });
      diagramPreviewPanel.webview.html = getDiagramPreviewHtml(escapeMermaidForScript(mermaid));
    }),
    vscode.commands.registerCommand("sruja.openSequenceDiagramPreview", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open sequence diagram preview.");
        return;
      }

      const symbols = await getDocumentSymbolsFromWasm(context, doc.getText(), doc.uri.fsPath);
      if (!symbols) {
        vscode.window.showWarningMessage(
          "Could not list scenarios/flows. Ensure the file parses and Sruja WASM is available."
        );
        return;
      }

      type ScenarioFlowPick = vscode.QuickPickItem & { seqKind: "scenario" | "flow"; id: string };

      const items: ScenarioFlowPick[] = symbols
        .filter((s) => s.kind === "scenario" || s.kind === "flow")
        .map((s) => ({
          label: s.name,
          description: s.kind === "flow" ? "Flow" : "Scenario",
          seqKind: s.kind === "flow" ? ("flow" as const) : ("scenario" as const),
          id: s.name,
        }))
        .sort((a, b) => a.label.localeCompare(b.label));

      if (items.length === 0) {
        vscode.window.showInformationMessage("No scenarios or flows found in this file.");
        return;
      }

      const picked = isTest
        ? items[0]
        : await vscode.window.showQuickPick<ScenarioFlowPick>(items, {
            placeHolder: "Choose a scenario or flow to render as a sequence diagram",
            matchOnDescription: true,
          });
      if (!picked) return;

      await vscode.commands.executeCommand("sruja.openSequenceDiagramPreviewAt", {
        docUri: doc.uri.toString(),
        kind: picked.seqKind,
        id: picked.id,
      });
    }),
    vscode.commands.registerCommand("sruja.markdownPreview", openMarkdownPreview),
    vscode.commands.registerCommand("sruja.openMarkdownPreview", openMarkdownPreview),
    vscode.commands.registerCommand("sruja.registerMcpServer", registerMcpServer),
    vscode.commands.registerCommand("sruja.runDrift", async () => {
      await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: "Sruja: Running drift…", cancellable: false },
        async () => {
          const channel = getCliOutputChannel();
          channel.clear();
          channel.show(true);
          channel.appendLine("Running sruja drift -r . ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(["drift", "-r", "."]);
            channel.append(stdout);
            if (stderr) channel.append(stderr);
            channel.appendLine("");
            if (code !== 0) {
              channel.appendLine(`(exit code ${code})`);
            }
            channel.appendLine("--- Done ---");
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage(
              "Sruja drift failed. Ensure the Sruja CLI is installed and on PATH, or set sruja.lsp.path."
            );
          }
        }
      );
    }),
    vscode.commands.registerCommand("sruja.refreshContext", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      if (!folder) {
        vscode.window.showWarningMessage("Open a workspace folder to refresh repo context.");
        return;
      }
      await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: "Sruja: Refreshing repo context…", cancellable: false },
        async () => {
          const channel = getCliOutputChannel();
          channel.show(true);
          channel.appendLine("Refreshing repo context (sruja sync -r . -f json) ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace([
              "sync",
              "-r",
              ".",
              "-f",
              "json",
            ]);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.append(stdout);
              vscode.window.showErrorMessage("Sruja sync failed. Is the CLI on PATH or set sruja.lsp.path?");
              return;
            }
            const contextPath = path.join(folder.uri.fsPath, ".sruja", "context.json");
            channel.appendLine(`Context written to ${contextPath}`);
            const parsed = parseJsonSafe<{ context_path?: string; truth_status?: string }>(stdout);
            if (parsed.ok && parsed.value.context_path) {
              channel.appendLine(`Baseline/truth: ${parsed.value.truth_status ?? "unknown"}`);
            }
            channel.appendLine("--- Done ---");
            vscode.window.showInformationMessage("Sruja: Repo context updated.");
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage("Sruja refresh context failed: " + msg);
          }
        }
      );
    }),
    vscode.commands.registerCommand("sruja.status", async () => {
      await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: "Sruja: Fetching status…", cancellable: false },
        async () => {
          const channel = getCliOutputChannel();
          channel.show(true);
          channel.appendLine("Running sruja status -r . --format json ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace([
              "status",
              "-r",
              ".",
              "--format",
              "json",
            ]);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.append(stdout);
              channel.appendLine("--- Status failed ---");
              return;
            }
            const parsed = parseJsonSafe<StatusJson>(stdout);
            if (!parsed.ok) {
              channel.appendLine(`Parse error: ${parsed.error}`);
              return;
            }
            for (const line of formatStatusLines(parsed.value)) {
              channel.appendLine(line);
            }
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage("Sruja status failed. Is the CLI on PATH or set sruja.lsp.path?");
          }
        }
      );
    }),
    vscode.commands.registerCommand("sruja.review", async () => {
      await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: "Sruja: Reviewing architecture update…", cancellable: false },
        async () => {
          const channel = getCliOutputChannel();
          channel.show(true);
          channel.appendLine("Running sruja review -r . --format json ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace([
              "review",
              "-r",
              ".",
              "--format",
              "json",
            ]);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.append(stdout);
              channel.appendLine("--- Review failed ---");
              return;
            }
            const parsed = parseJsonSafe<ReviewJson>(stdout);
            if (!parsed.ok) {
              channel.appendLine(`Parse error: ${parsed.error}`);
              return;
            }
            for (const line of formatReviewLines(parsed.value)) {
              channel.appendLine(line);
            }
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage("Sruja review failed. Is the CLI on PATH or set sruja.lsp.path?");
          }
        }
      );
    }),
  );
}

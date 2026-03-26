import * as vscode from "vscode";
import * as path from "path";
import {
  toFsPathOrUri,
  pickActiveSrujaDoc,
  formatRangeOneBased,
  truncateLines,
} from "./utils";
import { getDiagnosticCodeValue } from "./lintParser";
import { getSkillsRoot, getSkills } from "./skills";
import { getElementsFromWasm, getMermaidFromWasm } from "./wasm";
import { parseJsonSafe } from "./safeJson";

export async function buildContextPack(context: vscode.ExtensionContext): Promise<string> {
  const editor = vscode.window.activeTextEditor;
  const activeDoc = editor?.document;
  const folder = activeDoc ? vscode.workspace.getWorkspaceFolder(activeDoc.uri) : vscode.workspace.workspaceFolders?.[0];
  const rootPath = folder?.uri.fsPath;

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
      const elements = await getElementsFromWasm(context, activeSrujaDoc.getText(), activeSrujaPath, activeSrujaDoc.uri.toString(), activeSrujaDoc.version);
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

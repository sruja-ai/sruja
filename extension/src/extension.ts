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
  resolveDocUri,
  docUriExists,
} from "./providers";
import { exportMarkdownFromWasm, getDiagnosticsFromWasm, getMermaidFromWasm, getElementsFromWasm } from "./wasm";
import { getSrujaLspPath } from "./config";
import { runCli } from "./cliRunner";
import { parseJsonSafe } from "./safeJson";
import { formatStatusLines, formatReviewLines, type StatusJson, type ReviewJson } from "./cliOutput";
import { getDiagramPreviewHtml, escapeMermaidForScript } from "./diagramPreview";
import { SrujaMarkdownPreviewEditorProvider } from "./markdownPreviewEditor";

const DIAGNOSTIC_COLLECTION_ID = "sruja";
let diagnosticCollection: vscode.DiagnosticCollection | undefined;
let markdownPreviewPanel: vscode.WebviewPanel | undefined;
let diagramPreviewPanel: vscode.WebviewPanel | undefined;

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
  return `- [${severity}] ${where} ${msg}`;
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

  const visibleEditors = vscode.window.visibleTextEditors
    .map((e) => e.document.uri)
    .filter((u, idx, arr) => arr.findIndex((x) => x.toString() === u.toString()) === idx);

  const diagEntries = vscode.languages.getDiagnostics();
  const flattened = diagEntries
    .flatMap(([uri, diags]) => diags.map((d) => ({ uri, d })))
    .filter(({ d }) => d.severity === vscode.DiagnosticSeverity.Error || d.severity === vscode.DiagnosticSeverity.Warning);

  const diagLines = flattened.slice(0, 80).map(({ uri, d }) => formatDiagnosticSummary(rootPath, uri, d));
  const diagOmitted = Math.max(0, flattened.length - diagLines.length);

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
  lines.push("## Diagnostics");
  lines.push("");
  if (diagLines.length === 0) {
    lines.push("- none");
  } else {
    lines.push(...diagLines);
    if (diagOmitted > 0) lines.push(`- ... (${diagOmitted} more omitted)`);
  }
  lines.push("");
  lines.push("## Sruja");
  lines.push("");
  lines.push(skillsLine);
  if (contextJsonLine) lines.push(contextJsonLine);
  lines.push("");
  lines.push("## Ask");
  lines.push("");
  lines.push("- What is the root cause?");
  lines.push("- What is the smallest safe change?");
  lines.push("- Show exact files/lines to edit.");
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

  vscode.window.showInformationMessage(`Sruja MCP server registered in ${path.relative(folder.uri.fsPath, cursorConfigUri.fsPath)}.`);
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

  for (const doc of vscode.workspace.textDocuments) {
    if (doc.languageId === "sruja") runLintForDoc(doc);
  }

  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.commandCenter", async () => {
      const items: { label: string; detail?: string; command: string }[] = [
        { label: "Run validation", command: "sruja.runValidation" },
        { label: "Open Diagram Preview", command: "sruja.openDiagramPreview" },
        { label: "Open Focused Diagram Preview", command: "sruja.openFocusedDiagramPreview" },
        { label: "Export architecture to Markdown", command: "sruja.exportMarkdown" },
        { label: "Open Markdown Preview", command: "sruja.openMarkdownPreview" },
        { label: "Open component knowledge", command: "sruja.openComponentKnowledge" },
        { label: "Open Skills Overview", command: "sruja.openSkillsOverview" },
        { label: "Open Agent Guide (AGENTS.md)", command: "sruja.openAgentGuide" },
        { label: "List Rules…", command: "sruja.listRules" },
        { label: "Copy Rule for AI", command: "sruja.copyRuleForAI" },
        { label: "Copy Agent Guide for AI", command: "sruja.copyAgentGuideForAI" },
        { label: "Copy Context Pack for AI", command: "sruja.copyContextPackForAI" },
        { label: "Run drift (architecture health)", command: "sruja.runDrift" },
        { label: "Refresh repo context", command: "sruja.refreshContext" },
        { label: "Status", command: "sruja.status" },
        { label: "Review architecture update", command: "sruja.review" },
      ];
      const pick = isTest
        ? items[0]
        : await vscode.window.showQuickPick(
          items.map((i) => ({ label: i.label, detail: i.detail, item: i })),
          { placeHolder: "Sruja Command Center" }
        ).then((p) => p?.item);
      if (!pick) return;
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

  const codeActionProvider = new SrujaCodeActionProvider();
  context.subscriptions.push(
    vscode.languages.registerCodeActionsProvider("sruja", codeActionProvider, {
      providedCodeActionKinds: [vscode.CodeActionKind.QuickFix],
    })
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
      if (diagramPreviewPanel) {
        diagramPreviewPanel.dispose();
      }
      diagramPreviewPanel = vscode.window.createWebviewPanel(
        "srujaDiagramPreview",
         "Sruja – Context engineering for the AI era. – Diagram Preview",
        vscode.ViewColumn.Beside,
        { enableScripts: true }
      );
      diagramPreviewPanel.onDidDispose(() => {
        diagramPreviewPanel = undefined;
      });
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

      if (diagramPreviewPanel) {
        diagramPreviewPanel.dispose();
      }
      const titleSuffix = targetId ? ` – L${viewLevel}: ${targetId}` : ` – L${viewLevel}`;
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
    vscode.commands.registerCommand("sruja.markdownPreview", openMarkdownPreview),
    vscode.commands.registerCommand("sruja.openMarkdownPreview", openMarkdownPreview),
    vscode.commands.registerCommand("sruja.registerMcpServer", registerMcpServer),
    vscode.commands.registerCommand("sruja.runDrift", async () => {
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
    }),
    vscode.commands.registerCommand("sruja.refreshContext", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      if (!folder) {
        vscode.window.showWarningMessage("Open a workspace folder to refresh repo context.");
        return;
      }
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
    }),
    vscode.commands.registerCommand("sruja.status", async () => {
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
    }),
    vscode.commands.registerCommand("sruja.review", async () => {
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
    }),
  );
}

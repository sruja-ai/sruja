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

/** Output channel for architecture intelligence (drift, sync, status, review). */
let cliOutputChannel: vscode.OutputChannel | undefined;

function getCliOutputChannel(): vscode.OutputChannel {
  if (!cliOutputChannel) {
    cliOutputChannel = vscode.window.createOutputChannel("Sruja");
  }
  return cliOutputChannel;
}

/** Run a Sruja CLI command in the workspace root. Throws if no folder open. */
async function runCliInWorkspace(args: string[]): Promise<{ stdout: string; stderr: string; code: number }> {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) {
    throw new Error("No workspace folder open. Open a folder to run architecture intelligence.");
  }
  return runCli(getSrujaPath(), args, folder.uri.fsPath);
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
         "Sruja – Architecture intelligence for the AI era. – Diagram Preview",
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
        "Sruja – Architecture intelligence for the AI era. – Diagram Preview" + titleSuffix,
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
        "Sruja – Architecture intelligence for the AI era. – Diagram Preview" + titleSuffix,
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

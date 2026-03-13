import { execFile } from "child_process";
import { promisify } from "util";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as vscode from "vscode";

import { getSkills, getSkillsRoot } from "./skills";
import { SrujaSkillsTreeProvider } from "./skillsTree";
import { SrujaDefinitionProvider, SrujaHoverProvider, SrujaDocumentSymbolProvider } from "./providers";
import { exportMarkdownFromWasm, getDiagnosticsFromWasm, getMermaidFromWasm } from "./wasm";

const execFileAsync = promisify(execFile);

const DIAGNOSTIC_COLLECTION_ID = "sruja";
let diagnosticCollection: vscode.DiagnosticCollection | undefined;

/** Parse "sruja lint" stderr into VS Code diagnostics. Format:
 * [CODE] Error: message
 *   --> file:line:column
 */
function parseLintStderr(stderr: string, docUri: string): vscode.Diagnostic[] {
  const diagnostics: vscode.Diagnostic[] = [];
  const lines = stderr.split(/\r?\n/);
  const msgRe = /^\[([^\]]+)\]\s+(Error|Warning|Info):\s+(.+)$/;
  const locRe = /^\s+-->\s+(.+):(\d+):(\d+)$/;

  for (let i = 0; i < lines.length; i++) {
    const locMatch = lines[i].match(locRe);
    if (!locMatch) continue;
    const [, filePart, lineStr, colStr] = locMatch;
    const line = Math.max(0, parseInt(lineStr, 10) - 1);
    const character = Math.max(0, parseInt(colStr, 10) - 1);

    let code: string | undefined;
    let severity: vscode.DiagnosticSeverity = vscode.DiagnosticSeverity.Error;
    let message = "Validation error";

    if (i > 0) {
      const msgMatch = lines[i - 1].match(msgRe);
      if (msgMatch) {
        code = msgMatch[1];
        const sev = msgMatch[2];
        message = msgMatch[3].trim();
        if (sev === "Warning") severity = vscode.DiagnosticSeverity.Warning;
        else if (sev === "Info") severity = vscode.DiagnosticSeverity.Information;
      }
    }

    const range = new vscode.Range(line, character, line, character);
    const diag = new vscode.Diagnostic(range, message, severity);
    if (code) diag.code = code;
    diag.source = "sruja";
    diagnostics.push(diag);
  }

  return diagnostics;
}

/** Lint output when using --format json (see docs/LINT_JSON_OUTPUT.md). */
interface LintJsonOutput {
  ok: boolean;
  error_count: number;
  warning_count: number;
  diagnostics: Array<{
    code: string;
    severity: string;
    message: string;
    location?: { file: string; line: number; column: number };
  }>;
}

async function runLint(
  srujaPath: string,
  filePath: string
): Promise<{ stdout: string; stderr: string }> {
  return new Promise((resolve) => {
    execFile(
      srujaPath,
      ["lint", filePath],
      { encoding: "utf8", timeout: 15000, maxBuffer: 2 * 1024 * 1024 },
      (_err: Error | null, stdout: string, stderr: string) => {
        resolve({
          stdout: typeof stdout === "string" ? stdout : "",
          stderr: typeof stderr === "string" ? stderr : "",
        });
      }
    );
  });
}

/** Run lint with --format json for reliable diagnostics (DX + extraction quality). */
async function runLintJson(
  srujaPath: string,
  filePath: string
): Promise<{ stdout: string; stderr: string }> {
  return new Promise((resolve) => {
    execFile(
      srujaPath,
      ["lint", "--format", "json", filePath],
      { encoding: "utf8", timeout: 15000, maxBuffer: 2 * 1024 * 1024 },
      (_err: Error | null, stdout: string, stderr: string) => {
        resolve({
          stdout: typeof stdout === "string" ? stdout : "",
          stderr: typeof stderr === "string" ? stderr : "",
        });
      }
    );
  });
}

function parseLintJson(stdout: string, docUri: string): vscode.Diagnostic[] | null {
  try {
    const out = JSON.parse(stdout) as LintJsonOutput;
    if (!out.diagnostics || !Array.isArray(out.diagnostics)) return null;
    const diagnostics: vscode.Diagnostic[] = [];
    for (const d of out.diagnostics) {
      const line = d.location ? Math.max(0, (d.location.line || 1) - 1) : 0;
      const character = d.location ? Math.max(0, (d.location.column || 1) - 1) : 0;
      const severity =
        d.severity === "warning"
          ? vscode.DiagnosticSeverity.Warning
          : d.severity === "info"
            ? vscode.DiagnosticSeverity.Information
            : vscode.DiagnosticSeverity.Error;
      const range = new vscode.Range(line, character, line, character);
      const diag = new vscode.Diagnostic(range, d.message, severity);
      if (d.code) diag.code = d.code;
      diag.source = "sruja";
      diagnostics.push(diag);
    }
    return diagnostics;
  } catch {
    return null;
  }
}

/** Use WASM for lint/export unless user explicitly set sruja.lsp.path. WASM is always shipped with the extension. */
function useWasm(context: vscode.ExtensionContext): boolean {
  const config = vscode.workspace.getConfiguration("sruja").get<string>("lsp.path");
  return !config?.trim();
}

async function updateDiagnostics(
  context: vscode.ExtensionContext,
  doc: vscode.TextDocument
): Promise<void> {
  if (doc.languageId !== "sruja" || doc.uri.scheme !== "file") return;
  if (!diagnosticCollection) return;

  const uri = doc.uri;
  const filename = doc.uri.fsPath;

  if (useWasm(context)) {
    try {
      const diags = await getDiagnosticsFromWasm(context, doc.getText(), filename);
      diagnosticCollection.set(uri, diags);
    } catch {
      diagnosticCollection.set(uri, []);
    }
    return;
  }

  const srujaPath = getSrujaPath(context);
  const targetPath = doc.isDirty
    ? path.join(os.tmpdir(), `sruja-lint-${path.basename(filename)}`)
    : filename;
  if (doc.isDirty) {
    await fs.promises.writeFile(targetPath, doc.getText(), "utf8");
  }
  try {
    const { stdout, stderr } = await runLintJson(srujaPath, targetPath);
    const diagsFromJson = parseLintJson(stdout, uri.toString());
    const diags =
      diagsFromJson !== null
        ? diagsFromJson
        : parseLintStderr(stderr, uri.toString());
    diagnosticCollection.set(uri, diags);
  } finally {
    if (doc.isDirty) {
      await fs.promises.unlink(targetPath).catch(() => {});
    }
  }
}

function getSrujaPath(context: vscode.ExtensionContext): string {
  const config = vscode.workspace.getConfiguration("sruja").get<string>("lsp.path");
  if (config?.trim()) return config.trim();
  return "sruja";
}

/** Output channel for architecture intelligence (drift, analyze, why). */
let cliOutputChannel: vscode.OutputChannel | undefined;

function getCliOutputChannel(): vscode.OutputChannel {
  if (!cliOutputChannel) {
    cliOutputChannel = vscode.window.createOutputChannel("Sruja");
  }
  return cliOutputChannel;
}

/**
 * Run a Sruja CLI command in the workspace root. Architecture intelligence commands
 * (drift, analyze, why) require the CLI; they have no WASM equivalent.
 */
async function runCliInWorkspace(
  context: vscode.ExtensionContext,
  args: string[]
): Promise<{ stdout: string; stderr: string; code: number }> {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) {
    throw new Error("No workspace folder open. Open a folder to run architecture intelligence.");
  }
  const srujaPath = getSrujaPath(context);
  const cwd = folder.uri.fsPath;
  return new Promise((resolve) => {
    execFile(
      srujaPath,
      args,
      { encoding: "utf8", cwd, timeout: 120000, maxBuffer: 4 * 1024 * 1024 },
      (err: Error | null, stdout: string, stderr: string) => {
        resolve({
          stdout: typeof stdout === "string" ? stdout : "",
          stderr: typeof stderr === "string" ? stderr : "",
          code: err ? 1 : 0,
        });
      }
    );
  });
}

export function activate(context: vscode.ExtensionContext): void {
  diagnosticCollection = vscode.languages.createDiagnosticCollection(DIAGNOSTIC_COLLECTION_ID);
  context.subscriptions.push(diagnosticCollection);

  const runLintForDoc = (doc: vscode.TextDocument) => {
    if (doc.languageId !== "sruja") return;
    updateDiagnostics(context, doc).catch((err) => {
      if (diagnosticCollection && doc.uri) {
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
      const skill = skills.length === 1 ? skills[0] : await vscode.window.showQuickPick(
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
      const skill = withAgents.length === 1 ? withAgents[0] : await vscode.window.showQuickPick(
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
      const pick = await vscode.window.showQuickPick(
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
      const pick = await vscode.window.showQuickPick(
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
      const skill = withAgents.length === 1 ? withAgents[0] : await vscode.window.showQuickPick(
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

      let stdout: string | null = null;
      if (useWasm(context)) {
        stdout = await exportMarkdownFromWasm(context, dsl);
        if (stdout === null) {
          vscode.window.showErrorMessage(
            "Sruja WASM could not load. Reinstall the extension or set sruja.lsp.path to use the Sruja CLI."
          );
          return;
        }
      } else {
        const cliPath = getSrujaPath(context);
        let inputPath = filePath;
        let tmpPath: string | null = null;
        if (doc.isDirty || doc.uri.scheme !== "file") {
          tmpPath = path.join(os.tmpdir(), `sruja-export-${path.basename(filePath)}`);
          await fs.promises.writeFile(tmpPath, dsl, "utf8");
          inputPath = tmpPath;
        }
        try {
          const result = await execFileAsync(cliPath, ["export", "markdown", inputPath], {
            encoding: "utf8",
          });
          const out = Array.isArray(result) ? result[0] : (result as { stdout?: string }).stdout;
          stdout = out ?? "";
        } finally {
          if (tmpPath) await fs.promises.unlink(tmpPath).catch(() => {});
        }
      }

      if (stdout === null || stdout === undefined) {
        vscode.window.showErrorMessage("Export to Markdown failed.");
        return;
      }
      try {
        const mdDoc = await vscode.workspace.openTextDocument({
          content: stdout,
          language: "markdown",
        });
        await vscode.window.showTextDocument(mdDoc, { preview: false });
        // Open the Markdown preview so the user sees the full rendered document (not only the diagram).
        await vscode.commands.executeCommand("markdown.showPreview", mdDoc.uri);
        const save = await vscode.window.showInformationMessage(
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
    vscode.commands.registerCommand("sruja.openDiagramPreview", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to open diagram preview.");
        return;
      }
      const dsl = doc.getText();
      if (!useWasm(context)) {
        vscode.window.showInformationMessage(
          "Diagram preview uses bundled WASM. Clear sruja.lsp.path to use it, or use Sruja: Export to Markdown."
        );
        return;
      }
      const mermaid = await getMermaidFromWasm(context, dsl);
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          "Sruja WASM could not load or diagram is empty. Run npm run copy:assets if developing, or reinstall the extension."
        );
        return;
      }
      const panel = vscode.window.createWebviewPanel(
        "srujaDiagramPreview",
        "Sruja Diagram Preview",
        vscode.ViewColumn.Beside,
        { enableScripts: true }
      );
      const mermaidEscaped = mermaid
        .replace(/\\/g, "\\\\")
        .replace(/`/g, "\\`")
        .replace(/\$/g, "\\$")
        .replace(/<\/script>/gi, "<\\/script>");
      panel.webview.html = getDiagramPreviewHtml(mermaidEscaped);
    }),
    vscode.commands.registerCommand("sruja.runDrift", async () => {
      const channel = getCliOutputChannel();
      channel.clear();
      channel.show(true);
      channel.appendLine("Running sruja drift -r . ...");
      try {
        const { stdout, stderr, code } = await runCliInWorkspace(context, ["drift", "-r", "."]);
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
    vscode.commands.registerCommand("sruja.analyzeRepo", async () => {
      const channel = getCliOutputChannel();
      channel.clear();
      channel.show(true);
      channel.appendLine("Running sruja analyze -r . ...");
      try {
        const { stdout, stderr, code } = await runCliInWorkspace(context, ["analyze", "-r", "."]);
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
          "Sruja analyze failed. Ensure the Sruja CLI is installed and on PATH, or set sruja.lsp.path."
        );
      }
    }),
    vscode.commands.registerCommand("sruja.whyComponent", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      if (!folder) {
        vscode.window.showWarningMessage("Open a workspace folder to run Sruja Why.");
        return;
      }
      const question = await vscode.window.showInputBox({
        title: "Sruja: Why",
        prompt: "Ask about a component or dependency (e.g. component name or 'why does X depend on Y?')",
        placeHolder: "e.g. api_gateway or why does order_service depend on payment_service?",
      });
      if (question === undefined || question.trim() === "") return;
      const channel = getCliOutputChannel();
      channel.clear();
      channel.show(true);
      channel.appendLine(`Running sruja why "${question.trim()}" -r . ...`);
      try {
        const { stdout, stderr, code } = await runCliInWorkspace(context, [
          "why",
          question.trim(),
          "-r",
          ".",
        ]);
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
          "Sruja why failed. Ensure the Sruja CLI is installed and on PATH, or set sruja.lsp.path."
        );
      }
    })
  );
}

function getDiagramPreviewHtml(mermaidCodeEscaped: string): string {
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline' https://cdn.jsdelivr.net; style-src 'unsafe-inline' https://cdn.jsdelivr.net;">
  <title>Sruja Diagram Preview</title>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
</head>
<body>
  <div id="diagram" class="mermaid"></div>
  <script>
    (function() {
      var code = \`${mermaidCodeEscaped}\`;
      var el = document.getElementById('diagram');
      el.textContent = code;
      mermaid.initialize({ startOnLoad: false });
      mermaid.run({ nodes: [el] }).catch(function(err) {
        el.innerHTML = '<p style="color:#c00;font-family:sans-serif;">' + (err.message || String(err)) + '</p>';
      });
    })();
  </script>
</body>
</html>`;
}

export function deactivate(): void {
  diagnosticCollection?.dispose();
  diagnosticCollection = undefined;
}

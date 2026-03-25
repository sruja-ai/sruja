import * as vscode from "vscode";

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
} from "./providers";
import {
  getDiagnosticsFromWasm,
} from "./wasm";
import { getSrujaLspPath } from "./config";
import { SrujaMarkdownPreviewEditorProvider } from "./markdownPreviewEditor";
import { registerDiagramCommands } from "./commands/diagrams";
import { registerContextEngineeringCommands } from "./commands/contextEngineering";
import { registerSkillCommands } from "./commands/skills";
import { registerKnowledgeCommands, pushDocsThreadEntryFromActiveEditor } from "./commands/knowledge";
import { buildContextPack } from "./contextPack";
import { registerMcpServer } from "./mcpServer";

let diagnosticCollection: vscode.DiagnosticCollection | undefined;

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

export function activate(context: vscode.ExtensionContext) {
  const isTest = context.extensionMode === vscode.ExtensionMode.Test;

  diagnosticCollection = vscode.languages.createDiagnosticCollection("sruja");
  context.subscriptions.push(diagnosticCollection);

  // Register Modular Commands
  registerDiagramCommands(context, isTest);
  registerContextEngineeringCommands(context, getSrujaPath);
  registerSkillCommands(context, isTest);
  registerKnowledgeCommands(context);

  // Register Language Providers
  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider({ language: "sruja" }, new SrujaDefinitionProvider(context)),
    vscode.languages.registerHoverProvider({ language: "sruja" }, new SrujaHoverProvider(context)),
    vscode.languages.registerDocumentSymbolProvider({ language: "sruja" }, new SrujaDocumentSymbolProvider(context)),
    vscode.languages.registerCodeLensProvider({ language: "sruja" }, new SrujaDiagramCodeLensProvider(context)),
    vscode.languages.registerCodeActionsProvider({ language: "sruja" }, new SrujaCodeActionProvider(context)),
    vscode.languages.registerCompletionItemProvider({ language: "sruja" }, new SrujaCompletionItemProvider(context), " ", ">", "=", "."),
    vscode.languages.registerRenameProvider({ language: "sruja" }, new SrujaRenameProvider(context)),
    vscode.languages.registerReferenceProvider({ language: "sruja" }, new SrujaReferenceProvider(context)),
    vscode.languages.registerDocumentFormattingEditProvider({ language: "sruja" }, new SrujaDocumentFormattingEditProvider())
  );

  // Custom Editor Provider (Markdown Preview)
  context.subscriptions.push(
    vscode.window.registerCustomEditorProvider(
      SrujaMarkdownPreviewEditorProvider.viewType,
      new SrujaMarkdownPreviewEditorProvider(context)
    )
  );

  // Skills Tree View
  const skillsTreeProvider = new SrujaSkillsTreeProvider(context);
  vscode.window.registerTreeDataProvider("srujaSkills", skillsTreeProvider);
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.refreshSkills", () => skillsTreeProvider.refresh())
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

  // Remaining commands
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.registerMcpServer", () => registerMcpServer(getSrujaPath)),
    vscode.commands.registerCommand("sruja.copyContextPackForAI", async () => {
      try {
        const text = await buildContextPack(context);
        await vscode.env.clipboard.writeText(text);
        vscode.window.showInformationMessage(`Copied context pack to clipboard (${text.length} chars).`);
      } catch (err) {
        vscode.window.showErrorMessage(`Failed to build context pack: ${err instanceof Error ? err.message : String(err)}`);
      }
    }),
    vscode.commands.registerCommand("sruja.markdownPreview", openMarkdownPreview),
    vscode.commands.registerCommand("sruja.openMarkdownPreview", openMarkdownPreview)
  );

  // Event handlers
  vscode.workspace.onDidOpenTextDocument((doc) => updateDiagnostics(context, doc), null, context.subscriptions);
  vscode.workspace.onDidChangeTextDocument((e) => updateDiagnostics(context, e.document), null, context.subscriptions);
  vscode.workspace.onDidSaveTextDocument((doc) => updateDiagnostics(context, doc), null, context.subscriptions);

  // Docs thread sync on selection change
  vscode.window.onDidChangeTextEditorSelection(
    async (e) => {
      if (e.textEditor.document.languageId === "sruja") {
        await pushDocsThreadEntryFromActiveEditor(context);
      }
    },
    null,
    context.subscriptions
  );

  // Initial diagnostics
  for (const doc of vscode.workspace.textDocuments) {
    if (doc.languageId === "sruja") {
      updateDiagnostics(context, doc);
    }
  }
}

export function deactivate() {}

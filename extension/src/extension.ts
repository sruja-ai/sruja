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
import { getSrujaLspPath } from "./config";
import { SrujaMarkdownPreviewEditorProvider } from "./markdownPreviewEditor";
import { registerDiagramCommands } from "./commands/diagrams";
import { registerExplorerCommands } from "./architectureExplorer";
import { registerContextEngineeringCommands } from "./commands/contextEngineering";
import { registerSkillCommands } from "./commands/skills";
import { registerKnowledgeCommands, pushDocsThreadEntryFromActiveEditor } from "./commands/knowledge";
import { registerCommandCenter } from "./commands/commandCenter";
import { registerValidationCommands } from "./commands/validation";
import { registerExportCommands } from "./commands/export";
import { registerHumanCommands } from "./commands/human";
import { buildContextPack } from "./contextPack";
import { registerMcpServer } from "./mcpServer";
import { updateDiagnostics, getDiagnosticCollection } from "./diagnostics";
import { registerStatusBar } from "./statusBar";
import { ArchitectureTreeProvider } from "./architectureTree";
import { AgentRunsProvider, openAgentDiff } from "./agentRuns";
import { maybeShowWelcome, registerWelcomeCommand } from "./welcome";

function getLspPath(): string | undefined {
  return vscode.workspace.getConfiguration("sruja").get<string>("lsp.path");
}

function getSrujaPath(): string {
  return getSrujaLspPath(getLspPath());
}

export function activate(context: vscode.ExtensionContext) {
  const isTest = context.extensionMode === vscode.ExtensionMode.Test;

  const diagnosticCollection = getDiagnosticCollection();
  context.subscriptions.push(diagnosticCollection);

  // Register Modular Commands
  registerDiagramCommands(context, isTest);
  registerExplorerCommands(context);
  registerContextEngineeringCommands(context, getSrujaPath);
  registerSkillCommands(context, isTest);
  registerKnowledgeCommands(context);
  registerCommandCenter(context);
  registerValidationCommands(context, updateDiagnostics);
  registerExportCommands(context);
  registerHumanCommands(context, getSrujaPath);

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

  // Status Bar
  registerStatusBar(context, getSrujaPath);

  // Skills Tree View
  const skillsTreeProvider = new SrujaSkillsTreeProvider(context);
  vscode.window.registerTreeDataProvider("srujaSkills", skillsTreeProvider);
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.refreshSkills", () => skillsTreeProvider.refresh())
  );

  // Architecture Tree View
  const archTreeProvider = new ArchitectureTreeProvider(context);
  vscode.window.registerTreeDataProvider("srujaArchitecture", archTreeProvider);
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.refreshStatus", () => archTreeProvider.refresh())
  );

  // Agent Runs Tree View
  const agentRunsProvider = new AgentRunsProvider();
  vscode.window.registerTreeDataProvider("srujaAgentRuns", agentRunsProvider);
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.refreshAgentRuns", () => agentRunsProvider.refresh()),
    vscode.commands.registerCommand("sruja.openAgentDiff", openAgentDiff)
  );

  // Welcome walkthrough
  registerWelcomeCommand(context);
  maybeShowWelcome(context);

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

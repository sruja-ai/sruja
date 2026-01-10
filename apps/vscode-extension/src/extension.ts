// apps/vscode-extension/src/extension.ts
import * as vscode from "vscode";
import * as fs from "fs";
import { SrujaPreviewProvider } from "./previewProvider";
import { initializeWasmLsp, debugWasmLsp } from "./wasmLspProvider";

const STATUS_BAR_PRIORITY = 100;
const STATUS_BAR_ALIGNMENT = vscode.StatusBarAlignment.Right;

let statusBarItem: vscode.StatusBarItem;
let previewProvider: SrujaPreviewProvider;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext): void {
  try {
    outputChannel = vscode.window.createOutputChannel("Sruja");
    outputChannel.appendLine("Sruja extension activating...");

    const isFirstActivation = !context.globalState.get("sruja.hasActivated", false);
    if (isFirstActivation) {
      context.globalState.update("sruja.hasActivated", true);
      showWelcomeMessage(context);
    }

    statusBarItem = vscode.window.createStatusBarItem(STATUS_BAR_ALIGNMENT, STATUS_BAR_PRIORITY);
    statusBarItem.command = "sruja.previewArchitecture";
    updateStatusBar("Initializing...", "$(sync~spin)");
    statusBarItem.show();

    previewProvider = new SrujaPreviewProvider(context);
    const registration = vscode.workspace.registerTextDocumentContentProvider(
      SrujaPreviewProvider.scheme,
      previewProvider
    );
    context.subscriptions.push(registration);

    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.languageId === "sruja") {
        const previewUri = getPreviewUri(doc.uri);
        previewProvider.update(previewUri);
      }
    });

    const previewCmd = vscode.commands.registerCommand("sruja.previewArchitecture", () =>
      previewArchitecture(context)
    );
    const debugCmd = vscode.commands.registerCommand("sruja.debugWasmLsp", () => debugWasmLsp());
    const updateStatusCmd = vscode.commands.registerCommand("sruja.updateStatusBar", () =>
      updateStatusBarWithDiagnostics()
    );

    context.subscriptions.push(previewCmd, debugCmd, updateStatusCmd);

    vscode.window.onDidChangeActiveTextEditor(() => {
      updateStatusBarWithDiagnostics();
    });

    outputChannel.appendLine("Sruja extension commands registered");

    vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: "Initializing Sruja Language Server",
        cancellable: false,
      },
      async (progress) => {
        progress.report({ increment: 0, message: "Loading WASM module..." });

        try {
          await initializeWasmLsp(context);
          progress.report({ increment: 100, message: "Ready!" });

          outputChannel.appendLine("WASM LSP initialization completed");
          updateStatusBar("Ready", "$(check)");
          updateStatusBarWithDiagnostics();
        } catch (error: unknown) {
          progress.report({ increment: 100, message: "Initialization failed" });
          const errMsg = error instanceof Error ? error.message : String(error);
          const stack = error instanceof Error ? error.stack : undefined;
          outputChannel.appendLine(`Failed to initialize WASM LSP: ${errMsg}`);
          if (stack) {
            outputChannel.appendLine(`Stack: ${stack}`);
          }
          updateStatusBar("Error", "$(error)");
          vscode.window.showErrorMessage(
            `Failed to initialize Sruja Language Server: ${errMsg}. Check "Sruja" output channel for details.`
          );
        }
      }
    );
  } catch (e) {
    outputChannel?.appendLine(`Failed to activate Sruja extension: ${String(e)}`);
    console.error("Failed to activate Sruja extension:", e);
    vscode.window.showErrorMessage(`Sruja extension activation failed: ${e}`);
  }
}

export function deactivate(): Thenable<void> | undefined {
  statusBarItem?.hide();
  return undefined;
}

function updateStatusBar(text: string, icon?: string): void {
  if (!statusBarItem) return;
  statusBarItem.text = icon ? `${icon} Sruja: ${text}` : `Sruja: ${text}`;
  if (text.includes("Error")) {
    statusBarItem.backgroundColor = new vscode.ThemeColor("statusBarItem.errorBackground");
  } else {
    statusBarItem.backgroundColor = undefined;
  }
  statusBarItem.tooltip = `Sruja Language Support - ${text}`;
}

export function updateStatusBarWithDiagnostics(): void {
  if (!statusBarItem) return;

  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "sruja") {
    return;
  }

  const diagnostics = vscode.languages.getDiagnostics(editor.document.uri);
  const errorCount = diagnostics.filter(
    (d) => d.severity === vscode.DiagnosticSeverity.Error
  ).length;
  const warningCount = diagnostics.filter(
    (d) => d.severity === vscode.DiagnosticSeverity.Warning
  ).length;

  if (errorCount > 0) {
    updateStatusBar(`${errorCount} error${errorCount !== 1 ? "s" : ""}`, "$(error)");
  } else if (warningCount > 0) {
    updateStatusBar(`${warningCount} warning${warningCount !== 1 ? "s" : ""}`, "$(warning)");
  } else {
    updateStatusBar("Ready", "$(check)");
  }
}

function getPreviewUri(uri: vscode.Uri): vscode.Uri {
  const query = `original=${uri.fsPath}`;
  return vscode.Uri.parse(`${SrujaPreviewProvider.scheme}:${uri.path}.md?${query}`);
}

async function previewArchitecture(_context: vscode.ExtensionContext): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage("No active editor");
    return;
  }

  const doc = editor.document;
  if (doc.languageId !== "sruja") {
    vscode.window.showErrorMessage("Preview works only for Sruja (.sruja) files");
    return;
  }

  if (doc.isUntitled) {
    vscode.window.showErrorMessage("Please save the file before previewing");
    return;
  }

  if (doc.isDirty) {
    const saveFirst = await vscode.window.showWarningMessage(
      "The file has unsaved changes. Save before previewing?",
      "Save",
      "Cancel"
    );
    if (saveFirst === "Save") {
      await doc.save();
    } else {
      return;
    }
  }

  if (!fs.existsSync(doc.uri.fsPath)) {
    vscode.window.showErrorMessage("File not found. Please save the file first.");
    return;
  }

  const previewUri = getPreviewUri(doc.uri);

  try {
    if (previewProvider) {
      previewProvider.update(previewUri);
    }

    await vscode.commands.executeCommand("markdown.showPreview", previewUri);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    outputChannel?.appendLine(`Failed to open preview: ${errorMessage}`);
    vscode.window.showErrorMessage(`Failed to open preview: ${errorMessage}`);
  }
}

async function showWelcomeMessage(_context: vscode.ExtensionContext): Promise<void> {
  const action = await vscode.window.showInformationMessage(
    "Welcome to Sruja! 🎉",
    "Get Started",
    "View Documentation",
    "Dismiss"
  );

  if (action === "Get Started") {
    const sampleContent = `architecture "My Architecture" {
  person User "User"

  system MySystem "My System" {
    container API "API" {
      technology "Node.js"
    }
  }

  User -> MySystem.API "uses"
}`;

    const doc = await vscode.workspace.openTextDocument({
      language: "sruja",
      content: sampleContent,
    });
    await vscode.window.showTextDocument(doc);

    vscode.window.showInformationMessage(
      "Try hovering over symbols, using auto-completion (Ctrl+Space), or right-click to preview!"
    );
  } else if (action === "View Documentation") {
    vscode.env.openExternal(vscode.Uri.parse("https://sruja.ai/docs"));
  }
}

// apps/vscode-extension/src/extension.ts
import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import { SrujaPreviewProvider } from "./previewProvider";
import { initializeWasmLsp, debugWasmLsp } from "./wasmLspProvider";

let statusBarItem: vscode.StatusBarItem;
let previewProvider: SrujaPreviewProvider;

export function activate(context: vscode.ExtensionContext) {
  try {
    console.info("Sruja extension activating...");

    // Check if this is first activation
    const isFirstActivation = !context.globalState.get("sruja.hasActivated", false);
    if (isFirstActivation) {
      context.globalState.update("sruja.hasActivated", true);
      showWelcomeMessage(context);
    }

    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = "sruja.previewArchitecture";
    updateStatusBar("Initializing...", "$(sync~spin)");
    statusBarItem.show();

    previewProvider = new SrujaPreviewProvider(context);
    const registration = vscode.workspace.registerTextDocumentContentProvider(
      SrujaPreviewProvider.scheme,
      previewProvider
    );
    context.subscriptions.push(registration);

    // Update preview on save
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

    // Listen for active editor changes to update status bar
    vscode.window.onDidChangeActiveTextEditor(() => {
      updateStatusBarWithDiagnostics();
    });

    console.info("Sruja extension commands registered");

    // Initialize WASM-based LSP (no CLI dependency)
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

          console.info("WASM LSP initialization completed");
          updateStatusBar("Ready", "$(check)");
          // Update status bar with initial diagnostics
          updateStatusBarWithDiagnostics();
        } catch (error: unknown) {
          progress.report({ increment: 100, message: "Initialization failed" });
          const errMsg = error instanceof Error ? error.message : String(error);
          const stack = error instanceof Error ? error.stack : undefined;
          console.error("Failed to initialize WASM LSP:", errMsg);
          if (stack) {
            console.error("Stack:", stack);
          }
          updateStatusBar("Error", "$(error)");
          vscode.window.showErrorMessage(
            `Failed to initialize Sruja Language Server: ${errMsg}. Check "Sruja WASM LSP" output channel for details.`
          );
        }
      }
    );
  } catch (e) {
    console.error("Failed to activate Sruja extension:", e);
    vscode.window.showErrorMessage(`Sruja extension activation failed: ${e}`);
  }
}

export function deactivate(): Thenable<void> | undefined {
  statusBarItem?.hide();
  return undefined;
}

function updateStatusBar(text: string, icon?: string) {
  if (!statusBarItem) return;
  statusBarItem.text = icon ? `${icon} Sruja: ${text}` : `Sruja: ${text}`;
  if (text.includes("Error")) {
    statusBarItem.backgroundColor = new vscode.ThemeColor("statusBarItem.errorBackground");
  } else {
    statusBarItem.backgroundColor = undefined;
  }
  statusBarItem.tooltip = `Sruja Language Support - ${text}`;
}

// Update status bar with diagnostics count
export function updateStatusBarWithDiagnostics() {
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

async function previewArchitecture(context: vscode.ExtensionContext) {
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

  // Handle unsaved files - save first if dirty
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

  // Verify file exists
  if (!fs.existsSync(doc.uri.fsPath)) {
    vscode.window.showErrorMessage("File not found. Please save the file first.");
    return;
  }

  const previewUri = getPreviewUri(doc.uri);

  try {
    // Trigger preview update to refresh content
    if (previewProvider) {
      previewProvider.update(previewUri);
    }

    await vscode.commands.executeCommand("markdown.showPreview", previewUri);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    vscode.window.showErrorMessage(`Failed to open preview: ${errorMessage}`);
  }
}

async function showWelcomeMessage(context: vscode.ExtensionContext) {
  const action = await vscode.window.showInformationMessage(
    "Welcome to Sruja DSL Language Support! 🎉",
    "Get Started",
    "View Documentation",
    "Dismiss"
  );

  if (action === "Get Started") {
    // Open a sample file or show quick start
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

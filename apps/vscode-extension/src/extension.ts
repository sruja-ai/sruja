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
    console.log("Sruja extension activating...");

    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = "sruja.previewArchitecture";
    updateStatusBar("Ready", "$(check)");
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

    context.subscriptions.push(previewCmd, debugCmd);

    console.log("Sruja extension commands registered");

    // Initialize WASM-based LSP (no CLI dependency)
    initializeWasmLsp(context)
      .then(() => {
        console.log("WASM LSP initialization completed");
        updateStatusBar("Ready", "$(check)");
      })
      .catch((error) => {
        const errMsg = error instanceof Error ? error.message : String(error);
        const stack = error instanceof Error ? error.stack : undefined;
        console.error("Failed to initialize WASM LSP:", errMsg);
        if (stack) {
          console.error("Stack:", stack);
        }
        updateStatusBar("Error", "$(error)");
        vscode.window.showErrorMessage(
          `Failed to initialize WASM LSP: ${errMsg}. Check "Sruja WASM LSP" output channel for details.`
        );
      });
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

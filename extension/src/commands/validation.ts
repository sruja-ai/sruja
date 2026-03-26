import * as vscode from "vscode";

/**
 * Implementation of sruja.runValidation command.
 * This triggers a fresh diagnostic update for the active Sruja document.
 */
export function registerValidationCommands(context: vscode.ExtensionContext, updateDiagnostics: (context: vscode.ExtensionContext, doc: vscode.TextDocument) => void) {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.runValidation", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to run validation.");
        return;
      }

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: "Sruja: Validating...",
          cancellable: false,
        },
        async () => {
          updateDiagnostics(context, editor.document);
        }
      );
    })
  );
}

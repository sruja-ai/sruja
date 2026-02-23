import * as vscode from "vscode";

import { updateDiagnostics } from "../diagnostics";

export interface ValidationDeps {
  diagnosticCollection: vscode.DiagnosticCollection;
  pendingLint: Map<string, ReturnType<typeof setTimeout>>;
}

export function registerRunValidation(
  context: vscode.ExtensionContext,
  deps: ValidationDeps
): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.runValidation", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to run validation.");
        return;
      }
      const key = doc.uri.toString();
      const pending = deps.pendingLint.get(key);
      if (pending) {
        clearTimeout(pending);
        deps.pendingLint.delete(key);
      }
      try {
        await updateDiagnostics(context, doc, deps.diagnosticCollection);
        const diags = deps.diagnosticCollection.get(doc.uri) ?? [];
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
}

import * as vscode from "vscode";
import { getDiagnosticsFromWasm } from "./wasm";
import { applyGutterDecorations, clearGutterDecorations } from "./decorations";

let diagnosticCollection: vscode.DiagnosticCollection | undefined;

export function getDiagnosticCollection(): vscode.DiagnosticCollection {
  if (!diagnosticCollection) {
    diagnosticCollection = vscode.languages.createDiagnosticCollection("sruja");
  }
  return diagnosticCollection;
}

/** Return true if the document is still open in the workspace. */
function isDocumentStillOpen(uri: vscode.Uri): boolean {
  return vscode.workspace.textDocuments.some((d) => d.uri.toString() === uri.toString());
}

export async function updateDiagnostics(
  context: vscode.ExtensionContext,
  doc: vscode.TextDocument
): Promise<void> {
  if (doc.languageId !== "sruja" || doc.uri.scheme !== "file") return;
  const collection = getDiagnosticCollection();

  const uri = doc.uri;
  const filename = doc.uri.fsPath;

  const setDiagsIfDocOpen = (diags: vscode.Diagnostic[]) => {
    if (isDocumentStillOpen(uri)) {
      collection.set(uri, diags);
      // Apply gutter decorations to visible editors showing this document
      for (const editor of vscode.window.visibleTextEditors) {
        if (editor.document.uri.toString() === uri.toString()) {
          applyGutterDecorations(editor, diags);
        }
      }
    }
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

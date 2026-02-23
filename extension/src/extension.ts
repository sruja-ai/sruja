import * as vscode from "vscode";

import { registerRunValidation, registerSkillsCommands, registerExportCommands } from "./commands";
import { updateDiagnostics } from "./diagnostics";
import { SrujaDefinitionProvider, SrujaHoverProvider, SrujaDocumentSymbolProvider } from "./providers";
import { SrujaSkillsTreeProvider } from "./skillsTree";

const DIAGNOSTIC_COLLECTION_ID = "sruja";
let diagnosticCollection: vscode.DiagnosticCollection | undefined;

export function activate(context: vscode.ExtensionContext): void {
  diagnosticCollection = vscode.languages.createDiagnosticCollection(DIAGNOSTIC_COLLECTION_ID);
  context.subscriptions.push(diagnosticCollection);

  const pendingLint = new Map<string, ReturnType<typeof setTimeout>>();

  const runLintForDoc = (doc: vscode.TextDocument) => {
    if (doc.languageId !== "sruja") return;
    if (!diagnosticCollection) return;
    updateDiagnostics(context, doc, diagnosticCollection).catch((err) => {
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

  registerRunValidation(context, {
    diagnosticCollection,
    pendingLint,
  });
  registerSkillsCommands(context);
  registerExportCommands(context);

  const skillsTreeProvider = new SrujaSkillsTreeProvider(context);
  context.subscriptions.push(
    vscode.window.registerTreeDataProvider("srujaSkillsView", skillsTreeProvider)
  );

  const definitionProvider = new SrujaDefinitionProvider(context);
  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider("sruja", definitionProvider)
  );

  const hoverProvider = new SrujaHoverProvider(context);
  context.subscriptions.push(
    vscode.languages.registerHoverProvider("sruja", hoverProvider)
  );

  const documentSymbolProvider = new SrujaDocumentSymbolProvider(context);
  context.subscriptions.push(
    vscode.languages.registerDocumentSymbolProvider("sruja", documentSymbolProvider)
  );
}

export function deactivate(): void {
  diagnosticCollection?.dispose();
  diagnosticCollection = undefined;
}

import * as vscode from "vscode";

import { registerRunValidation, registerSkillsCommands, registerExportCommands } from "./commands";
import { updateDiagnostics } from "./diagnostics";
import { SrujaDefinitionProvider, SrujaHoverProvider, SrujaDocumentSymbolProvider } from "./providers";
import { SrujaSkillsTreeProvider } from "./skillsTree";

const DIAGNOSTIC_COLLECTION_ID = "sruja";
const LINT_DEBOUNCE_MS = 400;

class SrujaExtension {
  private diagnosticCollection: vscode.DiagnosticCollection;
  private pendingLint = new Map<string, ReturnType<typeof setTimeout>>();

  constructor(private context: vscode.ExtensionContext) {
    this.diagnosticCollection = vscode.languages.createDiagnosticCollection(DIAGNOSTIC_COLLECTION_ID);
    context.subscriptions.push(this.diagnosticCollection);
  }

  activate(): void {
    this.registerLintHandlers();
    this.registerCommands();
    this.registerProviders();
    this.registerTreeView();
    this.lintOpenDocuments();
  }

  deactivate(): void {
    this.diagnosticCollection.dispose();
  }

  private registerLintHandlers(): void {
    const { context, diagnosticCollection, pendingLint } = this;

    context.subscriptions.push(
      vscode.workspace.onDidOpenTextDocument((doc) => {
        if (doc.languageId === "sruja") this.runLintForDoc(doc);
      }),
      vscode.workspace.onDidSaveTextDocument((doc) => {
        if (doc.languageId === "sruja") this.runLintForDoc(doc);
      }),
      vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document.languageId === "sruja") this.scheduleLint(e.document);
      }),
      vscode.workspace.onDidCloseTextDocument((doc) => {
        if (doc.languageId === "sruja") {
          this.clearPendingLint(doc.uri.toString());
          diagnosticCollection.delete(doc.uri);
        }
      })
    );
  }

  private registerCommands(): void {
    registerRunValidation(this.context, {
      diagnosticCollection: this.diagnosticCollection,
      pendingLint: this.pendingLint,
    });
    registerSkillsCommands(this.context);
    registerExportCommands(this.context);
  }

  private registerProviders(): void {
    const { context } = this;

    context.subscriptions.push(
      vscode.languages.registerDefinitionProvider("sruja", new SrujaDefinitionProvider(context)),
      vscode.languages.registerHoverProvider("sruja", new SrujaHoverProvider(context)),
      vscode.languages.registerDocumentSymbolProvider("sruja", new SrujaDocumentSymbolProvider(context))
    );
  }

  private registerTreeView(): void {
    const skillsTreeProvider = new SrujaSkillsTreeProvider(this.context);
    this.context.subscriptions.push(
      vscode.window.registerTreeDataProvider("srujaSkillsView", skillsTreeProvider)
    );
  }

  private lintOpenDocuments(): void {
    for (const doc of vscode.workspace.textDocuments) {
      if (doc.languageId === "sruja") this.runLintForDoc(doc);
    }
  }

  private runLintForDoc(doc: vscode.TextDocument): void {
    if (!this.diagnosticCollection) return;

    updateDiagnostics(this.context, doc, this.diagnosticCollection).catch((err) => {
      if (this.diagnosticCollection && doc.uri) {
        this.diagnosticCollection.set(doc.uri, [
          new vscode.Diagnostic(
            new vscode.Range(0, 0, 0, 0),
            `Sruja lint failed: ${err instanceof Error ? err.message : String(err)}`,
            vscode.DiagnosticSeverity.Warning
          ),
        ]);
      }
    });
  }

  private scheduleLint(doc: vscode.TextDocument): void {
    const key = doc.uri.toString();
    this.clearPendingLint(key);

    const t = setTimeout(() => {
      this.pendingLint.delete(key);
      this.runLintForDoc(doc);
    }, LINT_DEBOUNCE_MS);

    this.pendingLint.set(key, t);
  }

  private clearPendingLint(key: string): void {
    const existing = this.pendingLint.get(key);
    if (existing) {
      clearTimeout(existing);
      this.pendingLint.delete(key);
    }
  }
}

let extension: SrujaExtension | undefined;

export function activate(context: vscode.ExtensionContext): void {
  extension = new SrujaExtension(context);
  extension.activate();
}

export function deactivate(): void {
  extension?.deactivate();
  extension = undefined;
}

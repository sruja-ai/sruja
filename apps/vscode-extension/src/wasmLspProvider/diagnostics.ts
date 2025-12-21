// Diagnostics provider for WASM LSP
import * as vscode from "vscode";
import type { Diagnostic } from "@sruja/shared/node/wasmAdapter";
import { log, getDebounceDelay, debounce } from "./utils";
import { getCachedDiagnostics } from "./cache";
import { getWasmApi } from "./initialization";

/**
 * Creates and registers the diagnostics provider.
 * 
 * @param context - VS Code extension context
 * 
 * @remarks
 * Sets up:
 * - Diagnostic collection
 * - Document change listener (debounced)
 * - Initial diagnostics for open documents
 */
export function registerDiagnosticsProvider(context: vscode.ExtensionContext): void {
  const diagnosticsCollection = vscode.languages.createDiagnosticCollection("sruja");
  context.subscriptions.push(diagnosticsCollection);

  const updateDiagnostics = async (document: vscode.TextDocument) => {
    const wasmApi = getWasmApi();
    if (document.languageId !== "sruja" || !wasmApi) return;

    try {
      const text = document.getText();
      // Use cached diagnostics when possible
      const diagnostics = await getCachedDiagnostics(document, text, wasmApi);

      const vscodeDiagnostics: vscode.Diagnostic[] = diagnostics.map((d: Diagnostic) => {
        const severity =
          d.severity === "Error"
            ? vscode.DiagnosticSeverity.Error
            : d.severity === "Warning"
              ? vscode.DiagnosticSeverity.Warning
              : vscode.DiagnosticSeverity.Information;

        // Calculate proper range - find the actual word/line length
        const line = Math.max(0, d.location.line - 1); // VS Code uses 0-based lines
        const startCol = Math.max(0, d.location.column - 1); // VS Code uses 0-based columns

        // Get the actual line text to calculate proper end position
        const lines = document.getText().split("\n");
        const lineText = lines[line] || "";
        // Find word boundaries or use line end
        let endCol = startCol;
        if (lineText.length > startCol) {
          // Try to find word end, or use a reasonable default
          const remaining = lineText.substring(startCol);
          const wordMatch = remaining.match(/^\S*/);
          if (wordMatch) {
            endCol = startCol + wordMatch[0].length;
          } else {
            endCol = Math.min(startCol + 20, lineText.length);
          }
        } else {
          endCol = startCol + 1;
        }

        const range = new vscode.Range(line, startCol, line, endCol);

        const diagnostic = new vscode.Diagnostic(range, d.message, severity);
        diagnostic.code = d.code;
        return diagnostic;
      });

      diagnosticsCollection.set(document.uri, vscodeDiagnostics);
      log(`Diagnostics updated for ${document.fileName}: ${vscodeDiagnostics.length} issues`);
      
      // Update status bar with diagnostics count (via event to avoid circular dependency)
      if (document === vscode.window.activeTextEditor?.document) {
        // Emit event that extension.ts can listen to
        vscode.commands.executeCommand("sruja.updateStatusBar");
      }
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      log(`WASM diagnostics failed for ${document.fileName}: ${errMsg}`, "error");
      diagnosticsCollection.set(document.uri, []);
    }
  };

  // Update diagnostics on document change (debounced for performance)
  const debouncedUpdateDiagnostics = debounce((doc: vscode.TextDocument) => {
    updateDiagnostics(doc);
  }, getDebounceDelay()); // Configurable debounce delay

  vscode.workspace.onDidChangeTextDocument((e) => {
    if (e.document.languageId === "sruja") {
      debouncedUpdateDiagnostics(e.document);
    }
  });

  // Initial diagnostics for open documents
  vscode.workspace.textDocuments.forEach((doc) => {
    if (doc.languageId === "sruja") {
      updateDiagnostics(doc);
    }
  });
}


// apps/vscode-extension/src/wasmLspProvider.ts
// WASM-based LSP provider for VS Code (no CLI dependency)
import * as vscode from "vscode";
import {
  initWasmNode,
  type Diagnostic,
  type HoverInfo,
  type CompletionItem,
  type Location,
} from "@sruja/shared/node/wasmAdapter";

let wasmApi: Awaited<ReturnType<typeof initWasmNode>> | null = null;

export async function initializeWasmLsp(context: vscode.ExtensionContext): Promise<void> {
  try {
    console.log("Initializing WASM LSP...");
    wasmApi = await initWasmNode({ extensionPath: context.extensionPath });

    // Verify WASM API is available
    if (!wasmApi) {
      throw new Error("WASM API initialization returned null");
    }

    // Test basic functionality
    try {
      await wasmApi.getDiagnostics('architecture "Test" {}');
      console.log("WASM LSP functions verified");
    } catch (testError) {
      console.warn("WASM function test failed:", testError);
      // Continue anyway - might work for some functions
    }

    // Register diagnostics provider
    const diagnosticsCollection = vscode.languages.createDiagnosticCollection("sruja");
    context.subscriptions.push(diagnosticsCollection);

    const updateDiagnostics = async (document: vscode.TextDocument) => {
      if (document.languageId !== "sruja" || !wasmApi) return;

      try {
        const text = document.getText();
        const diagnostics = await wasmApi.getDiagnostics(text);

        const vscodeDiagnostics: vscode.Diagnostic[] = diagnostics.map((d) => {
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
      } catch (error) {
        console.error("WASM diagnostics failed:", error);
        diagnosticsCollection.set(document.uri, []);
      }
    };

    // Update diagnostics on document change
    vscode.workspace.onDidChangeTextDocument((e) => {
      if (e.document.languageId === "sruja") {
        updateDiagnostics(e.document);
      }
    });

    // Initial diagnostics for open documents
    vscode.workspace.textDocuments.forEach((doc) => {
      if (doc.languageId === "sruja") {
        updateDiagnostics(doc);
      }
    });

    // Register hover provider
    const hoverProvider: vscode.HoverProvider = {
      async provideHover(document, position) {
        if (!wasmApi || document.languageId !== "sruja") return null;

        try {
          const text = document.getText();
          const hoverInfo = await wasmApi.hover(text, position.line + 1, position.character + 1);

          if (!hoverInfo) return null;

          return new vscode.Hover({
            language: "markdown",
            value: hoverInfo.contents,
          });
        } catch (error) {
          console.error("WASM hover failed:", error);
          return null;
        }
      },
    };

    context.subscriptions.push(vscode.languages.registerHoverProvider("sruja", hoverProvider));

    // Register completion provider
    const completionProvider: vscode.CompletionItemProvider = {
      async provideCompletionItems(document, position, token, context) {
        if (!wasmApi || document.languageId !== "sruja") return [];

        try {
          const text = document.getText();
          const completions = await wasmApi.completion(
            text,
            position.line + 1,
            position.character + 1
          );

          return completions.map((c) => {
            const item = new vscode.CompletionItem(c.label, vscode.CompletionItemKind.Keyword);
            item.detail = c.kind || "keyword";
            return item;
          });
        } catch (error) {
          console.error("WASM completion failed:", error);
          // Return basic keywords as fallback
          return [
            new vscode.CompletionItem("architecture", vscode.CompletionItemKind.Keyword),
            new vscode.CompletionItem("system", vscode.CompletionItemKind.Keyword),
            new vscode.CompletionItem("container", vscode.CompletionItemKind.Keyword),
            new vscode.CompletionItem("component", vscode.CompletionItemKind.Keyword),
            new vscode.CompletionItem("datastore", vscode.CompletionItemKind.Keyword),
            new vscode.CompletionItem("person", vscode.CompletionItemKind.Keyword),
            new vscode.CompletionItem("relation", vscode.CompletionItemKind.Keyword),
          ];
        }
      },
    };

    context.subscriptions.push(
      vscode.languages.registerCompletionItemProvider("sruja", completionProvider)
    );

    // Register definition provider
    const definitionProvider: vscode.DefinitionProvider = {
      async provideDefinition(document, position) {
        if (!wasmApi || document.languageId !== "sruja") return [];

        try {
          const text = document.getText();
          const location = await wasmApi.goToDefinition(
            text,
            position.line + 1,
            position.character + 1
          );

          if (!location) return [];

          // Calculate proper range for definition
          const line = Math.max(0, location.line - 1);
          const startCol = Math.max(0, location.column - 1);
          const lineText = document.getText().split("\n")[line] || "";
          const endCol = Math.min(startCol + 20, lineText.length);

          const range = new vscode.Range(line, startCol, line, endCol);

          return new vscode.Location(document.uri, range);
        } catch (error) {
          console.error("WASM goToDefinition failed:", error);
          return [];
        }
      },
    };

    context.subscriptions.push(
      vscode.languages.registerDefinitionProvider("sruja", definitionProvider)
    );

    // Register format provider
    const formatProvider: vscode.DocumentFormattingEditProvider = {
      async provideDocumentFormattingEdits(document) {
        if (!wasmApi || document.languageId !== "sruja") return [];

        try {
          const text = document.getText();
          const formatted = await wasmApi.format(text);

          const range = new vscode.Range(
            document.positionAt(0),
            document.positionAt(document.getText().length)
          );

          return [vscode.TextEdit.replace(range, formatted)];
        } catch (error) {
          console.error("WASM format failed:", error);
          return [];
        }
      },
    };

    context.subscriptions.push(
      vscode.languages.registerDocumentFormattingEditProvider("sruja", formatProvider)
    );

    console.log("WASM LSP providers registered successfully");
  } catch (error) {
    console.error("Failed to initialize WASM LSP:", error);
    vscode.window.showErrorMessage(
      `Failed to initialize WASM LSP: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

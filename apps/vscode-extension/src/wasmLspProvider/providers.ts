// LSP providers for WASM LSP
import * as vscode from "vscode";
import type { Diagnostic } from "@sruja/shared/node/wasmAdapter";
import { log, isIdentChar, mapSymbolKind, documentCache } from "./utils";
import { getWasmApi } from "./initialization";

/**
 * Registers all LSP providers for the Sruja language.
 *
 * @param context - VS Code extension context
 *
 * @remarks
 * Registers the following providers:
 * - Hover
 * - Completion
 * - Definition
 * - Format
 * - Document Symbols
 * - Workspace Symbols
 * - References
 * - Rename
 * - Code Actions
 * - Document Links
 * - Folding Ranges
 * - Semantic Tokens
 * - Inlay Hints
 * - Code Lenses
 * - Signature Help
 */
export function registerLspProviders(context: vscode.ExtensionContext): void {
  // Register hover provider
  const hoverProvider: vscode.HoverProvider = {
    async provideHover(document, position) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return null;

      try {
        const text = document.getText();
        // VSCode uses 0-based, Go WASM uses 1-based
        const line = position.line + 1;
        const column = position.character + 1;

        log(`[Hover] Request at line ${line}, column ${column} in ${document.fileName}`);
        const hoverInfo = await wasmApi.hover(text, line, column);

        if (!hoverInfo) {
          log(`[Hover] No hover info returned for line ${line}, column ${column}`);
          return null;
        }

        log(`[Hover] Success: ${hoverInfo.contents.substring(0, 100)}...`);
        return new vscode.Hover({
          language: "markdown",
          value: hoverInfo.contents,
        });
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM hover failed: ${errMsg}`, "error");
        return null;
      }
    },
  };

  context.subscriptions.push(vscode.languages.registerHoverProvider("sruja", hoverProvider));

  // Register completion provider
  const completionProvider: vscode.CompletionItemProvider = {
    async provideCompletionItems(document, position, _token, _context) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        const completions = await wasmApi.completion(
          text,
          position.line + 1,
          position.character + 1
        );

        log(`[Completion] Returning ${completions.length} items`);
        return completions.map((c) => {
          const item = new vscode.CompletionItem(c.label, vscode.CompletionItemKind.Keyword);
          item.detail = c.kind || "keyword";
          return item;
        });
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM completion failed: ${errMsg}`, "error");
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
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        // VSCode uses 0-based, Go WASM uses 1-based
        const line = position.line + 1;
        const column = position.character + 1;

        log(`[GoToDefinition] Request at line ${line}, column ${column} in ${document.fileName}`);
        const location = await wasmApi.goToDefinition(text, line, column);

        if (!location) {
          log(`[GoToDefinition] No definition found for line ${line}, column ${column}`);
          return [];
        }

        log(
          `[GoToDefinition] Found definition at line ${location.line}, column ${location.column}`
        );
        // Calculate proper range for definition
        // Go returns 1-based, VSCode needs 0-based
        const defLine = Math.max(0, location.line - 1);
        const startCol = Math.max(0, location.column - 1);
        const lineText = document.getText().split("\n")[defLine] || "";
        const endCol = Math.min(startCol + 20, lineText.length);

        const range = new vscode.Range(defLine, startCol, defLine, endCol);

        return new vscode.Location(document.uri, range);
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM goToDefinition failed: ${errMsg}`, "error");
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
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        const formatted = await wasmApi.format(text);

        const range = new vscode.Range(
          document.positionAt(0),
          document.positionAt(document.getText().length)
        );

        log(`[Format] Formatted document (${formatted.length} chars)`);
        return [vscode.TextEdit.replace(range, formatted)];
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM format failed: ${errMsg}`, "error");
        return [];
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider("sruja", formatProvider)
  );

  // Register document symbol provider (for Outline view)
  const documentSymbolProvider: vscode.DocumentSymbolProvider = {
    async provideDocumentSymbols(document) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        const symbols = await wasmApi.getSymbols(text);

        return symbols.map((sym) => {
          const kind = mapSymbolKind(sym.kind);
          const line = Math.max(0, sym.line - 1); // Convert to 0-based
          const range = new vscode.Range(line, 0, line, 1000); // Full line range

          return new vscode.DocumentSymbol(sym.name, sym.kind, kind, range, range);
        });
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM documentSymbols failed: ${errMsg}`, "error");
        return [];
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerDocumentSymbolProvider("sruja", documentSymbolProvider)
  );

  // Register workspace symbol provider (for "Go to Symbol in Workspace")
  const workspaceSymbolProvider: vscode.WorkspaceSymbolProvider = {
    async provideWorkspaceSymbols(query) {
      const wasmApi = getWasmApi();
      if (!wasmApi || !query) return [];

      try {
        const results: vscode.SymbolInformation[] = [];
        const queryLower = query.toLowerCase();

        // Search through all open Sruja documents
        for (const doc of vscode.workspace.textDocuments) {
          if (doc.languageId !== "sruja") continue;

          try {
            const text = doc.getText();
            const symbols = await wasmApi.getSymbols(text);

            for (const sym of symbols) {
              if (sym.name.toLowerCase().includes(queryLower)) {
                const kind = mapSymbolKind(sym.kind);
                const line = Math.max(0, sym.line - 1);
                const location = new vscode.Location(doc.uri, new vscode.Position(line, 0));

                results.push(new vscode.SymbolInformation(sym.name, kind, "", location));
              }
            }
          } catch {
            // Skip documents that fail to parse
            continue;
          }
        }

        return results;
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM workspaceSymbols failed: ${errMsg}`, "error");
        return [];
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerWorkspaceSymbolProvider(workspaceSymbolProvider)
  );

  // Register reference provider (Find All References)
  const referenceProvider: vscode.ReferenceProvider = {
    async provideReferences(document, position, _context) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        const line = position.line + 1;
        const column = position.character + 1;

        log(`[References] Request at line ${line}, column ${column} in ${document.fileName}`);
        const references = await wasmApi.findReferences(text, line, column);

        log(`[References] Found ${references.length} references`);

        return references.map((ref) => {
          const refLine = Math.max(0, ref.line - 1);
          const refCol = Math.max(0, ref.column - 1);
          const lineText = document.getText().split("\n")[refLine] || "";
          // Find the end of the symbol
          let endCol = refCol;
          while (
            endCol < lineText.length &&
            (isIdentChar(lineText[endCol]) || lineText[endCol] === ".")
          ) {
            endCol++;
          }

          const range = new vscode.Range(refLine, refCol, refLine, endCol);
          return new vscode.Location(document.uri, range);
        });
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM references failed: ${errMsg}`, "error");
        return [];
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerReferenceProvider("sruja", referenceProvider)
  );

  // Register rename provider
  const renameProvider: vscode.RenameProvider = {
    async provideRenameEdits(document, position, newName, _token) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return null;

      try {
        const text = document.getText();
        const line = position.line + 1;
        const column = position.character + 1;

        log(
          `[Rename] Request at line ${line}, column ${column} to "${newName}" in ${document.fileName}`
        );
        const renamedText = await wasmApi.rename(text, line, column, newName);

        if (renamedText === text) {
          log(`[Rename] No changes made`);
          return null;
        }

        // Create a workspace edit with the full document replacement
        const range = new vscode.Range(
          document.positionAt(0),
          document.positionAt(document.getText().length)
        );

        const edit = new vscode.WorkspaceEdit();
        edit.replace(document.uri, range, renamedText);
        log(`[Rename] Successfully renamed`);
        return edit;
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM rename failed: ${errMsg}`, "error");
        return null;
      }
    },
    async prepareRename(document, position) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return null;

      try {
        const text = document.getText();
        const line = position.line + 1;
        const column = position.character + 1;

        // Check if we can rename at this position by trying to find definition
        const def = await wasmApi.goToDefinition(text, line, column);
        if (!def) {
          return null; // Can't rename if no definition found
        }

        // Extract the symbol name at cursor
        const lineText = document.lineAt(position.line).text;
        let start = position.character;
        while (start > 0 && (isIdentChar(lineText[start - 1]) || lineText[start - 1] === ".")) {
          start--;
        }
        let end = position.character;
        while (end < lineText.length && (isIdentChar(lineText[end]) || lineText[end] === ".")) {
          end++;
        }

        if (start >= end) {
          return null;
        }

        const range = new vscode.Range(position.line, start, position.line, end);
        return range;
      } catch {
        return null;
      }
    },
  };

  context.subscriptions.push(vscode.languages.registerRenameProvider("sruja", renameProvider));

  // Register code action provider
  const codeActionProvider: vscode.CodeActionProvider = {
    async provideCodeActions(document, range, context) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        const diagnostics: Diagnostic[] = context.diagnostics.map((d) => ({
          code: d.code?.toString() || "",
          severity: (d.severity === vscode.DiagnosticSeverity.Error
            ? "Error"
            : d.severity === vscode.DiagnosticSeverity.Warning
              ? "Warning"
              : "Info") as "Error" | "Warning" | "Info",
          message: d.message,
          location: {
            file: document.fileName,
            line: d.range.start.line + 1,
            column: d.range.start.character + 1,
          },
        }));

        const actions = await wasmApi.codeActions(text, diagnostics);
        return actions.map((action) => {
          const codeAction = new vscode.CodeAction(action.title, vscode.CodeActionKind.QuickFix);
          codeAction.command = {
            title: action.title,
            command: action.command,
            arguments: action.arguments || [],
          };
          return codeAction;
        });
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM codeActions failed: ${errMsg}`, "error");
        return [];
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerCodeActionsProvider("sruja", codeActionProvider)
  );

  // Register document link provider
  const documentLinkProvider: vscode.DocumentLinkProvider = {
    async provideDocumentLinks(document) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        const links = await wasmApi.documentLinks(text);
        return links.map((link) => {
          const range = new vscode.Range(
            link.range.start.line,
            link.range.start.character,
            link.range.end.line,
            link.range.end.character
          );
          const vscodeLink = new vscode.DocumentLink(
            range,
            link.target ? vscode.Uri.parse(link.target) : undefined
          );
          if (link.tooltip) {
            vscodeLink.tooltip = link.tooltip;
          }
          return vscodeLink;
        });
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM documentLinks failed: ${errMsg}`, "error");
        return [];
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerDocumentLinkProvider("sruja", documentLinkProvider)
  );

  // Register folding range provider
  const foldingRangeProvider: vscode.FoldingRangeProvider = {
    async provideFoldingRanges(document) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        const ranges = await wasmApi.foldingRanges(text);
        return ranges.map((range) => {
          return new vscode.FoldingRange(
            range.startLine,
            range.endLine,
            range.kind === "region" || range.kind === "comment" || range.kind === "imports"
              ? vscode.FoldingRangeKind.Region
              : undefined
          );
        });
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM foldingRanges failed: ${errMsg}`, "error");
        return [];
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerFoldingRangeProvider("sruja", foldingRangeProvider)
  );

  // Register semantic tokens provider
  const semanticTokensLegend = new vscode.SemanticTokensLegend(
    ["keyword", "class", "module", "function", "struct", "enum", "variable", "operator", "string"],
    ["declaration"]
  );

  const semanticTokensProvider: vscode.DocumentSemanticTokensProvider = {
    async provideDocumentSemanticTokens(document) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") {
        return new vscode.SemanticTokens(new Uint32Array(0));
      }

      try {
        const text = document.getText();
        const tokens = await wasmApi.semanticTokens(text);
        return new vscode.SemanticTokens(new Uint32Array(tokens));
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM semanticTokens failed: ${errMsg}`, "error");
        return new vscode.SemanticTokens(new Uint32Array(0));
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerDocumentSemanticTokensProvider(
      "sruja",
      semanticTokensProvider,
      semanticTokensLegend
    )
  );

  // Register Inlay Hints provider (advanced LSP feature)
  const inlayHintsProvider: vscode.InlayHintsProvider = {
    async provideInlayHints(document, range) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        const hints: vscode.InlayHint[] = [];

        // Extract symbols to show type hints
        const symbols = await wasmApi.getSymbols(text);
        for (const sym of symbols) {
          const line = Math.max(0, sym.line - 1);
          if (line >= range.start.line && line <= range.end.line) {
            // Add type hint for systems, containers, etc.
            if (sym.kind === "system" || sym.kind === "container" || sym.kind === "component") {
              const position = new vscode.Position(line, 1000); // End of line
              const hint = new vscode.InlayHint(position, ` : ${sym.kind}`);
              hint.paddingLeft = true;
              hints.push(hint);
            }
          }
        }

        return hints;
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM inlayHints failed: ${errMsg}`, "error");
        return [];
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerInlayHintsProvider("sruja", inlayHintsProvider)
  );

  // Register Code Lenses provider (advanced LSP feature)
  const codeLensProvider: vscode.CodeLensProvider = {
    async provideCodeLenses(document) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return [];

      try {
        const text = document.getText();
        const symbols = await wasmApi.getSymbols(text);
        const lenses: vscode.CodeLens[] = [];

        for (const sym of symbols) {
          const line = Math.max(0, sym.line - 1);
          const range = new vscode.Range(line, 0, line, 1000);

          // Find references count
          try {
            const refs = await wasmApi.findReferences(text, sym.line, 1);
            const lens = new vscode.CodeLens(range, {
              title: `${refs.length} reference${refs.length !== 1 ? "s" : ""}`,
              command: "editor.action.showReferences",
              arguments: [document.uri, new vscode.Position(line, 0)],
            });
            lenses.push(lens);
          } catch {
            // If findReferences fails, skip this lens
          }
        }

        return lenses;
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM codeLenses failed: ${errMsg}`, "error");
        return [];
      }
    },
  };

  context.subscriptions.push(vscode.languages.registerCodeLensProvider("sruja", codeLensProvider));

  // Register Signature Help provider (advanced LSP feature)
  const signatureHelpProvider: vscode.SignatureHelpProvider = {
    async provideSignatureHelp(document, position) {
      const wasmApi = getWasmApi();
      if (!wasmApi || document.languageId !== "sruja") return null;

      try {
        const text = document.getText();
        const line = position.line + 1;
        const column = position.character + 1;

        // Get hover info which often contains signature-like information
        const hoverInfo = await wasmApi.hover(text, line, column);
        if (!hoverInfo) return null;

        // Try to extract signature from hover info
        const signatureMatch = hoverInfo.contents.match(/(\w+)\s*\([^)]*\)/);
        if (signatureMatch) {
          const signature = new vscode.SignatureHelp();
          const sigInfo = new vscode.SignatureInformation(signatureMatch[0], hoverInfo.contents);
          signature.signatures = [sigInfo];
          signature.activeSignature = 0;
          return signature;
        }

        return null;
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM signatureHelp failed: ${errMsg}`, "error");
        return null;
      }
    },
  };

  context.subscriptions.push(
    vscode.languages.registerSignatureHelpProvider("sruja", signatureHelpProvider, "(", ",")
  );

  // Clear cache on document close
  vscode.workspace.onDidCloseTextDocument((doc) => {
    if (doc.languageId === "sruja") {
      documentCache.delete(doc.uri.toString());
      log(`[Cache] Cleared cache for ${doc.fileName}`);
    }
  });

  // Clear cache on configuration change
  vscode.workspace.onDidChangeConfiguration((e) => {
    if (e.affectsConfiguration("sruja.performance")) {
      // Clear cache when performance settings change
      documentCache.clear();
      log("[Cache] Cleared all caches due to configuration change");
    }
  });

  log("WASM LSP providers registered successfully");
}

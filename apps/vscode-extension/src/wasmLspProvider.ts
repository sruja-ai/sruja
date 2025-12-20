// apps/vscode-extension/src/wasmLspProvider.ts
// WASM-based LSP provider for VS Code (no CLI dependency)
import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import {
  initWasmNode,
  type Diagnostic,
  type HoverInfo,
  type CompletionItem,
  type Location,
  type Symbol,
  type CodeAction,
  type DocumentLink,
  type FoldingRange,
} from "@sruja/shared/node/wasmAdapter";

let wasmApi: Awaited<ReturnType<typeof initWasmNode>> | null = null;
let outputChannel: vscode.OutputChannel | null = null;

function log(message: string, type: "info" | "warn" | "error" = "info") {
  const timestamp = new Date().toISOString();
  const prefix = `[${timestamp}] [${type.toUpperCase()}]`;
  const fullMessage = `${prefix} ${message}`;

  // Log to console (for development)
  if (type === "error") {
    console.error(fullMessage);
  } else if (type === "warn") {
    console.warn(fullMessage);
  } else {
    console.log(fullMessage);
  }

  // Log to output channel (for user visibility)
  if (outputChannel) {
    outputChannel.appendLine(fullMessage);
  }
}

// Intercept console.log from wasmAdapter to show in output channel
function setupConsoleInterception() {
  if (!outputChannel) return;

  const originalLog = console.log;
  const originalWarn = console.warn;
  const originalError = console.error;

  console.log = (...args: any[]) => {
    originalLog.apply(console, args);
    if (outputChannel && args.some((arg) => String(arg).includes("[WASM]"))) {
      outputChannel.appendLine(`[WASM] ${args.map((a) => String(a)).join(" ")}`);
    }
  };

  console.warn = (...args: any[]) => {
    originalWarn.apply(console, args);
    if (outputChannel && args.some((arg) => String(arg).includes("[WASM]"))) {
      outputChannel.appendLine(`[WASM] ${args.map((a) => String(a)).join(" ")}`);
    }
  };

  console.error = (...args: any[]) => {
    originalError.apply(console, args);
    if (outputChannel && args.some((arg) => String(arg).includes("[WASM]"))) {
      outputChannel.appendLine(`[WASM ERROR] ${args.map((a) => String(a)).join(" ")}`);
    }
  };
}

export function getWasmApi() {
  return wasmApi;
}

// Helper function to check if character is identifier character
function isIdentChar(c: string): boolean {
  return /[a-zA-Z0-9_]/.test(c);
}

// Map symbol kind from Go to VS Code SymbolKind
function mapSymbolKind(kind: string): vscode.SymbolKind {
  switch (kind.toLowerCase()) {
    case "system":
      return vscode.SymbolKind.Class;
    case "container":
      return vscode.SymbolKind.Module;
    case "component":
      return vscode.SymbolKind.Method;
    case "datastore":
      return vscode.SymbolKind.Variable; // Database not available in older VS Code versions
    case "person":
      return vscode.SymbolKind.Interface;
    default:
      return vscode.SymbolKind.Variable;
  }
}

export async function initializeWasmLsp(context: vscode.ExtensionContext): Promise<void> {
  // Create output channel for debugging
  outputChannel = vscode.window.createOutputChannel("Sruja WASM LSP");
  context.subscriptions.push(outputChannel);
  outputChannel.show(true); // Show output channel automatically for debugging

  // Intercept console logs from wasmAdapter
  setupConsoleInterception();

  try {
    log("Initializing WASM LSP...");

    // Check WASM files exist
    const wasmPath = path.join(context.extensionPath, "wasm", "sruja.wasm.gz");
    const wasmPathUncompressed = path.join(context.extensionPath, "wasm", "sruja.wasm");
    const wasmExecPath = path.join(context.extensionPath, "wasm", "wasm_exec.js");

    log(`Extension path: ${context.extensionPath}`);
    log(`Checking WASM files:`);
    log(`  - ${wasmPath}: ${fs.existsSync(wasmPath) ? "✅ Found" : "❌ Missing"}`);
    log(
      `  - ${wasmPathUncompressed}: ${fs.existsSync(wasmPathUncompressed) ? "✅ Found" : "❌ Missing"}`
    );
    log(`  - ${wasmExecPath}: ${fs.existsSync(wasmExecPath) ? "✅ Found" : "❌ Missing"}`);

    if (!fs.existsSync(wasmPath) && !fs.existsSync(wasmPathUncompressed)) {
      throw new Error(`WASM file not found. Expected at: ${wasmPath} or ${wasmPathUncompressed}`);
    }

    if (!fs.existsSync(wasmExecPath)) {
      throw new Error(`wasm_exec.js not found. Expected at: ${wasmExecPath}`);
    }

    log("WASM files verified. Initializing WASM module...");
    log("Calling initWasmNode...");
    const initStartTime = Date.now();

    // Add timeout to detect hanging
    const initTimeout = 30000; // 30 seconds
    const initPromise = initWasmNode({ extensionPath: context.extensionPath });
    const timeoutPromise = new Promise<never>((_, reject) => {
      setTimeout(() => {
        reject(new Error(`WASM initialization timed out after ${initTimeout}ms`));
      }, initTimeout);
    });

    try {
      wasmApi = await Promise.race([initPromise, timeoutPromise]);
      const initDuration = Date.now() - initStartTime;
      log(`WASM module loaded successfully (took ${initDuration}ms)`);
    } catch (initError) {
      const initDuration = Date.now() - initStartTime;
      const errMsg = initError instanceof Error ? initError.message : String(initError);
      const stack = initError instanceof Error ? initError.stack : undefined;
      log(`Failed to initialize WASM module after ${initDuration}ms: ${errMsg}`, "error");
      if (stack) {
        log(`Stack trace: ${stack}`, "error");
      }
      // Also check console for detailed logs from wasmAdapter
      log(
        "Check Developer Console (Help > Toggle Developer Tools) for detailed WASM loading logs",
        "warn"
      );
      // Continue anyway - providers will handle missing wasmApi gracefully
      log("Continuing without WASM API - LSP features will be limited", "warn");
      wasmApi = null; // Ensure it's null so providers know it failed
    }

    // Verify WASM API is available
    if (!wasmApi) {
      log("WASM API is null - initialization may have failed silently", "error");
      throw new Error("WASM API initialization returned null");
    }

    log("WASM API object created successfully");

    // Test all LSP functions
    log("Testing WASM LSP functions...");
    const testCode = 'architecture "Test" { system App "App" {} }';

    const testResults: Record<string, { success: boolean; error?: string }> = {};

    // Test diagnostics
    try {
      const diags = await wasmApi.getDiagnostics(testCode);
      testResults.diagnostics = { success: true };
      log(`✅ Diagnostics: OK (returned ${diags.length} diagnostics)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.diagnostics = { success: false, error: errMsg };
      log(`❌ Diagnostics: FAILED - ${errMsg}`, "error");
    }

    // Test hover
    try {
      const hover = await wasmApi.hover(testCode, 1, 15);
      testResults.hover = { success: true };
      log(`✅ Hover: OK (${hover ? "returned info" : "no info"})`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.hover = { success: false, error: errMsg };
      log(`❌ Hover: FAILED - ${errMsg}`, "error");
    }

    // Test completion
    try {
      const completions = await wasmApi.completion(testCode, 1, 15);
      testResults.completion = { success: true };
      log(`✅ Completion: OK (returned ${completions.length} items)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.completion = { success: false, error: errMsg };
      log(`❌ Completion: FAILED - ${errMsg}`, "error");
    }

    // Test goToDefinition
    try {
      const def = await wasmApi.goToDefinition(testCode, 1, 15);
      testResults.goToDefinition = { success: true };
      log(`✅ GoToDefinition: OK (${def ? "found definition" : "no definition"})`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.goToDefinition = { success: false, error: errMsg };
      log(`❌ GoToDefinition: FAILED - ${errMsg}`, "error");
    }

    // Test format
    try {
      const formatted = await wasmApi.format(testCode);
      testResults.format = { success: true };
      log(`✅ Format: OK (returned ${formatted.length} chars)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.format = { success: false, error: errMsg };
      log(`❌ Format: FAILED - ${errMsg}`, "error");
    }

    // Test getSymbols
    try {
      const symbols = await wasmApi.getSymbols(testCode);
      testResults.getSymbols = { success: true };
      log(`✅ GetSymbols: OK (returned ${symbols.length} symbols)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.getSymbols = { success: false, error: errMsg };
      log(`❌ GetSymbols: FAILED - ${errMsg}`, "error");
    }

    // Test findReferences
    try {
      const refs = await wasmApi.findReferences(testCode, 1, 15);
      testResults.findReferences = { success: true };
      log(`✅ FindReferences: OK (returned ${refs.length} references)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.findReferences = { success: false, error: errMsg };
      log(`❌ FindReferences: FAILED - ${errMsg}`, "error");
    }

    // Test rename
    try {
      const renamed = await wasmApi.rename(testCode, 1, 15, "TestApp");
      testResults.rename = { success: true };
      log(`✅ Rename: OK (returned ${renamed.length} chars)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.rename = { success: false, error: errMsg };
      log(`❌ Rename: FAILED - ${errMsg}`, "error");
    }

    // Test codeActions
    try {
      const diags = await wasmApi.getDiagnostics(testCode);
      const actions = await wasmApi.codeActions(testCode, diags);
      testResults.codeActions = { success: true };
      log(`✅ CodeActions: OK (returned ${actions.length} actions)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.codeActions = { success: false, error: errMsg };
      log(`❌ CodeActions: FAILED - ${errMsg}`, "error");
    }

    // Test semanticTokens
    try {
      const tokens = await wasmApi.semanticTokens(testCode);
      testResults.semanticTokens = { success: true };
      log(`✅ SemanticTokens: OK (returned ${tokens.length} tokens)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.semanticTokens = { success: false, error: errMsg };
      log(`❌ SemanticTokens: FAILED - ${errMsg}`, "error");
    }

    // Test documentLinks
    try {
      const links = await wasmApi.documentLinks(testCode);
      testResults.documentLinks = { success: true };
      log(`✅ DocumentLinks: OK (returned ${links.length} links)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.documentLinks = { success: false, error: errMsg };
      log(`❌ DocumentLinks: FAILED - ${errMsg}`, "error");
    }

    // Test foldingRanges
    try {
      const ranges = await wasmApi.foldingRanges(testCode);
      testResults.foldingRanges = { success: true };
      log(`✅ FoldingRanges: OK (returned ${ranges.length} ranges)`);
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      testResults.foldingRanges = { success: false, error: errMsg };
      log(`❌ FoldingRanges: FAILED - ${errMsg}`, "error");
    }

    const failedTests = Object.entries(testResults).filter(([_, result]) => !result.success);
    if (failedTests.length > 0) {
      log(`⚠️  ${failedTests.length} LSP function(s) failed tests, but continuing...`, "warn");
    } else {
      log("✅ All WASM LSP functions verified");
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
        log(`Diagnostics updated for ${document.fileName}: ${vscodeDiagnostics.length} issues`);
      } catch (error) {
        const errMsg = error instanceof Error ? error.message : String(error);
        log(`WASM diagnostics failed for ${document.fileName}: ${errMsg}`, "error");
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
      async provideCompletionItems(document, position, token, context) {
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
            } catch (error) {
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
      async provideReferences(document, position, context) {
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
      async provideRenameEdits(document, position, newName, token) {
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
        } catch (error) {
          return null;
        }
      },
    };

    context.subscriptions.push(vscode.languages.registerRenameProvider("sruja", renameProvider));

    // Register code action provider
    const codeActionProvider: vscode.CodeActionProvider = {
      async provideCodeActions(document, range, context) {
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
            const vscodeLink = new vscode.DocumentLink(range, link.target ? vscode.Uri.parse(link.target) : undefined);
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
      vscode.languages.registerDocumentSemanticTokensProvider("sruja", semanticTokensProvider, semanticTokensLegend)
    );

    log("WASM LSP providers registered successfully");
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    const stack = error instanceof Error ? error.stack : undefined;
    log(`Failed to initialize WASM LSP: ${errMsg}`, "error");
    if (stack) {
      log(`Stack trace: ${stack}`, "error");
    }
    vscode.window.showErrorMessage(
      `Failed to initialize WASM LSP: ${errMsg}. Check output channel "Sruja WASM LSP" for details.`
    );
  }
}

// Debug command to test WASM functions
export async function debugWasmLsp() {
  if (!outputChannel) {
    outputChannel = vscode.window.createOutputChannel("Sruja WASM LSP");
  }
  outputChannel.show(true);

  log("=== WASM LSP Debug Session ===");

  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "sruja") {
    log("Please open a .sruja file to test", "warn");
    vscode.window.showWarningMessage("Please open a .sruja file to test WASM LSP");
    return;
  }

  const document = editor.document;
  const text = document.getText();
  const position = editor.selection.active;

  log(`Testing with file: ${document.fileName}`);
  log(`File size: ${text.length} characters, ${text.split("\n").length} lines`);
  log(`Cursor position: line ${position.line + 1}, column ${position.character + 1}`);

  if (!wasmApi) {
    log("WASM API not initialized. Attempting to initialize...", "warn");
    try {
      const extensionPath = vscode.extensions.getExtension(
        "sruja-ai.sruja-language-support"
      )?.extensionPath;
      if (!extensionPath) {
        throw new Error("Extension path not found");
      }
      wasmApi = await initWasmNode({ extensionPath });
      log("WASM API initialized successfully");
    } catch (error) {
      const errMsg = error instanceof Error ? error.message : String(error);
      log(`Failed to initialize WASM API: ${errMsg}`, "error");
      vscode.window.showErrorMessage(`Failed to initialize WASM API: ${errMsg}`);
      return;
    }
  }

  // Test all functions
  log("\n--- Testing Diagnostics ---");
  try {
    const diags = await wasmApi.getDiagnostics(text);
    log(`✅ Diagnostics: ${diags.length} issues found`);
    diags.forEach((d, i) => {
      log(
        `  ${i + 1}. [${d.severity}] ${d.code}: ${d.message} at ${d.location.line}:${d.location.column}`
      );
    });
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ Diagnostics failed: ${errMsg}`, "error");
  }

  log("\n--- Testing Hover ---");
  try {
    const line = position.line + 1;
    const column = position.character + 1;
    const hover = await wasmApi.hover(text, line, column);
    if (hover) {
      log(`✅ Hover: Success`);
      log(`  Contents: ${hover.contents.substring(0, 200)}...`);
    } else {
      log(`⚠️  Hover: No info returned`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ Hover failed: ${errMsg}`, "error");
  }

  log("\n--- Testing Completion ---");
  try {
    const line = position.line + 1;
    const column = position.character + 1;
    const completions = await wasmApi.completion(text, line, column);
    log(`✅ Completion: ${completions.length} items`);
    completions.slice(0, 10).forEach((c, i) => {
      log(`  ${i + 1}. ${c.label} (${c.kind})`);
    });
    if (completions.length > 10) {
      log(`  ... and ${completions.length - 10} more`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ Completion failed: ${errMsg}`, "error");
  }

  log("\n--- Testing GoToDefinition ---");
  try {
    const line = position.line + 1;
    const column = position.character + 1;
    const def = await wasmApi.goToDefinition(text, line, column);
    if (def) {
      log(`✅ GoToDefinition: Found at line ${def.line}, column ${def.column}`);
    } else {
      log(`⚠️  GoToDefinition: No definition found`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ GoToDefinition failed: ${errMsg}`, "error");
  }

  log("\n--- Testing Format ---");
  try {
    const formatted = await wasmApi.format(text);
    log(`✅ Format: Success (${formatted.length} chars)`);
    if (formatted !== text) {
      log(`  Document was modified`);
    } else {
      log(`  Document unchanged`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ Format failed: ${errMsg}`, "error");
  }

  log("\n--- Testing GetSymbols ---");
  try {
    const symbols = await wasmApi.getSymbols(text);
    log(`✅ GetSymbols: ${symbols.length} symbols`);
    symbols.slice(0, 10).forEach((s, i) => {
      log(`  ${i + 1}. ${s.name} (${s.kind}) at line ${s.line}`);
    });
    if (symbols.length > 10) {
      log(`  ... and ${symbols.length - 10} more`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ GetSymbols failed: ${errMsg}`, "error");
  }

  log("\n--- Testing FindReferences ---");
  try {
    const line = position.line + 1;
    const column = position.character + 1;
    const refs = await wasmApi.findReferences(text, line, column);
    log(`✅ FindReferences: ${refs.length} references`);
    refs.slice(0, 10).forEach((ref, i) => {
      log(`  ${i + 1}. Line ${ref.line}, column ${ref.column}`);
    });
    if (refs.length > 10) {
      log(`  ... and ${refs.length - 10} more`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ FindReferences failed: ${errMsg}`, "error");
  }

  log("\n--- Testing Rename ---");
  try {
    const line = position.line + 1;
    const column = position.character + 1;
    const newName = "RenamedSymbol";
    const renamed = await wasmApi.rename(text, line, column, newName);
    log(`✅ Rename: Success (${renamed.length} chars)`);
    if (renamed !== text) {
      log(`  Document was modified`);
      log(`  Preview: ${renamed.substring(0, 200)}...`);
    } else {
      log(`  Document unchanged`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ Rename failed: ${errMsg}`, "error");
  }

  log("\n--- Testing CodeActions ---");
  try {
    const diags = await wasmApi.getDiagnostics(text);
    const actions = await wasmApi.codeActions(text, diags);
    log(`✅ CodeActions: ${actions.length} actions`);
    actions.slice(0, 5).forEach((a, i) => {
      log(`  ${i + 1}. ${a.title} (${a.command})`);
    });
    if (actions.length > 5) {
      log(`  ... and ${actions.length - 5} more`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ CodeActions failed: ${errMsg}`, "error");
  }

  log("\n--- Testing SemanticTokens ---");
  try {
    const tokens = await wasmApi.semanticTokens(text);
    log(`✅ SemanticTokens: ${tokens.length} tokens`);
    log(`  Token data length: ${tokens.length} (${tokens.length / 5} semantic tokens)`);
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ SemanticTokens failed: ${errMsg}`, "error");
  }

  log("\n--- Testing DocumentLinks ---");
  try {
    const links = await wasmApi.documentLinks(text);
    log(`✅ DocumentLinks: ${links.length} links`);
    links.slice(0, 5).forEach((link, i) => {
      log(`  ${i + 1}. Line ${link.range.start.line}, target: ${link.target || "none"}`);
    });
    if (links.length > 5) {
      log(`  ... and ${links.length - 5} more`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ DocumentLinks failed: ${errMsg}`, "error");
  }

  log("\n--- Testing FoldingRanges ---");
  try {
    const ranges = await wasmApi.foldingRanges(text);
    log(`✅ FoldingRanges: ${ranges.length} ranges`);
    ranges.slice(0, 5).forEach((range, i) => {
      log(`  ${i + 1}. Lines ${range.startLine}-${range.endLine} (${range.kind || "region"})`);
    });
    if (ranges.length > 5) {
      log(`  ... and ${ranges.length - 5} more`);
    }
  } catch (error) {
    const errMsg = error instanceof Error ? error.message : String(error);
    log(`❌ FoldingRanges failed: ${errMsg}`, "error");
  }

  log("\n=== Debug Session Complete ===");
  vscode.window.showInformationMessage(
    "WASM LSP debug complete. Check output channel for details."
  );
}

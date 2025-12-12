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

  log("\n=== Debug Session Complete ===");
  vscode.window.showInformationMessage(
    "WASM LSP debug complete. Check output channel for details."
  );
}

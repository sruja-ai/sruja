// Debug utilities for WASM LSP
import * as vscode from "vscode";
import { initWasmNode } from "@sruja/shared/node/wasmAdapter";
import { log, getOutputChannel, setOutputChannel } from "./utils";
import { getWasmApi, setWasmApi } from "./initialization";

/**
 * Debug command to test WASM functions.
 *
 * @remarks
 * Tests all WASM LSP functions with the currently open document
 * and logs results to the output channel.
 */
export async function debugWasmLsp(): Promise<void> {
  let outputChannel = getOutputChannel();
  if (!outputChannel) {
    outputChannel = vscode.window.createOutputChannel("Sruja WASM LSP");
    setOutputChannel(outputChannel);
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

  let wasmApi = getWasmApi();
  if (!wasmApi) {
    log("WASM API not initialized. Attempting to initialize...", "warn");
    try {
      const extensionPath = vscode.extensions.getExtension("srujaai.sruja")?.extensionPath;
      if (!extensionPath) {
        throw new Error("Extension path not found");
      }
      wasmApi = await initWasmNode({ extensionPath });
      setWasmApi(wasmApi);
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

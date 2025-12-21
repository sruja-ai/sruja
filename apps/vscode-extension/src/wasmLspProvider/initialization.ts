// WASM initialization and verification
import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import { initWasmNode } from "@sruja/shared/node/wasmAdapter";
import { log, setOutputChannel, setupConsoleInterception } from "./utils";

let wasmApi: Awaited<ReturnType<typeof initWasmNode>> | null = null;

/**
 * Gets the current WASM API instance.
 * 
 * @returns The WASM API instance or null if not initialized
 */
export function getWasmApi() {
  return wasmApi;
}

/**
 * Sets the WASM API instance (for testing or re-initialization).
 * 
 * @param api - The WASM API instance to set
 */
export function setWasmApi(api: Awaited<ReturnType<typeof initWasmNode>> | null) {
  wasmApi = api;
}

/**
 * Verifies that required WASM files exist.
 * 
 * @param extensionPath - Path to the VS Code extension
 * @throws Error if required files are missing
 */
function verifyWasmFiles(extensionPath: string): void {
  const wasmPath = path.join(extensionPath, "wasm", "sruja.wasm.gz");
  const wasmPathUncompressed = path.join(extensionPath, "wasm", "sruja.wasm");
  const wasmExecPath = path.join(extensionPath, "wasm", "wasm_exec.js");

  log(`Extension path: ${extensionPath}`);
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
}

/**
 * Tests all WASM LSP functions to verify they work correctly.
 * 
 * @returns Record of test results for each function
 */
async function testWasmFunctions(): Promise<Record<string, { success: boolean; error?: string }>> {
  if (!wasmApi) {
    throw new Error("WASM API not initialized");
  }

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

  return testResults;
}

/**
 * Initializes the WASM LSP module.
 * 
 * @param context - VS Code extension context
 * @throws Error if initialization fails
 * 
 * @remarks
 * This function:
 * 1. Creates output channel for logging
 * 2. Verifies WASM files exist
 * 3. Initializes WASM module with timeout protection
 * 4. Tests all LSP functions
 */
export async function initializeWasm(context: vscode.ExtensionContext): Promise<void> {
  // Create output channel for debugging
  const outputChannel = vscode.window.createOutputChannel("Sruja WASM LSP");
  context.subscriptions.push(outputChannel);
  outputChannel.show(true); // Show output channel automatically for debugging
  setOutputChannel(outputChannel);

  // Intercept console logs from wasmAdapter
  setupConsoleInterception();

  log("Initializing WASM LSP...");

  // Verify WASM files exist
  verifyWasmFiles(context.extensionPath);

  log("WASM files verified. Initializing WASM module...");
  log("Calling initWasmNode...");
  const initStartTime = Date.now();
  
  // Show progress in output channel
  log("⏳ Loading WASM module (this may take a few seconds on first load)...");

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
    log(`✅ WASM module loaded successfully (took ${initDuration}ms)`);
    
    if (initDuration > 2000) {
      log(`⚠️  WASM loading took ${initDuration}ms - consider using compressed WASM for faster startup`, "warn");
    }
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
    throw new Error(`WASM initialization failed: ${errMsg}`);
  }

  // Verify WASM API is available
  if (!wasmApi) {
    log("WASM API is null - initialization may have failed silently", "error");
    throw new Error("WASM API initialization returned null");
  }

  log("WASM API object created successfully");

  // Test all LSP functions
  await testWasmFunctions();
}


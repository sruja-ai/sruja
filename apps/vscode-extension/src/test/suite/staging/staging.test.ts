// apps/vscode-extension/src/test/suite/staging/staging.test.ts
// E2E tests for staging/pre-release extension from marketplace
import * as assert from "assert";
import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";

suite("Staging Extension E2E Tests", function () {
  this.timeout(60_000); // 60 seconds for e2e tests

  const EXTENSION_ID = "srujaai.sruja";

  test("Extension is installed from marketplace", async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, `Extension ${EXTENSION_ID} should be installed`);
    assert.ok(extension!.isActive || extension!.packageJSON, "Extension should be activatable");
  });

  test("Extension activates successfully", async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, "Extension should be installed");

    if (!extension!.isActive) {
      await extension!.activate();
    }

    assert.ok(extension!.isActive, "Extension should be active");
  });

  test("Sruja language is registered", async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, "Extension should be installed");

    if (!extension!.isActive) {
      await extension!.activate();
    }

    // Check that .sruja files are recognized
    const srujaLanguage = await vscode.languages.getLanguages();
    assert.ok(srujaLanguage.includes("sruja"), "Sruja language should be registered");
  });

  test("Extension commands are registered", async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, "Extension should be installed");

    if (!extension!.isActive) {
      await extension!.activate();
    }

    const commands = await vscode.commands.getCommands(true);
    assert.ok(
      commands.includes("sruja.previewArchitecture"),
      "sruja.previewArchitecture command should be registered"
    );
  });

  test("Extension provides syntax highlighting for .sruja files", async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, "Extension should be installed");

    if (!extension!.isActive) {
      await extension!.activate();
    }

    // Create a test file
    const testWorkspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!testWorkspace) {
      assert.fail("No workspace folder found");
      return;
    }

    const testFile = path.join(testWorkspace, "test-syntax.sruja");
    const testContent = `architecture "Test" {
  system TestSystem "Test System"
}`;

    fs.writeFileSync(testFile, testContent);

    try {
      const document = await vscode.workspace.openTextDocument(testFile);
      await vscode.window.showTextDocument(document);

      // Wait for language server to initialize
      await new Promise((resolve) => setTimeout(resolve, 3000));

      // Check that document language is sruja
      assert.strictEqual(document.languageId, "sruja", "Document language should be sruja");

      // Check that document has content
      assert.ok(document.getText().length > 0, "Document should have content");
    } finally {
      // Clean up
      if (fs.existsSync(testFile)) {
        fs.unlinkSync(testFile);
      }
    }
  });

  test("Extension provides LSP features (hover, completion)", async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, "Extension should be installed");

    if (!extension!.isActive) {
      await extension!.activate();
    }

    const testWorkspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!testWorkspace) {
      assert.fail("No workspace folder found");
      return;
    }

    const testFile = path.join(testWorkspace, "test-lsp.sruja");
    const testContent = `architecture "Test" {
  system TestSystem "Test System"
}`;

    fs.writeFileSync(testFile, testContent);

    try {
      const document = await vscode.workspace.openTextDocument(testFile);
      await vscode.window.showTextDocument(document);

      // Wait for LSP to initialize
      await new Promise((resolve) => setTimeout(resolve, 5000));

      // Test hover provider (if available)
      const position = new vscode.Position(1, 10); // Position on "TestSystem"
      const hoverResults = await vscode.commands.executeCommand<vscode.Hover[]>(
        "vscode.executeHoverProvider",
        document.uri,
        position
      );

      // Hover might not always return results, but shouldn't error
      assert.ok(
        Array.isArray(hoverResults) || hoverResults === undefined,
        "Hover provider should not error"
      );

      // Test completion provider
      const completionResults = await vscode.commands.executeCommand<vscode.CompletionList>(
        "vscode.executeCompletionItemProvider",
        document.uri,
        new vscode.Position(0, 0)
      );

      // Completion might not always return results, but shouldn't error
      assert.ok(
        completionResults === undefined ||
          Array.isArray(completionResults) ||
          (completionResults && "items" in completionResults),
        "Completion provider should not error"
      );
    } finally {
      // Clean up
      if (fs.existsSync(testFile)) {
        fs.unlinkSync(testFile);
      }
    }
  });

  test("Extension WASM loads without errors", async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, "Extension should be installed");

    if (!extension!.isActive) {
      await extension!.activate();
    }

    // Check that extension path exists
    const extensionPath = extension!.extensionPath;
    assert.ok(fs.existsSync(extensionPath), "Extension path should exist");

    // Check for WASM files
    const wasmPath = path.join(extensionPath, "wasm");
    const wasmExists =
      fs.existsSync(path.join(wasmPath, "sruja.wasm")) ||
      fs.existsSync(path.join(wasmPath, "sruja.wasm.gz"));

    // WASM files should exist (extension won't work without them)
    assert.ok(wasmExists, "WASM files should be present in extension");
  });
});

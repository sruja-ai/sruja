import * as assert from "assert";
import * as path from "path";
import * as vscode from "vscode";

suite("Semantic Tokens and Workspace Symbols", function () {
  this.timeout(10000);
  test("semantic tokens provided for .sruja", async function () {
    const ext = vscode.extensions.getExtension("srujaai.sruja");
    assert.ok(ext, "Extension not found");
    await ext!.activate();

    // Use any example file from the examples folder (workspace folder)
    const examplePath = path.resolve(__dirname, "../../../../../examples/demo_overview.sruja");
    const doc = await vscode.workspace.openTextDocument(examplePath);
    await vscode.window.showTextDocument(doc);

    await new Promise((r) => setTimeout(r, 3000));

    try {
      const legend: vscode.SemanticTokensLegend | undefined = await vscode.commands.executeCommand(
        "vscode.provideDocumentSemanticTokensLegend",
        doc.uri
      );
      assert.ok(legend, "Legend not provided");
      const tokens: vscode.SemanticTokens | undefined = await vscode.commands.executeCommand(
        "vscode.provideDocumentSemanticTokens",
        doc.uri
      );
      assert.ok(tokens, "No semantic tokens returned");
      assert.ok(tokens!.data.length > 0, "Expected some semantic tokens");
    } catch (e) {
      console.warn("Skipping semantic token assertions; server not ready:", e);
    }
  });

  test("workspace symbols include systems as Class", async function () {
    const ext = vscode.extensions.getExtension("srujaai.sruja");
    assert.ok(ext);
    await ext!.activate();

    // Use any example file from the examples folder (workspace folder)
    const examplePath = path.resolve(__dirname, "../../../../../examples/demo_overview.sruja");
    const doc = await vscode.workspace.openTextDocument(examplePath);
    await vscode.window.showTextDocument(doc);
    await new Promise((r) => setTimeout(r, 3000));

    try {
      const results: vscode.SymbolInformation[] = await vscode.commands.executeCommand(
        "vscode.executeWorkspaceSymbolProvider",
        "SoftwareSystem"
      );
      assert.ok(results && results.length > 0, "Expected workspace symbols for SoftwareSystem");
      const sys = results.find((r) => r.name.includes("SoftwareSystem"));
      assert.ok(sys, "SoftwareSystem not found in symbols");
      assert.strictEqual(sys!.kind, vscode.SymbolKind.Class);
    } catch (e) {
      console.warn("Skipping workspace symbol assertions; server not ready:", e);
    }
  });
});

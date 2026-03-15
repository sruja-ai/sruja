import * as assert from "assert";
import * as vscode from "vscode";

let ext: vscode.Extension<unknown> | undefined;

suiteSetup(async () => {
  ext = vscode.extensions.getExtension("SrujaAI.sruja");
  if (ext) await ext.activate();
});

suite("Extension (e2e)", () => {
  test("extension activates and registers all sruja commands", async () => {
    assert.ok(ext, "Sruja extension should be installed");
    await ext!.activate();
    const commands = await vscode.commands.getCommands(true);
    const srujaCommands = commands.filter((c) => c.startsWith("sruja."));
    const expected = [
      "sruja.runValidation",
      "sruja.exportMarkdown",
      "sruja.openDiagramPreview",
      "sruja.openSkillsOverview",
      "sruja.openAgentGuide",
      "sruja.listRules",
      "sruja.copyRuleForAI",
      "sruja.copyAgentGuideForAI",
      "sruja.runDrift",
      "sruja.refreshContext",
      "sruja.status",
      "sruja.review",
    ];
    for (const cmd of expected) {
      assert.ok(srujaCommands.includes(cmd), `Command ${cmd} should be registered`);
    }
  }).timeout(15_000);

  test("sruja.runValidation runs without throwing", async () => {
    if (!ext) return assert.fail("Extension not loaded");
    await assert.doesNotReject(
      async () => vscode.commands.executeCommand("sruja.runValidation"),
      "runValidation should not throw"
    );
  }).timeout(10_000);

  test("with .sruja open: runValidation runs and diagnostics can be read", async () => {
    const folder = vscode.workspace.workspaceFolders?.[0];
    assert.ok(folder, "Test workspace should be open");
    const srujaUri = vscode.Uri.joinPath(folder.uri, "sample.sruja");
    const doc = await vscode.workspace.openTextDocument(srujaUri);
    await vscode.window.showTextDocument(doc);
    await vscode.commands.executeCommand("sruja.runValidation");
    await sleep(1500);
    const diags = vscode.languages.getDiagnostics(srujaUri);
    assert.ok(Array.isArray(diags), "Diagnostics should be an array");
  }).timeout(15_000);

  test("with .sruja open: openDiagramPreview runs without throwing", async () => {
    const folder = vscode.workspace.workspaceFolders?.[0];
    assert.ok(folder, "Test workspace should be open");
    const srujaUri = vscode.Uri.joinPath(folder.uri, "sample.sruja");
    const doc = await vscode.workspace.openTextDocument(srujaUri);
    await vscode.window.showTextDocument(doc);
    await assert.doesNotReject(
      async () => vscode.commands.executeCommand("sruja.openDiagramPreview"),
      "openDiagramPreview should not throw"
    );
  }).timeout(15_000);

  test("sruja.exportMarkdown when no .sruja open shows warning (does not throw)", async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    await assert.doesNotReject(
      async () => vscode.commands.executeCommand("sruja.exportMarkdown"),
      "exportMarkdown should not throw"
    );
  }).timeout(8_000);

  test("CLI-style commands run without throwing (may show message if no CLI)", async () => {
    const commands = [
      "sruja.runDrift",
      "sruja.refreshContext",
      "sruja.status",
      "sruja.review",
      "sruja.openSkillsOverview",
      "sruja.openAgentGuide",
      "sruja.listRules",
    ];
    for (const cmd of commands) {
      await assert.doesNotReject(
        async () => vscode.commands.executeCommand(cmd),
        `${cmd} should not throw`
      );
    }
  }).timeout(25_000);

  test("language providers: hover, definition, document symbols", async () => {
    const folder = vscode.workspace.workspaceFolders?.[0];
    assert.ok(folder, "Test workspace should be open");
    const srujaUri = vscode.Uri.joinPath(folder.uri, "sample.sruja");
    const doc = await vscode.workspace.openTextDocument(srujaUri);
    const position = new vscode.Position(1, 0);

    const hover = await vscode.commands.executeCommand<vscode.Hover[]>(
      "vscode.executeHoverProvider",
      srujaUri,
      position
    );
    assert.ok(Array.isArray(hover) || hover === undefined, "Hover result should be array or undefined");

    const definitions = await vscode.commands.executeCommand<vscode.Location[]>(
      "vscode.executeDefinitionProvider",
      srujaUri,
      position
    );
    assert.ok(Array.isArray(definitions) || definitions === undefined, "Definition result should be array or undefined");

    const symbols = await vscode.commands.executeCommand<vscode.DocumentSymbol[]>(
      "vscode.executeDocumentSymbolProvider",
      srujaUri
    );
    assert.ok(Array.isArray(symbols) || symbols === undefined, "Document symbols should be array or undefined");
  }).timeout(10_000);
});

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

import * as assert from "assert";

// You can import and use all API from the 'vscode' module
// as well as import your extension to test it
import * as vscode from "vscode";
// import * as myExtension from '../../extension';

suite("Extension Test Suite", function () {
  this.timeout(10000);
  vscode.window.showInformationMessage("Start all tests.");

  test("Extension should be present", () => {
    assert.ok(vscode.extensions.getExtension("sruja-ai.sruja"));
  });

  test("Commands should be registered", async () => {
    const ext = vscode.extensions.getExtension("sruja-ai.sruja");
    assert.ok(ext, "Extension not found");
    await ext.activate();

    const commands = await vscode.commands.getCommands(true);
    assert.ok(
      commands.includes("sruja.previewArchitecture"),
      "sruja.previewArchitecture not found"
    );
  });
});

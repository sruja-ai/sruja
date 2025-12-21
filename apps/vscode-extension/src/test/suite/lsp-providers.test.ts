import * as assert from "assert";
import * as vscode from "vscode";
import { describe, it, before } from "mocha";

describe("LSP Provider Tests", function () {
  this.timeout(10000);

  let testDocument: vscode.TextDocument;

  before(async () => {
    const testContent = `
architecture "E-Commerce" {
  person Customer "Customer"
  
  system OrderService "Order Service" {
    container OrderAPI "Order API" {
      technology "Spring Boot"
    }
  }
  
  Customer -> OrderService.OrderAPI "places orders"
}
`;
    testDocument = await vscode.workspace.openTextDocument({
      language: "sruja",
      content: testContent,
    });
  });

  it("should provide hover information", async () => {
    const position = new vscode.Position(4, 10); // Position on "OrderService"
    const hovers = await vscode.commands.executeCommand<vscode.Hover[]>(
      "vscode.executeHoverProvider",
      testDocument.uri,
      position
    );
    
    assert.ok(hovers, "Hover provider should return results");
  });

  it("should provide completion items", async () => {
    const position = new vscode.Position(2, 0);
    const completions = await vscode.commands.executeCommand<vscode.CompletionList>(
      "vscode.executeCompletionItemProvider",
      testDocument.uri,
      position
    );
    
    assert.ok(completions, "Completion provider should return results");
    if (completions && "items" in completions) {
      assert.ok(Array.isArray(completions.items), "Completions should be an array");
    }
  });

  it("should provide document symbols", async () => {
    const symbols = await vscode.commands.executeCommand<vscode.DocumentSymbol[]>(
      "vscode.executeDocumentSymbolProvider",
      testDocument.uri
    );
    
    assert.ok(symbols, "Document symbol provider should return results");
    assert.ok(Array.isArray(symbols), "Symbols should be an array");
  });

  it("should provide diagnostics", async () => {
    const diagnostics = vscode.languages.getDiagnostics(testDocument.uri);
    
    assert.ok(Array.isArray(diagnostics), "Diagnostics should be an array");
  });

  it("should provide formatting", async () => {
    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      "vscode.executeFormatDocumentProvider",
      testDocument.uri,
      { tabSize: 2, insertSpaces: true }
    );
    
    assert.ok(Array.isArray(edits), "Format provider should return edits array");
  });

  it("should provide code lenses", async () => {
    const lenses = await vscode.commands.executeCommand<vscode.CodeLens[]>(
      "vscode.executeCodeLensProvider",
      testDocument.uri
    );
    
    assert.ok(Array.isArray(lenses), "Code lens provider should return lenses array");
  });

  it("should provide inlay hints", async () => {
    const range = new vscode.Range(0, 0, 10, 0);
    const hints = await vscode.commands.executeCommand<vscode.InlayHint[]>(
      "vscode.executeInlayHintProvider",
      testDocument.uri,
      range
    );
    
    assert.ok(Array.isArray(hints), "Inlay hints provider should return hints array");
  });
});


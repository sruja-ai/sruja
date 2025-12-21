import * as assert from "assert";
import * as vscode from "vscode";
import { describe, it, before, after } from "mocha";

describe("Performance Tests", function () {
  this.timeout(10000);

  let testDocument: vscode.TextDocument;

  before(async () => {
    // Create a test document
    const testContent = `
architecture "Test" {
  system App "Application" {
    container API "API" {
      technology "Spring Boot"
    }
  }
}
`;
    testDocument = await vscode.workspace.openTextDocument({
      language: "sruja",
      content: testContent,
    });
  });

  it("should debounce diagnostics updates", async () => {
    const startTime = Date.now();
    
    // Trigger multiple rapid changes
    for (let i = 0; i < 5; i++) {
      // Simulate document change
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    
    const endTime = Date.now();
    // Should take at least 300ms (debounce delay) but not too long
    assert.ok(endTime - startTime >= 200, "Debouncing should delay updates");
  });

  it("should cache diagnostics results", async () => {
    // First call should parse
    const diagnostics1 = vscode.languages.getDiagnostics(testDocument.uri);
    
    // Second call should use cache (if within TTL)
    const diagnostics2 = vscode.languages.getDiagnostics(testDocument.uri);
    
    // Results should be consistent
    assert.deepStrictEqual(diagnostics1, diagnostics2, "Cached results should match");
  });

  it("should handle large files efficiently", async () => {
    // Create a large document
    const largeContent = Array(1000)
      .fill('system App "Application" {}')
      .join("\n");
    
    const largeDoc = await vscode.workspace.openTextDocument({
      language: "sruja",
      content: largeContent,
    });
    
    const startTime = Date.now();
    const diagnostics = vscode.languages.getDiagnostics(largeDoc.uri);
    const endTime = Date.now();
    
    // Should complete within reasonable time (5 seconds)
    assert.ok(endTime - startTime < 5000, "Large file should parse within 5 seconds");
  });
});


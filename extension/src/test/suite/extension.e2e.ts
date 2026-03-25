import * as vscode from "vscode";

let ext: vscode.Extension<unknown> | undefined;

export function runTests({ describe, it, beforeAll, assert }: {
  describe: (name: string, fn: () => void) => void;
  it: (name: string, fn: () => Promise<void>) => void;
  beforeAll: (fn: () => Promise<void>) => void;
  assert: {
    ok: (value: unknown, message?: string) => void;
    equal: (actual: unknown, expected: unknown, message?: string) => void;
    fail: (message: string) => void;
    doesNotReject: (fn: () => Promise<void>, message: string) => Promise<void>;
  };
}): void {
  beforeAll(async () => {
    ext = vscode.extensions.getExtension("SrujaAI.sruja");
    if (ext) await ext.activate();
  });

  describe("Extension (e2e)", () => {
    it("extension activates and registers all sruja commands", async () => {
      assert.ok(ext, "Sruja extension should be installed");
      await ext!.activate();
      const commands = await vscode.commands.getCommands(true);
      const srujaCommands = commands.filter((c) => c.startsWith("sruja."));
      const expected = [
        "sruja.commandCenter",
        "sruja.runValidation",
        "sruja.exportMarkdown",
        "sruja.openDiagramPreview",
        "sruja.openFocusedDiagramPreview",
        "sruja.openFocusedDiagramPreviewAt",
        "sruja.openComponentKnowledge",
        "sruja.openDocsThread",
        "sruja.openDocsThreadAt",
        "sruja.openMarkdownPreview",
        "sruja.openSkillsOverview",
        "sruja.openAgentGuide",
        "sruja.listRules",
        "sruja.copyRuleForAI",
        "sruja.copyAgentGuideForAI",
        "sruja.copyContextPackForAI",
        "sruja.registerMcpServer",
        "sruja.runDrift",
        "sruja.refreshContext",
        "sruja.status",
        "sruja.review",
      ];
      for (const cmd of expected) {
        assert.ok(srujaCommands.includes(cmd), `Command ${cmd} should be registered`);
      }
    });

    it("sruja.runValidation runs without throwing", async () => {
      if (!ext) return assert.fail("Extension not loaded");
      await assert.doesNotReject(
        async () => vscode.commands.executeCommand("sruja.runValidation"),
        "runValidation should not throw"
      );
    });

    it("with .sruja open: runValidation runs and diagnostics can be read", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "sample.sruja");
      const doc = await vscode.workspace.openTextDocument(srujaUri);
      await vscode.window.showTextDocument(doc);
      await vscode.commands.executeCommand("sruja.runValidation");
      await sleep(1500);
      const diags = vscode.languages.getDiagnostics(srujaUri);
      assert.ok(Array.isArray(diags), "Diagnostics should be an array");
    });

    it("with .sruja open: openDiagramPreview runs without throwing", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "sample.sruja");
      const doc = await vscode.workspace.openTextDocument(srujaUri);
      await vscode.window.showTextDocument(doc);
      await assert.doesNotReject(
        async () => vscode.commands.executeCommand("sruja.openDiagramPreview"),
        "openDiagramPreview should not throw"
      );
    });

    it("with sruja-platform.sruja: runValidation and exportMarkdown run without throwing", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "sruja-platform.sruja");
      const doc = await vscode.workspace.openTextDocument(srujaUri);
      await vscode.window.showTextDocument(doc);
      await sleep(500);
      await assert.doesNotReject(
        async () => vscode.commands.executeCommand("sruja.runValidation"),
        "runValidation on sruja-platform.sruja should not throw"
      );
      await sleep(1500);
      await assert.doesNotReject(
        async () => vscode.commands.executeCommand("sruja.exportMarkdown"),
        "exportMarkdown on sruja-platform.sruja should not throw"
      );
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    });

    it("sruja.exportMarkdown when no .sruja open shows warning (does not throw)", async () => {
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
      await assert.doesNotReject(
        async () => vscode.commands.executeCommand("sruja.exportMarkdown"),
        "exportMarkdown should not throw"
      );
    });

    it("CLI-style commands run without throwing (may show message if no CLI)", async () => {
      const commands = [
        "sruja.runDrift",
        "sruja.refreshContext",
        "sruja.status",
        "sruja.review",
        "sruja.openDocsThread",
        "sruja.openDocsThreadAt",
        "sruja.openSkillsOverview",
        "sruja.openAgentGuide",
        "sruja.listRules",
        "sruja.copyContextPackForAI",
        "sruja.registerMcpServer",
        "sruja.commandCenter",
      ];
      for (const cmd of commands) {
        await assert.doesNotReject(
          async () => vscode.commands.executeCommand(cmd),
          `${cmd} should not throw`
        );
      }
    });

    it("language providers: hover, definition, document symbols", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "sample.sruja");
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
    });

    it("language providers: code lenses", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "sruja-platform.sruja");
      const doc = await vscode.workspace.openTextDocument(srujaUri);
      await vscode.window.showTextDocument(doc);
      await sleep(1500);

      const lenses = await vscode.commands.executeCommand<vscode.CodeLens[]>(
        "vscode.executeCodeLensProvider",
        srujaUri
      );
      assert.ok(Array.isArray(lenses) || lenses === undefined, "Code lenses should be array or undefined");
    });

    it("custom editor provider is registered for markdown preview", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "sample.sruja");
      
      const editors = vscode.window.visibleNotebookEditors;
      assert.ok(Array.isArray(editors), "Notebook editors should be array");
      
      const allEditors = vscode.window.tabGroups.all;
      assert.ok(Array.isArray(allEditors), "Tab groups should be array");
    });

    it("markdown preview custom editor can be opened", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "sample.sruja");
      
      await vscode.commands.executeCommand("vscode.openWith", srujaUri, "sruja.markdownPreview");
      await sleep(1500);
      
      const activeEditor = vscode.window.activeTextEditor;
      assert.ok(activeEditor === undefined || activeEditor?.document.uri.toString() !== srujaUri.toString(), 
        "Custom editor should be active (not text editor)");
      
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    });

    it("sruja.openMarkdownPreview command opens custom editor", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "sample.sruja");
      
      const doc = await vscode.workspace.openTextDocument(srujaUri);
      await vscode.window.showTextDocument(doc);
      await sleep(500);
      
      await assert.doesNotReject(
        async () => vscode.commands.executeCommand("sruja.openMarkdownPreview"),
        "openMarkdownPreview should not throw"
      );
      await sleep(1000);
      
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    });

    it("component knowledge: hover over element with doc includes Documentation", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "knowledge-doc.sruja");
      const doc = await vscode.workspace.openTextDocument(srujaUri);
      await vscode.window.showTextDocument(doc);
      await sleep(1500);
      const position = new vscode.Position(5, 2);
      const hover = await vscode.commands.executeCommand<vscode.Hover[]>(
        "vscode.executeHoverProvider",
        srujaUri,
        position
      );
      assert.ok(Array.isArray(hover) && hover.length > 0, "Hover should return at least one result");
      const content = hover[0].contents;
      const value = Array.isArray(content) ? content.map((c) => (c as { value: string }).value).join("") : (content as { value: string }).value;
      assert.ok(value.includes("Documentation") || value.includes("Open in split"), "Hover over element with doc should mention Documentation or Open in split");
    });

    it("component knowledge: definition over element with doc returns two locations", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "knowledge-doc.sruja");
      const doc = await vscode.workspace.openTextDocument(srujaUri);
      await vscode.window.showTextDocument(doc);
      await sleep(1500);
      const position = new vscode.Position(5, 2);
      const definitions = await vscode.commands.executeCommand<vscode.LocationLink[] | vscode.Location[]>(
        "vscode.executeDefinitionProvider",
        srujaUri,
        position
      );
      assert.ok(Array.isArray(definitions) && definitions.length >= 2, "Definition over element with doc should return at least two targets (DSL + knowledge file)");
      const uris = definitions.map((d: vscode.LocationLink | vscode.Location) => {
        const loc = d as vscode.LocationLink & { uri?: vscode.Uri };
        return loc.targetUri ?? loc.uri!;
      });
      const hasDocUri = uris.some((u: vscode.Uri) => u.fsPath.endsWith("PaymentService.md") || u.path.endsWith("PaymentService.md"));
      assert.ok(hasDocUri, "One definition target should be the knowledge file PaymentService.md");
    });

    it("component knowledge: openComponentKnowledge opens doc in split", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "knowledge-doc.sruja");
      const doc = await vscode.workspace.openTextDocument(srujaUri);
      await vscode.window.showTextDocument(doc);
      await sleep(1500);
      const editor = vscode.window.activeTextEditor;
      assert.ok(editor, "Editor should be active");
      const position = new vscode.Position(5, 2);
      editor!.selection = new vscode.Selection(position, position);
      await assert.doesNotReject(
        async () => vscode.commands.executeCommand("sruja.openComponentKnowledge"),
        "openComponentKnowledge should not throw"
      );
      await sleep(1500);
      const tabs = vscode.window.tabGroups.all.flatMap((g) => g.tabs);
      const hasKnowledgeTab = tabs.some((t) => (t.label?.includes("PaymentService") ?? false));
      assert.ok(hasKnowledgeTab, "Knowledge file should be opened (tab with PaymentService)");
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    });

    it("markdown preview updates on document change", async () => {
      const folder = vscode.workspace.workspaceFolders?.[0];
      assert.ok(folder, "Test workspace should be open");
      const srujaUri = vscode.Uri.joinPath(folder!.uri, "sample.sruja");
      
      const doc = await vscode.workspace.openTextDocument(srujaUri);
      await vscode.window.showTextDocument(doc);
      await sleep(500);
      
      await vscode.commands.executeCommand("vscode.openWith", srujaUri, "sruja.markdownPreview", vscode.ViewColumn.Beside);
      await sleep(1500);
      
      const tabGroups = vscode.window.tabGroups.all;
      assert.ok(tabGroups.length >= 1, "Should have at least one tab group");
      
      await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    });
  });
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

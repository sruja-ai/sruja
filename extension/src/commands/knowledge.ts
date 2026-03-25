import * as vscode from "vscode";
import {
  getElementsFromWasm,
  wasmRangeToVscodeRange,
} from "../wasm";
import { resolveDocUri, docUriExists } from "../providers";
import {
  docsThreadState,
  renderDocsThreadHtml,
  DocsThreadRef,
  DocsThreadEntry,
  DocsThreadDoc,
} from "../docsThread";
import { truncateLines, formatRelPath } from "../utils";

let docsThreadPanel: vscode.WebviewPanel | undefined;

export function registerKnowledgeCommands(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.openComponentKnowledge", async (docUriArg?: string) => {
      async function openDocInSplit(uri: vscode.Uri): Promise<void> {
        const isMarkdown = uri.fsPath.toLowerCase().endsWith(".md");
        if (isMarkdown) {
          await vscode.workspace.openTextDocument(uri);
          await vscode.commands.executeCommand("markdown.showPreviewToSide", uri);
        } else {
          const doc = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(doc, {
            viewColumn: vscode.ViewColumn.Beside,
            preserveFocus: false,
          });
        }
      }

      if (docUriArg) {
        try {
          const uri = vscode.Uri.parse(docUriArg);
          await openDocInSplit(uri);
        } catch {
          vscode.window.showWarningMessage("Could not open component knowledge file.");
        }
        return;
      }

      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage("Place the cursor on an element in a .sruja file, or use the hover link to open documentation.");
        return;
      }
      const position = editor.selection.active;
      const elements = await getElementsFromWasm(context, doc.getText(), doc.uri.fsPath, doc.uri.toString(), doc.version);
      if (!elements?.length) {
        if (elements === null) {
          vscode.window.showWarningMessage("Sruja WASM is not available for component lookup. Reinstall the extension or run npm run copy:assets if developing.");
        }
        return;
      }
      const wordRange = doc.getWordRangeAtPosition(position);
      const word = wordRange ? doc.getText(wordRange).trim() : "";
      if (!word) return;
      const element = elements.find((e) => e.id === word || e.id.endsWith(`.${word}`));
      if (!element?.doc) {
        vscode.window.showInformationMessage(`Element "${word}" has no doc link. Add doc ".sruja/knowledge/..." to the element in the DSL.`);
        return;
      }
      const docUri = resolveDocUri(element.doc, doc);
      if (!docUri || !(await docUriExists(docUri))) {
        vscode.window.showWarningMessage(`Knowledge file not found: ${element.doc}`);
        return;
      }
      await openDocInSplit(docUri);
    }),

    vscode.commands.registerCommand("sruja.openDocsThread", async () => {
      docsThreadState.followCursor = true;
      createDocsThreadPanel(context);
      await pushDocsThreadEntryFromActiveEditor(context);
    }),

    vscode.commands.registerCommand("sruja.openDocsThreadAt", async (arg?: unknown) => {
      docsThreadState.followCursor = true;
      createDocsThreadPanel(context);

      const parsed =
        typeof arg === "object" && arg !== null
          ? (arg as { docUri?: unknown; elementId?: unknown })
          : undefined;
      const docUriRaw = typeof parsed?.docUri === "string" ? parsed.docUri : undefined;
      const elementId = typeof parsed?.elementId === "string" ? parsed.elementId : undefined;

      if (docUriRaw && elementId) {
        const uri = vscode.Uri.parse(docUriRaw);
        const doc = await vscode.workspace.openTextDocument(uri);
        const elements = await getElementsFromWasm(context, doc.getText(), doc.uri.fsPath, doc.uri.toString(), doc.version);
        const element = elements?.find((e) => e.id === elementId || e.id.endsWith(`.${elementId}`));
        if (element) {
          await pushDocsThreadEntryForElement(context, doc, element);
          return;
        }
      }

      await pushDocsThreadEntryFromActiveEditor(context);
    })
  );
}

export function createDocsThreadPanel(context: vscode.ExtensionContext): vscode.WebviewPanel {
  if (docsThreadPanel) {
    docsThreadPanel.dispose();
  }
  docsThreadPanel = vscode.window.createWebviewPanel(
    "srujaDocsThread",
    "Sruja – Docs & References",
    vscode.ViewColumn.Beside,
    { enableScripts: true, retainContextWhenHidden: true }
  );
  docsThreadPanel.onDidDispose(() => {
    docsThreadPanel = undefined;
  });
  docsThreadPanel.webview.onDidReceiveMessage(
    async (message) => {
      if (!message || typeof message !== "object") return;
      const type = (message as { type?: unknown }).type;
      if (type === "toggleFollow") {
        docsThreadState.followCursor = !docsThreadState.followCursor;
        if (docsThreadPanel) docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
        return;
      }
      if (type === "clear") {
        docsThreadState.entries = [];
        docsThreadState.lastPushedKey = "";
        if (docsThreadPanel) docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
        return;
      }
      if (type === "pop") {
        docsThreadState.entries.pop();
        docsThreadState.lastPushedKey = docsThreadState.entries.at(-1)?.key ?? "";
        if (docsThreadPanel) docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
        return;
      }
      if (type === "openDoc") {
        const uriRaw = (message as { uri?: unknown }).uri;
        if (typeof uriRaw !== "string") return;
        const uri = vscode.Uri.parse(uriRaw);
        const isMarkdown = uri.fsPath.toLowerCase().endsWith(".md");
        if (isMarkdown) {
          await vscode.workspace.openTextDocument(uri);
          await vscode.commands.executeCommand("markdown.showPreviewToSide", uri);
        } else {
          const doc = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(doc, { viewColumn: vscode.ViewColumn.Beside, preserveFocus: false });
        }
        return;
      }
      if (type === "openLocation") {
        const uriRaw = (message as { uri?: unknown }).uri;
        const line = (message as { line?: unknown }).line;
        const character = (message as { character?: unknown }).character;
        if (typeof uriRaw !== "string") return;
        const pos = new vscode.Position(
          typeof line === "number" ? line : 0,
          typeof character === "number" ? character : 0
        );
        const uri = vscode.Uri.parse(uriRaw);
        const doc = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(doc, { preserveFocus: false });
        editor.selection = new vscode.Selection(pos, pos);
        editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
        return;
      }
    },
    undefined,
    context.subscriptions
  );
  docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
  return docsThreadPanel;
}

export async function pushDocsThreadEntryForElement(
  context: vscode.ExtensionContext,
  doc: vscode.TextDocument,
  element: { id: string; kind: string; title: string | null; doc?: string | null; range: { start: { line: number; character: number }; end: { line: number; character: number } } }
): Promise<void> {
  if (!docsThreadPanel) return;
  const range = wasmRangeToVscodeRange(element.range);
  const key = `${doc.uri.toString()}::${element.id}`;
  if (key === docsThreadState.lastPushedKey) return;

  const parentId = (() => {
    const dot = element.id.lastIndexOf(".");
    return dot === -1 ? null : element.id.slice(0, dot);
  })();

  const docInfo = element.doc ? await readDocPreview(doc, element.doc) : null;

  let refs: DocsThreadRef[] = [];
  try {
    const locations = await vscode.commands.executeCommand<vscode.Location[]>(
      "vscode.executeReferenceProvider",
      doc.uri,
      range.start
    );
    const locs = Array.isArray(locations) ? locations : [];
    refs = locs.map((loc) => {
      const uri = loc.uri.toString();
      const rel = formatRelPath(loc.uri);
      const line = loc.range.start.line;
      const character = loc.range.start.character;
      const lineText =
        loc.uri.toString() === doc.uri.toString() && line >= 0 && line < doc.lineCount
          ? doc.lineAt(line).text
          : null;
      return { uri, rel, line, character, lineText };
    });
  } catch {
    refs = [];
  }

  const entry: DocsThreadEntry = {
    key,
    sourceUri: doc.uri.toString(),
    elementId: element.id,
    kind: element.kind,
    title: element.title,
    parentId,
    range: {
      startLine: range.start.line,
      startCharacter: range.start.character,
      endLine: range.end.line,
      endCharacter: range.end.character,
    },
    doc: docInfo,
    refs,
    createdAtMs: Date.now(),
  };

  docsThreadState.entries.push(entry);
  docsThreadState.lastPushedKey = key;
  if (docsThreadState.entries.length > 20) docsThreadState.entries.shift();
  if (docsThreadPanel) docsThreadPanel.webview.html = renderDocsThreadHtml(docsThreadPanel.webview);
}

export async function pushDocsThreadEntryFromActiveEditor(context: vscode.ExtensionContext): Promise<void> {
  if (!docsThreadPanel || !docsThreadState.followCursor) return;
  const editor = vscode.window.activeTextEditor;
  const doc = editor?.document;
  if (!doc || doc.languageId !== "sruja") return;

  const position = editor.selection.active;
  const elements = await getElementsFromWasm(context, doc.getText(), doc.uri.fsPath, doc.uri.toString(), doc.version);
  if (!elements) return;

  const element = elements.find((e) => {
    const r = wasmRangeToVscodeRange(e.range);
    return r.contains(position);
  });
  if (element) {
    await pushDocsThreadEntryForElement(context, doc, element);
  }
}

async function readDocPreview(
  sourceDoc: vscode.TextDocument,
  docPath: string
): Promise<DocsThreadDoc | null> {
  const resolved = resolveDocUri(docPath, sourceDoc);
  if (!resolved) {
    return {
      path: docPath,
      uri: "",
      exists: false,
      isMarkdown: docPath.toLowerCase().endsWith(".md"),
      previewText: null,
      omittedLines: 0,
    };
  }
  const exists = await docUriExists(resolved);
  if (!exists) {
    return {
      path: docPath,
      uri: resolved.toString(),
      exists: false,
      isMarkdown: resolved.fsPath.toLowerCase().endsWith(".md"),
      previewText: null,
      omittedLines: 0,
    };
  }
  try {
    const raw = await vscode.workspace.fs.readFile(resolved);
    const text = Buffer.from(raw).toString("utf8");
    const { body, omittedLines } = truncateLines(text, 40, 2000);
    return {
      path: docPath,
      uri: resolved.toString(),
      exists: true,
      isMarkdown: resolved.fsPath.toLowerCase().endsWith(".md"),
      previewText: body,
      omittedLines,
    };
  } catch {
    return {
      path: docPath,
      uri: resolved.toString(),
      exists: true,
      isMarkdown: resolved.fsPath.toLowerCase().endsWith(".md"),
      previewText: null,
      omittedLines: 0,
    };
  }
}

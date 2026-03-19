/**
 * VS Code providers for Sruja DSL
 */

import * as path from "path";
import * as vscode from "vscode";
import {
  getElementsFromWasm,
  getDocumentSymbolsFromWasm,
  wasmRangeToVscodeRange,
  SrujaElement,
} from "./wasm";

export interface ElementRange {
  start: { line: number; character: number };
  end: { line: number; character: number };
}

export interface DefinitionResult {
  originSelectionRange: ElementRange;
  targetRange: ElementRange;
  targetSelectionRange: ElementRange;
}

/**
 * Resolve a doc path (relative to workspace root) to a file URI.
 * Returns undefined if no workspace folder or path is empty.
 */
export function resolveDocUri(
  docPath: string | null | undefined,
  document: vscode.TextDocument
): vscode.Uri | undefined {
  if (!docPath || !docPath.trim()) return undefined;
  const folder = vscode.workspace.getWorkspaceFolder(document.uri);
  if (!folder) return undefined;
  const absolute = path.resolve(folder.uri.fsPath, docPath.trim());
  return vscode.Uri.file(absolute);
}

/**
 * Check if a file URI exists in the workspace.
 */
export async function docUriExists(uri: vscode.Uri): Promise<boolean> {
  try {
    await vscode.workspace.fs.stat(uri);
    return true;
  } catch {
    return false;
  }
}

export function buildDefinitionLinks(
  word: string,
  wordRange: ElementRange,
  elements: SrujaElement[]
): DefinitionResult[] | undefined {
  const element = elements.find(e => e.id === word || e.id.endsWith(`.${word}`));
  if (!element) return undefined;

  return [{
    originSelectionRange: wordRange,
    targetRange: element.range,
    targetSelectionRange: element.range,
  }];
}

/**
 * Go to Definition Provider
 * Allows navigating from element references to their definitions
 */
export class SrujaDefinitionProvider implements vscode.DefinitionProvider {
  constructor(private context: vscode.ExtensionContext) {}

  /**
   * Provide definition for element references
   * Returns the location of the element definition
   */
  async provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): Promise<vscode.LocationLink[] | undefined> {
    if (document.languageId !== "sruja") return undefined;

    const elements = await getElementsFromWasm(this.context, document.getText(), document.uri.fsPath);
    if (token.isCancellationRequested || !elements) return undefined;

    const wordRange = document.getWordRangeAtPosition(position);
    if (!wordRange) return undefined;

    const word = document.getText(wordRange).trim();
    if (!word) return undefined;

    const links = buildDefinitionLinks(
      word,
      { start: { line: wordRange.start.line, character: wordRange.start.character }, end: { line: wordRange.end.line, character: wordRange.end.character } },
      elements
    );
    if (!links) return undefined;

    const element = elements.find((e) => e.id === word || e.id.endsWith(`.${word}`));
    const originStart = new vscode.Position(wordRange.start.line, wordRange.start.character);
    const originEnd = new vscode.Position(wordRange.end.line, wordRange.end.character);
    const originRange = new vscode.Range(originStart, originEnd);

    const result: vscode.LocationLink[] = links.map((link) => {
      const targetRange = wasmRangeToVscodeRange(link.targetRange);
      return {
        originSelectionRange: originRange,
        targetUri: document.uri,
        targetRange,
        targetSelectionRange: targetRange,
      };
    });

    // When element has doc, add a second definition target (knowledge file) if it exists
    if (element?.doc) {
      const docUri = resolveDocUri(element.doc, document);
      if (docUri && (await docUriExists(docUri))) {
        result.push({
          originSelectionRange: originRange,
          targetUri: docUri,
          targetRange: new vscode.Range(0, 0, 0, 0),
          targetSelectionRange: new vscode.Range(0, 0, 0, 0),
        });
      }
    }

    return result;
  }
}

/**
 * Hover Provider
 * Shows element information when hovering over element references
 */
export class SrujaHoverProvider implements vscode.HoverProvider {
  constructor(private context: vscode.ExtensionContext) {}

  /**
   * Provide hover information for element references
   */
  async provideHover(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): Promise<vscode.Hover | undefined> {
    if (document.languageId !== "sruja") return undefined;

    const elements = await getElementsFromWasm(this.context, document.getText(), document.uri.fsPath);
    if (token.isCancellationRequested || !elements) return undefined;

    const wordRange = document.getWordRangeAtPosition(position);
    if (!wordRange) return undefined;

    const word = document.getText(wordRange).trim();
    if (!word) return undefined;

    // Find element matching the word
    const element = elements.find(e => e.id === word || e.id.endsWith(`.${word}`));
    if (!element) return undefined;

    // Build hover markdown
    const markdown = new vscode.MarkdownString();
    markdown.appendMarkdown(`**${element.id}**\n\n`);
    markdown.appendMarkdown(`*Kind:* ${element.kind}\n\n`);

    if (element.title) {
      markdown.appendMarkdown(`*Title:* ${element.title}\n\n`);
    }

    // Add parent info for nested elements
    const dotIndex = element.id.lastIndexOf('.');
    if (dotIndex !== -1) {
      const parentId = element.id.substring(0, dotIndex);
      markdown.appendMarkdown(`*Parent:* \`${parentId}\`\n\n`);
    }

    // When element has doc, add clickable link to open knowledge file in split (Markdown as preview)
    if (element.doc) {
      const docUri = resolveDocUri(element.doc, document);
      if (docUri) {
        const args = [docUri.toString()];
        const cmdUri = vscode.Uri.parse(
          `command:sruja.openComponentKnowledge?${encodeURIComponent(JSON.stringify(args))}`
        );
        const isMd = docUri.fsPath.toLowerCase().endsWith(".md");
        const label = isMd ? "Open doc (preview)" : "Open in split";
        markdown.appendMarkdown(`*Documentation:* [${label}](${cmdUri})\n\n`);
        markdown.isTrusted = true;
      }
    }

    const viewLevel = elementKindToViewLevel(element.kind);
    if (viewLevel !== 1) {
      const args = [{ docUri: document.uri.toString(), viewLevel, targetId: element.id }];
      const cmdUri = vscode.Uri.parse(
        `command:sruja.openFocusedDiagramPreviewAt?${encodeURIComponent(JSON.stringify(args))}`
      );
      markdown.appendMarkdown(`*Diagram:* [Open focused diagram (L${viewLevel})](${cmdUri})\n\n`);
      markdown.isTrusted = true;
    }

    return new vscode.Hover(markdown, wordRange);
  }
}

/**
 * Document Symbol Provider
 * Provides outline view for Sruja documents
 */
export class SrujaDocumentSymbolProvider implements vscode.DocumentSymbolProvider {
  constructor(private context: vscode.ExtensionContext) {}

  /**
   * Provide document symbols for outline view
   */
  async provideDocumentSymbols(
    document: vscode.TextDocument,
    token: vscode.CancellationToken
  ): Promise<vscode.DocumentSymbol[] | undefined> {
    if (document.languageId !== "sruja") return undefined;

    const symbols = await getDocumentSymbolsFromWasm(this.context, document.getText(), document.uri.fsPath);
    if (token.isCancellationRequested || !symbols) return undefined;

    // Convert Sruja symbols to VS Code document symbols (WASM uses 1-based line/character)
    const documentSymbols: vscode.DocumentSymbol[] = symbols.map((symbol) => {
      const kind = this.kindToSymbolKind(symbol.kind);
      const range = wasmRangeToVscodeRange(symbol.range);

      return new vscode.DocumentSymbol(
        symbol.name,
        symbol.detail,
        kind,
        range,
        range
      );
    });

    return documentSymbols;
  }

  /**
   * Convert Sruja symbol kind to VS Code SymbolKind
   */
  private kindToSymbolKind(kind: string): vscode.SymbolKind {
    switch (kind) {
      case "element":
        return vscode.SymbolKind.Class;
      case "view":
        return vscode.SymbolKind.Interface;
      case "scenario":
        return vscode.SymbolKind.Method;
      case "flow":
        return vscode.SymbolKind.Function;
      case "requirement":
        return vscode.SymbolKind.Boolean;
      case "adr":
        return vscode.SymbolKind.Enum;
      case "policy":
        return vscode.SymbolKind.Event;
      default:
        return vscode.SymbolKind.Object;
    }
  }
}

type FocusDiagramArgs = {
  docUri: string;
  viewLevel: 1 | 2 | 3;
  targetId?: string;
};

function elementKindToViewLevel(kind: string): 1 | 2 | 3 {
  const k = kind.toLowerCase();
  if (k.includes("component")) return 3;
  if (k.includes("container")) return 2;
  if (k.includes("database")) return 2;
  if (k.includes("system")) return 2;
  return 1;
}

function findElementsByRegex(document: vscode.TextDocument): Array<{ id: string; kind: string; range: vscode.Range }> {
  const re = /^\s*([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)?)\s*=\s*(container|component|system|database)\b/;
  const out: Array<{ id: string; kind: string; range: vscode.Range }> = [];
  for (let i = 0; i < document.lineCount; i++) {
    const line = document.lineAt(i).text;
    const m = re.exec(line);
    if (!m) continue;
    const id = m[1];
    const kind = m[2];
    out.push({
      id,
      kind,
      range: new vscode.Range(new vscode.Position(i, 0), new vscode.Position(i, 0)),
    });
  }
  return out;
}

export class SrujaDiagramCodeLensProvider implements vscode.CodeLensProvider {
  constructor(private context: vscode.ExtensionContext) {}

  async provideCodeLenses(
    document: vscode.TextDocument,
    token: vscode.CancellationToken
  ): Promise<vscode.CodeLens[]> {
    if (document.languageId !== "sruja") return [];

    const elements = await getElementsFromWasm(this.context, document.getText(), document.uri.fsPath);
    if (token.isCancellationRequested) return [];

    const lenses: vscode.CodeLens[] = [];
    const topRange = new vscode.Range(new vscode.Position(0, 0), new vscode.Position(0, 0));
    lenses.push(
      new vscode.CodeLens(topRange, {
        title: "Open diagram preview",
        command: "sruja.openDiagramPreview",
      }),
      new vscode.CodeLens(topRange, {
        title: "Open focused diagram preview…",
        command: "sruja.openFocusedDiagramPreview",
      })
    );

    const fallback = !elements?.length ? findElementsByRegex(document) : null;

    const elementLike = elements?.length
      ? elements.map((e) => ({
          id: e.id,
          kind: e.kind,
          range: wasmRangeToVscodeRange(e.range),
        }))
      : fallback;

    if (!elementLike?.length) return lenses;
    for (const element of elementLike) {
      const viewLevel = elementKindToViewLevel(element.kind);
      if (viewLevel === 1) continue;

      const lensRange = new vscode.Range(element.range.start, element.range.start);
      const args: FocusDiagramArgs = {
        docUri: document.uri.toString(),
        viewLevel,
        targetId: element.id,
      };
      lenses.push(
        new vscode.CodeLens(lensRange, {
          title: `Open focused diagram (L${viewLevel})`,
          command: "sruja.openFocusedDiagramPreviewAt",
          arguments: [args],
        })
      );
    }

    return lenses;
  }
}

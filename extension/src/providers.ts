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
  SrujaDocumentSymbol,
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
    if (!elements) return undefined;

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
    if (!elements) return undefined;

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

    // When element has doc, add link to open knowledge file in split
    if (element.doc) {
      const docUri = resolveDocUri(element.doc, document);
      if (docUri) {
        const args = [docUri.toString()];
        const cmdUri = vscode.Uri.parse(
          `command:sruja.openComponentKnowledge?${encodeURIComponent(JSON.stringify(args))}`
        );
        markdown.appendMarkdown(`*Documentation:* [Open in split](${cmdUri})\n\n`);
        markdown.isTrusted = true;
      }
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
    if (!symbols) return undefined;

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

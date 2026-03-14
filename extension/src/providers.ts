/**
 * VS Code providers for Sruja DSL
 */

import * as vscode from "vscode";
import { getElementsFromWasm, getDocumentSymbolsFromWasm, SrujaElement, SrujaDocumentSymbol } from "./wasm";

export interface ElementRange {
  start: { line: number; character: number };
  end: { line: number; character: number };
}

export interface DefinitionResult {
  originSelectionRange: ElementRange;
  targetRange: ElementRange;
  targetSelectionRange: ElementRange;
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

    return links.map(link => {
      const start = new vscode.Position(link.targetRange.start.line, link.targetRange.start.character);
      const end = new vscode.Position(link.targetRange.end.line, link.targetRange.end.character);
      const range = new vscode.Range(start, end);
      const originStart = new vscode.Position(link.originSelectionRange.start.line, link.originSelectionRange.start.character);
      const originEnd = new vscode.Position(link.originSelectionRange.end.line, link.originSelectionRange.end.character);

      return {
        originSelectionRange: new vscode.Range(originStart, originEnd),
        targetUri: document.uri,
        targetRange: range,
        targetSelectionRange: range,
      };
    });
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

    // Convert Sruja symbols to VS Code document symbols
    const documentSymbols: vscode.DocumentSymbol[] = symbols.map(symbol => {
      const kind = this.kindToSymbolKind(symbol.kind);
      const start = new vscode.Position(symbol.range.start.line, symbol.range.start.character);
      const end = new vscode.Position(symbol.range.end.line, symbol.range.end.character);
      const range = new vscode.Range(start, end);

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

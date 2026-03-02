import * as vscode from "vscode";
import { getDocumentSymbolsFromWasm, SrujaDocumentSymbol } from "./wasm";
import { findElementAtPosition, toVscodeRange } from "./utils";

export class SrujaDefinitionProvider implements vscode.DefinitionProvider {
  constructor(private context: vscode.ExtensionContext) {}

  async provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
    _token: vscode.CancellationToken
  ): Promise<vscode.LocationLink[] | undefined> {
    const result = await findElementAtPosition(this.context, document, position);
    if (!result) return undefined;

    const { element, wordRange } = result;
    const targetRange = toVscodeRange(element.range);

    return [
      {
        originSelectionRange: wordRange,
        targetUri: document.uri,
        targetRange,
      },
    ];
  }
}

export class SrujaHoverProvider implements vscode.HoverProvider {
  constructor(private context: vscode.ExtensionContext) {}

  async provideHover(
    document: vscode.TextDocument,
    position: vscode.Position,
    _token: vscode.CancellationToken
  ): Promise<vscode.Hover | undefined> {
    const result = await findElementAtPosition(this.context, document, position);
    if (!result) return undefined;

    const { element, wordRange } = result;
    const markdown = new vscode.MarkdownString();
    markdown.appendMarkdown(`**${element.id}**\n\n`);
    markdown.appendMarkdown(`*Kind:* ${element.kind}\n\n`);

    if (element.title) {
      markdown.appendMarkdown(`*Title:* ${element.title}\n\n`);
    }

    const dotIndex = element.id.lastIndexOf(".");
    if (dotIndex !== -1) {
      const parentId = element.id.substring(0, dotIndex);
      markdown.appendMarkdown(`*Parent:* \`${parentId}\`\n\n`);
    }

    return new vscode.Hover(markdown, wordRange);
  }
}

export class SrujaDocumentSymbolProvider implements vscode.DocumentSymbolProvider {
  constructor(private context: vscode.ExtensionContext) {}

  async provideDocumentSymbols(
    document: vscode.TextDocument,
    _token: vscode.CancellationToken
  ): Promise<vscode.DocumentSymbol[] | undefined> {
    if (document.languageId !== "sruja") return undefined;

    const symbols = await getDocumentSymbolsFromWasm(
      this.context,
      document.getText(),
      document.uri.fsPath
    );
    if (!symbols) return undefined;

    return symbols.map((symbol) => this.toDocumentSymbol(symbol));
  }

  private toDocumentSymbol(symbol: SrujaDocumentSymbol): vscode.DocumentSymbol {
    const kind = this.kindToSymbolKind(symbol.kind);
    const range = toVscodeRange(symbol.range);

    return new vscode.DocumentSymbol(symbol.name, symbol.detail, kind, range, range);
  }

  private kindToSymbolKind(kind: string): vscode.SymbolKind {
    const kindMap: Record<string, vscode.SymbolKind> = {
      element: vscode.SymbolKind.Class,
      view: vscode.SymbolKind.Interface,
      scenario: vscode.SymbolKind.Method,
      flow: vscode.SymbolKind.Function,
      requirement: vscode.SymbolKind.Boolean,
      adr: vscode.SymbolKind.Enum,
      policy: vscode.SymbolKind.Event,
    };
    return kindMap[kind] ?? vscode.SymbolKind.Object;
  }
}

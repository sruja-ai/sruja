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
import { extractMissingFieldName, getDiagnosticCodeValue } from "./lintParser";
import { findElementById, findAllElementsById } from "./elementLookup";

function isWordChar(ch: string | undefined): boolean {
  if (!ch) return false;
  return /[A-Za-z0-9_]/.test(ch);
}

function findWholeWordOccurrences(text: string, needle: string): number[] {
  if (!needle) return [];
  const first = needle[0];
  const last = needle[needle.length - 1];

  const out: number[] = [];
  let fromIndex = 0;
  while (fromIndex <= text.length - needle.length) {
    const idx = text.indexOf(needle, fromIndex);
    if (idx === -1) break;

    const startBoundary = idx === 0 || isWordChar(text[idx - 1]) !== isWordChar(first);
    const endIdx = idx + needle.length;
    const endBoundary = endIdx === text.length || isWordChar(text[endIdx]) !== isWordChar(last);

    if (startBoundary && endBoundary) out.push(idx);
    fromIndex = idx + needle.length;
  }

  return out;
}

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
  const root = folder.uri.fsPath;
  const absolute = path.resolve(root, docPath.trim());
  const rel = path.relative(root, absolute);
  if (!rel || rel.startsWith("..") || path.isAbsolute(rel)) return undefined;
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
  const element = findElementById(elements, word);
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

    const elements = await getElementsFromWasm(this.context, document.getText(), document.uri.fsPath, document.uri.toString(), document.version);
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

    const element = findElementById(elements, word);
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

    const elements = await getElementsFromWasm(this.context, document.getText(), document.uri.fsPath, document.uri.toString(), document.version);
    if (token.isCancellationRequested || !elements) return undefined;

    const wordRange = document.getWordRangeAtPosition(position);
    if (!wordRange) return undefined;

    const word = document.getText(wordRange).trim();
    if (!word) return undefined;

    // Find element matching the word
    const element = findElementById(elements, word);
    if (!element) return undefined;

    // Build hover markdown
    const markdown = new vscode.MarkdownString();
    markdown.appendMarkdown(`**${element.id}**\n\n`);
    markdown.appendMarkdown(`*Kind:* ${element.kind}\n\n`);

    if (element.title) {
      markdown.appendMarkdown(`*Title:* ${element.title}\n\n`);
    }

    // Show enriched requirement fields
    if (element.kind === "requirement") {
      const symbols = await getDocumentSymbolsFromWasm(this.context, document.getText(), document.uri.fsPath, document.uri.toString(), document.version);
      const sym = symbols?.find(s => s.kind === "requirement" && s.name === element.id);
      if (sym && sym.kind === "requirement") {
        if (sym.priority) {
          markdown.appendMarkdown(`*Priority:* ${sym.priority}\n\n`);
        }
        if (sym.status) {
          markdown.appendMarkdown(`*Status:* ${sym.status}\n\n`);
        }
        if (sym.source) {
          markdown.appendMarkdown(`*Source:* ${sym.source}\n\n`);
        }
        if (sym.scenarios && sym.scenarios.length > 0) {
          markdown.appendMarkdown(`*Scenarios:* ${sym.scenarios.join(", ")}\n\n`);
        }
        if (sym.adrs && sym.adrs.length > 0) {
          markdown.appendMarkdown(`*ADRs:* ${sym.adrs.join(", ")}\n\n`);
        }
        if (sym.affects && sym.affects.length > 0) {
          markdown.appendMarkdown(`*Affects:* ${sym.affects.join(", ")}\n\n`);
        }
      }
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

    {
      const args = [{ docUri: document.uri.toString(), elementId: element.id }];
      const cmdUri = vscode.Uri.parse(
        `command:sruja.openDocsThreadAt?${encodeURIComponent(JSON.stringify(args))}`
      );
      markdown.appendMarkdown(`*Docs & refs:* [Open thread](${cmdUri})\n\n`);
      markdown.isTrusted = true;
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

    const symbols = await getDocumentSymbolsFromWasm(this.context, document.getText(), document.uri.fsPath, document.uri.toString(), document.version);
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
  if (k.includes("container")) return 3;
  if (k.includes("database")) return 3;
  if (k.includes("system")) return 2;
  return 1;
}

function kindToCodeLensViewLevel(kind: string): 2 | 3 | null {
  const k = kind.toLowerCase();
  if (k === "system") return 2;
  if (k === "container" || k === "database") return 3;
  return null;
}

function findFocusableElementsByBlockParsing(document: vscode.TextDocument): Array<{ id: string; viewLevel: 2 | 3; range: vscode.Range }> {
  const declRe = /^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(system|container|component|database)\b/;
  const stack: Array<{ id: string; kind: string }> = [];
  const out: Array<{ id: string; viewLevel: 2 | 3; range: vscode.Range }> = [];

  for (let i = 0; i < document.lineCount; i++) {
    const line = document.lineAt(i).text;
    const m = declRe.exec(line);
    if (m) {
      const localId = m[1];
      const kind = m[2];
      const parent = stack.length > 0 ? stack[stack.length - 1] : undefined;
      const fullId = parent ? `${parent.id}.${localId}` : localId;

      const viewLevel = kindToCodeLensViewLevel(kind);
      if (viewLevel) {
        out.push({
          id: fullId,
          viewLevel,
          range: new vscode.Range(new vscode.Position(i, 0), new vscode.Position(i, 0)),
        });
      }

      if (line.includes("{")) {
        stack.push({ id: fullId, kind });
      }
    }

    const closeCount = (line.match(/\}/g) ?? []).length;
    for (let c = 0; c < closeCount; c++) {
      stack.pop();
    }
  }

  return out;
}

function findScenarioAndFlowDefsByLineParsing(
  document: vscode.TextDocument
): Array<{ id: string; kind: "scenario" | "flow"; range: vscode.Range }> {
  const assignRe = /^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(scenario|story|flow)\b/;
  const keywordRe = /^\s*(scenario|story|flow)\s+([A-Za-z_][A-Za-z0-9_]*)\b/;
  const out: Array<{ id: string; kind: "scenario" | "flow"; range: vscode.Range }> = [];

  for (let i = 0; i < document.lineCount; i++) {
    const line = document.lineAt(i).text;
    const m1 = assignRe.exec(line);
    if (m1) {
      const id = m1[1];
      const k = m1[2];
      out.push({
        id,
        kind: k === "flow" ? "flow" : "scenario",
        range: new vscode.Range(new vscode.Position(i, 0), new vscode.Position(i, 0)),
      });
      continue;
    }
    const m2 = keywordRe.exec(line);
    if (m2) {
      const k = m2[1];
      const id = m2[2];
      out.push({
        id,
        kind: k === "flow" ? "flow" : "scenario",
        range: new vscode.Range(new vscode.Position(i, 0), new vscode.Position(i, 0)),
      });
    }
  }

  return out;
}

export class SrujaDiagramCodeLensProvider implements vscode.CodeLensProvider {
  constructor(_context: vscode.ExtensionContext) {}

  async provideCodeLenses(
    document: vscode.TextDocument,
    token: vscode.CancellationToken
  ): Promise<vscode.CodeLens[]> {
    if (document.languageId !== "sruja") return [];

    const lenses: vscode.CodeLens[] = [];
    const topRange = new vscode.Range(new vscode.Position(0, 0), new vscode.Position(0, 0));
    lenses.push(
      new vscode.CodeLens(topRange, {
        title: "Open diagram preview (L1)",
        command: "sruja.openDiagramPreview",
      }),
      new vscode.CodeLens(topRange, {
        title: "Open Architecture Explorer",
        command: "sruja.openArchitectureExplorer",
      })
    );

    const focusable = findFocusableElementsByBlockParsing(document);
    if (token.isCancellationRequested) return lenses;

    for (const element of focusable) {
      const lensRange = new vscode.Range(element.range.start, element.range.start);
      const args: FocusDiagramArgs = {
        docUri: document.uri.toString(),
        viewLevel: element.viewLevel,
        targetId: element.id,
      };
      lenses.push(
        new vscode.CodeLens(lensRange, {
          title: `Open focused diagram (L${element.viewLevel})`,
          command: "sruja.openFocusedDiagramPreviewAt",
          arguments: [args],
        })
      );
    }

    const seqDefs = findScenarioAndFlowDefsByLineParsing(document);
    if (token.isCancellationRequested) return lenses;
    for (const def of seqDefs) {
      const lensRange = new vscode.Range(def.range.start, def.range.start);
      lenses.push(
        new vscode.CodeLens(lensRange, {
          title: "Open sequence diagram",
          command: "sruja.openSequenceDiagramPreviewAt",
          arguments: [
            {
              docUri: document.uri.toString(),
              kind: def.kind,
              id: def.id,
            },
          ],
        })
      );
    }

    return lenses;
  }
}

function findLineIndexToInsertAfterOpenBrace(
  document: vscode.TextDocument,
  startLine: number
): number | null {
  const maxLookahead = Math.min(document.lineCount - 1, startLine + 10);
  for (let i = startLine; i <= maxLookahead; i++) {
    const line = document.lineAt(i).text;
    if (line.includes("{")) return i;
    if (line.includes("}")) return null;
  }
  return null;
}

function blockContainsField(
  document: vscode.TextDocument,
  openBraceLine: number,
  fieldName: string
): boolean {
  let depth = 0;
  for (let i = openBraceLine; i < document.lineCount; i++) {
    const line = document.lineAt(i).text;
    depth += (line.match(/\{/g) ?? []).length;
    depth -= (line.match(/\}/g) ?? []).length;
    if (i > openBraceLine) {
      if (new RegExp(`^\\s*${fieldName}\\b`).test(line)) return true;
    }
    if (depth <= 0 && i > openBraceLine) break;
  }
  return false;
}

function buildAddMissingFieldQuickFix(
  document: vscode.TextDocument,
  diagnostic: vscode.Diagnostic
): vscode.CodeAction | null {
  const code = getDiagnosticCodeValue(diagnostic);
  if (code !== "E302") return null;

  const fieldName = extractMissingFieldName(diagnostic.message);
  if (!fieldName) return null;

  const openBraceLine = findLineIndexToInsertAfterOpenBrace(
    document,
    diagnostic.range.start.line
  );
  if (openBraceLine == null) return null;

  if (blockContainsField(document, openBraceLine, fieldName)) return null;

  const openLineText = document.lineAt(openBraceLine).text;
  const indent = (openLineText.match(/^\s*/)?.[0] ?? "") + "  ";
  const insertLine = openBraceLine + 1;
  const pos = new vscode.Position(insertLine, 0);
  const newText = `${indent}${fieldName} "..."` + "\n";

  const action = new vscode.CodeAction(
    `Add ${fieldName} "..."`,
    vscode.CodeActionKind.QuickFix
  );
  action.diagnostics = [diagnostic];
  action.edit = new vscode.WorkspaceEdit();
  action.edit.insert(document.uri, pos, newText);
  return action;
}

function buildSpellingCorrectionQuickFix(
  document: vscode.TextDocument,
  diagnostic: vscode.Diagnostic,
  elements: SrujaElement[]
): vscode.CodeAction[] {
  const code = getDiagnosticCodeValue(diagnostic);
  if (code !== "E202") return [];

  const wordRange = diagnostic.range;
  const word = document.getText(wordRange).trim();
  if (!word) return [];

  // Suggest elements that are similar to the misspelled word
  const suggestions = elements
    .filter(e => {
      const id = e.id;
      // Simple similarity check: contains or very close length
      return id.toLowerCase().includes(word.toLowerCase()) || 
             word.toLowerCase().includes(id.toLowerCase());
    })
    .slice(0, 3);

  return suggestions.map(s => {
    const action = new vscode.CodeAction(
      `Replace with "${s.id}"`,
      vscode.CodeActionKind.QuickFix
    );
    action.diagnostics = [diagnostic];
    action.edit = new vscode.WorkspaceEdit();
    action.edit.replace(document.uri, wordRange, s.id);
    return action;
  });
}

export class SrujaCodeActionProvider implements vscode.CodeActionProvider {
  constructor(private context: vscode.ExtensionContext) {}

  async provideCodeActions(
    document: vscode.TextDocument,
    _range: vscode.Range,
    context: vscode.CodeActionContext,
    token: vscode.CancellationToken
  ): Promise<vscode.CodeAction[]> {
    if (document.languageId !== "sruja") return [];
    
    const elements = await getElementsFromWasm(this.context, document.getText(), document.uri.fsPath, document.uri.toString(), document.version);
    if (token.isCancellationRequested || !elements) return [];

    const actions: vscode.CodeAction[] = [];
    for (const d of context.diagnostics) {
      const missingFieldFix = buildAddMissingFieldQuickFix(document, d);
      if (missingFieldFix) actions.push(missingFieldFix);

      const spellingFixes = buildSpellingCorrectionQuickFix(document, d, elements);
      actions.push(...spellingFixes);
    }
    return actions;
  }
}

/**
 * Completion Item Provider
 * Provides completions for keywords and existing element IDs
 */
export class SrujaCompletionItemProvider implements vscode.CompletionItemProvider {
  constructor(private context: vscode.ExtensionContext) {}

  async provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): Promise<vscode.CompletionItem[] | undefined> {
    if (document.languageId !== "sruja") return undefined;

    const items: vscode.CompletionItem[] = [];

    // Keywords
    const keywords = [
      "system", "container", "component", "database", "person", "software",
      "story", "flow", "scenario", "architecture", "description",
      "technology", "doc", "external"
    ];

    for (const kw of keywords) {
      const item = new vscode.CompletionItem(kw, vscode.CompletionItemKind.Keyword);
      items.push(item);
    }

    // Element IDs for relationships
    const linePrefix = document.lineAt(position).text.substring(0, position.character);
    if (linePrefix.includes("->") || linePrefix.includes("=")) {
      const elements = await getElementsFromWasm(this.context, document.getText(), document.uri.fsPath, document.uri.toString(), document.version);
      if (token.isCancellationRequested || !elements) return items;

      for (const el of elements) {
        const item = new vscode.CompletionItem(el.id, vscode.CompletionItemKind.Variable);
        item.detail = el.kind;
        if (el.title) item.documentation = el.title;
        items.push(item);
      }
    }

    return items;
  }
}

/**
 * Rename Provider
 * Allows renaming an element and updating all references
 */
export class SrujaRenameProvider implements vscode.RenameProvider {
  constructor(private context: vscode.ExtensionContext) {}

  async provideRenameEdits(
    document: vscode.TextDocument,
    position: vscode.Position,
    newName: string,
    token: vscode.CancellationToken
  ): Promise<vscode.WorkspaceEdit | undefined> {
    const elements = await getElementsFromWasm(this.context, document.getText(), document.uri.fsPath, document.uri.toString(), document.version);
    if (token.isCancellationRequested || !elements) return undefined;

    const wordRange = document.getWordRangeAtPosition(position);
    if (!wordRange) return undefined;

    const oldName = document.getText(wordRange).trim();
    const element = findElementById(elements, oldName);
    if (!element) return undefined;

    const edit = new vscode.WorkspaceEdit();
    const files = await vscode.workspace.findFiles("**/*.sruja", "**/{.git,node_modules,target,dist,out}/**");

    for (const fileUri of files) {
      if (token.isCancellationRequested) return undefined;
      const doc = await vscode.workspace.openTextDocument(fileUri);
      const text = doc.getText();
      const matches = findWholeWordOccurrences(text, element.id);
      for (const idx of matches) {
        const startPos = doc.positionAt(idx);
        const endPos = doc.positionAt(idx + element.id.length);
        edit.replace(fileUri, new vscode.Range(startPos, endPos), newName);
      }
    }

    return edit;
  }
}

/**
 * Reference Provider
 * Finds all usages of an element ID
 */
export class SrujaReferenceProvider implements vscode.ReferenceProvider {
  constructor(private context: vscode.ExtensionContext) {}

  async provideReferences(
    document: vscode.TextDocument,
    position: vscode.Position,
    _context: vscode.ReferenceContext,
    token: vscode.CancellationToken
  ): Promise<vscode.Location[] | undefined> {
    const elements = await getElementsFromWasm(this.context, document.getText(), document.uri.fsPath, document.uri.toString(), document.version);
    if (token.isCancellationRequested || !elements) return undefined;

    const wordRange = document.getWordRangeAtPosition(position);
    if (!wordRange) return undefined;

    const word = document.getText(wordRange).trim();
    const element = findElementById(elements, word);
    if (!element) return undefined;

    const locations: vscode.Location[] = [];
    const files = await vscode.workspace.findFiles("**/*.sruja", "**/{.git,node_modules,target,dist,out}/**");

    for (const fileUri of files) {
      if (token.isCancellationRequested) return undefined;
      const doc = await vscode.workspace.openTextDocument(fileUri);
      const text = doc.getText();
      const matches = findWholeWordOccurrences(text, element.id);
      for (const idx of matches) {
        const startPos = doc.positionAt(idx);
        const endPos = doc.positionAt(idx + element.id.length);
        locations.push(new vscode.Location(fileUri, new vscode.Range(startPos, endPos)));
      }
    }

    return locations;
  }
}

/**
 * Document Formatting Edit Provider
 * Standardizes indentation and spacing
 */
export class SrujaDocumentFormattingEditProvider implements vscode.DocumentFormattingEditProvider {
  provideDocumentFormattingEdits(
    document: vscode.TextDocument,
    _options: vscode.FormattingOptions,
    _token: vscode.CancellationToken
  ): vscode.TextEdit[] {
    const edits: vscode.TextEdit[] = [];
    let indentLevel = 0;
    const tabSize = 2;

    for (let i = 0; i < document.lineCount; i++) {
      const line = document.lineAt(i);
      const text = line.text.trim();
      
      if (text.length === 0) continue;

      // Adjust indent level BEFORE formatting for closing brace
      if (text.startsWith("}")) {
        indentLevel = Math.max(0, indentLevel - 1);
      }

      const expectedIndent = " ".repeat(indentLevel * tabSize);
      
      // Basic rule: replace line with formatted version (indent + trimmed text)
      // and ensure space around ->
      let formattedText = text;
      if (text.includes("->")) {
        formattedText = text.replace(/\s*->\s*/, " -> ");
      }
      if (text.includes(" = ")) {
         // already has spaces
      } else if (text.includes("=")) {
         formattedText = formattedText.replace(/\s*=\s*/, " = ");
      }

      const newText = expectedIndent + formattedText;

      if (newText !== line.text) {
        edits.push(vscode.TextEdit.replace(line.range, newText));
      }

      // Adjust indent level AFTER formatting for opening brace
      if (text.endsWith("{") || text.includes("{") && !text.includes("}")) {
        indentLevel++;
      }
    }

    return edits;
  }
}

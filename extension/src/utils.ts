import * as vscode from "vscode";
import { getElementsFromWasm, SrujaElement } from "./wasm";

export async function findElementAtPosition(
  context: vscode.ExtensionContext,
  document: vscode.TextDocument,
  position: vscode.Position
): Promise<{ element: SrujaElement; wordRange: vscode.Range } | undefined> {
  if (document.languageId !== "sruja") return undefined;

  const elements = await getElementsFromWasm(context, document.getText(), document.uri.fsPath);
  if (!elements) return undefined;

  const wordRange = document.getWordRangeAtPosition(position);
  if (!wordRange) return undefined;

  const word = document.getText(wordRange).trim();
  if (!word) return undefined;

  const element = elements.find((e) => e.id === word || e.id.endsWith(`.${word}`));
  if (!element) return undefined;

  return { element, wordRange };
}

export function toVscodeRange(
  range: SrujaElement["range"]
): vscode.Range {
  const start = new vscode.Position(range.start.line, range.start.character);
  const end = new vscode.Position(range.end.line, range.end.character);
  return new vscode.Range(start, end);
}

export async function pickSingleOrMany<T>(
  items: T[],
  toPickItem: (item: T) => { label: string; description?: string; item: T },
  placeHolder: string
): Promise<T | undefined> {
  if (items.length === 0) return undefined;
  if (items.length === 1) return items[0];

  const picks = items.map(toPickItem);
  const selected = await vscode.window.showQuickPick(picks, {
    placeHolder,
    matchOnDescription: true,
  });
  return selected?.item;
}

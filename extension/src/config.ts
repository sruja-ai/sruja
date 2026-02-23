import * as vscode from "vscode";

/** Use WASM for lint/export unless user explicitly set sruja.lsp.path. WASM is always shipped with the extension. */
export function useWasm(context: vscode.ExtensionContext): boolean {
  const config = vscode.workspace.getConfiguration("sruja").get<string>("lsp.path");
  return !config?.trim();
}

export function getSrujaPath(context: vscode.ExtensionContext): string {
  const config = vscode.workspace.getConfiguration("sruja").get<string>("lsp.path");
  if (config?.trim()) return config.trim();
  return "sruja";
}

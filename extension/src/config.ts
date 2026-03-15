/**
 * Sruja extension config: pure helpers for LSP path and WASM vs CLI choice.
 * Testable without vscode; extension passes config values from getConfiguration().
 */

/** Default CLI command when no path is configured. */
export const DEFAULT_SRUJA_CMD = "sruja";

/**
 * Resolve the Sruja CLI/LSP path from config. Returns default if empty or unset.
 */
export function getSrujaLspPath(lspPath: string | undefined): string {
  const trimmed = lspPath?.trim();
  return trimmed ? trimmed : DEFAULT_SRUJA_CMD;
}

/**
 * Whether to use bundled WASM for lint/export. Use WASM when LSP path is not set.
 */
export function useWasmFromLspPath(lspPath: string | undefined): boolean {
  return !lspPath?.trim();
}

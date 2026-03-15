/**
 * Parse "sruja lint" output (stderr or --format json) into VS Code diagnostics.
 * Pure logic for testability; depends only on vscode types for Diagnostic/Range.
 */

import * as vscode from "vscode";

/** Lint output when using --format json (see docs/LINT_JSON_OUTPUT.md). */
export interface LintJsonOutput {
  ok: boolean;
  error_count: number;
  warning_count: number;
  diagnostics: Array<{
    code: string;
    severity: string;
    message: string;
    location?: { file: string; line: number; column: number };
  }>;
}

const MSG_RE = /^\[([^\]]+)\]\s+(Error|Warning|Info):\s+(.+)$/;
const LOC_RE = /^\s+-->\s+(.+):(\d+):(\d+)$/;

/**
 * Parse "sruja lint" stderr into diagnostics. Format:
 * [CODE] Error: message
 *   --> file:line:column
 */
export function parseLintStderr(stderr: string, _docUri: string): vscode.Diagnostic[] {
  const diagnostics: vscode.Diagnostic[] = [];
  const lines = stderr.split(/\r?\n/);

  for (let i = 0; i < lines.length; i++) {
    const locMatch = lines[i].match(LOC_RE);
    if (!locMatch) continue;
    const [, , lineStr, colStr] = locMatch;
    const line = Math.max(0, parseInt(lineStr, 10) - 1);
    const character = Math.max(0, parseInt(colStr, 10) - 1);

    let code: string | undefined;
    let severity: vscode.DiagnosticSeverity = vscode.DiagnosticSeverity.Error;
    let message = "Validation error";

    if (i > 0) {
      const msgMatch = lines[i - 1].match(MSG_RE);
      if (msgMatch) {
        code = msgMatch[1];
        const sev = msgMatch[2];
        message = msgMatch[3].trim();
        if (sev === "Warning") severity = vscode.DiagnosticSeverity.Warning;
        else if (sev === "Info") severity = vscode.DiagnosticSeverity.Information;
      }
    }

    const range = new vscode.Range(line, character, line, character);
    const diag = new vscode.Diagnostic(range, message, severity);
    if (code) diag.code = code;
    diag.source = "sruja";
    diagnostics.push(diag);
  }

  return diagnostics;
}

/**
 * Parse lint JSON stdout into diagnostics. Returns null on invalid or missing diagnostics.
 */
export function parseLintJson(stdout: string, _docUri: string): vscode.Diagnostic[] | null {
  try {
    const out = JSON.parse(stdout) as LintJsonOutput;
    if (!out.diagnostics || !Array.isArray(out.diagnostics)) return null;
    const diagnostics: vscode.Diagnostic[] = [];
    for (const d of out.diagnostics) {
      const line = d.location ? Math.max(0, (d.location.line || 1) - 1) : 0;
      const character = d.location ? Math.max(0, (d.location.column || 1) - 1) : 0;
      const severity =
        d.severity === "warning"
          ? vscode.DiagnosticSeverity.Warning
          : d.severity === "info"
            ? vscode.DiagnosticSeverity.Information
            : vscode.DiagnosticSeverity.Error;
      const range = new vscode.Range(line, character, line, character);
      const diag = new vscode.Diagnostic(range, d.message, severity);
      if (d.code) diag.code = d.code;
      diag.source = "sruja";
      diagnostics.push(diag);
    }
    return diagnostics;
  } catch {
    return null;
  }
}

/**
 * Combine lint JSON stdout and stderr: prefer JSON diagnostics, fallback to stderr.
 */
export function parseLintOutput(
  stdout: string,
  stderr: string,
  docUri: string
): vscode.Diagnostic[] {
  const fromJson = parseLintJson(stdout, docUri);
  if (fromJson !== null) return fromJson;
  return parseLintStderr(stderr, docUri);
}

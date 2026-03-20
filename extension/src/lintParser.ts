/**
 * Parse "sruja lint" output (stderr or --format json) into VS Code diagnostics.
 * Pure logic for testability; depends only on vscode types for Diagnostic/Range.
 */

import * as vscode from "vscode";
import * as path from "path";

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
const LOC_RE = /^\s*-->\s+(.+):(\d+):(\d+)\s*$/;

export function getDiagnosticCodeValue(
  diag: vscode.Diagnostic
): string | number | undefined {
  const code = diag.code;
  if (typeof code === "string" || typeof code === "number") return code;
  if (
    code &&
    typeof code === "object" &&
    "value" in code &&
    (typeof (code as { value: unknown }).value === "string" ||
      typeof (code as { value: unknown }).value === "number")
  ) {
    return (code as { value: string | number }).value;
  }
  return undefined;
}

function getBasename(p: string): string {
  const normalized = p.replace(/\\/g, "/");
  const parts = normalized.split("/");
  return parts[parts.length - 1] ?? normalized;
}

function docUriToFsPath(docUri: string): string | undefined {
  try {
    const parsed = vscode.Uri.parse(docUri);
    return parsed.scheme === "file" ? parsed.fsPath : undefined;
  } catch {
    return undefined;
  }
}

function lintFileMatchesDoc(lintFile: string, docUri: string): boolean {
  const docFsPath = docUriToFsPath(docUri);
  if (!docFsPath) return true;

  const lintFileTrimmed = lintFile.trim();
  if (lintFileTrimmed === docFsPath) return true;

  const resolvedLint = (() => {
    try {
      return path.resolve(lintFileTrimmed);
    } catch {
      return lintFileTrimmed;
    }
  })();
  if (resolvedLint === docFsPath) return true;

  const docBase = getBasename(docFsPath);
  const lintBase = getBasename(lintFileTrimmed);
  return docBase === lintBase;
}

function normalizeFieldCandidate(candidate: string): string | null {
  let c = candidate.trim();
  c = c.replace(/^[`"'“”‘’]+/, "").replace(/[`"'“”‘’]+$/, "");
  c = c.replace(/[),.;:]+$/, "").trim();
  if (c.includes(",") || /\band\b/i.test(c)) return null;
  const token = (c.match(/^[A-Za-z_][A-Za-z0-9_]*/) ?? [])[0];
  if (!token) return null;
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(token)) return null;
  return token;
}

export function extractMissingFieldName(message: string): string | null {
  const text = String(message);

  const quoted = [
    /Missing required fields?\s+`([^`]+)`/i,
    /Missing required fields?\s+"([^"]+)"/i,
    /Missing required fields?\s+'([^']+)'/i,
    /Missing required fields?\s+“([^”]+)”/i,
    /Missing required fields?\s+‘([^’]+)’/i,
    /Missing fields?\s+`([^`]+)`/i,
    /Missing fields?\s+"([^"]+)"/i,
    /Missing fields?\s+'([^']+)'/i,
    /Missing fields?\s+“([^”]+)”/i,
    /Missing fields?\s+‘([^’]+)’/i,
  ];
  for (const re of quoted) {
    const m = re.exec(text);
    if (!m) continue;
    const normalized = normalizeFieldCandidate(m[1]);
    if (normalized) return normalized;
    return null;
  }

  const unquoted = [
    /Missing required fields?\b\s*[:=]?\s*([A-Za-z_][A-Za-z0-9_]*)/i,
    /Missing fields?\b\s*[:=]?\s*([A-Za-z_][A-Za-z0-9_]*)/i,
    /Required fields?\b\s*[:=]?\s*([A-Za-z_][A-Za-z0-9_]*)\s+(?:is|are)\s+missing/i,
    /\bmissing_required_field\b\s*[:=]\s*([A-Za-z_][A-Za-z0-9_]*)/i,
  ];
  for (const re of unquoted) {
    const m = re.exec(text);
    if (!m) continue;
    const normalized = normalizeFieldCandidate(m[1]);
    if (normalized) return normalized;
    return null;
  }

  return null;
}

/**
 * Parse "sruja lint" stderr into diagnostics. Format:
 * [CODE] Error: message
 *   --> file:line:column
 */
export function parseLintStderr(stderr: string, docUri: string): vscode.Diagnostic[] {
  const diagnostics: vscode.Diagnostic[] = [];
  const lines = stderr.split(/\r?\n/);

  let pending: { code?: string; severity: vscode.DiagnosticSeverity; messageLines: string[] } | null = null;

  for (let i = 0; i < lines.length; i++) {
    const lineText = lines[i];

    const msgMatch = lineText.match(MSG_RE);
    if (msgMatch) {
      const code = msgMatch[1];
      const sev = msgMatch[2];
      const message = msgMatch[3].trim();
      const severity =
        sev === "Warning"
          ? vscode.DiagnosticSeverity.Warning
          : sev === "Info"
            ? vscode.DiagnosticSeverity.Information
            : vscode.DiagnosticSeverity.Error;
      pending = { code, severity, messageLines: message ? [message] : [] };
      continue;
    }

    const locMatch = lineText.match(LOC_RE);
    if (!locMatch) {
      if (pending && /^\s+/.test(lineText) && !/^\s*\|/.test(lineText)) {
        const extra = lineText.trim();
        if (extra.length > 0) pending.messageLines.push(extra);
      }
      continue;
    }

    const [, fileStr, lineStr, colStr] = locMatch;
    if (!lintFileMatchesDoc(fileStr, docUri)) {
      pending = null;
      continue;
    }

    const line = Math.max(0, parseInt(lineStr, 10) - 1);
    const character = Math.max(0, parseInt(colStr, 10) - 1);

    const code = pending?.code;
    const severity = pending?.severity ?? vscode.DiagnosticSeverity.Error;
    const message =
      pending?.messageLines.length && pending.messageLines.join(" ").trim().length > 0
        ? pending.messageLines.join(" ").replace(/\s+/g, " ").trim()
        : "Validation error";

    const range = new vscode.Range(line, character, line, character);
    const diag = new vscode.Diagnostic(range, message, severity);
    if (code) diag.code = code;
    diag.source = "sruja";
    diagnostics.push(diag);
    pending = null;
  }

  return diagnostics;
}

/**
 * Parse lint JSON stdout into diagnostics. Returns null on invalid or missing diagnostics.
 */
export function parseLintJson(stdout: string, docUri: string): vscode.Diagnostic[] | null {
  try {
    const out = JSON.parse(stdout) as LintJsonOutput;
    if (!out || !out.diagnostics || !Array.isArray(out.diagnostics)) return null;
    const diagnostics: vscode.Diagnostic[] = [];
    for (const d of out.diagnostics) {
      const locFile = d.location?.file;
      if (typeof locFile === "string" && locFile.trim().length > 0) {
        if (!lintFileMatchesDoc(locFile, docUri)) continue;
      }

      const line = d.location ? Math.max(0, (d.location.line || 1) - 1) : 0;
      const character = d.location ? Math.max(0, (d.location.column || 1) - 1) : 0;
      const sev = String(d.severity ?? "").toLowerCase();
      const severity =
        sev === "warning" || sev === "warn"
          ? vscode.DiagnosticSeverity.Warning
          : sev === "info" || sev === "information"
            ? vscode.DiagnosticSeverity.Information
            : sev === "hint"
              ? vscode.DiagnosticSeverity.Hint
              : vscode.DiagnosticSeverity.Error;
      const range = new vscode.Range(line, character, line, character);
      const diag = new vscode.Diagnostic(range, String(d.message ?? ""), severity);
      if (typeof d.code === "string" || typeof d.code === "number") diag.code = d.code as unknown as vscode.Diagnostic["code"];
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

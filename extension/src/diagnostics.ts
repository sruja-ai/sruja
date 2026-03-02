import { execFile } from "child_process";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as vscode from "vscode";

import { getDiagnosticsFromWasm } from "./wasm";
import { getSrujaPath, useWasm } from "./config";

const LINT_TIMEOUT_MS = 15000;
const LINT_MAX_BUFFER_BYTES = 2 * 1024 * 1024;
const LINT_ERROR_REGEX = /^\[([^\]]+)\]\s+(Error|Warning|Info):\s+(.+)$/;
const LINT_LOCATION_REGEX = /^\s+-->\s+(.+):(\d+):(\d+)$/;

export function parseLintStderr(stderr: string): vscode.Diagnostic[] {
  const diagnostics: vscode.Diagnostic[] = [];
  const lines = stderr.split(/\r?\n/);

  for (let i = 0; i < lines.length; i++) {
    const locMatch = lines[i].match(LINT_LOCATION_REGEX);
    if (!locMatch) continue;

    const [, _filePart, lineStr, colStr] = locMatch;
    const line = Math.max(0, parseInt(lineStr, 10) - 1);
    const character = Math.max(0, parseInt(colStr, 10) - 1);

    let code: string | undefined;
    let severity: vscode.DiagnosticSeverity = vscode.DiagnosticSeverity.Error;
    let message = "Validation error";

    if (i > 0) {
      const msgMatch = lines[i - 1].match(LINT_ERROR_REGEX);
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

export async function runLint(srujaPath: string, filePath: string): Promise<{ stderr: string }> {
  return new Promise((resolve) => {
    execFile(
      srujaPath,
      ["lint", filePath],
      { encoding: "utf8", timeout: LINT_TIMEOUT_MS, maxBuffer: LINT_MAX_BUFFER_BYTES },
      (err: Error | null, _stdout: string, stderr: string) => {
        const out = typeof stderr === "string" ? stderr : err?.message ?? "";
        resolve({ stderr: out });
      }
    );
  });
}

async function withTempFile(content: string, baseName: string, fn: (tmpPath: string) => Promise<void>): Promise<void> {
  const tmp = path.join(os.tmpdir(), `sruja-lint-${baseName}`);
  await fs.promises.writeFile(tmp, content, "utf8");
  try {
    await fn(tmp);
  } finally {
    await fs.promises.unlink(tmp).catch(() => {});
  }
}

export async function updateDiagnostics(
  context: vscode.ExtensionContext,
  doc: vscode.TextDocument,
  diagnosticCollection: vscode.DiagnosticCollection
): Promise<void> {
  if (doc.languageId !== "sruja" || doc.uri.scheme !== "file") return;

  const uri = doc.uri;
  const filename = doc.uri.fsPath;

  if (useWasm(context)) {
    try {
      const diags = await getDiagnosticsFromWasm(context, doc.getText(), filename);
      diagnosticCollection.set(uri, diags);
    } catch {
      diagnosticCollection.set(uri, []);
    }
    return;
  }

  const srujaPath = getSrujaPath(context);

  if (doc.isDirty) {
    await withTempFile(doc.getText(), path.basename(filename), async (tmp) => {
      const { stderr } = await runLint(srujaPath, tmp);
      diagnosticCollection.set(uri, parseLintStderr(stderr));
    });
    return;
  }

  const { stderr } = await runLint(srujaPath, filename);
  diagnosticCollection.set(uri, parseLintStderr(stderr));
}

import { execFile } from "child_process";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as vscode from "vscode";

import { getDiagnosticsFromWasm } from "./wasm";
import { getSrujaPath, useWasm } from "./config";

/** Parse "sruja lint" stderr into VS Code diagnostics. Format:
 * [CODE] Error: message
 *   --> file:line:column
 */
export function parseLintStderr(stderr: string, _docUri: string): vscode.Diagnostic[] {
  const diagnostics: vscode.Diagnostic[] = [];
  const lines = stderr.split(/\r?\n/);
  const msgRe = /^\[([^\]]+)\]\s+(Error|Warning|Info):\s+(.+)$/;
  const locRe = /^\s+-->\s+(.+):(\d+):(\d+)$/;

  for (let i = 0; i < lines.length; i++) {
    const locMatch = lines[i].match(locRe);
    if (!locMatch) continue;
    const [, _filePart, lineStr, colStr] = locMatch;
    const line = Math.max(0, parseInt(lineStr, 10) - 1);
    const character = Math.max(0, parseInt(colStr, 10) - 1);

    let code: string | undefined;
    let severity: vscode.DiagnosticSeverity = vscode.DiagnosticSeverity.Error;
    let message = "Validation error";

    if (i > 0) {
      const msgMatch = lines[i - 1].match(msgRe);
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

export async function runLint(
  srujaPath: string,
  filePath: string
): Promise<{ stderr: string }> {
  return new Promise((resolve) => {
    execFile(
      srujaPath,
      ["lint", filePath],
      { encoding: "utf8", timeout: 15000, maxBuffer: 2 * 1024 * 1024 },
      (err: Error | null, _stdout: string, stderr: string) => {
        const out = typeof stderr === "string" ? stderr : err?.message ?? "";
        resolve({ stderr: out });
      }
    );
  });
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
    const tmp = path.join(os.tmpdir(), `sruja-lint-${path.basename(filename)}`);
    await fs.promises.writeFile(tmp, doc.getText(), "utf8");
    try {
      const { stderr } = await runLint(srujaPath, tmp);
      const diags = parseLintStderr(stderr, uri.toString());
      diagnosticCollection.set(uri, diags);
    } finally {
      await fs.promises.unlink(tmp).catch(() => {});
    }
    return;
  }

  const { stderr } = await runLint(srujaPath, filename);
  const diags = parseLintStderr(stderr, uri.toString());
  diagnosticCollection.set(uri, diags);
}

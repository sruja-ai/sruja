import { promisify } from "util";
import { execFile } from "child_process";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as vscode from "vscode";

import { getSrujaPath, useWasm } from "../config";
import { exportMarkdownFromWasm, getMermaidFromWasm } from "../wasm";

const execFileAsync = promisify(execFile);

export function getDiagramPreviewHtml(mermaidCodeEscaped: string): string {
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline' https://cdn.jsdelivr.net; style-src 'unsafe-inline' https://cdn.jsdelivr.net;">
  <title>Sruja Diagram Preview</title>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
</head>
<body>
  <div id="diagram" class="mermaid"></div>
  <script>
    (function() {
      var code = \`${mermaidCodeEscaped}\`;
      var el = document.getElementById('diagram');
      el.textContent = code;
      mermaid.initialize({ startOnLoad: false });
      mermaid.run({ nodes: [el] }).catch(function(err) {
        el.innerHTML = '<p style="color:#c00;font-family:sans-serif;">' + (err.message || String(err)) + '</p>';
      });
    })();
  </script>
</body>
</html>`;
}

export function registerExportCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.exportMarkdown", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to export to Markdown.");
        return;
      }
      const doc = editor.document;
      const dsl = doc.getText();
      const filePath =
        doc.uri.scheme === "file"
          ? doc.uri.fsPath
          : path.join(os.tmpdir(), "document.sruja");
      const outPath = filePath.replace(/\.sruja$/i, ".md");

      let stdout: string | null = null;
      if (useWasm(context)) {
        stdout = await exportMarkdownFromWasm(context, dsl);
        if (stdout === null) {
          vscode.window.showErrorMessage(
            "Sruja WASM could not load. Reinstall the extension or set sruja.lsp.path to use the Sruja CLI."
          );
          return;
        }
      } else {
        const cliPath = getSrujaPath(context);
        let inputPath = filePath;
        let tmpPath: string | null = null;
        if (doc.isDirty || doc.uri.scheme !== "file") {
          tmpPath = path.join(
            os.tmpdir(),
            `sruja-export-${path.basename(filePath)}`
          );
          await fs.promises.writeFile(tmpPath, dsl, "utf8");
          inputPath = tmpPath;
        }
        try {
          const result = await execFileAsync(cliPath, [
            "export",
            "markdown",
            inputPath,
          ], {
            encoding: "utf8",
          });
          const out =
            Array.isArray(result) ? result[0] : (result as { stdout?: string }).stdout;
          stdout = out ?? "";
        } finally {
          if (tmpPath) await fs.promises.unlink(tmpPath).catch(() => {});
        }
      }

      if (stdout === null || stdout === undefined) {
        vscode.window.showErrorMessage("Export to Markdown failed.");
        return;
      }
      try {
        const mdDoc = await vscode.workspace.openTextDocument({
          content: stdout,
          language: "markdown",
        });
        await vscode.window.showTextDocument(mdDoc, { preview: false });
        await vscode.commands.executeCommand("markdown.showPreview", mdDoc.uri);
        const save = await vscode.window.showInformationMessage(
          "Markdown generated from DSL. Save to file?",
          "Save",
          "Cancel"
        );
        if (save === "Save" && doc.uri.scheme === "file") {
          const uri = vscode.Uri.file(outPath);
          await vscode.workspace.fs.writeFile(uri, Buffer.from(stdout, "utf8"));
          const saved = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(saved);
        }
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        vscode.window.showErrorMessage(`Export to Markdown failed: ${msg}`);
      }
    }),
    vscode.commands.registerCommand("sruja.openDiagramPreview", async () => {
      const editor = vscode.window.activeTextEditor;
      const doc = editor?.document;
      if (!doc || doc.languageId !== "sruja") {
        vscode.window.showWarningMessage(
          "Open a .sruja file to open diagram preview."
        );
        return;
      }
      const dsl = doc.getText();
      if (!useWasm(context)) {
        vscode.window.showInformationMessage(
          "Diagram preview uses bundled WASM. Clear sruja.lsp.path to use it, or use Sruja: Export to Markdown."
        );
        return;
      }
      const mermaid = await getMermaidFromWasm(context, dsl);
      if (mermaid === null || mermaid.trim() === "") {
        vscode.window.showErrorMessage(
          "Sruja WASM could not load or diagram is empty. Run npm run copy:assets if developing, or reinstall the extension."
        );
        return;
      }
      const panel = vscode.window.createWebviewPanel(
        "srujaDiagramPreview",
        "Sruja Diagram Preview",
        vscode.ViewColumn.Beside,
        { enableScripts: true }
      );
      const mermaidEscaped = mermaid
        .replace(/\\/g, "\\\\")
        .replace(/`/g, "\\`")
        .replace(/\$/g, "\\$")
        .replace(/<\/script>/gi, "<\\/script>");
      panel.webview.html = getDiagramPreviewHtml(mermaidEscaped);
    })
  );
}

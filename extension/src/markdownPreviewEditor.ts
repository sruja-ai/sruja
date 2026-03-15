import * as vscode from "vscode";
import { exportMarkdownFromWasm, initWasm } from "./wasm";

interface SrujaMarkdownDocument extends vscode.CustomDocument {
  uri: vscode.Uri;
}

export class SrujaMarkdownPreviewEditorProvider implements vscode.CustomEditorProvider<SrujaMarkdownDocument> {
  private readonly _onDidChangeCustomDocument = new vscode.EventEmitter<vscode.CustomDocumentEditEvent<SrujaMarkdownDocument>>();
  onDidChangeCustomDocument = this._onDidChangeCustomDocument.event;

  constructor(private context: vscode.ExtensionContext) {}

  async openCustomDocument(
    uri: vscode.Uri,
    _openContext: vscode.CustomDocumentOpenContext,
    _token: vscode.CancellationToken
  ): Promise<SrujaMarkdownDocument> {
    return { uri, dispose: () => {} };
  }

  async saveCustomDocument(_document: SrujaMarkdownDocument, _cancellation: vscode.CancellationToken): Promise<void> {}
  async saveCustomDocumentAs(_document: SrujaMarkdownDocument, _destination: vscode.Uri, _cancellation: vscode.CancellationToken): Promise<void> {}
  async revertCustomDocument(_document: SrujaMarkdownDocument, _cancellation: vscode.CancellationToken): Promise<void> {}
  async backupCustomDocument(_document: SrujaMarkdownDocument, _context: vscode.CustomDocumentBackupContext, _cancellation: vscode.CancellationToken): Promise<vscode.CustomDocumentBackup> {
    return { id: "", delete: () => {} };
  }

  async resolveCustomEditor(
    document: SrujaMarkdownDocument,
    webviewPanel: vscode.WebviewPanel,
    _token: vscode.CancellationToken
  ): Promise<void> {
    webviewPanel.webview.options = { enableScripts: true };

    const mod = await initWasm(this.context);
    if (!mod) {
      webviewPanel.webview.html = this.getErrorHtml(
        "WASM not available. Run npm run copy:assets or reinstall the extension."
      );
      return;
    }

    let disposed = false;
    const updatePreview = async (): Promise<void> => {
      if (disposed) return;
      try {
        const editor = vscode.window.visibleTextEditors.find(
          (e) => e.document.uri.toString() === document.uri.toString()
        );
        const dsl = editor?.document.getText() ?? "";
        if (!dsl) return;

        const md = await exportMarkdownFromWasm(this.context, dsl);
        if (disposed) return;
        if (md) {
          webviewPanel.webview.html = this.getMarkdownHtml(md);
        } else {
          webviewPanel.webview.html = this.getErrorHtml("Failed to generate markdown.");
        }
      } catch (err) {
        if (disposed) return;
        const msg = err instanceof Error ? err.message : String(err);
        webviewPanel.webview.html = this.getErrorHtml(`Error: ${msg}`);
      }
    };

    await updatePreview();

    let debounceTimer: ReturnType<typeof setTimeout> | undefined;
    const changeSub = vscode.workspace.onDidChangeTextDocument((e) => {
      if (e.document.uri.toString() === document.uri.toString()) {
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => updatePreview(), 300);
      }
    });

    const saveSub = vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.uri.toString() === document.uri.toString()) {
        updatePreview();
      }
    });

    webviewPanel.onDidDispose(() => {
      disposed = true;
      changeSub.dispose();
      saveSub.dispose();
      if (debounceTimer) clearTimeout(debounceTimer);
    });
  }

  private getMarkdownHtml(markdown: string): string {
    const escaped = JSON.stringify(markdown);
    return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline' https://cdn.jsdelivr.net; style-src 'unsafe-inline' https://cdn.jsdelivr.net;">
  <title>Sruja Markdown Preview</title>
  <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; padding: 16px; max-width: 900px; margin: 0 auto; line-height: 1.6; }
    h1, h2, h3 { margin-top: 1.5em; }
    code { background: #f4f4f4; padding: 2px 6px; border-radius: 3px; font-size: 0.9em; }
    pre { background: #f4f4f4; padding: 12px; border-radius: 6px; overflow-x: auto; }
    pre code { background: none; padding: 0; }
    table { border-collapse: collapse; width: 100%; margin: 1em 0; }
    th, td { border: 1px solid #ddd; padding: 8px 12px; text-align: left; }
    th { background: #f4f4f4; }
  </style>
</head>
<body>
  <div id="content">Loading...</div>
  <script>
    try {
      const md = ${escaped};
      document.getElementById('content').innerHTML = marked.parse(md);
    } catch(e) {
      document.getElementById('content').innerHTML = '<p style="color:#c00">Error: ' + e.message + '</p>';
    }
  </script>
</body>
</html>`;
  }

  private getErrorHtml(message: string): string {
    return `<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><style>body{font-family:sans-serif;padding:20px;color:#c00;}</style></head>
<body><p>${message}</p></body>
</html>`;
  }
}

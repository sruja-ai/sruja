import * as vscode from "vscode";
import { exportMarkdownFromWasm, initWasm } from "./wasm";

interface SrujaMarkdownDocument extends vscode.CustomDocument {
  uri: vscode.Uri;
}

export class SrujaMarkdownPreviewEditorProvider implements vscode.CustomEditorProvider<SrujaMarkdownDocument> {
  public static readonly viewType = "sruja.markdownPreview";
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
    const updatePreview = async (isInitial = false): Promise<void> => {
      if (disposed) return;
      try {
        const inMemoryDoc = vscode.workspace.textDocuments.find((d) => d.uri.toString() === document.uri.toString());
        const dsl = inMemoryDoc?.getText() ?? Buffer.from(await vscode.workspace.fs.readFile(document.uri)).toString("utf8");
        if (!dsl) return;

        const md = await exportMarkdownFromWasm(this.context, dsl);
        if (disposed) return;
        if (md) {
          if (isInitial) {
            webviewPanel.webview.html = this.getMarkdownHtml(webviewPanel.webview, md);
          } else {
            webviewPanel.webview.postMessage({ type: "update", markdown: md });
          }
        } else {
          webviewPanel.webview.html = this.getErrorHtml("Failed to generate markdown.");
        }
      } catch (err) {
        if (disposed) return;
        const msg = err instanceof Error ? err.message : String(err);
        webviewPanel.webview.html = this.getErrorHtml(`Error: ${msg}`);
      }
    };

    await updatePreview(true);

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

  private getMarkdownHtml(webview: vscode.Webview, markdown: string): string {
    const n = Date.now().toString(16);
    const nonce = n + n;
    const csp = `default-src 'none'; img-src ${webview.cspSource} https:; style-src ${webview.cspSource} 'unsafe-inline' https://cdn.jsdelivr.net; script-src ${webview.cspSource} 'nonce-${nonce}' https://cdn.jsdelivr.net;`;

    return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="${csp}">
  <title>Sruja Markdown Preview</title>
  <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/dompurify/dist/purify.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
  <style>
    body { 
      font-family: var(--vscode-font-family); 
      font-weight: var(--vscode-font-weight);
      font-size: var(--vscode-font-size);
      color: var(--vscode-editor-foreground);
      background-color: var(--vscode-editor-background);
      padding: 16px; 
      max-width: 900px; 
      margin: 0 auto; 
      line-height: 1.6; 
    }
    h1, h2, h3 { color: var(--vscode-descriptionForeground); margin-top: 1.5em; border-bottom: 1px solid var(--vscode-textSeparator-foreground); padding-bottom: 0.3em; }
    code { 
      background: var(--vscode-textCodeBlock-background); 
      padding: 2px 6px; 
      border-radius: 3px; 
      font-family: var(--vscode-editor-font-family);
    }
    pre { 
      background: var(--vscode-textCodeBlock-background); 
      padding: 12px; 
      border-radius: 6px; 
      overflow-x: auto; 
      border: 1px solid var(--vscode-textSeparator-foreground);
    }
    pre code { background: none; padding: 0; }
    table { border-collapse: collapse; width: 100%; margin: 1em 0; }
    th, td { border: 1px solid var(--vscode-textSeparator-foreground); padding: 8px 12px; text-align: left; }
    th { background: var(--vscode-textCodeBlock-background); }
    blockquote { border-left: 4px solid var(--vscode-textLink-foreground); padding-left: 16px; color: var(--vscode-descriptionForeground); margin: 1em 0; }
    .mermaid { margin: 2em 0; display: flex; justify-content: center; }
  </style>
</head>
<body class="vscode-body">
  <div id="content">Loading...</div>
  <script nonce="${nonce}">
    (function() {
      const content = document.getElementById('content');
      
      function getTheme() {
        return document.body.classList.contains('vscode-light') ? 'default' : 'dark';
      }

      function initMermaid() {
        mermaid.initialize({ 
          startOnLoad: false, 
          theme: getTheme(),
          flowchart: { useMaxWidth: false, htmlLabels: true }
        });
      }

      function renderMarkdown(markdown) {
        try {
          const mermaidBlocks = [];
          const mdWithPlaceholders = markdown.replace(
            /\`\`\`mermaid\\n([\\s\\S]*?)\`\`\`/g,
            (match, code) => {
              mermaidBlocks.push(code.trim());
              return '\\n\\n<div class="mermaid-placeholder" data-index="' + (mermaidBlocks.length - 1) + '"></div>\\n\\n';
            }
          );

          const parsedHtml = marked.parse(mdWithPlaceholders);
          content.innerHTML = DOMPurify.sanitize(parsedHtml);

          document.querySelectorAll('.mermaid-placeholder').forEach(el => {
            const idx = el.getAttribute('data-index');
            el.className = 'mermaid';
            el.textContent = mermaidBlocks[idx];
          });

          if (mermaidBlocks.length > 0) {
            mermaid.run({ querySelector: '.mermaid' }).catch(err => console.error('Mermaid error:', err));
          }
        } catch (e) {
          content.innerHTML = '<p style="color:var(--vscode-errorForeground)">Error parsing markdown: ' + e.message + '</p>';
        }
      }

      initMermaid();
      renderMarkdown(${JSON.stringify(markdown)});

      window.addEventListener('message', event => {
        const message = event.data;
        if (message.type === 'update') {
          renderMarkdown(message.markdown);
        }
      });

      const observer = new MutationObserver(() => {
        initMermaid();
        mermaid.run({ querySelector: '.mermaid' });
      });
      observer.observe(document.body, { attributes: true, attributeFilter: ['class'] });
    })();
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

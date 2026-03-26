export function escapeMermaidForScript(mermaid: string): string {
  return mermaid
    .replace(/\\/g, "\\\\")
    .replace(/`/g, "\\`")
    .replace(/\$/g, "\\$")
    .replace(/<\/script>/gi, "<\\/script>");
}

export function getDiagramPreviewHtml(mermaidCodeEscaped: string, sourceUri: string): string {
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline' https://cdn.jsdelivr.net; style-src 'unsafe-inline' https://cdn.jsdelivr.net;">
  <style>
    body { 
      background-color: var(--vscode-editor-background); 
      color: var(--vscode-editor-foreground); 
      padding: 0; 
      margin: 0;
      font-family: var(--vscode-font-family);
      overflow: hidden;
      width: 100vw;
      height: 100vh;
    }
    #diagram-container {
      width: 100%;
      height: 100%;
      display: flex;
      flex-direction: column;
    }
    #diagram { 
      flex-grow: 1;
      width: 100%;
      height: 100%;
      overflow: hidden;
    }
    .mermaid { background: transparent; display: flex; justify-content: center; align-items: center; height: 100%; }
    .mermaid .node { cursor: pointer; transition: opacity 0.2s; }
    .mermaid .node:hover { opacity: 0.8; }
    
    .status-bar {
      position: fixed;
      bottom: 0;
      left: 0;
      right: 0;
      background: var(--vscode-statusBar-background);
      color: var(--vscode-statusBar-foreground);
      font-size: 11px;
      padding: 2px 8px;
      display: flex;
      justify-content: space-between;
      z-index: 100;
      opacity: 0.8;
    }
  </style>
  <title>Sruja – Diagram Preview</title>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
</head>
<body>
  <div id="diagram-container">
    <div id="diagram" class="mermaid"></div>
  </div>
  <div class="status-bar">
    <span>Sruja Live Preview</span>
    <span>Click to Jump • Scroll to Zoom • Drag to Pan</span>
  </div>

  <script>
    (function() {
      const vscode = acquireVsCodeApi();
      let currentCode = \`${mermaidCodeEscaped}\`;
      const sourceUri = \`${sourceUri}\`;
      const el = document.getElementById('diagram');
      
      function getTheme() {
        return document.body.classList.contains('vscode-light') ? 'default' : 'dark';
      }

      function initMermaid() {
        mermaid.initialize({ 
          startOnLoad: false,
          theme: getTheme(),
          securityLevel: 'loose',
          flowchart: { useMaxWidth: false, htmlLabels: true }
        });
      }

      function bindNodeClicks() {
        const nodes = document.querySelectorAll('.node');
        nodes.forEach(node => {
          node.addEventListener('click', () => {
             const nodeId = node.id || '';
             const match = nodeId.match(/flowchart-([^-]+)-/) || nodeId.match(/node-([^-]+)-/);
             let elementId = match ? match[1] : node.textContent.trim();
             
             vscode.postMessage({
               command: 'jumpToElement',
               elementId: elementId,
               sourceUri: sourceUri
             });
          });
        });
      }

      async function render(code) {
        currentCode = code;
        el.removeAttribute('data-processed');
        el.textContent = code;
        try {
          await mermaid.run({ nodes: [el] });
          bindNodeClicks();
        } catch (err) {
          el.innerHTML = '<p style="color:var(--vscode-errorForeground);padding:20px;">' + (err.message || String(err)) + '</p>';
        }
      }

      initMermaid();
      render(currentCode);

      window.addEventListener('message', event => {
        const message = event.data;
        if (message.command === 'update') {
          render(message.code);
        }
      });

      const observer = new MutationObserver(() => {
        initMermaid();
        render(currentCode);
      });
      observer.observe(document.body, { attributes: true, attributeFilter: ['class'] });
    })();
  </script>
</body>
</html>
`;
}

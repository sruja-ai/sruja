/**
 * Diagram preview HTML generator. Pure function for testability.
 */

/**
 * Escape mermaid code for safe embedding in a script template (backticks, backslash, script tag).
 */
export function escapeMermaidForScript(mermaid: string): string {
  return mermaid
    .replace(/\\/g, "\\\\")
    .replace(/`/g, "\\`")
    .replace(/\$/g, "\\$")
    .replace(/<\/script>/gi, "<\\/script>");
}

/**
 * Build the webview HTML for the diagram preview. Pass already-escaped mermaid code.
 */
export function getDiagramPreviewHtml(mermaidCodeEscaped: string, sourceUri: string): string {
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline' https://cdn.jsdelivr.net; style-src 'unsafe-inline' https://cdn.jsdelivr.net;">
  <style>
    body { background-color: var(--vscode-editor-background); color: var(--vscode-editor-foreground); padding: 20px; font-family: sans-serif; }
    .mermaid { background: transparent; }
    /* Make nodes look clickable */
    .mermaid .node { cursor: pointer; transition: opacity 0.2s; }
    .mermaid .node:hover { opacity: 0.8; }
  </style>
  <title>Sruja – Context engineering for the AI era. – Diagram Preview</title>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
</head>
<body>
  <div id="diagram" class="mermaid"></div>
  <script>
    (function() {
      const vscode = acquireVsCodeApi();
      const code = \`${mermaidCodeEscaped}\`;
      const sourceUri = \`${sourceUri}\`;
      const el = document.getElementById('diagram');
      el.textContent = code;
      
      mermaid.initialize({ 
        startOnLoad: false,
        theme: 'dark',
        securityLevel: 'loose' 
      });

      mermaid.run({ nodes: [el] }).then(function() {
        // Add click listeners to nodes after rendering
        const nodes = document.querySelectorAll('.node');
        nodes.forEach(node => {
          node.addEventListener('click', () => {
             // Try to find the element ID from the node text or ID
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
      }).catch(function(err) {
        el.innerHTML = '<p style="color:#c00;font-family:sans-serif;">' + (err.message || String(err)) + '</p>';
      });
    })();
  </script>
</body>
</html>
`;
}

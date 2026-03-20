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
export function getDiagramPreviewHtml(mermaidCodeEscaped: string): string {
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline' https://cdn.jsdelivr.net; style-src 'unsafe-inline' https://cdn.jsdelivr.net;">
  <title>Sruja – Context engineering for the AI era. – Diagram Preview</title>
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
</html>
`;
}

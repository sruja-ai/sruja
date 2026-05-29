import * as vscode from "vscode";

// SVG data URIs for gutter icons (16x16 circles/triangles)
const ERROR_ICON = vscode.Uri.parse(
  `data:image/svg+xml,${encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><circle cx="8" cy="8" r="6" fill="#e51400"/></svg>'
  )}`
);

const WARNING_ICON = vscode.Uri.parse(
  `data:image/svg+xml,${encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><polygon points="8,2 14,14 2,14" fill="#e3a000"/></svg>'
  )}`
);

const INFO_ICON = vscode.Uri.parse(
  `data:image/svg+xml,${encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><circle cx="8" cy="8" r="6" fill="#3794ff"/></svg>'
  )}`
);

const errorDecorationType = vscode.window.createTextEditorDecorationType({
  gutterIconPath: ERROR_ICON,
  gutterIconSize: "70%",
});

const warningDecorationType = vscode.window.createTextEditorDecorationType({
  gutterIconPath: WARNING_ICON,
  gutterIconSize: "70%",
});

const infoDecorationType = vscode.window.createTextEditorDecorationType({
  gutterIconPath: INFO_ICON,
  gutterIconSize: "70%",
});

/**
 * Apply gutter decorations to an editor based on its diagnostics.
 * Call this after setting diagnostics on the diagnostic collection.
 */
export function applyGutterDecorations(
  editor: vscode.TextEditor,
  diagnostics: readonly vscode.Diagnostic[]
): void {
  const errors: vscode.DecorationOptions[] = [];
  const warnings: vscode.DecorationOptions[] = [];
  const infos: vscode.DecorationOptions[] = [];

  for (const diag of diagnostics) {
    const opts: vscode.DecorationOptions = { range: diag.range };
    switch (diag.severity) {
      case vscode.DiagnosticSeverity.Error:
        errors.push(opts);
        break;
      case vscode.DiagnosticSeverity.Warning:
        warnings.push(opts);
        break;
      case vscode.DiagnosticSeverity.Information:
      case vscode.DiagnosticSeverity.Hint:
        infos.push(opts);
        break;
    }
  }

  editor.setDecorations(errorDecorationType, errors);
  editor.setDecorations(warningDecorationType, warnings);
  editor.setDecorations(infoDecorationType, infos);
}

/**
 * Clear gutter decorations from an editor.
 */
export function clearGutterDecorations(editor: vscode.TextEditor): void {
  editor.setDecorations(errorDecorationType, []);
  editor.setDecorations(warningDecorationType, []);
  editor.setDecorations(infoDecorationType, []);
}

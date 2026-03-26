import * as vscode from "vscode";
import { getMermaidFromWasm } from "../wasm";

/**
 * Implementation of sruja.exportMarkdown command.
 * Generates an architectural markdown representation and displays it.
 */
export function registerExportCommands(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.exportMarkdown", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "sruja") {
        vscode.window.showWarningMessage("Open a .sruja file to export markdown.");
        return;
      }

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: "Sruja: Generating Markdown...",
          cancellable: false,
        },
        async () => {
          try {
            const mermaid = await getMermaidFromWasm(context, editor.document.getText());
            if (!mermaid) {
               vscode.window.showErrorMessage("Failed to generate architecture export.");
               return;
            }

            const markdown = `# Architecture Export\n\nGenerated from \`${editor.document.uri.fsPath}\`\n\n\`\`\`mermaid\n${mermaid}\n\`\`\`\n`;
            
            // Create a new untitled markdown file
            const doc = await vscode.workspace.openTextDocument({
              content: markdown,
              language: "markdown",
            });
            await vscode.window.showTextDocument(doc, vscode.ViewColumn.Beside);
          } catch (err) {
            vscode.window.showErrorMessage(`Export failed: ${err instanceof Error ? err.message : String(err)}`);
          }
        }
      );
    })
  );
}

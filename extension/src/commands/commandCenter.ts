import * as vscode from "vscode";

export function registerCommandCenter(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.commandCenter", async () => {
      const items: (vscode.QuickPickItem & { command: string })[] = [
        {
          label: "$(check-all) Validate",
          description: "Run validation (check after AI/edit)",
          command: "sruja.runValidation",
        },
        {
          label: "$(markdown) Export",
          description: "Export architecture to Markdown",
          command: "sruja.exportMarkdown",
        },
        {
          label: "$(graph) Diagram",
          description: "Open Diagram Preview",
          command: "sruja.openDiagramPreview",
        },
        {
          label: "$(filter) Focused Diagram",
          description: "Open Focused Diagram Preview",
          command: "sruja.openFocusedDiagramPreview",
        },
        {
          label: "$(sync) Refresh Context",
          description: "Refresh repo context",
          command: "sruja.refreshContext",
        },
        {
          label: "$(pulse) Run Drift",
          description: "Run drift (architecture health)",
          command: "sruja.runDrift",
        },
        {
          label: "$(info) Status",
          description: "Sruja Status",
          command: "sruja.status",
        },
        {
          label: "$(eye) Review",
          description: "Review architecture update",
          command: "sruja.review",
        },
      ];

      const selected = await vscode.window.showQuickPick(items, {
        placeHolder: "Sruja: Select an action",
        title: "Sruja Command Center",
      });

      if (selected) {
        await vscode.commands.executeCommand(selected.command);
      }
    })
  );
}

import * as vscode from "vscode";

export function registerCommandCenter(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.commandCenter", async () => {
      const items: (vscode.QuickPickItem & { command: string })[] = [
        {
          label: "$(check) Verify task",
          description: "Host gate after AI edits (verify-task)",
          command: "sruja.verifyTask",
        },
        {
          label: "$(lightbulb) Brief file",
          description: "Task-scoped focus briefing",
          command: "sruja.briefThisFile",
        },
        {
          label: "$(pulse) Structural drift",
          description: "Scan structural drift (no repo.sruja required)",
          command: "sruja.runDrift",
        },
        {
          label: "$(plug) Register MCP",
          description: "Register Sruja MCP in Cursor",
          command: "sruja.registerMcpServer",
        },
        {
          label: "$(copy) Context pack",
          description: "Copy context pack for AI",
          command: "sruja.copyContextPackForAI",
        },
        {
          label: "$(check-all) Validate .sruja",
          description: "Lint/validate architecture file",
          command: "sruja.runValidation",
        },
        {
          label: "$(sync) Refresh Context",
          description: "Refresh repo context (sync)",
          command: "sruja.refreshContext",
        },
        {
          label: "$(info) Status",
          description: "Truth freshness and baseline",
          command: "sruja.status",
        },
        {
          label: "$(eye) Review",
          description: "Daily/review architecture update",
          command: "sruja.review",
        },
        {
          label: "$(graph) Diagram",
          description: "Open diagram preview (Tier 2)",
          command: "sruja.openDiagramPreview",
        },
        {
          label: "$(markdown) Export",
          description: "Export architecture to Markdown",
          command: "sruja.exportMarkdown",
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

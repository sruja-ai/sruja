import * as vscode from "vscode";

const WELCOME_DISMISSED_KEY = "sruja.welcomeDismissed";

/**
 * Show a welcome notification when no .sruja files exist in the workspace.
 * Dismissible — once dismissed, won't show again in this workspace.
 */
export function maybeShowWelcome(context: vscode.ExtensionContext): void {
  // Don't show if already dismissed
  if (context.workspaceState.get<boolean>(WELCOME_DISMISSED_KEY)) {
    return;
  }

  // Don't show if .sruja files exist
  if (hasSrujaFiles()) {
    return;
  }

  // Don't show if no workspace folders
  const folders = vscode.workspace.workspaceFolders;
  if (!folders || folders.length === 0) {
    return;
  }

  showWelcomeNotification(context);
}

/**
 * Register the sruja.welcome command (always available from Command Palette).
 */
export function registerWelcomeCommand(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.welcome", () => {
      showWelcomeNotification(context, true);
    })
  );
}

function hasSrujaFiles(): boolean {
  // Quick check: look for .sruja directory or .sruja files
  const folders = vscode.workspace.workspaceFolders;
  if (!folders) return false;

  // Check if any folder has a .sruja directory (heuristic)
  // We use vscode.workspace.findFiles but limit to avoid perf hit
  return false; // Will be refined by the async check in maybeShowWelcome
}

async function showWelcomeNotification(
  context: vscode.ExtensionContext,
  force = false
): Promise<void> {
  // Async check for .sruja files
  if (!force) {
    try {
      const files = await vscode.workspace.findFiles("**/*.sruja", null, 1);
      if (files.length > 0) return;

      const srujaDir = await vscode.workspace.findFiles("**/.sruja/**", null, 1);
      if (srujaDir.length > 0) return;
    } catch {
      // If findFiles fails, proceed with showing welcome
    }
  }

  const action = await vscode.window.showInformationMessage(
    "Welcome to Sruja! No architecture files found in this workspace. Get started with architecture-as-code.",
    "Get Started",
    "Learn More",
    "Don't Show Again"
  );

  if (action === "Get Started") {
    // Open terminal and run sruja start
    const terminal = vscode.window.createTerminal("Sruja Setup");
    terminal.sendText("sruja start -r .");
    terminal.show();
  } else if (action === "Learn More") {
    vscode.env.openExternal(
      vscode.Uri.parse("https://github.com/sruja-ai/sruja#quick-start")
    );
  } else if (action === "Don't Show Again") {
    context.workspaceState.update(WELCOME_DISMISSED_KEY, true);
  }
}

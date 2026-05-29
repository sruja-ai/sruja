import * as vscode from "vscode";
import { runCli } from "./cliRunner";
import { parseJsonSafe } from "./safeJson";

export interface StatusState {
  healthScore?: number;
  truthStatus?: string;
  violationsCount?: number;
  lastUpdated?: number;
}

let statusBarItem: vscode.StatusBarItem | undefined;
let refreshTimer: ReturnType<typeof setInterval> | undefined;
let currentState: StatusState = {};

const REFRESH_INTERVAL_MS = 60_000;
const DEBOUNCE_SAVE_MS = 5_000;
let saveDebounce: ReturnType<typeof setTimeout> | undefined;

export function registerStatusBar(
  context: vscode.ExtensionContext,
  getSrujaPath: () => string
): void {
  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100
  );
  statusBarItem.command = "sruja.commandCenter";
  statusBarItem.tooltip = "Sruja — click for Command Center";
  context.subscriptions.push(statusBarItem);

  // Initial fetch
  refreshStatus(getSrujaPath);

  // Periodic refresh
  refreshTimer = setInterval(() => refreshStatus(getSrujaPath), REFRESH_INTERVAL_MS);
  context.subscriptions.push({ dispose: () => clearInterval(refreshTimer) });

  // Refresh on save (debounced)
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.languageId === "sruja" || doc.fileName.endsWith(".sruja")) {
        if (saveDebounce) clearTimeout(saveDebounce);
        saveDebounce = setTimeout(() => refreshStatus(getSrujaPath), DEBOUNCE_SAVE_MS);
      }
    })
  );
}

export function getStatusState(): StatusState {
  return currentState;
}

async function refreshStatus(getSrujaPath: () => string): Promise<void> {
  if (!statusBarItem) return;

  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) {
    statusBarItem.text = "$(heart) Sruja: no workspace";
    statusBarItem.color = undefined;
    statusBarItem.show();
    return;
  }

  statusBarItem.text = "$(loading~spin) Sruja: checking…";
  statusBarItem.show();

  try {
    const { stdout, code } = await runCli(
      getSrujaPath(),
      ["status", "-r", ".", "--format", "json"],
      folder.uri.fsPath
    );

    if (code !== 0) {
      statusBarItem.text = "$(warning) Sruja: error";
      statusBarItem.color = new vscode.ThemeColor("statusBarItem.warningForeground");
      return;
    }

    const parsed = parseJsonSafe<{
      health_score?: number;
      truth_status?: string;
      violations_count?: number;
    }>(stdout);

    if (!parsed.ok) {
      statusBarItem.text = "$(warning) Sruja: parse error";
      statusBarItem.color = new vscode.ThemeColor("statusBarItem.warningForeground");
      return;
    }

    const { health_score, truth_status, violations_count } = parsed.value;
    currentState = {
      healthScore: health_score,
      truthStatus: truth_status,
      violationsCount: violations_count,
      lastUpdated: Date.now(),
    };

    const score = health_score ?? 0;
    const violations = violations_count ?? 0;
    const truth = truth_status ?? "unknown";

    // Health icon based on score
    let healthIcon: string;
    if (score >= 90) {
      healthIcon = "$(heart)";
    } else if (score >= 70) {
      healthIcon = "$(heart)";
    } else {
      healthIcon = "$(heart)";
    }

    // Truth status icon
    let truthIcon: string;
    if (truth === "reviewed") {
      truthIcon = "$(pass-filled)";
    } else if (truth === "drifted") {
      truthIcon = "$(warning)";
    } else {
      truthIcon = "$(circle-filled)";
    }

    // Build status bar text
    let text = `${healthIcon} Sruja: ${score}/100  ${truthIcon} ${truth}`;
    if (violations > 0) {
      text += `  $(warning) ${violations}`;
    }

    statusBarItem.text = text;

    // Color based on health
    if (score >= 90) {
      statusBarItem.color = undefined; // default (green in most themes)
    } else if (score >= 70) {
      statusBarItem.color = new vscode.ThemeColor("statusBarItem.warningForeground");
    } else {
      statusBarItem.color = new vscode.ThemeColor("statusBarItem.errorForeground");
    }
  } catch (err) {
    // CLI not available or other error — show degraded state
    statusBarItem.text = "$(heart) Sruja: unavailable";
    statusBarItem.color = undefined;
  }
}

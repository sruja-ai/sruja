import * as vscode from "vscode";
import { getStatusState, type StatusState } from "./statusBar";

// Tree item types
type ArchItemType = "health" | "status" | "violations" | "violation" | "components" | "sync" | "info" | "action";

interface ArchTreeItemOpts {
  type: ArchItemType;
  icon?: string;
  description?: string;
  command?: vscode.Command;
  contextValue?: string;
  fileUri?: vscode.Uri;
  line?: number;
}

export class ArchitectureTreeProvider implements vscode.TreeDataProvider<ArchTreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<ArchTreeItem | undefined | void>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  constructor(private context: vscode.ExtensionContext) {}

  refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(element: ArchTreeItem): vscode.TreeItem {
    return element;
  }

  async getChildren(element?: ArchTreeItem): Promise<ArchTreeItem[]> {
    if (element) {
      // Violations are expandable
      if (element.itemType === "violations") {
        return this.getViolationChildren();
      }
      return [];
    }

    // Root items
    const state = getStatusState();
    const items: ArchTreeItem[] = [];

    // Health
    const score = state.healthScore;
    const healthLabel = score != null ? `Health: ${score}/100` : "Health: —";
    const healthIcon = score != null
      ? score >= 90 ? "$(heart)" : score >= 70 ? "$(heart)" : "$(heart)"
      : "$(heart)";
    items.push(
      new ArchTreeItem(healthLabel, {
        type: "health",
        icon: healthIcon,
        description: score != null ? this.healthDescription(score) : "not available",
        contextValue: "srujaHealth",
      })
    );

    // Truth status
    const truth = state.truthStatus ?? "unknown";
    const truthIcon = truth === "reviewed" ? "$(pass-filled)" : truth === "drifted" ? "$(warning)" : "$(circle-filled)";
    items.push(
      new ArchTreeItem(`Status: ${truth}`, {
        type: "status",
        icon: truthIcon,
        contextValue: "srujaStatus",
      })
    );

    // Violations
    const violations = state.violationsCount ?? 0;
    if (violations > 0) {
      items.push(
        new ArchTreeItem(`Drift Findings (${violations})`, {
          type: "violations",
          icon: "$(warning)",
          contextValue: "srujaViolations",
        })
      );
    } else {
      items.push(
        new ArchTreeItem("No drift findings", {
          type: "violations",
          icon: "$(pass-filled)",
          contextValue: "srujaViolations",
        })
      );
    }

    // Last sync
    if (state.lastUpdated) {
      const elapsed = Date.now() - state.lastUpdated;
      const elapsedStr = this.formatElapsed(elapsed);
      items.push(
        new ArchTreeItem(`Last sync: ${elapsedStr}`, {
          type: "sync",
          icon: "$(history)",
          contextValue: "srujaSync",
        })
      );
    }

    // Quick actions
    items.push(
      new ArchTreeItem("Run Drift Scan", {
        type: "action",
        icon: "$(pulse)",
        command: { command: "sruja.runDrift", title: "Run Drift" },
        contextValue: "srujaAction",
      })
    );
    items.push(
      new ArchTreeItem("Open Explorer", {
        type: "action",
        icon: "$(graph)",
        command: { command: "sruja.openArchitectureExplorer", title: "Open Explorer" },
        contextValue: "srujaAction",
      })
    );
    items.push(
      new ArchTreeItem("Refresh Status", {
        type: "action",
        icon: "$(refresh)",
        command: { command: "sruja.refreshStatus", title: "Refresh" },
        contextValue: "srujaAction",
      })
    );

    return items;
  }

  private async getViolationChildren(): Promise<ArchTreeItem[]> {
    // Parse violations from the last CLI output (cached in output channel)
    // For now, return a placeholder — violations are populated by the status refresh
    // which stores them in the output channel. A future enhancement could parse
    // the full drift JSON and store structured violations.
    return [
      new ArchTreeItem("Run drift scan to see findings", {
        type: "info",
        icon: "$(info)",
        contextValue: "srujaInfo",
      }),
    ];
  }

  private healthDescription(score: number): string {
    if (score >= 90) return "excellent";
    if (score >= 70) return "good";
    if (score >= 40) return "fair";
    return "poor";
  }

  private formatElapsed(ms: number): string {
    const secs = Math.floor(ms / 1000);
    if (secs < 60) return "just now";
    const mins = Math.floor(secs / 60);
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  }
}

class ArchTreeItem extends vscode.TreeItem {
  itemType: ArchItemType;

  constructor(label: string, opts: ArchTreeItemOpts) {
    super(
      label,
      opts.type === "violations"
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None
    );
    this.itemType = opts.type;
    // $(icon) syntax in labels is rendered by VS Code automatically.
    // For iconPath, extract the icon name from $(icon) format.
    if (opts.icon) {
      const match = opts.icon.match(/\$\(([^)]+)\)/);
      this.iconPath = match ? new vscode.ThemeIcon(match[1]) : undefined;
    }
    if (opts.description) this.description = opts.description;
    if (opts.command) this.command = opts.command;
    if (opts.contextValue) this.contextValue = opts.contextValue;
  }
}

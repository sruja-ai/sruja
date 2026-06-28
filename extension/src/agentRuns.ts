import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import { execSync } from "child_process";

// ---------------------------------------------------------------------------
// Types matching the LoopResult schema from sruja-agent
// ---------------------------------------------------------------------------

interface Usage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  estimated_cost_usd?: number;
}

interface StepResult {
  subtask_id: string;
  status: "ok" | "failed" | "skipped";
  output: string;
}

interface Plan {
  goal: string;
  subtasks: { id: string; description: string }[];
}

interface AgentRunResult {
  goal: string;
  comprehension: { summary: string };
  plan?: Plan;
  step_results: StepResult[];
}

interface LoopIteration {
  iteration: number;
  replanned: boolean;
  plan_goal: string;
  subtask_count: number;
  succeeded: number;
  failed: number;
  critique_approved: boolean;
  critique_score: number;
  critique_issues?: string[];
  verify_failed?: string[];
}

interface LoopResult {
  goal: string;
  iterations: LoopIteration[];
  converged: boolean;
  termination: string;
  total_usage: Usage;
  final_result: AgentRunResult;
}

// ---------------------------------------------------------------------------
// Tree item types
// ---------------------------------------------------------------------------

type TreeNode =
  | { kind: "run"; runId: string; result: LoopResult; workspaceRoot: string }
  | { kind: "file"; filePath: string; workspaceRoot: string; status: string; added: number; deleted: number }
  | { kind: "iteration"; iteration: LoopIteration; steps: StepResult[] }
  | { kind: "step"; step: StepResult; description: string }
  | { kind: "info"; message: string };

class AgentRunItem extends vscode.TreeItem {
  constructor(
    label: string,
    public readonly node: TreeNode,
    collapsibleState: vscode.TreeItemCollapsibleState
  ) {
    super(label, collapsibleState);
  }
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

export class AgentRunsProvider implements vscode.TreeDataProvider<AgentRunItem> {
  private _onDidChange = new vscode.EventEmitter<AgentRunItem | undefined | void>();
  readonly onDidChangeTreeData = this._onDidChange.event;

  refresh(): void {
    this._onDidChange.fire();
  }

  getTreeItem(element: AgentRunItem): vscode.TreeItem {
    return element;
  }

  async getChildren(element?: AgentRunItem): Promise<AgentRunItem[]> {
    const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!root) return [];

    if (!element) {
      return this.getRunList(root);
    }

    const node = element.node;
    if (node.kind === "run") {
      return this.getRunChildren(node);
    }
    if (node.kind === "iteration") {
      return this.getStepItems(node);
    }
    return [];
  }

  // -----------------------------------------------------------------------
  // Top-level: list recent runs
  // -----------------------------------------------------------------------

  private async getRunList(root: string): Promise<AgentRunItem[]> {
    const runsDir = path.join(root, ".sruja", "runs");
    if (!fs.existsSync(runsDir)) {
      return [this.infoItem("No agent runs found. Run 'sruja agent loop' to see results here.")];
    }

    const entries = fs.readdirSync(runsDir, { withFileTypes: true })
      .filter((e) => e.isDirectory())
      .map((e) => {
        const runDir = path.join(runsDir, e.name);
        const loopJson = path.join(runDir, "loop.json");
        let mtime = 0;
        try { mtime = fs.statSync(loopJson).mtimeMs; } catch { /* no loop.json */ }
        return { name: e.name, runDir, loopJson, mtime };
      })
      .filter((e) => fs.existsSync(e.loopJson))
      .sort((a, b) => b.mtime - a.mtime);

    if (entries.length === 0) {
      return [this.infoItem("No completed agent runs found (runs with loop.json).")];
    }

    const items: AgentRunItem[] = [];
    for (const entry of entries.slice(0, 10)) {
      try {
        const raw = fs.readFileSync(entry.loopJson, "utf-8");
        const result: LoopResult = JSON.parse(raw);
        const cost = result.total_usage?.estimated_cost_usd;
        const costStr = cost != null ? `$${cost.toFixed(2)}` : "";
        const icon = result.converged ? "$(check)" : "$(error)";
        const iterStr = `${result.iterations.length} iter`;
        const label = `${icon} ${result.goal.length > 60 ? result.goal.slice(0, 57) + "..." : result.goal}`;
        const desc = [iterStr, costStr].filter(Boolean).join(", ");
        const item = new AgentRunItem(
          label,
          { kind: "run", runId: entry.name, result, workspaceRoot: root },
          vscode.TreeItemCollapsibleState.Expanded
        );
        item.description = desc;
        item.tooltip = `${result.goal}\n${result.iterations.length} iterations | ${costStr} | ${result.converged ? "Converged" : result.termination}`;
        items.push(item);
      } catch {
        // skip unparseable
      }
    }
    return items;
  }

  // -----------------------------------------------------------------------
  // Run children: changed files + iterations
  // -----------------------------------------------------------------------

  private async getRunChildren(node: Extract<TreeNode, { kind: "run" }>): Promise<AgentRunItem[]> {
    const result = node.result;

    // --- Changed files via git diff (click to open diff editor) ---
    const changedFiles = this.getGitDiff(node.workspaceRoot);
    const fileItems = changedFiles.map((f) => {
      const statusIcon = f.status === "A" ? "$(diff-add)" : f.status === "D" ? "$(diff-remove)" : "$(diff-modified)";
      const item = new AgentRunItem(
        `${statusIcon} ${path.basename(f.filePath)}`,
        { kind: "file", filePath: f.filePath, workspaceRoot: node.workspaceRoot, status: f.status, added: f.added, deleted: f.deleted },
        vscode.TreeItemCollapsibleState.None
      );
      item.description = `${f.filePath.length > 45 ? "..." + f.filePath.slice(-42) : f.filePath}  (+${f.added} -${f.deleted})`;
      item.tooltip = f.filePath;
      item.command = {
        command: "sruja.openAgentDiff",
        title: "Open Diff",
        arguments: [f.filePath, node.workspaceRoot],
      };
      item.contextValue = "agentFile";
      return item;
    });

    // --- Iterations ---
    const iterItems = result.iterations.map((iter) => {
      const status = iter.critique_approved ? "$(check)" : "$(x)";
      const label = `${status} Iteration ${iter.iteration}`;
      const item = new AgentRunItem(
        label,
        { kind: "iteration", iteration: iter, steps: result.final_result.step_results },
        vscode.TreeItemCollapsibleState.Collapsed
      );
      const parts = [
        `${iter.succeeded}/${iter.subtask_count} ok`,
        `score ${iter.critique_score.toFixed(1)}`,
      ];
      if (iter.replanned) parts.push("replanned");
      item.description = parts.join(", ");
      item.tooltip = `Plan: ${iter.plan_goal}\nCritique: ${iter.critique_approved ? "approved" : "rejected"}\nIssues: ${(iter.critique_issues ?? []).join("; ")}`;
      return item;
    });

    return [...fileItems, ...iterItems];
  }

  // -----------------------------------------------------------------------
  // Steps under an iteration
  // -----------------------------------------------------------------------

  private getStepItems(node: Extract<TreeNode, { kind: "iteration" }>): AgentRunItem[] {
    const steps = node.steps;
    if (!steps || steps.length === 0) {
      return [this.infoItem("No step results recorded.")];
    }

    return steps.map((step) => {
      const icon = step.status === "ok" ? "$(check)" : step.status === "failed" ? "$(error)" : "$(dash)";
      const desc = step.subtask_id.replace(/_/g, " ");
      const item = new AgentRunItem(
        `${icon} ${desc.length > 55 ? desc.slice(0, 52) + "..." : desc}`,
        { kind: "step", step, description: desc },
        vscode.TreeItemCollapsibleState.None
      );
      item.tooltip = step.output.slice(0, 500);
      item.contextValue = "agentStep";
      return item;
    });
  }

  // -----------------------------------------------------------------------
  // Git diff helper
  // -----------------------------------------------------------------------

  private getGitDiff(root: string): { filePath: string; status: string; added: number; deleted: number }[] {
    try {
      const numstatOutput = execSync("git diff --numstat HEAD", {
        cwd: root,
        encoding: "utf-8",
        timeout: 5000,
        stdio: ["pipe", "pipe", "pipe"],
      }).trim();

      const nameStatusOutput = execSync("git diff --name-status HEAD", {
        cwd: root,
        encoding: "utf-8",
        timeout: 5000,
        stdio: ["pipe", "pipe", "pipe"],
      }).trim();

      if (!numstatOutput) return [];

      const statusMap = new Map<string, string>();
      for (const line of nameStatusOutput.split("\n")) {
        const [status, ...pathParts] = line.trim().split("\t");
        if (status && pathParts.length > 0) {
          statusMap.set(pathParts.join("\t"), status.charAt(0));
        }
      }

      return numstatOutput.split("\n").map((line) => {
        const parts = line.trim().split("\t");
        const added = parts[0] === "-" ? 0 : parseInt(parts[0], 10) || 0;
        const deleted = parts[1] === "-" ? 0 : parseInt(parts[1], 10) || 0;
        const filePath = parts[2] ?? "";
        const status = statusMap.get(filePath) ?? "M";
        return { filePath, status, added, deleted };
      }).filter((f) => f.filePath);
    } catch {
      return [];
    }
  }

  // -----------------------------------------------------------------------
  // Utils
  // -----------------------------------------------------------------------

  private infoItem(message: string): AgentRunItem {
    return new AgentRunItem(
      `$(info) ${message}`,
      { kind: "info", message },
      vscode.TreeItemCollapsibleState.None
    );
  }
}

// ---------------------------------------------------------------------------
// Command registration
// ---------------------------------------------------------------------------

export async function openAgentDiff(filePath: string, workspaceRoot: string): Promise<void> {
  const uri = vscode.Uri.file(path.join(workspaceRoot, filePath));
  try {
    await vscode.commands.executeCommand("git.openChange", uri);
  } catch {
    await vscode.commands.executeCommand("vscode.open", uri);
  }
}

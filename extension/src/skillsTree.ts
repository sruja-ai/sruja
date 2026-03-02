import * as vscode from "vscode";
import { getSkills, SkillInfo } from "./skills";

export class SrujaSkillsTreeProvider implements vscode.TreeDataProvider<SkillsTreeItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<SkillsTreeItem | undefined | void>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  constructor(private context: vscode.ExtensionContext) {}

  refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(element: SkillsTreeItem): vscode.TreeItem {
    return element;
  }

  async getChildren(element?: SkillsTreeItem): Promise<SkillsTreeItem[]> {
    const skills = await getSkills(this.context);

    if (skills.length === 0) {
      return [
        new SkillsTreeItem("No skills found. Open Sruja repo or set sruja.skills.path", {
          type: "info",
        }),
      ];
    }

    if (!element) {
      return skills.map((s) => new SkillsTreeItem(s.name, { skillInfo: s, type: "skill" }));
    }

    const info = element.skillInfo;
    if (info) {
      const items: SkillsTreeItem[] = [
        new SkillsTreeItem("SKILL.md", { uri: info.skillUri, type: "file" }),
      ];

      if (info.agentsUri) {
        items.push(new SkillsTreeItem("AGENTS.md", { uri: info.agentsUri, type: "file" }));
      }

      for (const r of info.ruleUris) {
        items.push(new SkillsTreeItem(r.label + ".md", { uri: r.uri, type: "file" }));
      }

      return items;
    }

    return [];
  }
}

export class SkillsTreeItem extends vscode.TreeItem {
  skillInfo?: SkillInfo;

  constructor(
    label: string,
    opts: { uri?: vscode.Uri; skillInfo?: SkillInfo; type: "skill" | "file" | "info" }
  ) {
    super(
      label,
      opts.type === "skill"
        ? vscode.TreeItemCollapsibleState.Expanded
        : vscode.TreeItemCollapsibleState.None
    );

    if (opts.uri && opts.type === "file") {
      this.command = { command: "vscode.open", title: "Open", arguments: [opts.uri] };
      this.resourceUri = opts.uri;
    }

    if (opts.type === "file") this.iconPath = new vscode.ThemeIcon("file");
    if (opts.type === "skill") this.iconPath = new vscode.ThemeIcon("book");

    this.skillInfo = opts.skillInfo;
  }
}

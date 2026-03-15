import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import * as vscode from "vscode";
import { ExtensionContext } from "./__mocks__/vscode";
import { SrujaSkillsTreeProvider, SkillsTreeItem } from "./skillsTree";

function asContext(ctx: ExtensionContext): vscode.ExtensionContext {
  return ctx as unknown as vscode.ExtensionContext;
}

describe("SrujaSkillsTreeProvider", () => {
  it("getTreeItem returns the element", () => {
    const item = new SkillsTreeItem("label", { type: "info" });
    const provider = new SrujaSkillsTreeProvider(asContext(new ExtensionContext()));
    expect(provider.getTreeItem(item)).toBe(item);
  });

  it("getChildren with no skills returns info item", async () => {
    const context = new ExtensionContext();
    (vscode.workspace as { workspaceFolders?: unknown[] }).workspaceFolders = [];
    const provider = new SrujaSkillsTreeProvider(asContext(context));
    const children = await provider.getChildren();
    expect(children).toHaveLength(1);
    expect(children[0].label).toBe("No skills found. Open Sruja repo or set sruja.skills.path");
  });

  it("refresh can be called without throwing", () => {
    const provider = new SrujaSkillsTreeProvider(asContext(new ExtensionContext()));
    expect(() => provider.refresh()).not.toThrow();
  });

  it("getChildren returns skill items when skills exist", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-tree-test-"));
    try {
      const skillsDir = path.join(tmpDir, "skills");
      const skillDir = path.join(skillsDir, "my-skill");
      fs.mkdirSync(skillDir, { recursive: true });
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# Skill", "utf8");
      (vscode.workspace as { workspaceFolders?: { uri: vscode.Uri }[] }).workspaceFolders = [
        { uri: vscode.Uri.file(tmpDir) },
      ];
      const context = new ExtensionContext();
      context.extensionUri = vscode.Uri.file("/nonexistent");
      const provider = new SrujaSkillsTreeProvider(asContext(context));
      const children = await provider.getChildren();
      expect(children).toHaveLength(1);
      expect(children[0].label).toBe("my-skill");
      expect(children[0].skillInfo?.name).toBe("my-skill");
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it("getChildren returns file items for a skill element", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-tree-children-"));
    try {
      const skillsDir = path.join(tmpDir, "skills");
      const skillDir = path.join(skillsDir, "arch");
      const rulesDir = path.join(skillDir, "rules");
      fs.mkdirSync(rulesDir, { recursive: true });
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# Arch", "utf8");
      fs.writeFileSync(path.join(skillDir, "AGENTS.md"), "# Agents", "utf8");
      fs.writeFileSync(path.join(rulesDir, "c4.md"), "# C4", "utf8");
      (vscode.workspace as { workspaceFolders?: { uri: vscode.Uri }[] }).workspaceFolders = [
        { uri: vscode.Uri.file(tmpDir) },
      ];
      const context = new ExtensionContext();
      context.extensionUri = vscode.Uri.file("/nonexistent");
      const provider = new SrujaSkillsTreeProvider(asContext(context));
      const topChildren = await provider.getChildren();
      expect(topChildren.length).toBeGreaterThanOrEqual(1);
      const skillItem = topChildren[0];
      const fileChildren = await provider.getChildren(skillItem);
      expect(fileChildren.map((c) => c.label)).toContain("SKILL.md");
      expect(fileChildren.map((c) => c.label)).toContain("AGENTS.md");
      expect(fileChildren.some((c) => String(c.label).includes("c4"))).toBe(true);
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  it("getChildren returns empty array for file item (no skillInfo)", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-tree-file-"));
    try {
      const skillsDir = path.join(tmpDir, "skills");
      const skillDir = path.join(skillsDir, "one");
      fs.mkdirSync(skillDir, { recursive: true });
      fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# One", "utf8");
      (vscode.workspace as { workspaceFolders?: { uri: vscode.Uri }[] }).workspaceFolders = [
        { uri: vscode.Uri.file(tmpDir) },
      ];
      const context = new ExtensionContext();
      context.extensionUri = vscode.Uri.file("/nonexistent");
      const provider = new SrujaSkillsTreeProvider(asContext(context));
      const fileItem = new SkillsTreeItem("SKILL.md", {
        uri: vscode.Uri.file(path.join(skillDir, "SKILL.md")),
        type: "file",
      });
      const children = await provider.getChildren(fileItem);
      expect(children).toEqual([]);
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });
});

describe("SkillsTreeItem", () => {
  it("creates info item without command", () => {
    const item = new SkillsTreeItem("No skills", { type: "info" });
    expect(item.label).toBe("No skills");
    expect(item.command).toBeUndefined();
  });

  it("creates file item with command and icon", () => {
    const uri = vscode.Uri.file("/path/to/SKILL.md");
    const item = new SkillsTreeItem("SKILL.md", { uri, type: "file" });
    expect(item.command).toEqual({ command: "vscode.open", title: "Open", arguments: [uri] });
    expect(item.resourceUri).toBe(uri);
  });

  it("creates skill item with expanded state", () => {
    const item = new SkillsTreeItem("arch", {
      type: "skill",
      skillInfo: {
        name: "arch",
        path: "/p",
        skillUri: vscode.Uri.file("/p/SKILL.md"),
        agentsUri: null,
        ruleUris: [],
      },
    });
    expect(item.collapsibleState).toBe(vscode.TreeItemCollapsibleState.Expanded);
    expect(item.skillInfo?.name).toBe("arch");
  });
});

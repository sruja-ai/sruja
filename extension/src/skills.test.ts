import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import * as vscode from "vscode";
import { ExtensionContext } from "./__mocks__/vscode";
import { getSkills, getSkillsRoot } from "./skills";

/** Context type compatible with getSkills/getSkillsRoot (mock has fewer props than real vscode.ExtensionContext). */
function asContext(ctx: ExtensionContext): vscode.ExtensionContext {
  return ctx as unknown as vscode.ExtensionContext;
}

describe("skills", () => {
  let tmpDir: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "sruja-skills-test-"));
    (vscode.workspace as { workspaceFolders?: { uri: vscode.Uri }[] }).workspaceFolders = [
      { uri: vscode.Uri.file(tmpDir) },
    ];
  });

  afterEach(() => {
    try {
      fs.rmSync(tmpDir, { recursive: true });
    } catch {
      // ignore
    }
  });

  it("getSkillsRoot returns first workspace skills root", () => {
    const skillsDir = path.join(tmpDir, "skills");
    fs.mkdirSync(skillsDir, { recursive: true });
    const context = new ExtensionContext();
    context.extensionUri = vscode.Uri.file("/nonexistent");
    const root = getSkillsRoot(asContext(context));
    expect(root).not.toBeNull();
    expect(root!.fsPath).toBe(skillsDir);
  });

  it("getSkillsRoot returns null when no skills directory exists", () => {
    const context = new ExtensionContext();
    const root = getSkillsRoot(asContext(context));
    expect(root).toBeNull();
  });

  it("getSkills returns empty when no skills dir", () => {
    const context = new ExtensionContext();
    expect(getSkills(asContext(context))).toEqual([]);
  });

  it("getSkills discovers skill with SKILL.md", () => {
    const skillsDir = path.join(tmpDir, "skills");
    const skillDir = path.join(skillsDir, "my-skill");
    fs.mkdirSync(skillDir, { recursive: true });
    fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# My Skill", "utf8");

    const context = new ExtensionContext();
    const skills = getSkills(asContext(context));
    expect(skills).toHaveLength(1);
    expect(skills[0].name).toBe("my-skill");
    expect(skills[0].skillUri.fsPath).toContain("SKILL.md");
    expect(skills[0].agentsUri).toBeNull();
    expect(skills[0].ruleUris).toEqual([]);
  });

  it("getSkills includes AGENTS.md and rules when present", () => {
    const skillsDir = path.join(tmpDir, "skills");
    const skillDir = path.join(skillsDir, "arch");
    const rulesDir = path.join(skillDir, "rules");
    fs.mkdirSync(rulesDir, { recursive: true });
    fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# Arch", "utf8");
    fs.writeFileSync(path.join(skillDir, "AGENTS.md"), "# Agents", "utf8");
    fs.writeFileSync(path.join(rulesDir, "c4.md"), "# C4", "utf8");

    const context = new ExtensionContext();
    const skills = getSkills(asContext(context));
    expect(skills).toHaveLength(1);
    expect(skills[0].agentsUri).not.toBeNull();
    expect(skills[0].agentsUri!.fsPath).toContain("AGENTS.md");
    expect(skills[0].ruleUris).toHaveLength(1);
    expect(skills[0].ruleUris[0].label).toBe("c4");
  });

  it("getSkills skips directories without SKILL.md", () => {
    const skillsDir = path.join(tmpDir, "skills");
    fs.mkdirSync(path.join(skillsDir, "no-skill"), { recursive: true });
    const context = new ExtensionContext();
    expect(getSkills(asContext(context))).toEqual([]);
  });

  it("getSkillsRoot uses sruja.skills.path when set", () => {
    const customSkillsRoot = path.join(tmpDir, "custom-skills");
    const skillDir = path.join(customSkillsRoot, "custom-skill");
    fs.mkdirSync(skillDir, { recursive: true });
    fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# Custom", "utf8");
    const origGetConfig = (vscode.workspace as { getConfiguration?: (s: string) => { get: (k: string) => string } }).getConfiguration;
    (vscode.workspace as { getConfiguration: (s: string) => { get: (k: string) => string } }).getConfiguration = () => ({
      get: (k: string) => (k === "skills.path" ? customSkillsRoot : ""),
    });
    try {
      const context = new ExtensionContext();
      context.extensionUri = vscode.Uri.file("/nonexistent");
      const root = getSkillsRoot(asContext(context));
      expect(root?.fsPath).toBe(customSkillsRoot);
      const skills = getSkills(asContext(context));
      expect(skills).toHaveLength(1);
      expect(skills[0].name).toBe("custom-skill");
    } finally {
      (vscode.workspace as { getConfiguration: typeof origGetConfig }).getConfiguration = origGetConfig!;
    }
  });

  it("getSkillsRoot falls back to workspace when custom path does not exist", () => {
    const skillsDir = path.join(tmpDir, "skills");
    const skillDir = path.join(skillsDir, "fallback-skill");
    fs.mkdirSync(skillDir, { recursive: true });
    fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# Fallback", "utf8");
    const origGetConfig = (vscode.workspace as { getConfiguration?: (s: string) => { get: (k: string) => string } }).getConfiguration;
    (vscode.workspace as { getConfiguration: (s: string) => { get: (k: string) => string } }).getConfiguration = () => ({
      get: (k: string) => (k === "skills.path" ? "/nonexistent/custom/skills" : ""),
    });
    try {
      const context = new ExtensionContext();
      context.extensionUri = vscode.Uri.file("/nonexistent");
      const root = getSkillsRoot(asContext(context));
      expect(root?.fsPath).toBe(skillsDir);
      const skills = getSkills(asContext(context));
      expect(skills).toHaveLength(1);
      expect(skills[0].name).toBe("fallback-skill");
    } finally {
      (vscode.workspace as { getConfiguration: typeof origGetConfig }).getConfiguration = origGetConfig!;
    }
  });

  it("getSkillsRoot uses extension skills when no workspace folder has skills", () => {
    const extDir = path.join(tmpDir, "ext");
    const extSkillsDir = path.join(extDir, "skills");
    const skillDir = path.join(extSkillsDir, "ext-skill");
    fs.mkdirSync(skillDir, { recursive: true });
    fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# Ext", "utf8");
    (vscode.workspace as { workspaceFolders?: { uri: vscode.Uri }[] }).workspaceFolders = [
      { uri: vscode.Uri.file(tmpDir) },
    ];
    const context = new ExtensionContext();
    context.extensionUri = vscode.Uri.file(extDir);
    const root = getSkillsRoot(asContext(context));
    expect(root?.fsPath).toBe(extSkillsDir);
    const skills = getSkills(asContext(context));
    expect(skills).toHaveLength(1);
    expect(skills[0].name).toBe("ext-skill");
  });

  it("getSkills deduplicates when same root appears twice in workspace folders", () => {
    const skillsDir = path.join(tmpDir, "skills");
    const skillDir = path.join(skillsDir, "dup");
    fs.mkdirSync(skillDir, { recursive: true });
    fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# Dup", "utf8");
    const uri = vscode.Uri.file(tmpDir);
    (vscode.workspace as { workspaceFolders?: { uri: vscode.Uri }[] }).workspaceFolders = [
      { uri },
      { uri },
    ];
    const context = new ExtensionContext();
    context.extensionUri = vscode.Uri.file("/nonexistent");
    const skills = getSkills(asContext(context));
    expect(skills).toHaveLength(1);
    expect(skills[0].name).toBe("dup");
  });

  it("getSkillsRoot uses file: URI when sruja.skills.path is file: URI", () => {
    const customSkillsRoot = path.join(tmpDir, "file-uri-skills");
    const skillDir = path.join(customSkillsRoot, "uri-skill");
    fs.mkdirSync(skillDir, { recursive: true });
    fs.writeFileSync(path.join(skillDir, "SKILL.md"), "# URI Skill", "utf8");
    const fileUriString = vscode.Uri.file(customSkillsRoot).toString();
    const origGetConfig = (vscode.workspace as { getConfiguration?: (s: string) => { get: (k: string) => string } }).getConfiguration;
    (vscode.workspace as { getConfiguration: (s: string) => { get: (k: string) => string } }).getConfiguration = () => ({
      get: (k: string) => (k === "skills.path" ? fileUriString : ""),
    });
    try {
      const context = new ExtensionContext();
      context.extensionUri = vscode.Uri.file("/nonexistent");
      const root = getSkillsRoot(asContext(context));
      expect(root?.fsPath).toBe(customSkillsRoot);
      const skills = getSkills(asContext(context));
      expect(skills).toHaveLength(1);
      expect(skills[0].name).toBe("uri-skill");
    } finally {
      (vscode.workspace as { getConfiguration: typeof origGetConfig }).getConfiguration = origGetConfig!;
    }
  });
});

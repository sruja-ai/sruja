import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";

export interface SkillInfo {
  name: string;
  path: string;
  skillUri: vscode.Uri;
  agentsUri: vscode.Uri | null;
  ruleUris: { label: string; uri: vscode.Uri }[];
}

async function pathExists(fsPath: string): Promise<boolean> {
  try {
    await fs.promises.access(fsPath);
    return true;
  } catch {
    return false;
  }
}

async function isDirectory(fsPath: string): Promise<boolean> {
  try {
    const stat = await fs.promises.stat(fsPath);
    return stat.isDirectory();
  } catch {
    return false;
  }
}

async function getSkillsRoots(context: vscode.ExtensionContext): Promise<vscode.Uri[]> {
  const config = vscode.workspace.getConfiguration("sruja");
  const custom = config.get<string>("skills.path")?.trim();
  if (custom) {
    const uri = custom.startsWith("file:") ? vscode.Uri.parse(custom) : vscode.Uri.file(custom);
    if (await pathExists(uri.fsPath)) return [uri];
  }

  const roots: vscode.Uri[] = [];
  const folders = vscode.workspace.workspaceFolders ?? [];

  for (const folder of folders) {
    const wsSkills = vscode.Uri.joinPath(folder.uri, "skills");
    if (await pathExists(wsSkills.fsPath)) roots.push(wsSkills);
  }

  if (roots.length > 0) return roots;

  const extSkills = vscode.Uri.joinPath(context.extensionUri, "skills");
  if (await pathExists(extSkills.fsPath)) return [extSkills];

  return [];
}

export async function getSkillsRoot(context: vscode.ExtensionContext): Promise<vscode.Uri | null> {
  const roots = await getSkillsRoots(context);
  return roots[0] ?? null;
}

export async function getSkills(context: vscode.ExtensionContext): Promise<SkillInfo[]> {
  const roots = await getSkillsRoots(context);
  if (roots.length === 0) return [];

  const seen = new Set<string>();
  const skills: SkillInfo[] = [];

  for (const root of roots) {
    const entries = await fs.promises.readdir(root.fsPath, { withFileTypes: true });
    const names = entries.filter((d) => d.isDirectory() && !d.name.startsWith(".")).map((d) => d.name);

    for (const name of names) {
      const skillDir = path.join(root.fsPath, name);
      const skillMd = path.join(skillDir, "SKILL.md");
      const agentsMd = path.join(skillDir, "AGENTS.md");
      const rulesDir = path.join(skillDir, "rules");

      if (!(await pathExists(skillMd))) continue;

      const key = `${root.fsPath}:${name}`;
      if (seen.has(key)) continue;
      seen.add(key);

      const skillUri = vscode.Uri.file(skillMd);
      const agentsUri: vscode.Uri | null = (await pathExists(agentsMd)) ? vscode.Uri.file(agentsMd) : null;
      const ruleUris: { label: string; uri: vscode.Uri }[] = [];

      if (await isDirectory(rulesDir)) {
        const ruleFiles = (await fs.promises.readdir(rulesDir))
          .filter((f) => f.endsWith(".md"))
          .sort();

        for (const f of ruleFiles) {
          const label = f.replace(/\.md$/, "");
          ruleUris.push({ label, uri: vscode.Uri.file(path.join(rulesDir, f)) });
        }
      }

      skills.push({ name, path: skillDir, skillUri, agentsUri, ruleUris });
    }
  }

  return skills.sort((a, b) => a.name.localeCompare(b.name));
}

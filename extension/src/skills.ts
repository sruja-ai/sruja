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

/**
 * Resolve skills roots for multi-root workspaces.
 * Order: sruja.skills.path (single) > each workspace folder's "skills" > extension "skills".
 */
function getSkillsRoots(context: vscode.ExtensionContext): vscode.Uri[] {
  const config = vscode.workspace.getConfiguration("sruja");
  const custom = config.get<string>("skills.path")?.trim();
  if (custom) {
    try {
      const uri = custom.startsWith("file:")
        ? vscode.Uri.parse(custom)
        : vscode.Uri.file(custom);
      if (fs.existsSync(uri.fsPath)) return [uri];
    } catch {
      // Fall through to workspace/extension roots
    }
  }
  const roots: vscode.Uri[] = [];
  const folders = vscode.workspace.workspaceFolders ?? [];
  for (const folder of folders) {
    try {
      const wsSkills = vscode.Uri.joinPath(folder.uri, "skills");
      if (fs.existsSync(wsSkills.fsPath)) roots.push(wsSkills);
    } catch {
      continue;
    }
  }
  if (roots.length > 0) return roots;
  try {
    const extSkills = vscode.Uri.joinPath(context.extensionUri, "skills");
    if (fs.existsSync(extSkills.fsPath)) return [extSkills];
  } catch {
    // Ignore
  }
  return [];
}

/** First skills root, if any (for commands that need a single root). */
export function getSkillsRoot(context: vscode.ExtensionContext): vscode.Uri | null {
  const roots = getSkillsRoots(context);
  return roots[0] ?? null;
}

export function getSkills(context: vscode.ExtensionContext): SkillInfo[] {
  const roots = getSkillsRoots(context);
  if (roots.length === 0) return [];

  const seen = new Set<string>();
  const skills: SkillInfo[] = [];

  for (const root of roots) {
    let names: string[];
    try {
      names = fs.readdirSync(root.fsPath, { withFileTypes: true })
        .filter((d: fs.Dirent) => d.isDirectory() && !d.name.startsWith("."))
        .map((d: fs.Dirent) => d.name);
    } catch {
      continue;
    }

    for (const name of names) {
      try {
        const skillDir = path.join(root.fsPath, name);
        const skillMd = path.join(skillDir, "SKILL.md");
        const agentsMd = path.join(skillDir, "AGENTS.md");
        const rulesDir = path.join(skillDir, "rules");

        if (!fs.existsSync(skillMd)) continue;
        const key = `${root.fsPath}:${name}`;
        if (seen.has(key)) continue;
        seen.add(key);

        const skillUri = vscode.Uri.file(skillMd);
        let agentsUri: vscode.Uri | null = fs.existsSync(agentsMd)
          ? vscode.Uri.file(agentsMd)
          : null;
        const ruleUris: { label: string; uri: vscode.Uri }[] = [];

        if (fs.existsSync(rulesDir) && fs.statSync(rulesDir).isDirectory()) {
          const ruleFiles = fs.readdirSync(rulesDir)
            .filter((f: string) => f.endsWith(".md"))
            .sort();
          for (const f of ruleFiles) {
            const label = f.replace(/\.md$/, "");
            ruleUris.push({ label, uri: vscode.Uri.file(path.join(rulesDir, f)) });
          }
        }

        skills.push({
          name,
          path: skillDir,
          skillUri,
          agentsUri,
          ruleUris,
        });
      } catch {
        continue;
      }
    }
  }

  return skills.sort((a, b) => a.name.localeCompare(b.name));
}

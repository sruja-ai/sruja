import * as vscode from "vscode";

import { getSkills, getSkillsRoot, SkillInfo } from "../skills";
import { pickSingleOrMany } from "../utils";

export function registerSkillsCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.openSkillsOverview", async () => {
      const root = await getSkillsRoot(context);
      if (!root) {
        vscode.window.showWarningMessage(
          "No skills root. Set sruja.skills.path or open a workspace with a skills folder."
        );
        return;
      }

      const skills = await getSkills(context);
      if (skills.length === 0) {
        vscode.window.showWarningMessage("No skills found in the skills root.");
        return;
      }

      const skill = await pickSingleOrMany(
        skills,
        (s) => ({ label: s.name, item: s }),
        "Select a skill"
      );

      if (skill) await vscode.window.showTextDocument(skill.skillUri);
    }),

    vscode.commands.registerCommand("sruja.openAgentGuide", async () => {
      const skills = await getSkills(context);
      const withAgents = skills.filter((s) => s.agentsUri != null);

      if (withAgents.length === 0) {
        vscode.window.showWarningMessage(
          "No AGENTS.md found. Set sruja.skills.path or open a workspace with skills."
        );
        return;
      }

      const skill = await pickSingleOrMany(
        withAgents,
        (s) => ({ label: s.name, item: s }),
        "Select skill"
      );

      if (skill?.agentsUri) await vscode.window.showTextDocument(skill.agentsUri);
    }),

    vscode.commands.registerCommand("sruja.listRules", async () => {
      const skills = await getSkills(context);
      const allRules: { label: string; description: string; uri: vscode.Uri }[] = [];

      for (const s of skills) {
        for (const r of s.ruleUris) {
          allRules.push({ label: r.label, description: s.name, uri: r.uri });
        }
      }

      if (allRules.length === 0) {
        vscode.window.showWarningMessage(
          "No rules found. Set sruja.skills.path or open a workspace with a skills folder."
        );
        return;
      }

      const pick = await vscode.window.showQuickPick(allRules, {
        placeHolder: "Open a rule",
        matchOnDescription: true,
      });

      if (pick) await vscode.window.showTextDocument(pick.uri);
    }),

    vscode.commands.registerCommand("sruja.copyRuleForAI", async () => {
      const skills = await getSkills(context);
      const allRules: { label: string; uri: vscode.Uri }[] = [];

      for (const s of skills) {
        for (const r of s.ruleUris) {
          allRules.push({ label: `${s.name} / ${r.label}`, uri: r.uri });
        }
      }

      if (allRules.length === 0) {
        vscode.window.showWarningMessage("No rules found.");
        return;
      }

      const pick = await vscode.window.showQuickPick(allRules, {
        placeHolder: "Copy which rule for AI?",
      });

      if (!pick) return;

      try {
        const content = await vscode.workspace.fs.readFile(pick.uri);
        const text = Buffer.from(content).toString("utf8");
        await vscode.env.clipboard.writeText(text);
        vscode.window.showInformationMessage(`Copied "${pick.label}" to clipboard.`);
      } catch (e) {
        vscode.window.showErrorMessage(
          `Failed to read rule: ${e instanceof Error ? e.message : String(e)}`
        );
      }
    }),

    vscode.commands.registerCommand("sruja.copyAgentGuideForAI", async () => {
      const skills = await getSkills(context);
      const withAgents = skills.filter((s) => s.agentsUri != null);

      if (withAgents.length === 0) {
        vscode.window.showWarningMessage("No AGENTS.md found.");
        return;
      }

      const skill = await pickSingleOrMany(
        withAgents,
        (s) => ({ label: s.name, item: s }),
        "Copy which agent guide?"
      );

      if (!skill?.agentsUri) return;

      try {
        const content = await vscode.workspace.fs.readFile(skill.agentsUri);
        const text = Buffer.from(content).toString("utf8");
        await vscode.env.clipboard.writeText(text);
        vscode.window.showInformationMessage(`Copied "${skill.name}" agent guide to clipboard.`);
      } catch (e) {
        vscode.window.showErrorMessage(
          `Failed to read agent guide: ${e instanceof Error ? e.message : String(e)}`
        );
      }
    })
  );
}

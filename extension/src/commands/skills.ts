import * as vscode from "vscode";

import { getSkills, getSkillsRoot } from "../skills";

export function registerSkillsCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.openSkillsOverview", async () => {
      const root = getSkillsRoot(context);
      if (!root) {
        vscode.window.showWarningMessage(
          "No skills root. Set sruja.skills.path or open a workspace with a skills folder."
        );
        return;
      }
      const skills = getSkills(context);
      if (skills.length === 0) {
        vscode.window.showWarningMessage("No skills found in the skills root.");
        return;
      }
      const skill =
        skills.length === 1
          ? skills[0]
          : await vscode.window
              .showQuickPick(
                skills.map((s) => ({ label: s.name, skill: s })),
                { placeHolder: "Select a skill" }
              )
              .then((p) => p?.skill);
      if (skill) await vscode.window.showTextDocument(skill.skillUri);
    }),
    vscode.commands.registerCommand("sruja.openAgentGuide", async () => {
      const skills = getSkills(context);
      const withAgents = skills.filter((s) => s.agentsUri != null);
      if (withAgents.length === 0) {
        vscode.window.showWarningMessage(
          "No AGENTS.md found. Set sruja.skills.path or open a workspace with skills."
        );
        return;
      }
      const skill =
        withAgents.length === 1
          ? withAgents[0]
          : await vscode.window
              .showQuickPick(
                withAgents.map((s) => ({ label: s.name, skill: s })),
                { placeHolder: "Select skill" }
              )
              .then((p) => p?.skill);
      if (skill?.agentsUri) await vscode.window.showTextDocument(skill.agentsUri);
    }),
    vscode.commands.registerCommand("sruja.listRules", async () => {
      const skills = getSkills(context);
      const allRules: { label: string; uri: vscode.Uri; skillName: string }[] = [];
      for (const s of skills) {
        for (const r of s.ruleUris) {
          allRules.push({ label: r.label, uri: r.uri, skillName: s.name });
        }
      }
      if (allRules.length === 0) {
        vscode.window.showWarningMessage(
          "No rules found. Set sruja.skills.path or open a workspace with a skills folder."
        );
        return;
      }
      const pick = await vscode.window.showQuickPick(
        allRules.map((r) => ({ label: r.label, description: r.skillName, rule: r })),
        { placeHolder: "Open a rule", matchOnDescription: true }
      );
      if (pick) await vscode.window.showTextDocument(pick.rule.uri);
    }),
    vscode.commands.registerCommand("sruja.copyRuleForAI", async () => {
      const skills = getSkills(context);
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
      const pick = await vscode.window.showQuickPick(
        allRules.map((r) => ({ label: r.label, rule: r })),
        { placeHolder: "Copy which rule for AI?" }
      );
      if (!pick) return;
      try {
        const content = await vscode.workspace.fs.readFile(pick.rule.uri);
        const text = Buffer.from(content).toString("utf8");
        await vscode.env.clipboard.writeText(text);
        vscode.window.showInformationMessage(`Copied "${pick.rule.label}" to clipboard.`);
      } catch (e) {
        vscode.window.showErrorMessage(
          `Failed to read rule: ${e instanceof Error ? e.message : String(e)}`
        );
      }
    }),
    vscode.commands.registerCommand("sruja.copyAgentGuideForAI", async () => {
      const skills = getSkills(context);
      const withAgents = skills.filter((s) => s.agentsUri != null);
      if (withAgents.length === 0) {
        vscode.window.showWarningMessage("No AGENTS.md found.");
        return;
      }
      const skill =
        withAgents.length === 1
          ? withAgents[0]
          : await vscode.window
              .showQuickPick(
                withAgents.map((s) => ({ label: s.name, skill: s })),
                { placeHolder: "Copy which agent guide?" }
              )
              .then((p) => p?.skill);
      if (!skill?.agentsUri) return;
      try {
        const content = await vscode.workspace.fs.readFile(skill.agentsUri);
        const text = Buffer.from(content).toString("utf8");
        await vscode.env.clipboard.writeText(text);
        vscode.window.showInformationMessage(
          `Copied "${skill.name}" agent guide to clipboard.`
        );
      } catch (e) {
        vscode.window.showErrorMessage(
          `Failed to read agent guide: ${e instanceof Error ? e.message : String(e)}`
        );
      }
    })
  );
}

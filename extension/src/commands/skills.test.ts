import * as vscode from "vscode";
import { ExtensionContext } from "../__mocks__/vscode";

jest.mock("../skills", () => {
  return {
    getSkillsRoot: jest.fn(),
    getSkills: jest.fn(),
  };
});

import { registerSkillCommands } from "./skills";

describe("registerSkillCommands", () => {
  let registered: Map<string, () => Promise<void> | void>;

  beforeEach(() => {
    registered = new Map<string, () => Promise<void> | void>();
    (vscode.commands as any).registerCommand = jest.fn((id: string, cb: () => Promise<void> | void) => {
      registered.set(id, cb);
      return { dispose: () => {} };
    });

    (vscode.window as any).showWarningMessage = jest.fn();
    (vscode.window as any).showInformationMessage = jest.fn();
    (vscode.window as any).showErrorMessage = jest.fn();
    (vscode.window as any).showTextDocument = jest.fn();

    (vscode.env as any).clipboard = { writeText: jest.fn().mockResolvedValue(undefined) };
    (vscode.workspace as any).fs = {
      readFile: jest.fn().mockResolvedValue(Buffer.from("rule text", "utf8")),
      writeFile: jest.fn(),
      stat: jest.fn(),
      createDirectory: jest.fn(),
    };
  });

  it("openSkillsOverview warns when no skills root", async () => {
    const { getSkillsRoot, getSkills } = await import("../skills");
    (getSkillsRoot as jest.Mock).mockReturnValue(null);
    (getSkills as jest.Mock).mockReturnValue([]);

    const ctx = new ExtensionContext();
    registerSkillCommands(ctx as any, true);

    const cb = registered.get("sruja.openSkillsOverview");
    if (!cb) throw new Error("Command not registered: sruja.openSkillsOverview");
    await cb();

    expect(vscode.window.showWarningMessage).toHaveBeenCalledWith(
      "No skills root. Set sruja.skills.path or open a workspace with a skills folder."
    );
  });

  it("openSkillsOverview opens the only skill when present", async () => {
    const { getSkillsRoot, getSkills } = await import("../skills");
    (getSkillsRoot as jest.Mock).mockReturnValue(vscode.Uri.file("/ws/skills"));
    (getSkills as jest.Mock).mockReturnValue([
      {
        name: "one",
        path: "/ws/skills/one",
        skillUri: vscode.Uri.file("/ws/skills/one/SKILL.md"),
        agentsUri: null,
        ruleUris: [],
      },
    ]);

    const ctx = new ExtensionContext();
    registerSkillCommands(ctx as any, true);

    const cb = registered.get("sruja.openSkillsOverview");
    if (!cb) throw new Error("Command not registered: sruja.openSkillsOverview");
    await cb();

    expect(vscode.window.showTextDocument).toHaveBeenCalledWith(vscode.Uri.file("/ws/skills/one/SKILL.md"));
  });

  it("copyRuleForAI reads rule file and writes to clipboard", async () => {
    const { getSkillsRoot, getSkills } = await import("../skills");
    (getSkillsRoot as jest.Mock).mockReturnValue(vscode.Uri.file("/ws/skills"));
    (getSkills as jest.Mock).mockReturnValue([
      {
        name: "one",
        path: "/ws/skills/one",
        skillUri: vscode.Uri.file("/ws/skills/one/SKILL.md"),
        agentsUri: null,
        ruleUris: [{ label: "r1", uri: vscode.Uri.file("/ws/skills/one/rules/r1.md") }],
      },
    ]);

    const ctx = new ExtensionContext();
    registerSkillCommands(ctx as any, true);

    const cb = registered.get("sruja.copyRuleForAI");
    if (!cb) throw new Error("Command not registered: sruja.copyRuleForAI");
    await cb();

    expect((vscode.env as any).clipboard.writeText).toHaveBeenCalledWith("rule text");
    expect(vscode.window.showInformationMessage).toHaveBeenCalledWith('Copied "one / r1" to clipboard.');
  });
});


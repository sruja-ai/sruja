import * as vscode from "vscode";
import { ExtensionContext } from "./__mocks__/vscode";

jest.mock("./skills", () => {
  return {
    getSkillsRoot: jest.fn(),
    getSkills: jest.fn(),
  };
});

jest.mock("./wasm", () => {
  const actual = jest.requireActual<typeof import("./wasm")>("./wasm");
  return {
    ...actual,
    getElementsFromWasm: jest.fn(),
    getMermaidFromWasm: jest.fn(),
  };
});

import { buildContextPack } from "./contextPack";

function asContext(ctx: ExtensionContext): vscode.ExtensionContext {
  return ctx as unknown as vscode.ExtensionContext;
}

describe("buildContextPack", () => {
  beforeEach(() => {
    (vscode.workspace as any).workspaceFolders = [];
    (vscode.workspace as any).textDocuments = [];
    (vscode.window as any).activeTextEditor = undefined;
    (vscode.window as any).visibleTextEditors = [];
    (vscode.languages as any).getDiagnostics = () => [];
    (vscode.workspace as any).fs = {
      stat: jest.fn().mockResolvedValue({ size: 0, mtime: 0 }),
      readFile: jest.fn().mockResolvedValue(Buffer.from("", "utf8")),
      writeFile: jest.fn(),
      createDirectory: jest.fn(),
    };
    (vscode.workspace as any).getWorkspaceFolder = (uri: vscode.Uri) => {
      if (uri.fsPath.startsWith("/ws")) {
        const folders = (vscode.workspace as any).workspaceFolders;
        return folders && folders.length > 0 ? folders[0] : undefined;
      }
      return undefined;
    };
  });

  it("renders basic pack when no workspace and no sruja open", async () => {
    const { getSkillsRoot, getSkills } = await import("./skills");
    (getSkillsRoot as jest.Mock).mockReturnValue(null);
    (getSkills as jest.Mock).mockReturnValue([]);

    const pack = await buildContextPack(asContext(new ExtensionContext()));
    expect(pack).toContain("# Sruja Context Pack");
    expect(pack).toContain("- workspace=none");
    expect(pack).toContain("- activeSrujaFile=none");
    expect(pack).toContain("- elements=unavailable (no .sruja file open)");
    expect(pack).toContain("- mermaid=unavailable (no .sruja file open)");
    expect(pack).toContain("- skills root=none");
  });

  it("includes selection, diagnostics, skills, elements, and mermaid when available", async () => {
    const { getSkillsRoot, getSkills } = await import("./skills");
    const { getElementsFromWasm, getMermaidFromWasm } = await import("./wasm");

    (vscode.workspace as any).workspaceFolders = [{ uri: vscode.Uri.file("/ws"), name: "ws" }];

    const activeDoc = {
      uri: vscode.Uri.file("/ws/a.ts"),
      languageId: "typescript",
      version: 1,
      lineCount: 3,
      getText: (_range?: unknown) => "const x = 1;\n",
      lineAt: (i: number) => ({ text: ["l1", "l2", "l3"][i] ?? "" }),
    } as unknown as vscode.TextDocument;

    const activeEditor = {
      document: activeDoc,
      selection: {
        isEmpty: false,
        active: { line: 0 },
      },
    };
    activeDoc.getText = (range?: unknown) => (range ? "selected\n" : "const x = 1;\n");
    (vscode.window as any).activeTextEditor = activeEditor;

    const srujaDoc = {
      uri: vscode.Uri.file("/ws/arch.sruja"),
      languageId: "sruja",
      version: 2,
      getText: () => 'Payments = system "Payments" { description "x" }',
    } as unknown as vscode.TextDocument;

    (vscode.window as any).visibleTextEditors = [{ document: activeDoc }, { document: srujaDoc }];

    const diag1 = new vscode.Diagnostic(new vscode.Range(0, 0, 0, 1), "Bad", vscode.DiagnosticSeverity.Error);
    (diag1 as any).code = "SR001";
    const diag2 = new vscode.Diagnostic(new vscode.Range(1, 2, 1, 3), "Warn", vscode.DiagnosticSeverity.Warning);
    (diag2 as any).code = "SR002";

    (vscode.languages as any).getDiagnostics = (uri?: vscode.Uri) => {
      if (uri) return [diag1, diag2];
      return [[vscode.Uri.file("/ws/a.ts"), [diag1, diag2]]];
    };

    (vscode.workspace as any).fs.stat.mockResolvedValue({ size: 100, mtime: 123 });
    (vscode.workspace as any).fs.readFile.mockResolvedValue(Buffer.from('{"truth_status":"fresh"}', "utf8"));

    (getSkillsRoot as jest.Mock).mockReturnValue(vscode.Uri.file("/ws/skills"));
    (getSkills as jest.Mock).mockReturnValue([
      {
        name: "my-skill",
        path: "/ws/skills/my-skill",
        skillUri: vscode.Uri.file("/ws/skills/my-skill/SKILL.md"),
        agentsUri: vscode.Uri.file("/ws/skills/my-skill/AGENTS.md"),
        ruleUris: [{ label: "rule1", uri: vscode.Uri.file("/ws/skills/my-skill/rules/rule1.md") }],
      },
    ]);

    (getElementsFromWasm as jest.Mock).mockResolvedValue([
      { id: "Payments", kind: "system" },
      { id: "Payments.Api", kind: "component" },
      { id: "Payments.Db", kind: "database" },
    ]);

    const mermaidLines = Array.from({ length: 200 }, (_, i) => `line${i + 1}`).join("\n");
    (getMermaidFromWasm as jest.Mock).mockResolvedValue(mermaidLines);

    const pack = await buildContextPack(asContext(new ExtensionContext()));

    expect(pack).toContain("```typescript");
    expect(pack).toContain("selected");
    expect(pack).toContain("- activeFile=a.ts");
    expect(pack).toContain("- context.json");
    expect(pack).toContain("truth=fresh");
    expect(pack).toContain("- skills root=/ws/skills");
    expect(pack).toContain("- my-skill agents=yes rules=rule1");
    expect(pack).toContain("- elements=3");
    expect(pack).toContain("```mermaid");
    expect(pack).toContain("- mermaid omittedLines=");
    expect(pack).toContain("## Diagnostics");
    expect(pack).toContain("### Active File");
    expect(pack).toContain("### Workspace (Top Files)");
  });

  it("handles wasm failures gracefully", async () => {
    const { getSkillsRoot, getSkills } = await import("./skills");
    const { getElementsFromWasm, getMermaidFromWasm } = await import("./wasm");

    (vscode.workspace as any).workspaceFolders = [{ uri: vscode.Uri.file("/ws"), name: "ws" }];
    const srujaDoc = {
      uri: vscode.Uri.file("/ws/arch.sruja"),
      languageId: "sruja",
      version: 2,
      getText: () => "x",
    } as unknown as vscode.TextDocument;
    (vscode.window as any).activeTextEditor = { document: srujaDoc, selection: { isEmpty: true, active: { line: 0 } } };
    (vscode.window as any).visibleTextEditors = [{ document: srujaDoc }];

    (getSkillsRoot as jest.Mock).mockReturnValue(null);
    (getSkills as jest.Mock).mockReturnValue([]);
    (getElementsFromWasm as jest.Mock).mockRejectedValue(new Error("boom"));
    (getMermaidFromWasm as jest.Mock).mockRejectedValue(new Error("boom"));

    const pack = await buildContextPack(asContext(new ExtensionContext()));
    expect(pack).toContain("- elements=unavailable");
    expect(pack).toContain("- mermaid=unavailable");
  });
});


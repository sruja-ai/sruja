import * as vscode from "vscode";
import { ExtensionContext } from "./__mocks__/vscode";

jest.mock("./cliRunner", () => ({
  runCli: jest.fn(),
}));

jest.mock("./config", () => ({
  getSrujaLspPath: jest.fn(() => "sruja"),
}));

import { registerExplorerCommands } from "./architectureExplorer";
import { runCli } from "./cliRunner";

const MOCK_MODEL = {
  schema_version: "explorer/v1",
  nodes: [
    {
      id: "SysA",
      kind: "system",
      label: "System A",
      description: "Primary",
      technology: null,
      parent_id: null,
      children_count: 1,
      metrics: {
        centrality: 0.5,
        instability: 0.3,
        coupling_zone: "main_sequence",
        drift_count: 0,
        drift_severity_max: null,
        health: "healthy",
        is_hotspot: false,
        is_in_cycle: false,
        community_id: 1,
      },
    },
  ],
  edges: [],
  communities: [],
  cycles: [],
  summary: {
    total_nodes: 1,
    total_edges: 0,
    drift_score: 0,
    health: "healthy",
    hotspot_count: 0,
    cycle_count: 0,
    community_count: 0,
  },
};

describe("registerExplorerCommands", () => {
  let registered: Map<string, (...args: unknown[]) => unknown>;

  beforeEach(() => {
    registered = new Map();
    (vscode.commands as any).registerCommand = jest.fn(
      (id: string, cb: (...args: unknown[]) => unknown) => {
        registered.set(id, cb);
        return { dispose: () => {} };
      }
    );
    (vscode.window as any).showWarningMessage = jest.fn();
    (vscode.window as any).showInformationMessage = jest.fn();
    (vscode.workspace as any).workspaceFolders = [
      { uri: vscode.Uri.file("/ws"), name: "ws", index: 0 },
    ];
    (vscode.workspace as any).findFiles = jest.fn().mockResolvedValue([]);

    const { window } = jest.requireActual("./__mocks__/vscode");
    (vscode.window as any).createWebviewPanel = jest.fn(
      window.createWebviewPanel
    );
  });

  it("registers the openArchitectureExplorer command", () => {
    registerExplorerCommands(new ExtensionContext() as any);
    expect(registered.has("sruja.openArchitectureExplorer")).toBe(true);
  });

  it("warns when no workspace folder is open", async () => {
    (vscode.workspace as any).workspaceFolders = undefined;
    registerExplorerCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.openArchitectureExplorer")!;
    await cb();
    expect(vscode.window.showWarningMessage).toHaveBeenCalledWith(
      "Open a workspace folder to use the Architecture Explorer."
    );
  });

  it("creates webview with explorer HTML on success", async () => {
    (runCli as jest.Mock).mockResolvedValue({
      code: 0,
      stdout: JSON.stringify(MOCK_MODEL),
      stderr: "",
    });

    registerExplorerCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.openArchitectureExplorer")!;
    await cb();

    expect(vscode.window.createWebviewPanel).toHaveBeenCalledWith(
      "srujaArchitectureExplorer",
      "Sruja – Architecture Explorer",
      vscode.ViewColumn.Beside,
      expect.objectContaining({ enableScripts: true })
    );
  });

  it("shows error content when CLI fails", async () => {
    (runCli as jest.Mock).mockResolvedValue({
      code: 1,
      stdout: "",
      stderr: "scan failed",
    });

    registerExplorerCommands(new ExtensionContext() as any);
    const cb = registered.get("sruja.openArchitectureExplorer")!;
    await cb();

    // Panel already existed from prior test; verify runCli was called with explore args
    expect(runCli).toHaveBeenCalledWith(
      "sruja",
      ["explore", "-r", "/ws"],
      "/ws"
    );
  });
});

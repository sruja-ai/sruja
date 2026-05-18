import * as vscode from "vscode";
import { ExtensionContext } from "../__mocks__/vscode";

jest.mock("../cliRunner", () => {
  return {
    runCli: jest.fn(),
  };
});

describe("registerContextEngineeringCommands", () => {
  const channel = {
    clear: jest.fn(),
    show: jest.fn(),
    append: jest.fn(),
    appendLine: jest.fn(),
  };

  let registered: Map<string, (...args: any[]) => any>;

  beforeEach(() => {
    jest.clearAllMocks();
    (vscode.workspace as any).workspaceFolders = [];
    (vscode.window as any).activeTextEditor = undefined;
    (vscode.window as any).showWarningMessage = jest.fn();
    (vscode.window as any).showErrorMessage = jest.fn();
    (vscode.window as any).showInformationMessage = jest.fn();

    channel.clear.mockClear();
    channel.show.mockClear();
    channel.append.mockClear();
    channel.appendLine.mockClear();
    (vscode.window as any).createOutputChannel = jest.fn().mockReturnValue(channel);

    registered = new Map<string, (...args: any[]) => any>();
    (vscode.commands as any).registerCommand = jest.fn((id: string, cb: (...args: any[]) => any) => {
      registered.set(id, cb);
      return { dispose: () => {} };
    });
  });

  it("refreshContext warns when no workspace folder", async () => {
    const { registerContextEngineeringCommands } = await import("./contextEngineering");
    const ctx = new ExtensionContext();
    registerContextEngineeringCommands(ctx as any, () => "/bin/sruja");

    const cb = registered.get("sruja.refreshContext");
    if (!cb) throw new Error("Command not registered: sruja.refreshContext");
    await cb();
    expect(vscode.window.showWarningMessage).toHaveBeenCalledWith("Select a workspace folder to refresh repo context.");
  });

  it("refreshArchitectureState runs drift-state CLI", async () => {
    const { runCli } = await import("../cliRunner");
    (runCli as jest.Mock).mockResolvedValue({
      stdout: JSON.stringify({
        schema_version: "drift_state/v1",
        truth_status: "Unknown",
        health_score: 90,
        violation_count: 0,
        violations: [],
      }),
      stderr: "",
      code: 0,
    });

    const folder = { uri: vscode.Uri.file("/ws"), name: "ws" };
    (vscode.workspace as any).workspaceFolders = [folder];
    (vscode.env as any).clipboard = { writeText: jest.fn().mockResolvedValue(undefined) };

    const { registerContextEngineeringCommands } = await import("./contextEngineering");
    registerContextEngineeringCommands(new ExtensionContext() as any, () => "/bin/sruja");

    const cb = registered.get("sruja.refreshArchitectureState");
    if (!cb) throw new Error("Command not registered: sruja.refreshArchitectureState");
    await cb();

    expect(runCli).toHaveBeenCalledWith(
      "/bin/sruja",
      ["drift", "-r", ".", "-f", "drift-state"],
      "/ws"
    );
    expect(vscode.env.clipboard.writeText).toHaveBeenCalled();
  });

  it("runDrift shows error when no workspace folder", async () => {
    const { registerContextEngineeringCommands } = await import("./contextEngineering");
    const ctx = new ExtensionContext();
    registerContextEngineeringCommands(ctx as any, () => "/bin/sruja");

    const cb = registered.get("sruja.runDrift");
    if (!cb) throw new Error("Command not registered: sruja.runDrift");
    await cb();
    expect(vscode.window.showErrorMessage).toHaveBeenCalledWith(
      "Sruja drift failed. Ensure the Sruja CLI is installed and on PATH, or set sruja.lsp.path."
    );
  });

  it("refreshContext runs sync once in the active workspace folder", async () => {
    const { runCli } = await import("../cliRunner");
    (runCli as jest.Mock).mockResolvedValue({
      stdout: JSON.stringify({ context_path: "/ws-b/.sruja/context.json", truth_status: "fresh" }),
      stderr: "",
      code: 0,
    });

    const folderA = { uri: vscode.Uri.file("/ws-a"), name: "ws-a" };
    const folderB = { uri: vscode.Uri.file("/ws-b"), name: "ws-b" };
    (vscode.workspace as any).workspaceFolders = [folderA, folderB];
    (vscode.workspace as any).getWorkspaceFolder = (uri: vscode.Uri) => {
      if (uri.fsPath.startsWith("/ws-b")) return folderB;
      if (uri.fsPath.startsWith("/ws-a")) return folderA;
      return undefined;
    };
    (vscode.window as any).activeTextEditor = {
      document: { uri: vscode.Uri.file("/ws-b/src/main.ts") },
    };

    const { registerContextEngineeringCommands } = await import("./contextEngineering");
    const ctx = new ExtensionContext();
    registerContextEngineeringCommands(ctx as any, () => "/bin/sruja");

    const cb = registered.get("sruja.refreshContext");
    if (!cb) throw new Error("Command not registered: sruja.refreshContext");
    await cb();

    expect(runCli).toHaveBeenCalledTimes(1);
    expect(runCli).toHaveBeenCalledWith("/bin/sruja", ["sync", "-r", ".", "-f", "json"], "/ws-b");
    expect(channel.appendLine).toHaveBeenCalledWith("Context written to /ws-b/.sruja/context.json");
    expect(channel.appendLine).toHaveBeenCalledWith("Baseline/truth: fresh");
    expect(vscode.window.showInformationMessage).toHaveBeenCalledWith("Sruja: Repo context updated.");
  });

  it("status prints formatted lines when CLI returns valid JSON", async () => {
    const { runCli } = await import("../cliRunner");
    (runCli as jest.Mock).mockResolvedValue({
      stdout: JSON.stringify({ truth_status: "fresh", violations_count: 1 }),
      stderr: "",
      code: 0,
    });

    (vscode.workspace as any).workspaceFolders = [{ uri: vscode.Uri.file("/ws"), name: "ws" }];

    const { registerContextEngineeringCommands } = await import("./contextEngineering");
    const ctx = new ExtensionContext();
    registerContextEngineeringCommands(ctx as any, () => "/bin/sruja");

    const cb = registered.get("sruja.status");
    if (!cb) throw new Error("Command not registered: sruja.status");
    await cb();

    expect(channel.appendLine).toHaveBeenCalledWith("Baseline: (none)");
    expect(channel.appendLine).toHaveBeenCalledWith(expect.stringContaining("Truth: fresh (1 violation(s))"));
  });

  it("review prints parse error when CLI returns invalid JSON", async () => {
    const { runCli } = await import("../cliRunner");
    (runCli as jest.Mock).mockResolvedValue({
      stdout: "not-json",
      stderr: "",
      code: 0,
    });

    (vscode.workspace as any).workspaceFolders = [{ uri: vscode.Uri.file("/ws"), name: "ws" }];

    const { registerContextEngineeringCommands } = await import("./contextEngineering");
    const ctx = new ExtensionContext();
    registerContextEngineeringCommands(ctx as any, () => "/bin/sruja");

    const cb = registered.get("sruja.review");
    if (!cb) throw new Error("Command not registered: sruja.review");
    await cb();

    expect(channel.appendLine).toHaveBeenCalledWith(expect.stringContaining("Parse error:"));
  });
});

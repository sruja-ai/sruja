import * as vscode from "vscode";
import { registerMcpServer } from "./mcpServer";

describe("registerMcpServer", () => {
  beforeEach(() => {
    (vscode.workspace as any).workspaceFolders = [];
    (vscode.workspace as any).fs = {
      createDirectory: jest.fn(),
      readFile: jest.fn(),
      writeFile: jest.fn(),
    };
    (vscode.window as any).showWarningMessage = jest.fn();
    (vscode.window as any).showInformationMessage = jest.fn();
    (vscode.window as any).showErrorMessage = jest.fn();
    delete (vscode as any).cursor;
  });

  it("warns when no workspace folder is open", async () => {
    await registerMcpServer(() => "/bin/sruja");
    expect(vscode.window.showWarningMessage).toHaveBeenCalledWith(
      "Open a workspace folder to register the MCP server."
    );
  });

  it("registers via Cursor MCP API when available", async () => {
    (vscode.workspace as any).workspaceFolders = [{ uri: vscode.Uri.file("/ws"), name: "ws" }];
    const registerServer = jest.fn().mockResolvedValue(undefined);
    (vscode as any).cursor = { mcp: { registerServer } };

    await registerMcpServer(() => "/bin/sruja");

    expect(registerServer).toHaveBeenCalledWith({
      name: "sruja",
      type: "stdio",
      command: "/bin/sruja",
      args: ["mcp", "--root", "/ws"],
    });
    expect(vscode.window.showInformationMessage).toHaveBeenCalledWith("Sruja MCP server registered in Cursor.");
    expect((vscode.workspace as any).fs.writeFile).not.toHaveBeenCalled();
  });

  it("falls back to writing .cursor/mcp.json when Cursor MCP API fails", async () => {
    (vscode.workspace as any).workspaceFolders = [{ uri: vscode.Uri.file("/ws"), name: "ws" }];
    const registerServer = jest.fn().mockRejectedValue(new Error("nope"));
    (vscode as any).cursor = { mcp: { registerServer } };

    (vscode.workspace as any).fs.readFile.mockRejectedValue(new Error("missing"));
    (vscode.workspace as any).fs.writeFile.mockResolvedValue(undefined);

    await registerMcpServer(() => "/bin/sruja");

    expect((vscode.workspace as any).fs.createDirectory).toHaveBeenCalled();
    expect((vscode.workspace as any).fs.writeFile).toHaveBeenCalled();

    const writeArgs = (vscode.workspace as any).fs.writeFile.mock.calls[0];
    const content = Buffer.from(writeArgs[1]).toString("utf8");
    expect(content).toContain('"mcpServers"');
    expect(content).toContain('"sruja"');
    expect(content).toContain('"/bin/sruja"');
    expect(vscode.window.showInformationMessage).toHaveBeenCalledWith(
      "Sruja MCP server registered in .cursor/mcp.json for ws."
    );
  });

  it("shows error when fallback write fails", async () => {
    (vscode.workspace as any).workspaceFolders = [{ uri: vscode.Uri.file("/ws"), name: "ws" }];
    (vscode.workspace as any).fs.createDirectory.mockRejectedValue(new Error("no perms"));

    await registerMcpServer(() => "/bin/sruja");

    expect(vscode.window.showErrorMessage).toHaveBeenCalled();
  });
});


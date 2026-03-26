import * as vscode from "vscode";
import { parseJsonSafe } from "./safeJson";

export async function registerMcpServer(getSrujaPath: () => string): Promise<void> {
  const folders = vscode.workspace.workspaceFolders;
  if (!folders || folders.length === 0) {
    vscode.window.showWarningMessage("Open a workspace folder to register the MCP server.");
    return;
  }

  let folder = folders[0];
  if (folders.length > 1) {
    const picked = await vscode.window.showQuickPick(
      folders.map((f) => ({ label: f.name, folder: f })),
      { placeHolder: "Select workspace folder to register Sruja MCP server" }
    );
    if (!picked) return;
    folder = picked.folder;
  }

  const serverName = "sruja";
  const command = getSrujaPath();
  const args = ["mcp", "--root", folder.uri.fsPath];

  const cursorMcp = (vscode as any).cursor?.mcp;

  if (cursorMcp?.registerServer) {
    try {
      await cursorMcp.registerServer({ name: serverName, type: "stdio", command, args });
      vscode.window.showInformationMessage("Sruja MCP server registered in Cursor.");
      return;
    } catch {
      // Fallback to manual file update if registerServer fails
    }
  }

  const cursorDir = vscode.Uri.joinPath(folder.uri, ".cursor");
  const cursorConfigUri = vscode.Uri.joinPath(cursorDir, "mcp.json");

  try {
    await vscode.workspace.fs.createDirectory(cursorDir);

    let existing: Record<string, any> | undefined;
    try {
      const raw = await vscode.workspace.fs.readFile(cursorConfigUri);
      const text = Buffer.from(raw).toString("utf8");
      const parsed = parseJsonSafe<Record<string, any>>(text);
      if (parsed.ok) existing = parsed.value;
    } catch {
      existing = undefined;
    }

    const base = existing || {};
    const mcpServers = (base.mcpServers && typeof base.mcpServers === "object" ? base.mcpServers : {}) as Record<string, any>;

    mcpServers[serverName] = {
      type: "stdio",
      command,
      args,
    };

    const next = { ...base, mcpServers };
    const jsonText = JSON.stringify(next, null, 2) + "\n";
    await vscode.workspace.fs.writeFile(cursorConfigUri, Buffer.from(jsonText, "utf8"));

    vscode.window.showInformationMessage(`Sruja MCP server registered in .cursor/mcp.json for ${folder.name}.`);
  } catch (err) {
    vscode.window.showErrorMessage(
      `Failed to register Sruja MCP server: ${err instanceof Error ? err.message : String(err)}`
    );
  }
}

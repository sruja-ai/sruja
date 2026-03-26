import * as vscode from "vscode";
import * as path from "path";
import { runCli } from "../cliRunner";
import { parseJsonSafe } from "../safeJson";
import { StatusJson, ReviewJson, formatStatusLines, formatReviewLines } from "../cliOutput";

let cliOutputChannel: vscode.OutputChannel | undefined;

function getCliOutputChannel(): vscode.OutputChannel {
  if (!cliOutputChannel) {
    cliOutputChannel = vscode.window.createOutputChannel("Sruja");
  }
  return cliOutputChannel;
}

async function getTargetWorkspaceFolder(): Promise<vscode.WorkspaceFolder | undefined> {
  const folders = vscode.workspace.workspaceFolders;
  if (!folders || folders.length === 0) return undefined;
  if (folders.length === 1) return folders[0];

  // Try to use the folder of the active document
  const activeEditor = vscode.window.activeTextEditor;
  if (activeEditor) {
    const folder = vscode.workspace.getWorkspaceFolder(activeEditor.document.uri);
    if (folder) return folder;
  }

  // Ask the user to pick a folder
  const picked = await vscode.window.showQuickPick(
    folders.map((f) => ({ label: f.name, folder: f })),
    { placeHolder: "Select workspace folder for Sruja action" }
  );
  return picked?.folder;
}

async function runCliInWorkspace(getSrujaPath: () => string, args: string[]): Promise<{ stdout: string; stderr: string; code: number; folder?: vscode.WorkspaceFolder }> {
  const folder = await getTargetWorkspaceFolder();
  if (!folder) {
    throw new Error("No workspace folder selected.");
  }
  const res = await runCli(getSrujaPath(), args, folder.uri.fsPath);
  return { ...res, folder };
}

export function registerContextEngineeringCommands(context: vscode.ExtensionContext, getSrujaPath: () => string) {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.runDrift", async () => {
      await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: "Sruja: Running drift…", cancellable: false },
        async () => {
          const channel = getCliOutputChannel();
          channel.clear();
          channel.show(true);
          channel.appendLine("Running sruja drift -r . ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, ["drift", "-r", "."]);
            channel.append(stdout);
            if (stderr) channel.append(stderr);
            channel.appendLine("");
            if (code !== 0) {
              channel.appendLine(`(exit code ${code})`);
            }
            channel.appendLine("--- Done ---");
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage(
              "Sruja drift failed. Ensure the Sruja CLI is installed and on PATH, or set sruja.lsp.path."
            );
          }
        }
      );
    }),

    vscode.commands.registerCommand("sruja.refreshContext", async () => {
      const targetFolder = await getTargetWorkspaceFolder();
      if (!targetFolder) {
        vscode.window.showWarningMessage("Select a workspace folder to refresh repo context.");
        return;
      }
      await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: "Sruja: Refreshing repo context…", cancellable: false },
        async () => {
          const channel = getCliOutputChannel();
          channel.show(true);
          channel.appendLine("Refreshing repo context (sruja sync -r . -f json) ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, [
              "sync",
              "-r",
              ".",
              "-f",
              "json",
            ]);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.append(stdout);
              vscode.window.showErrorMessage("Sruja sync failed. Is the CLI on PATH or set sruja.lsp.path?");
              return;
            }
            const folder = (await runCliInWorkspace(getSrujaPath, ["status"])).folder; // Should not happen as we just ran it, but for type safety if we didn't pass folder back
            if (!folder) return;
            const contextPath = path.join(folder.uri.fsPath, ".sruja", "context.json");
            channel.appendLine(`Context written to ${contextPath}`);
            const parsed = parseJsonSafe<{ context_path?: string; truth_status?: string }>(stdout);
            if (parsed.ok && parsed.value.context_path) {
              channel.appendLine(`Baseline/truth: ${parsed.value.truth_status ?? "unknown"}`);
            }
            channel.appendLine("--- Done ---");
            vscode.window.showInformationMessage("Sruja: Repo context updated.");
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage("Sruja refresh context failed: " + msg);
          }
        }
      );
    }),

    vscode.commands.registerCommand("sruja.status", async () => {
      await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: "Sruja: Fetching status…", cancellable: false },
        async () => {
          const channel = getCliOutputChannel();
          channel.show(true);
          channel.appendLine("Running sruja status -r . --format json ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, [
              "status",
              "-r",
              ".",
              "--format",
              "json",
            ]);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.append(stdout);
              channel.appendLine("--- Status failed ---");
              return;
            }
            const parsed = parseJsonSafe<StatusJson>(stdout);
            if (!parsed.ok) {
              channel.appendLine(`Parse error: ${parsed.error}`);
              return;
            }
            for (const line of formatStatusLines(parsed.value)) {
              channel.appendLine(line);
            }
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage("Sruja status failed. Is the CLI on PATH or set sruja.lsp.path?");
          }
        }
      );
    }),

    vscode.commands.registerCommand("sruja.review", async () => {
      await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: "Sruja: Reviewing architecture update…", cancellable: false },
        async () => {
          const channel = getCliOutputChannel();
          channel.show(true);
          channel.appendLine("Running sruja review -r . --format json ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, [
              "review",
              "-r",
              ".",
              "--format",
              "json",
            ]);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.append(stdout);
              channel.appendLine("--- Review failed ---");
              return;
            }
            const parsed = parseJsonSafe<ReviewJson>(stdout);
            if (!parsed.ok) {
              channel.appendLine(`Parse error: ${parsed.error}`);
              return;
            }
            for (const line of formatReviewLines(parsed.value)) {
              channel.appendLine(line);
            }
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage("Sruja review failed. Is the CLI on PATH or set sruja.lsp.path?");
          }
        }
      );
    })
  );
}

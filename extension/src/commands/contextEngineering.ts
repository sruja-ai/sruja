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

async function runCliInFolder(
  getSrujaPath: () => string,
  folder: vscode.WorkspaceFolder,
  args: string[]
): Promise<{ stdout: string; stderr: string; code: number; folder: vscode.WorkspaceFolder }> {
  const res = await runCli(getSrujaPath(), args, folder.uri.fsPath);
  return { ...res, folder };
}

async function runCliInWorkspace(
  getSrujaPath: () => string,
  args: string[]
): Promise<{ stdout: string; stderr: string; code: number; folder: vscode.WorkspaceFolder }> {
  const folder = await getTargetWorkspaceFolder();
  if (!folder) {
    throw new Error("No workspace folder selected.");
  }
  return runCliInFolder(getSrujaPath, folder, args);
}

export function registerContextEngineeringCommands(context: vscode.ExtensionContext, getSrujaPath: () => string) {
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.verifyTask", async () => {
      const profilePick = await vscode.window.showQuickPick(
        [
          { label: "coding", description: "Default — lint, check, drift when applicable", profile: "coding" },
          { label: "bugfix", description: "Focused fix — check + intent", profile: "bugfix" },
          { label: "review", description: "Pre-merge — review + intent + drift", profile: "review" },
          { label: "arch", description: "Architecture/DSL changes", profile: "arch" },
        ],
        { placeHolder: "Sruja verify-task profile" }
      );
      if (!profilePick) return;

      const editor = vscode.window.activeTextEditor;
      const args = ["verify-task", "--profile", profilePick.profile, "-r", ".", "-f", "json"];
      if (profilePick.profile === "bugfix" && editor) {
        const folder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
        if (folder) {
          const rel = path.relative(folder.uri.fsPath, editor.document.uri.fsPath);
          args.push("--file", rel.split(path.sep).join("/"));
        }
      }

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: `Sruja: verify-task (${profilePick.profile})…`,
          cancellable: false,
        },
        async () => {
          const channel = getCliOutputChannel();
          channel.clear();
          channel.show(true);
          channel.appendLine(`Running sruja ${args.join(" ")} ...`);
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, args);
            channel.append(stdout);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.appendLine(`(exit code ${code})`);
              vscode.window.showErrorMessage(
                "Sruja verify-task failed. See Sruja output; fix issues or run with a narrower profile."
              );
              return;
            }
            const parsed = parseJsonSafe<{ all_passed?: boolean }>(stdout);
            if (parsed.ok && parsed.value.all_passed === false) {
              vscode.window.showErrorMessage(
                "Sruja verify-task: one or more steps failed (see output)."
              );
              return;
            }
            if (!parsed.ok) {
              vscode.window.showWarningMessage(
                "Sruja verify-task exited 0 but JSON could not be parsed; review output."
              );
              return;
            }
            vscode.window.showInformationMessage(
              `Sruja verify-task (${profilePick.profile}): all steps passed.`
            );
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage(
              "Sruja verify-task failed. Ensure the CLI is on PATH or set sruja.lsp.path."
            );
          }
        }
      );
    }),

    vscode.commands.registerCommand("sruja.runDrift", async () => {
      await vscode.window.withProgress(
        { location: vscode.ProgressLocation.Notification, title: "Sruja: Running drift…", cancellable: false },
        async () => {
          const channel = getCliOutputChannel();
          channel.clear();
          channel.show(true);
          channel.appendLine("Running sruja drift -r . --structural-only --advisory ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, [
              "drift",
              "-r",
              ".",
              "--structural-only",
              "--advisory",
            ]);
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
            const { stdout, stderr, code, folder } = await runCliInFolder(getSrujaPath, targetFolder, [
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

    vscode.commands.registerCommand("sruja.refreshArchitectureState", async () => {
      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: "Sruja: Refresh architecture state…",
          cancellable: false,
        },
        async () => {
          const channel = getCliOutputChannel();
          channel.show(true);
          channel.appendLine("Running sruja drift -r . -f drift-state ...");
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, [
              "drift",
              "-r",
              ".",
              "-f",
              "drift-state",
            ]);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.append(stdout);
              channel.appendLine("--- drift -f drift-state failed ---");
              vscode.window.showErrorMessage(
                "Sruja drift -f drift-state failed. Is the CLI on PATH or set sruja.lsp.path?"
              );
              return;
            }
            const parsed = parseJsonSafe<{
              schema_version?: string;
              truth_status?: string;
              health_score?: number;
              violation_count?: number;
              violations?: Array<{ message?: string; location?: string; severity?: string }>;
            }>(stdout);
            if (!parsed.ok) {
              channel.appendLine(`Parse error: ${parsed.error}`);
              channel.append(stdout);
              return;
            }
            const s = parsed.value;
            channel.appendLine(`schema: ${s.schema_version ?? "unknown"}`);
            channel.appendLine(`truth_status: ${s.truth_status ?? "unknown"}`);
            channel.appendLine(`health_score: ${s.health_score ?? "?"}`);
            channel.appendLine(`violations: ${s.violation_count ?? 0}`);
            for (const v of (s.violations ?? []).slice(0, 8)) {
              channel.appendLine(
                `  - [${v.severity ?? "?"}] ${v.location ?? "?"}: ${v.message ?? ""}`
              );
            }
            channel.appendLine("--- JSON (for host injection) ---");
            channel.append(stdout);
            await vscode.env.clipboard.writeText(stdout.trim());
            vscode.window.showInformationMessage(
              "Sruja: Architecture state refreshed (JSON copied to clipboard)."
            );
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            channel.appendLine(`Error: ${msg}`);
            vscode.window.showErrorMessage("Sruja refresh architecture state failed: " + msg);
          }
        }
      );
    }),

    vscode.commands.registerCommand("sruja.briefThisFile", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showWarningMessage("Open a source file to brief.");
        return;
      }
      const folder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
      if (!folder) {
        vscode.window.showWarningMessage("File is not in a workspace folder.");
        return;
      }
      const rel = path.relative(folder.uri.fsPath, editor.document.uri.fsPath);
      const relCli = rel.split(path.sep).join("/");
      const channel = getCliOutputChannel();
      channel.clear();
      channel.show(true);
      channel.appendLine(`Running sruja focus -r . --file ${relCli} ...`);
      try {
        const { stdout, stderr, code } = await runCliInFolder(getSrujaPath, folder, [
          "focus",
          "-r",
          ".",
          "--file",
          relCli,
        ]);
        channel.append(stdout);
        if (stderr) channel.append(stderr);
        if (code !== 0) {
          channel.appendLine(`(exit code ${code})`);
        }
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        channel.appendLine(`Error: ${msg}`);
        vscode.window.showErrorMessage("Sruja focus failed. Is the CLI on PATH or set sruja.lsp.path?");
      }
    }),

    vscode.commands.registerCommand("sruja.openAiSetup", async () => {
      const pick = await vscode.window.showQuickPick(
        [
          { label: "Verify task (host gate)", command: "sruja.verifyTask" },
          { label: "Brief this file (focus)", command: "sruja.briefThisFile" },
          { label: "Run structural drift", command: "sruja.runDrift" },
          { label: "Register MCP (Cursor)", command: "sruja.registerMcpServer" },
          { label: "Copy context pack for AI", command: "sruja.copyContextPackForAI" },
          { label: "Run validation (.sruja)", command: "sruja.runValidation" },
          { label: "Open skills overview", command: "sruja.openSkillsOverview" },
          { label: "Open agent guide (AGENTS.md)", command: "sruja.openAgentGuide" },
          { label: "List rules…", command: "sruja.listRules" },
        ],
        { placeHolder: "Sruja harness setup" }
      );
      if (pick) {
        await vscode.commands.executeCommand(pick.command);
      }
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

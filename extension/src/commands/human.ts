import * as vscode from "vscode";
import * as path from "path";
import { runCli } from "../cliRunner";
import { parseJsonSafe } from "../safeJson";

let cliOutputChannel: vscode.OutputChannel | undefined;

function getCliOutputChannel(): vscode.OutputChannel {
  if (!cliOutputChannel) {
    cliOutputChannel = vscode.window.createOutputChannel("Sruja Human");
  }
  return cliOutputChannel;
}

async function getTargetWorkspaceFolder(): Promise<vscode.WorkspaceFolder | undefined> {
  const folders = vscode.workspace.workspaceFolders;
  if (!folders || folders.length === 0) return undefined;
  if (folders.length === 1) return folders[0];

  const activeEditor = vscode.window.activeTextEditor;
  if (activeEditor) {
    const folder = vscode.workspace.getWorkspaceFolder(activeEditor.document.uri);
    if (folder) return folder;
  }

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

export function registerHumanCommands(context: vscode.ExtensionContext, getSrujaPath: () => string) {
  // Sruja: Human Map — "How does the system work?"
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.humanMap", async () => {
      const formatPick = await vscode.window.showQuickPick(
        [
          { label: "Text", description: "Human-readable text output", format: "text" },
          { label: "JSON", description: "Machine-readable JSON output", format: "json" },
        ],
        { placeHolder: "Output format" }
      );
      if (!formatPick) return;

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: "Sruja: Generating system map…",
          cancellable: false,
        },
        async () => {
          const channel = getCliOutputChannel();
          channel.clear();
          channel.show(true);
          const args = ["human", "map", "-r", ".", "-f", formatPick.format];
          channel.appendLine(`Running sruja ${args.join(" ")} ...`);
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, args);
            channel.append(stdout);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.appendLine(`(exit code ${code})`);
              vscode.window.showErrorMessage("Sruja human map failed. See Sruja output.");
            } else {
              vscode.window.showInformationMessage("Sruja human map completed.");
            }
          } catch (err: any) {
            channel.appendLine(`Error: ${err.message}`);
            vscode.window.showErrorMessage(`Sruja human map failed: ${err.message}`);
          }
        }
      );
    })
  );

  // Sruja: Human Trace — "What happens when..."
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.humanTrace", async () => {
      const query = await vscode.window.showInputBox({
        prompt: "What flow do you want to trace?",
        placeHolder: 'e.g. "user clicks checkout" or "sruja-cli"',
      });
      if (!query) return;

      const formatPick = await vscode.window.showQuickPick(
        [
          { label: "Text", description: "Human-readable text output", format: "text" },
          { label: "JSON", description: "Machine-readable JSON output", format: "json" },
        ],
        { placeHolder: "Output format" }
      );
      if (!formatPick) return;

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: `Sruja: Tracing "${query}"…`,
          cancellable: false,
        },
        async () => {
          const channel = getCliOutputChannel();
          channel.clear();
          channel.show(true);
          const args = ["human", "trace", query, "-r", ".", "-f", formatPick.format];
          channel.appendLine(`Running sruja ${args.join(" ")} ...`);
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, args);
            channel.append(stdout);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.appendLine(`(exit code ${code})`);
              vscode.window.showErrorMessage("Sruja human trace failed. See Sruja output.");
            } else {
              vscode.window.showInformationMessage("Sruja human trace completed.");
            }
          } catch (err: any) {
            channel.appendLine(`Error: ${err.message}`);
            vscode.window.showErrorMessage(`Sruja human trace failed: ${err.message}`);
          }
        }
      );
    })
  );

  // Sruja: Human Explain — "What is this thing?"
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.humanExplain", async () => {
      const editor = vscode.window.activeTextEditor;
      let defaultTarget = "";
      if (editor) {
        const fileName = path.basename(editor.document.fileName, path.extname(editor.document.fileName));
        defaultTarget = fileName;
      }

      const target = await vscode.window.showInputBox({
        prompt: "What element do you want to explain?",
        placeHolder: 'e.g. "PaymentService" or "sruja-agent"',
        value: defaultTarget,
      });
      if (!target) return;

      const formatPick = await vscode.window.showQuickPick(
        [
          { label: "Text", description: "Human-readable text output", format: "text" },
          { label: "JSON", description: "Machine-readable JSON output", format: "json" },
          { label: "Markdown", description: "Markdown output", format: "md" },
        ],
        { placeHolder: "Output format" }
      );
      if (!formatPick) return;

      const persistPick = await vscode.window.showQuickPick(
        [
          { label: "No", description: "Don't save to disk", persist: false },
          { label: "Yes", description: "Save to docs/architecture/<element>.md", persist: true },
        ],
        { placeHolder: "Save explanation to disk?" }
      );
      if (!persistPick) return;

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: `Sruja: Explaining "${target}"…`,
          cancellable: false,
        },
        async () => {
          const channel = getCliOutputChannel();
          channel.clear();
          channel.show(true);
          const args = ["human", "explain", target, "-r", ".", "-f", formatPick.format];
          if (persistPick.persist) {
            args.push("--persist");
          }
          channel.appendLine(`Running sruja ${args.join(" ")} ...`);
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, args);
            channel.append(stdout);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.appendLine(`(exit code ${code})`);
              vscode.window.showErrorMessage("Sruja human explain failed. See Sruja output.");
            } else {
              vscode.window.showInformationMessage("Sruja human explain completed.");
            }
          } catch (err: any) {
            channel.appendLine(`Error: ${err.message}`);
            vscode.window.showErrorMessage(`Sruja human explain failed: ${err.message}`);
          }
        }
      );
    })
  );

  // Sruja: Human Before — "What will I break?"
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.humanBefore", async () => {
      const editor = vscode.window.activeTextEditor;
      let defaultFile = "";
      if (editor) {
        const folder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
        if (folder) {
          defaultFile = path.relative(folder.uri.fsPath, editor.document.uri.fsPath);
        }
      }

      const file = await vscode.window.showInputBox({
        prompt: "What file are you about to change?",
        placeHolder: 'e.g. "src/payment.rs" or "crates/sruja-agent/src/memory/mod.rs"',
        value: defaultFile,
      });
      if (!file) return;

      const formatPick = await vscode.window.showQuickPick(
        [
          { label: "Text", description: "Human-readable text output", format: "text" },
          { label: "JSON", description: "Machine-readable JSON output", format: "json" },
        ],
        { placeHolder: "Output format" }
      );
      if (!formatPick) return;

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: `Sruja: Checking impact of "${file}"…`,
          cancellable: false,
        },
        async () => {
          const channel = getCliOutputChannel();
          channel.clear();
          channel.show(true);
          const args = ["human", "before", file, "-r", ".", "-f", formatPick.format];
          channel.appendLine(`Running sruja ${args.join(" ")} ...`);
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, args);
            channel.append(stdout);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.appendLine(`(exit code ${code})`);
              vscode.window.showErrorMessage("Sruja human before failed. See Sruja output.");
            } else {
              vscode.window.showInformationMessage("Sruja human before completed.");
            }
          } catch (err: any) {
            channel.appendLine(`Error: ${err.message}`);
            vscode.window.showErrorMessage(`Sruja human before failed: ${err.message}`);
          }
        }
      );
    })
  );

  // Sruja: Human What-If — "What if I change X?"
  context.subscriptions.push(
    vscode.commands.registerCommand("sruja.humanWhatIf", async () => {
      const query = await vscode.window.showInputBox({
        prompt: "What change do you want to model?",
        placeHolder: 'e.g. "remove FraudService sync check" or "sruja-agent"',
      });
      if (!query) return;

      const formatPick = await vscode.window.showQuickPick(
        [
          { label: "Text", description: "Human-readable text output", format: "text" },
          { label: "JSON", description: "Machine-readable JSON output", format: "json" },
        ],
        { placeHolder: "Output format" }
      );
      if (!formatPick) return;

      await vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: `Sruja: Modeling "${query}"…`,
          cancellable: false,
        },
        async () => {
          const channel = getCliOutputChannel();
          channel.clear();
          channel.show(true);
          const args = ["human", "what-if", query, "-r", ".", "-f", formatPick.format];
          channel.appendLine(`Running sruja ${args.join(" ")} ...`);
          try {
            const { stdout, stderr, code } = await runCliInWorkspace(getSrujaPath, args);
            channel.append(stdout);
            if (stderr) channel.append(stderr);
            if (code !== 0) {
              channel.appendLine(`(exit code ${code})`);
              vscode.window.showErrorMessage("Sruja human what-if failed. See Sruja output.");
            } else {
              vscode.window.showInformationMessage("Sruja human what-if completed.");
            }
          } catch (err: any) {
            channel.appendLine(`Error: ${err.message}`);
            vscode.window.showErrorMessage(`Sruja human what-if failed: ${err.message}`);
          }
        }
      );
    })
  );
}

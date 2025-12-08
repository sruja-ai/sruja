import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';
import { execFile } from 'child_process';
import { SrujaPreviewProvider } from './previewProvider';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
  ExecutableOptions
} from 'vscode-languageclient/node';

let client: LanguageClient;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
  try {
    console.log('Sruja extension activating...');

    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = 'sruja.showOutput';
    updateStatusBar('Initializing...');
    statusBarItem.show();

    const provider = new SrujaPreviewProvider(context);
    const registration = vscode.workspace.registerTextDocumentContentProvider(SrujaPreviewProvider.scheme, provider);
    context.subscriptions.push(registration);

    // Update preview on save
    vscode.workspace.onDidSaveTextDocument(doc => {
      if (doc.languageId === 'sruja') {
        const previewUri = getPreviewUri(doc.uri);
        provider.update(previewUri);
      }
    });

    const previewCmd = vscode.commands.registerCommand('sruja.previewArchitecture', () => previewArchitecture(context));
    const restartCmd = vscode.commands.registerCommand('sruja.restartServer', restartLanguageServer);
    const outputCmd = vscode.commands.registerCommand('sruja.showOutput', showServerOutput);

    context.subscriptions.push(previewCmd, restartCmd, outputCmd);

    console.log('Sruja extension commands registered');

    startLanguageServer(context);
  } catch (e) {
    console.error('Failed to activate Sruja extension:', e);
    vscode.window.showErrorMessage(`Sruja extension activation failed: ${e}`);
  }
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) return undefined;
  statusBarItem?.hide();
  return client.stop();
}

async function startLanguageServer(_context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration('srujaLanguageServer');
  let serverPath = resolveServerPath(config);
  const enableLogging = config.get<boolean>('enableLogging') || false;
  const logLevel = config.get<string>('logLevel') || 'info';

  const ws = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (ws) {
    const localBin = path.join(ws, 'bin', 'sruja');
    if (!fs.existsSync(serverPath) && fs.existsSync(localBin) === false) {
      updateStatusBar('Building CLI...', '$(tools)');
      try {
        await new Promise<void>((resolve, reject) => {
          execFile('make', ['build'], { cwd: ws }, (err) => {
            if (err) reject(err); else resolve();
          });
        });
        if (fs.existsSync(localBin)) {
          serverPath = localBin;
          config.update('path', serverPath, vscode.ConfigurationTarget.Workspace);
          updateStatusBar('CLI Ready', '$(check)');
        }
      } catch {
        // ignore; fallback to messaging below
      }
    }
  }

  const serverOptions: ServerOptions = {
    run: { command: serverPath, args: ['lsp'], transport: TransportKind.stdio, options: getExecutableOptions(enableLogging, logLevel) },
    debug: { command: serverPath, args: ['lsp'], transport: TransportKind.stdio, options: getExecutableOptions(true, 'debug') }
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'sruja' }],
    synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher('**/*.sruja') },
    outputChannelName: 'Sruja Language Server'
  };

  client = new LanguageClient('srujaLanguageServer', 'Sruja Language Server', serverOptions, clientOptions);
  
  // Single unified state change handler
  client.onDidChangeState((event: { newState: number }) => {
    switch (event.newState) {
      case 1: // Starting
        updateStatusBar('Starting...');
        break;
      case 2: // Connected
        updateStatusBar('Connected', '$(check)');
        break;
      case 3: // Disconnected
        updateStatusBar('Disconnected', '$(x)');
        // Show error message if disconnected (likely CLI not found)
        setTimeout(() => {
          vscode.window.showErrorMessage(
            'Sruja CLI not found. Please install it:\n' +
            'curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash\n\n' +
            'Or configure the path in settings: srujaLanguageServer.path',
            'Open Settings'
          ).then(selection => {
            if (selection === 'Open Settings') {
              vscode.commands.executeCommand('workbench.action.openSettings', 'srujaLanguageServer.path');
            }
          });
        }, 2000);
        break;
    }
  });

  client.start().catch(error => {
    if (error.message && (error.message.includes('ENOENT') || error.message.includes('spawn'))) {
      updateStatusBar('CLI Not Found', '$(error)');
      vscode.window.showErrorMessage(
        `Cannot find 'sruja' command. Please install the Sruja CLI:\n\n` +
        `curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash\n\n` +
        `Or set the path in settings: srujaLanguageServer.path`,
        'Open Settings'
      ).then(selection => {
        if (selection === 'Open Settings') {
          vscode.commands.executeCommand('workbench.action.openSettings', 'srujaLanguageServer.path');
        }
      });
    } else {
      updateStatusBar('Error', '$(error)');
      vscode.window.showErrorMessage(`Failed to start Sruja Language Server: ${error.message}`);
    }
  });

  registerSemanticTokensProvider(_context);
}

function resolveServerPath(config: vscode.WorkspaceConfiguration): string {
  const configured = config.get<string>('path');
  const ws = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  const candidates: string[] = [];
  if (configured) candidates.push(configured);
  if (ws) candidates.push(path.join(ws, 'bin', 'sruja'));
  candidates.push('sruja');

  for (const p of candidates) {
    if (p && p !== 'sruja') {
      try {
        if (fs.existsSync(p)) {
          if (!configured || configured !== p) {
            config.update('path', p, vscode.ConfigurationTarget.Workspace);
          }
          return p;
        }
      } catch {}
    }
  }
  return configured || 'sruja';
}

function getExecutableOptions(enableLogging: boolean, logLevel: string): ExecutableOptions {
  const options: ExecutableOptions = {};
  if (enableLogging) {
    options.env = { ...process.env, SRUJA_LSP_LOG_LEVEL: logLevel };
  }
  return options;
}

function restartLanguageServer() {
  if (client) {
    client.restart().then(
      () => vscode.window.showInformationMessage('Sruja Language Server restarted successfully'),
      (error: any) => vscode.window.showErrorMessage(`Failed to restart Sruja Language Server: ${error}`)
    );
  }
}

function showServerOutput() { if (client) client.outputChannel.show(); }

function updateStatusBar(text: string, icon?: string) {
  if (!statusBarItem) return;
  statusBarItem.text = icon ? `${icon} Sruja: ${text}` : `Sruja: ${text}`;
  if (text.includes('Disconnected') || text.includes('Error')) {
    statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
  } else {
    statusBarItem.backgroundColor = undefined;
  }
}

function getPreviewUri(uri: vscode.Uri): vscode.Uri {
  const query = `original=${uri.fsPath}`;
  return vscode.Uri.parse(`${SrujaPreviewProvider.scheme}:${uri.path}.md?${query}`);
}

async function previewArchitecture(context: vscode.ExtensionContext) {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('No active editor');
    return;
  }

  const doc = editor.document;
  if (doc.languageId !== 'sruja') {
    vscode.window.showErrorMessage('Preview works only for Sruja (.sruja) files');
    return;
  }

  if (doc.isUntitled) {
    vscode.window.showErrorMessage('Please save the file before previewing');
    return;
  }

  const previewUri = getPreviewUri(doc.uri);

  try {
    await vscode.commands.executeCommand('markdown.showPreview', previewUri);
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to open preview: ${error}`);
  }
}

function registerSemanticTokensProvider(context: vscode.ExtensionContext) {
  const tokenTypes = [
    'keyword', 'class', 'module', 'function', 'struct', 'enum', 'variable', 'operator', 'string'
  ];
  const tokenModifiers = ['declaration'];
  const legend = new vscode.SemanticTokensLegend(tokenTypes, tokenModifiers);

  const provider: vscode.DocumentSemanticTokensProvider = {
    async provideDocumentSemanticTokens(document: vscode.TextDocument): Promise<vscode.SemanticTokens | null> {
      if (!client) return null;
      try {
        const uri = document.uri.toString();
        interface SemanticTokensResponse { data: number[] }
        const resp = await client.sendRequest<SemanticTokensResponse>('textDocument/semanticTokens/full', { textDocument: { uri } });
        if (!resp || !Array.isArray(resp.data)) return null;
        const data: number[] = resp.data;
        const builder = new vscode.SemanticTokensBuilder(legend);
        let prevLine = 0;
        let prevChar = 0;
        for (let i = 0; i + 4 < data.length; i += 5) {
          const deltaLine = data[i];
          const deltaStart = data[i + 1];
          const length = data[i + 2];
          const tokenType = data[i + 3];
          const tokenMods = data[i + 4];
          const line = prevLine + deltaLine;
          const startChar = deltaLine === 0 ? prevChar + deltaStart : deltaStart;
          builder.push(line, startChar, length, tokenType, tokenMods);
          prevLine = line;
          prevChar = startChar;
        }
        return builder.build();
      } catch {
        return null;
      }
    }
  };

  const selector = { language: 'sruja', scheme: 'file' };
  const reg = vscode.languages.registerDocumentSemanticTokensProvider(selector, provider, legend);
  context.subscriptions.push(reg);
}

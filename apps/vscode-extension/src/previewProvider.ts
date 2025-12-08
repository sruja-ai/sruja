import * as vscode from 'vscode';
import { execFile } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';

export class SrujaPreviewProvider implements vscode.TextDocumentContentProvider {
    static readonly scheme = 'sruja-preview';

    private _onDidChange = new vscode.EventEmitter<vscode.Uri>();
    get onDidChange(): vscode.Event<vscode.Uri> {
        return this._onDidChange.event;
    }

    constructor(private context: vscode.ExtensionContext) { }

    update(uri: vscode.Uri) {
        this._onDidChange.fire(uri);
    }

    provideTextDocumentContent(uri: vscode.Uri): Promise<string> {
        return new Promise((resolve, reject) => {
            const query = new URLSearchParams(uri.query);
            const originalPath = query.get('original');

            if (!originalPath) {
                return reject(new Error('Original file path not found in URI query'));
            }

            const config = vscode.workspace.getConfiguration('srujaLanguageServer');
            let srujaPath = config.get<string>('path') || 'sruja';
            const ws = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            const localBin = ws ? path.join(ws, 'bin', 'sruja') : undefined;
            if (localBin && fs.existsSync(localBin)) {
                srujaPath = localBin;
            }

            execFile(srujaPath, ['export', 'markdown', originalPath], { env: process.env }, (err, stdout, stderr) => {
                if (err) {
                    const errorMsg = stderr?.toString() || err.message || 'Unknown error';
                    return resolve(`
# Error Generating Preview

Failed to run sruja export:
\`\`\`
${errorMsg}
\`\`\`

Please ensure the Sruja CLI is installed and configured correctly.
`);
                }
                resolve(stdout.toString());
            });
        });
    }
}

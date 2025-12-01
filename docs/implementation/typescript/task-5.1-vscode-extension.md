# Task 5.1: VS Code Extension

**Priority**: 🟢 Medium (Enhances developer experience)
**Technology**: TypeScript
**Estimated Time**: 5-7 days
**Dependencies**: Task 1.7 (LSP), Task 4.1 (Studio Core)

## Overview

Create VS Code extension for Sruja that provides:
- LSP integration (syntax highlighting, errors, completion, hover)
- Studio webview (visual editor)
- File operations (read/write `.sruja` files)
- Git integration (SCM status, PR workflows)

## Features

### 1. LSP Integration

**Language Support**:
- Syntax highlighting (`.sruja` files)
- Error diagnostics (red squiggles)
- Code completion
- Hover information
- Go to definition
- Find references
- Quick fixes (code actions)
- Formatting (format on save)

**Implementation**:
- Extension launches LSP server (`sruja lsp`)
- Communicates via stdio
- Displays diagnostics in Problems panel

### 2. Studio Webview

**Visual Editor**:
- Embed Local Studio in webview
- Same UI as CLI Studio
- File operations via VS Code APIs
- Real-time sync with editor

**Implementation**:
- Reuse `local-studio/` React app
- Adapt for VS Code webview context
- Use VS Code file system APIs

### 3. File Operations

**VS Code APIs**:
- Read/write `.sruja` files
- Watch for file changes
- Handle file saves
- Workspace file management

### 4. Git Integration

**SCM Features**:
- Show `.sruja` files in Source Control
- Diff view for changes
- Commit `.sruja` files
- Create PRs (via GitHub extension)

## Extension Structure

```
vscode-extension/
  ├── src/
  │   ├── extension.ts          # Main extension entry
  │   ├── lsp/
  │   │   ├── client.ts         # LSP client
  │   │   └── server.ts         # LSP server launcher
  │   ├── studio/
  │   │   ├── webview.ts        # Studio webview provider
  │   │   └── panel.ts          # Studio panel
  │   ├── commands/
  │   │   ├── openStudio.ts     # Open Studio command
  │   │   ├── formatDocument.ts # Format command
  │   │   └── validate.ts       # Validate command
  │   └── utils/
  │       └── fileSystem.ts     # File operations
  ├── package.json
  ├── tsconfig.json
  └── .vscodeignore
```

## Implementation

### Extension Entry Point

```typescript
// src/extension.ts
import * as vscode from 'vscode';
import { SrujaLanguageClient } from './lsp/client';
import { StudioWebviewProvider } from './studio/webview';

export function activate(context: vscode.ExtensionContext) {
    // Initialize LSP client
    const lspClient = new SrujaLanguageClient(context);
    await lspClient.start();
    
    // Register Studio webview
    const studioProvider = new StudioWebviewProvider(context);
    vscode.window.registerWebviewViewProvider('sruja.studio', studioProvider);
    
    // Register commands
    vscode.commands.registerCommand('sruja.openStudio', () => {
        studioProvider.show();
    });
    
    vscode.commands.registerCommand('sruja.formatDocument', async () => {
        const editor = vscode.window.activeTextEditor;
        if (editor && editor.document.languageId === 'sruja') {
            await vscode.commands.executeCommand('editor.action.formatDocument');
        }
    });
    
    // Register language features
    vscode.languages.registerDocumentFormattingEditProvider('sruja', {
        provideDocumentFormattingEdits(document: vscode.TextDocument): vscode.TextEdit[] {
            // Format DSL using LSP
            return lspClient.formatDocument(document);
        }
    });
}
```

### LSP Client

```typescript
// src/lsp/client.ts
import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

export class SrujaLanguageClient {
    private client: LanguageClient;
    
    constructor(context: vscode.ExtensionContext) {
        const serverOptions: ServerOptions = {
            command: 'sruja',
            args: ['lsp'],
            options: {
                env: { ...process.env }
            }
        };
        
        const clientOptions: LanguageClientOptions = {
            documentSelector: [{ scheme: 'file', language: 'sruja' }],
            synchronize: {
                fileEvents: vscode.workspace.createFileSystemWatcher('**/*.sruja')
            }
        };
        
        this.client = new LanguageClient(
            'sruja',
            'Sruja Language Server',
            serverOptions,
            clientOptions
        );
    }
    
    async start() {
        await this.client.start();
    }
    
    async stop() {
        await this.client.stop();
    }
}
```

### Studio Webview

```typescript
// src/studio/webview.ts
import * as vscode from 'vscode';
import * as path from 'path';

export class StudioWebviewProvider implements vscode.WebviewViewProvider {
    private _view?: vscode.WebviewView;
    
    constructor(private context: vscode.ExtensionContext) {}
    
    resolveWebviewView(
        webviewView: vscode.WebviewView,
        context: vscode.WebviewViewResolveContext,
        token: vscode.CancellationToken
    ) {
        this._view = webviewView;
        
        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [
                vscode.Uri.file(path.join(this.context.extensionPath, 'dist'))
            ]
        };
        
        // Load Studio React app
        const studioUri = vscode.Uri.file(
            path.join(this.context.extensionPath, 'dist', 'studio', 'index.html')
        );
        
        webviewView.webview.html = this.getWebviewContent(studioUri);
        
        // Handle messages from Studio
        webviewView.webview.onDidReceiveMessage(async (message) => {
            switch (message.command) {
                case 'loadFile':
                    const content = await vscode.workspace.fs.readFile(
                        vscode.Uri.file(message.path)
                    );
                    webviewView.webview.postMessage({
                        command: 'fileLoaded',
                        content: content.toString()
                    });
                    break;
                    
                case 'saveFile':
                    await vscode.workspace.fs.writeFile(
                        vscode.Uri.file(message.path),
                        Buffer.from(message.content)
                    );
                    break;
            }
        });
    }
    
    private getWebviewContent(studioUri: vscode.Uri): string {
        return `
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
            </head>
            <body>
                <iframe src="${studioUri}" style="width: 100%; height: 100vh; border: none;"></iframe>
                <script>
                    const vscode = acquireVsCodeApi();
                    window.addEventListener('message', event => {
                        const message = event.data;
                        switch (message.command) {
                            case 'fileLoaded':
                                // Forward to Studio iframe
                                break;
                        }
                    });
                </script>
            </body>
            </html>
        `;
    }
    
    show() {
        if (this._view) {
            this._view.show();
        }
    }
}
```

### Package.json Configuration

```json
{
  "name": "sruja",
  "displayName": "Sruja",
  "description": "Sruja Architecture as Code language support",
  "version": "0.1.0",
  "engines": {
    "vscode": "^1.80.0"
  },
  "categories": ["Languages", "Other"],
  "activationEvents": [
    "onLanguage:sruja",
    "onCommand:sruja.openStudio"
  ],
  "main": "./dist/extension.js",
  "contributes": {
    "languages": [{
      "id": "sruja",
      "aliases": ["Sruja", "sruja"],
      "extensions": [".sruja"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "sruja",
      "scopeName": "source.sruja",
      "path": "./syntaxes/sruja.tmLanguage.json"
    }],
    "commands": [
      {
        "command": "sruja.openStudio",
        "title": "Open Studio",
        "icon": "$(graph)"
      },
      {
        "command": "sruja.formatDocument",
        "title": "Format Document"
      },
      {
        "command": "sruja.validate",
        "title": "Validate"
      }
    ],
    "views": {
      "explorer": [{
        "id": "sruja.studio",
        "name": "Sruja Studio",
        "when": "sruja:enabled"
      }]
    },
    "configuration": {
      "title": "Sruja",
      "properties": {
        "sruja.lsp.path": {
          "type": "string",
          "default": "sruja",
          "description": "Path to Sruja CLI"
        },
        "sruja.formatOnSave": {
          "type": "boolean",
          "default": true,
          "description": "Format document on save"
        }
      }
    }
  }
}
```

## Syntax Highlighting

**TextMate Grammar** (`syntaxes/sruja.tmLanguage.json`):
- Keywords (architecture, system, container, etc.)
- Strings
- Comments
- Identifiers
- Relations

## Language Configuration

**Language Config** (`language-configuration.json`):
- Comments (single-line `//`, multi-line `/* */`)
- Brackets matching
- Auto-closing pairs
- Indentation rules

## Acceptance Criteria

- [ ] Extension installs and activates
- [ ] Syntax highlighting works for `.sruja` files
- [ ] LSP integration works (errors, completion, hover)
- [ ] Studio webview opens and works
- [ ] File operations work (load/save via VS Code APIs)
- [ ] Format document command works
- [ ] Format on save works
- [ ] Quick fixes work (code actions)
- [ ] Go to definition works
- [ ] Find references works
- [ ] Problems panel shows errors
- [ ] Works with Git (SCM integration)

## Dependencies

- Task 1.7 (LSP) - Language server protocol
- Task 4.1 (Studio Core) - Studio React app
- `vscode-languageclient` - LSP client library
- `@types/vscode` - VS Code API types

## Notes

- **Reuse Studio**: Embed `local-studio/` React app in webview
- **LSP Server**: Extension launches `sruja lsp` command
- **File Operations**: Use VS Code file system APIs (not direct file access)
- **Performance**: Lazy load Studio webview (only when opened)


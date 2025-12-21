// apps/vscode-extension/src/previewProvider.ts
import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import { convertDslToMarkdown, initWasmNode } from "@sruja/shared/node/wasmAdapter";

function createFileNotFoundError(filePath: string): string {
  return `# Error Generating Preview

File not found:
\`\`\`
${filePath}
\`\`\`

Please ensure the file is saved and try again.`;
}

function createConversionError(): string {
  return `# Error Generating Preview

Failed to convert DSL to Markdown.

**Troubleshooting:**
1. Check that the file contains valid Sruja DSL
2. Ensure WASM files are bundled with extension
3. Try rebuilding the extension: \`npm run vscode:prepublish\``;
}

function createWasmError(errorMsg: string): string {
  return `# Error Generating Preview

WASM parser failed: ${errorMsg}

**Troubleshooting:**
1. Ensure WASM files are bundled with extension (\`wasm/sruja.wasm\` and \`wasm/wasm_exec.js\`)
2. Check that extension was built correctly: \`npm run vscode:prepublish\`
3. Verify WASM files exist in extension directory
4. Try rebuilding: \`make wasm && npm run vscode:prepublish\``;
}

export class SrujaPreviewProvider implements vscode.TextDocumentContentProvider {
  static readonly scheme = "sruja-preview";

  private _onDidChange = new vscode.EventEmitter<vscode.Uri>();
  get onDidChange(): vscode.Event<vscode.Uri> {
    return this._onDidChange.event;
  }

  constructor(private context: vscode.ExtensionContext) {}

  update(uri: vscode.Uri) {
    this._onDidChange.fire(uri);
  }

  async provideTextDocumentContent(uri: vscode.Uri): Promise<string> {
    const query = new URLSearchParams(uri.query);
    const originalPath = query.get("original");

    if (!originalPath) {
      throw new Error("Original file path not found in URI query");
    }

    if (!fs.existsSync(originalPath)) {
      return createFileNotFoundError(originalPath);
    }

    const absolutePath = path.isAbsolute(originalPath) ? originalPath : path.resolve(originalPath);

    try {
      const dslContent = fs.readFileSync(absolutePath, "utf-8");
      const extensionPath = this.context.extensionPath;
      const wasmApi = await initWasmNode({ extensionPath });
      const markdown = await convertDslToMarkdown(dslContent, wasmApi, path.basename(absolutePath));

      if (markdown) {
        return markdown;
      }
      return createConversionError();
    } catch (wasmErr) {
      const errorMsg = wasmErr instanceof Error ? wasmErr.message : String(wasmErr);
      return createWasmError(errorMsg);
    }
  }
}

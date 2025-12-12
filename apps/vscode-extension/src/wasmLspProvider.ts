// apps/vscode-extension/src/wasmLspProvider.ts
// WASM-based LSP provider for VS Code (no CLI dependency)
import * as vscode from 'vscode';
import { initWasmNode, type Diagnostic, type HoverInfo, type CompletionItem, type Location } from '@sruja/shared/node/wasmAdapter';

let wasmApi: Awaited<ReturnType<typeof initWasmNode>> | null = null;

export async function initializeWasmLsp(context: vscode.ExtensionContext): Promise<void> {
  try {
    wasmApi = await initWasmNode({ extensionPath: context.extensionPath });
    
    // Register diagnostics provider
    const diagnosticsCollection = vscode.languages.createDiagnosticCollection('sruja');
    context.subscriptions.push(diagnosticsCollection);
    
    const updateDiagnostics = async (document: vscode.TextDocument) => {
      if (document.languageId !== 'sruja' || !wasmApi) return;
      
      try {
        const text = document.getText();
        const diagnostics = await wasmApi.getDiagnostics(text);
        
        const vscodeDiagnostics: vscode.Diagnostic[] = diagnostics.map(d => {
          const severity = d.severity === 'Error' 
            ? vscode.DiagnosticSeverity.Error
            : d.severity === 'Warning'
            ? vscode.DiagnosticSeverity.Warning
            : vscode.DiagnosticSeverity.Information;
          
          const range = new vscode.Range(
            d.location.line - 1, // VS Code uses 0-based lines
            d.location.column - 1, // VS Code uses 0-based columns
            d.location.line - 1,
            d.location.column + 10 // Approximate end
          );
          
          const diagnostic = new vscode.Diagnostic(range, d.message, severity);
          diagnostic.code = d.code;
          return diagnostic;
        });
        
        diagnosticsCollection.set(document.uri, vscodeDiagnostics);
      } catch (error) {
        console.error('WASM diagnostics failed:', error);
        diagnosticsCollection.set(document.uri, []);
      }
    };
    
    // Update diagnostics on document change
    vscode.workspace.onDidChangeTextDocument(e => {
      if (e.document.languageId === 'sruja') {
        updateDiagnostics(e.document);
      }
    });
    
    // Initial diagnostics for open documents
    vscode.workspace.textDocuments.forEach(doc => {
      if (doc.languageId === 'sruja') {
        updateDiagnostics(doc);
      }
    });
    
    // Register hover provider
    const hoverProvider: vscode.HoverProvider = {
      async provideHover(document, position) {
        if (!wasmApi || document.languageId !== 'sruja') return null;
        
        try {
          const text = document.getText();
          const hoverInfo = await wasmApi.hover(text, position.line + 1, position.character + 1);
          
          if (!hoverInfo) return null;
          
          return new vscode.Hover({
            language: 'markdown',
            value: hoverInfo.contents,
          });
        } catch (error) {
          console.error('WASM hover failed:', error);
          return null;
        }
      },
    };
    
    context.subscriptions.push(
      vscode.languages.registerHoverProvider('sruja', hoverProvider)
    );
    
    // Register completion provider
    const completionProvider: vscode.CompletionItemProvider = {
      async provideCompletionItems(document, position) {
        if (!wasmApi || document.languageId !== 'sruja') return [];
        
        try {
          const text = document.getText();
          const completions = await wasmApi.completion(text, position.line + 1, position.character + 1);
          
          return completions.map(c => {
            const item = new vscode.CompletionItem(c.label, vscode.CompletionItemKind.Keyword);
            return item;
          });
        } catch (error) {
          console.error('WASM completion failed:', error);
          return [];
        }
      },
    };
    
    context.subscriptions.push(
      vscode.languages.registerCompletionItemProvider('sruja', completionProvider)
    );
    
    // Register definition provider
    const definitionProvider: vscode.DefinitionProvider = {
      async provideDefinition(document, position) {
        if (!wasmApi || document.languageId !== 'sruja') return [];
        
        try {
          const text = document.getText();
          const location = await wasmApi.goToDefinition(text, position.line + 1, position.character + 1);
          
          if (!location) return [];
          
          const range = new vscode.Range(
            location.line - 1,
            location.column - 1,
            location.line - 1,
            location.column + 10
          );
          
          return new vscode.Location(document.uri, range);
        } catch (error) {
          console.error('WASM goToDefinition failed:', error);
          return [];
        }
      },
    };
    
    context.subscriptions.push(
      vscode.languages.registerDefinitionProvider('sruja', definitionProvider)
    );
    
    // Register format provider
    const formatProvider: vscode.DocumentFormattingEditProvider = {
      async provideDocumentFormattingEdits(document) {
        if (!wasmApi || document.languageId !== 'sruja') return [];
        
        try {
          const text = document.getText();
          const formatted = await wasmApi.format(text);
          
          const range = new vscode.Range(
            document.positionAt(0),
            document.positionAt(document.getText().length)
          );
          
          return [vscode.TextEdit.replace(range, formatted)];
        } catch (error) {
          console.error('WASM format failed:', error);
          return [];
        }
      },
    };
    
    context.subscriptions.push(
      vscode.languages.registerDocumentFormattingEditProvider('sruja', formatProvider)
    );
    
    console.log('WASM LSP providers registered successfully');
  } catch (error) {
    console.error('Failed to initialize WASM LSP:', error);
    vscode.window.showErrorMessage(
      `Failed to initialize WASM LSP: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

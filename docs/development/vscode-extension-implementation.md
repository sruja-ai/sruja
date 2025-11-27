# VS Code Extension Implementation Plan

**Goal**: Implement all essential productivity features for maximum developer productivity.

[← Back to VS Code Extension](./vscode-extension.md)

## Implementation Order

### Phase 1: Foundation (Day 1)
1. ✅ Syntax highlighting
2. ✅ Code snippets
3. ✅ File association
4. ✅ Basic extension structure

### Phase 2: Essential Features (Days 2-4)

**Priority Order**:
1. **Hover Documentation** (2-4 hours) - Easiest, high value
2. **Keyword Completion** (2-4 hours) - Quick win
3. **Mermaid Diagram Preview** (4-6 hours) - Essential for visual feedback
4. **Context-Aware Completion** (1-2 days) - Essential
5. **Go-to-Definition** (1-2 days) - Essential

**Total**: ~4-5 days

---

## Implementation Details

### 1. Hover Documentation

**File**: `src/extension.ts`

```typescript
import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export function activate(context: vscode.ExtensionContext) {
  // Hover provider
  const hoverProvider = vscode.languages.registerHoverProvider('sruja', {
    async provideHover(document: vscode.TextDocument, position: vscode.Position) {
      try {
        const line = position.line + 1;
        const col = position.character + 1;
        const cmd = `sruja hover "${document.fileName}" ${line} ${col}`;
        
        const { stdout } = await execAsync(cmd);
        const info = JSON.parse(stdout);
        
        if (info.description) {
          const markdown = new vscode.MarkdownString();
          markdown.appendMarkdown(`**${info.type}**: ${info.name}\n\n`);
          markdown.appendMarkdown(info.description);
          
          if (info.technology) {
            markdown.appendMarkdown(`\n\n*Technology: ${info.technology}*`);
          }
          
          return new vscode.Hover(markdown);
        }
      } catch (error) {
        // Silently fail - hover is optional
      }
      return null;
    }
  });
  
  context.subscriptions.push(hoverProvider);
}
```

**Go CLI Command Needed**: `cmd/sruja/hover.go`

```go
package main

import (
    "encoding/json"
    "fmt"
    "os"
    "strconv"
    
    "github.com/sruja-ai/sruja/pkg/language"
    "github.com/sruja-ai/sruja/pkg/compiler"
)

type HoverInfo struct {
    Type        string `json:"type"`
    Name        string `json:"name"`
    Description string `json:"description"`
    Technology  string `json:"technology,omitempty"`
}

func hoverCommand(file string, line, col int) error {
    // Read file
    content, err := os.ReadFile(file)
    if err != nil {
        return err
    }
    
    // Parse
    parser := language.NewParser(string(content))
    ast, err := parser.Parse()
    if err != nil {
        return err
    }
    
    // Find element at position
    element := findElementAtPosition(ast, line, col)
    if element == nil {
        return fmt.Errorf("no element at position")
    }
    
    // Build hover info
    info := HoverInfo{
        Type:        element.Type,
        Name:        element.Name,
        Description: element.Description,
        Technology:  element.Technology,
    }
    
    // Output JSON
    json.NewEncoder(os.Stdout).Encode(info)
    return nil
}

func findElementAtPosition(ast *language.AST, line, col int) *language.Element {
    // Traverse AST to find element at position
    // Implementation depends on AST structure
    // ...
}
```

---

### 2. Keyword Completion

**File**: `src/extension.ts`

```typescript
const keywordProvider = vscode.languages.registerCompletionItemProvider('sruja', {
  provideCompletionItems() {
    const keywords = [
      { label: 'workspace', kind: vscode.CompletionItemKind.Keyword, detail: 'Workspace block' },
      { label: 'model', kind: vscode.CompletionItemKind.Keyword, detail: 'Model block' },
      { label: 'system', kind: vscode.CompletionItemKind.Keyword, detail: 'System declaration' },
      { label: 'container', kind: vscode.CompletionItemKind.Keyword, detail: 'Container declaration' },
      { label: 'component', kind: vscode.CompletionItemKind.Keyword, detail: 'Component declaration' },
      { label: 'requirement', kind: vscode.CompletionItemKind.Keyword, detail: 'Requirement' },
      { label: 'adr', kind: vscode.CompletionItemKind.Keyword, detail: 'Architecture Decision Record' },
      { label: 'import', kind: vscode.CompletionItemKind.Keyword, detail: 'Import statement' },
    ];
    
    return keywords.map(kw => {
      const item = new vscode.CompletionItem(kw.label, kw.kind);
      item.detail = kw.detail;
      return item;
    });
  }
}, ' ', '\t', '\n');
```

---

### 3. Context-Aware Completion

**File**: `src/extension.ts`

```typescript
const contextAwareProvider = vscode.languages.registerCompletionItemProvider('sruja', {
  async provideCompletionItems(document: vscode.TextDocument, position: vscode.Position) {
    try {
      const line = position.line + 1;
      const col = position.character + 1;
      const cmd = `sruja complete "${document.fileName}" ${line} ${col}`;
      
      const { stdout } = await execAsync(cmd);
      const suggestions = JSON.parse(stdout);
      
      return suggestions.map((s: any) => {
        const item = new vscode.CompletionItem(s.text, s.kind);
        item.detail = s.detail;
        item.documentation = s.documentation;
        return item;
      });
    } catch (error) {
      // Fall back to keyword completion
      return [];
    }
  }
}, ' ', '\t', '\n', '>', '-', ':');
```

**Go CLI Command Needed**: `cmd/sruja/complete.go`

```go
package main

import (
    "encoding/json"
    "fmt"
    "os"
    "strconv"
    
    "github.com/sruja-ai/sruja/pkg/language"
)

type CompletionSuggestion struct {
    Text         string `json:"text"`
    Kind         string `json:"kind"` // "keyword", "element", "property"
    Detail       string `json:"detail,omitempty"`
    Documentation string `json:"documentation,omitempty"`
}

func completeCommand(file string, line, col int) error {
    // Read file
    content, err := os.ReadFile(file)
    if err != nil {
        return err
    }
    
    // Parse up to cursor (may be incomplete)
    parser := language.NewParser(string(content))
    ast, err := parser.ParsePartial(line, col)
    if err != nil {
        // If parse fails, return keywords only
        return returnKeywords()
    }
    
    // Determine context
    context := determineContext(ast, line, col)
    
    // Generate suggestions based on context
    suggestions := generateSuggestions(context, ast)
    
    // Output JSON
    json.NewEncoder(os.Stdout).Encode(suggestions)
    return nil
}

func determineContext(ast *language.AST, line, col int) string {
    // Determine what context we're in:
    // - Start of statement -> keywords
    // - After "->" -> element names
    // - In metadata block -> properties
    // - In import -> module paths
    // ...
}

func generateSuggestions(context string, ast *language.AST) []CompletionSuggestion {
    switch context {
    case "statement_start":
        return keywordSuggestions()
    case "after_arrow":
        return elementNameSuggestions(ast)
    case "metadata":
        return propertySuggestions()
    case "import":
        return importPathSuggestions()
    default:
        return keywordSuggestions()
    }
}
```

---

### 4. Go-to-Definition

**File**: `src/extension.ts`

```typescript
const definitionProvider = vscode.languages.registerDefinitionProvider('sruja', {
  async provideDefinition(document: vscode.TextDocument, position: vscode.Position) {
    try {
      const line = position.line + 1;
      const col = position.character + 1;
      const cmd = `sruja definition "${document.fileName}" ${line} ${col}`;
      
      const { stdout } = await execAsync(cmd);
      const result = JSON.parse(stdout);
      
      if (result.file && result.line) {
        const uri = vscode.Uri.file(result.file);
        const pos = new vscode.Position(result.line - 1, result.column - 1);
        return new vscode.Location(uri, pos);
      }
    } catch (error) {
      // Silently fail
    }
    return null;
  }
});
```

**Go CLI Command Needed**: `cmd/sruja/definition.go`

```go
package main

import (
    "encoding/json"
    "fmt"
    "os"
    
    "github.com/sruja-ai/sruja/pkg/language"
    "github.com/sruja-ai/sruja/pkg/composition"
)

type DefinitionResult struct {
    File   string `json:"file"`
    Line   int    `json:"line"`
    Column int    `json:"column"`
}

func definitionCommand(file string, line, col int) error {
    // Read file
    content, err := os.ReadFile(file)
    if err != nil {
        return err
    }
    
    // Parse
    parser := language.NewParser(string(content))
    ast, err := parser.Parse()
    if err != nil {
        return err
    }
    
    // Find identifier at position
    identifier := findIdentifierAtPosition(ast, line, col)
    if identifier == "" {
        return fmt.Errorf("no identifier at position")
    }
    
    // Resolve reference
    definition := resolveDefinition(identifier, ast, file)
    if definition == nil {
        return fmt.Errorf("definition not found")
    }
    
    // Output JSON
    result := DefinitionResult{
        File:   definition.File,
        Line:   definition.Line,
        Column: definition.Column,
    }
    json.NewEncoder(os.Stdout).Encode(result)
    return nil
}

func resolveDefinition(identifier string, ast *language.AST, currentFile string) *Definition {
    // Search in current file first
    if def := findInAST(ast, identifier); def != nil {
        return &Definition{
            File:   currentFile,
            Line:   def.Line,
            Column: def.Column,
        }
    }
    
    // Search in imported modules (if multi-file support)
    // ...
}
```

---

### 5. Mermaid Diagram Preview

**What it does**: Show live Mermaid diagram preview as you code.

**Implementation Options**:

#### Option A: Webview Panel (Recommended - 4-6 hours)

**File**: `src/extension.ts`

```typescript
import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

let previewPanel: vscode.WebviewPanel | undefined = undefined;

export function activate(context: vscode.ExtensionContext) {
  // Command to show preview
  const previewCommand = vscode.commands.registerCommand('sruja.preview', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor || !editor.document.fileName.endsWith('.sruja')) {
      vscode.window.showWarningMessage('Open a .sruja file to preview');
      return;
    }

    // Create or reveal preview panel
    if (!previewPanel) {
      previewPanel = vscode.window.createWebviewPanel(
        'srujaPreview',
        'Sruja Diagram Preview',
        vscode.ViewColumn.Beside,
        {
          enableScripts: true,
          retainContextWhenHidden: true
        }
      );

      previewPanel.onDidDispose(() => {
        previewPanel = undefined;
      });
    }

    // Update preview
    updatePreview(editor.document);
    
    // Auto-update on document change
    const changeListener = vscode.workspace.onDidChangeTextDocument(e => {
      if (e.document === editor.document) {
        updatePreview(e.document);
      }
    });

    context.subscriptions.push(changeListener);
  });

  context.subscriptions.push(previewCommand);
}

async function updatePreview(document: vscode.TextDocument) {
  if (!previewPanel) return;

  try {
    // Compile to Mermaid
    const cmd = `sruja compile "${document.fileName}"`;
    const { stdout } = await execAsync(cmd);
    
    // Get Mermaid code
    const mermaidCode = stdout.trim();
    
    // Create HTML with Mermaid.js
    const html = getPreviewHtml(mermaidCode);
    previewPanel.webview.html = html;
  } catch (error) {
    previewPanel.webview.html = getErrorHtml(error.message);
  }
}

function getPreviewHtml(mermaidCode: string): string {
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
  <style>
    body {
      margin: 0;
      padding: 20px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      background: var(--vscode-editor-background);
      color: var(--vscode-editor-foreground);
    }
    .mermaid {
      background: white;
      padding: 20px;
      border-radius: 4px;
    }
    .error {
      color: var(--vscode-errorForeground);
      padding: 20px;
    }
  </style>
</head>
<body>
  <div class="mermaid">
${mermaidCode}
  </div>
  <script>
    mermaid.initialize({ 
      startOnLoad: true,
      theme: 'default',
      securityLevel: 'loose'
    });
  </script>
</body>
</html>`;
}

function getErrorHtml(error: string): string {
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <style>
    body {
      margin: 0;
      padding: 20px;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      background: var(--vscode-editor-background);
      color: var(--vscode-editor-foreground);
    }
    .error {
      color: var(--vscode-errorForeground);
      padding: 20px;
      border: 1px solid var(--vscode-errorForeground);
      border-radius: 4px;
    }
  </style>
</head>
<body>
  <div class="error">
    <h3>Preview Error</h3>
    <pre>${error}</pre>
  </div>
</body>
</html>`;
}
```

**Add command to `package.json`**:

```json
{
  "contributes": {
    "commands": [
      {
        "command": "sruja.preview",
        "title": "Show Diagram Preview",
        "icon": "$(preview)"
      }
    ],
    "menus": {
      "editor/title": [
        {
          "command": "sruja.preview",
          "when": "resourceExtname == .sruja",
          "group": "navigation"
        }
      ],
      "commandPalette": [
        {
          "command": "sruja.preview",
          "title": "Sruja: Show Diagram Preview"
        }
      ]
    }
  }
}
```

#### Option B: Custom Editor (Advanced - 1-2 days)

For a more integrated experience, you can create a custom editor that shows the diagram side-by-side:

```typescript
// Register custom editor
vscode.window.registerCustomEditorProvider('sruja.preview', {
  async resolveCustomTextEditor(document, webviewPanel) {
    // Similar to webview panel but more integrated
  }
});
```

**Recommendation**: Start with Option A (Webview Panel) - simpler and sufficient.

---

## Complete Extension Structure

```
.vscode-extension/
├── package.json
├── tsconfig.json
├── src/
│   └── extension.ts          # All providers here
├── syntaxes/
│   └── sruja.tmLanguage.json
├── snippets/
│   └── sruja.json
└── language-configuration.json
```

**Complete `extension.ts`**:

```typescript
import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export function activate(context: vscode.ExtensionContext) {
  // Hover
  const hoverProvider = vscode.languages.registerHoverProvider('sruja', {
    async provideHover(document, position) {
      // ... hover implementation
    }
  });
  
  // Keyword completion
  const keywordProvider = vscode.languages.registerCompletionItemProvider('sruja', {
    provideCompletionItems() {
      // ... keyword completion
    }
  }, ' ', '\t', '\n');
  
  // Context-aware completion
  const contextAwareProvider = vscode.languages.registerCompletionItemProvider('sruja', {
    async provideCompletionItems(document, position) {
      // ... context-aware completion
    }
  }, ' ', '\t', '\n', '>', '-', ':');
  
  // Go-to-definition
  const definitionProvider = vscode.languages.registerDefinitionProvider('sruja', {
    async provideDefinition(document, position) {
      // ... go-to-definition
    }
  });
  
  // Mermaid preview command
  const previewCommand = vscode.commands.registerCommand('sruja.preview', () => {
    // ... preview implementation
  });
  
  context.subscriptions.push(
    hoverProvider,
    keywordProvider,
    contextAwareProvider,
    definitionProvider,
    previewCommand
  );
}

export function deactivate() {}
```

---

## Go CLI Commands to Implement

Add these commands to `cmd/sruja/`:

1. **`hover.go`** - Returns element info at position
2. **`complete.go`** - Returns completion suggestions
3. **`definition.go`** - Returns definition location

All commands:
- Take file, line, column as arguments
- Parse the file
- Return JSON output
- Handle errors gracefully

---

## Testing

### Test Hover
1. Open a `.sruja` file
2. Hover over an element name
3. Should show element info

### Test Completion
1. Type `sys` and press Ctrl+Space
2. Should suggest `system`
3. Type `->` and press Ctrl+Space
4. Should suggest element names

### Test Go-to-Definition
1. Ctrl+Click on an element reference
2. Should jump to definition

---

## Timeline

- **Day 1**: Foundation (syntax highlighting, snippets)
- **Day 2**: Hover + Keyword completion
- **Day 2-3**: Mermaid diagram preview
- **Day 3-4**: Context-aware completion
- **Day 4-5**: Go-to-definition

**Total**: 4-5 days for all essential features

## Usage

### Opening Preview

1. **Command Palette**: `Ctrl+Shift+P` → "Sruja: Show Diagram Preview"
2. **Editor Title Bar**: Click preview icon (if configured)
3. **Keyboard Shortcut**: Add custom keybinding (optional)

### Auto-Update

Preview automatically updates when you:
- Type in the `.sruja` file
- Save the file
- Switch between tabs

### Preview Features

- ✅ Live Mermaid diagram rendering
- ✅ Auto-refresh on changes
- ✅ Side-by-side view (opens in adjacent column)
- ✅ Error display if compilation fails
- ✅ Responsive layout

---

*All features are essential for developer productivity. Implement all of them.*


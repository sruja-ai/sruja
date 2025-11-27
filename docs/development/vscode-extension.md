# VS Code Extension (Local Development)

**Purpose**: Local development productivity tool, **not for publishing**

This extension provides essential productivity features for developing Sruja language locally.

[← Back to Documentation Index](../README.md)

## Overview

A **developer productivity-focused** VS Code extension for local development that provides:
- ✅ Syntax highlighting for `.sruja` files
- ✅ Code snippets for common patterns
- ✅ File association for `.sruja` files
- ✅ **Hover documentation** (essential for productivity)
- ✅ **Code completion** (essential for productivity)
- ✅ **Go-to-definition** (essential for productivity)
- ✅ **Mermaid diagram preview** (essential for productivity - see diagram as you code)
- ✅ Basic validation (via CLI integration)

**Note**: This is a local development extension (not for publishing), but includes all essential productivity features.

---

## Features

### 1. Syntax Highlighting

**File**: `syntaxes/sruja.tmLanguage.json`

TextMate grammar for `.sruja` files:

```json
{
  "scopeName": "source.sruja",
  "fileTypes": ["sruja"],
  "patterns": [
    {
      "match": "\\b(workspace|model|system|container|component|requirement|adr)\\b",
      "name": "keyword.control.sruja"
    },
    {
      "match": "->",
      "name": "keyword.operator.relation.sruja"
    },
    {
      "match": "\"[^\"]*\"",
      "name": "string.quoted.double.sruja"
    },
    {
      "match": "//.*$",
      "name": "comment.line.sruja"
    }
  ]
}
```

### 2. Code Snippets

**File**: `snippets/sruja.json`

Common patterns:

```json
{
  "Workspace": {
    "prefix": "workspace",
    "body": [
      "workspace {",
      "  model {",
      "    $0",
      "  }",
      "}"
    ],
    "description": "Workspace block"
  },
  "System": {
    "prefix": "system",
    "body": [
      "system ${1:Name} \"${2:Label}\" {",
      "  $0",
      "}"
    ],
    "description": "System declaration"
  },
  "Container": {
    "prefix": "container",
    "body": [
      "container ${1:Name} \"${2:Label}\" {",
      "  technology \"${3:Tech}\"",
      "  $0",
      "}"
    ],
    "description": "Container declaration"
  },
  "Component": {
    "prefix": "component",
    "body": [
      "component ${1:Name} \"${2:Label}\"",
      "$0"
    ],
    "description": "Component declaration"
  },
  "Relation": {
    "prefix": "rel",
    "body": [
      "${1:From} -> ${2:To} \"${3:Label}\"",
      "$0"
    ],
    "description": "Relationship"
  },
  "Requirement": {
    "prefix": "req",
    "body": [
      "requirement ${1:R1}: ${2|functional,constraint,performance,security|} \"${3:Description}\"",
      "$0"
    ],
    "description": "Requirement"
  },
  "ADR": {
    "prefix": "adr",
    "body": [
      "adr ${1:ADR001}: \"${2:Decision}\"",
      "$0"
    ],
    "description": "Architecture Decision Record"
  }
}
```

### 3. File Association

**File**: `package.json`

```json
{
  "contributes": {
    "languages": [
      {
        "id": "sruja",
        "aliases": ["Sruja", "sruja"],
        "extensions": [".sruja"],
        "configuration": "./language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "sruja",
        "scopeName": "source.sruja",
        "path": "./syntaxes/sruja.tmLanguage.json"
      }
    ],
    "snippets": [
      {
        "language": "sruja",
        "path": "./snippets/sruja.json"
      }
    ]
  }
}
```

### 4. Basic Validation (Optional)

**File**: `package.json` (add task)

```json
{
  "tasks": {
    "tasks": [
      {
        "label": "sruja: validate",
        "type": "shell",
        "command": "sruja",
        "args": ["validate", "${file}"],
        "problemMatcher": {
          "owner": "sruja",
          "fileLocation": ["relative", "${workspaceFolder}"],
          "pattern": {
            "regexp": "^(.+):(\\d+):(\\d+):\\s+(.+)$",
            "file": 1,
            "line": 2,
            "column": 3,
            "message": 4
          }
        }
      }
    ]
  }
}
```

---

## Project Structure

```
.vscode-extension/
├── package.json              # Extension manifest
├── language-configuration.json  # Language config (brackets, comments)
├── syntaxes/
│   └── sruja.tmLanguage.json   # TextMate grammar
├── snippets/
│   └── sruja.json              # Code snippets
└── README.md                   # Extension docs
```

---

## Implementation

### Step 1: Initialize Extension

```bash
cd .vscode-extension
npm init -y
npm install --save-dev @types/vscode
```

### Step 2: Create `package.json`

```json
{
  "name": "sruja-local",
  "displayName": "Sruja (Local Dev)",
  "description": "Local development extension for Sruja language",
  "version": "0.0.1",
  "engines": {
    "vscode": "^1.80.0"
  },
  "categories": ["Programming Languages"],
  "contributes": {
    "languages": [
      {
        "id": "sruja",
        "aliases": ["Sruja", "sruja"],
        "extensions": [".sruja"],
        "configuration": "./language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "sruja",
        "scopeName": "source.sruja",
        "path": "./syntaxes/sruja.tmLanguage.json"
      }
    ],
    "snippets": [
      {
        "language": "sruja",
        "path": "./snippets/sruja.json"
      }
    ]
  }
}
```

### Step 3: Create Language Configuration

**File**: `language-configuration.json`

```json
{
  "comments": {
    "lineComment": "//"
  },
  "brackets": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"]
  ],
  "autoClosingPairs": [
    { "open": "{", "close": "}" },
    { "open": "[", "close": "]" },
    { "open": "(", "close": ")" },
    { "open": "\"", "close": "\"" }
  ],
  "surroundingPairs": [
    { "open": "{", "close": "}" },
    { "open": "[", "close": "]" },
    { "open": "(", "close": ")" },
    { "open": "\"", "close": "\"" }
  ]
}
```

### Step 4: Create TextMate Grammar

**File**: `syntaxes/sruja.tmLanguage.json`

Use a TextMate grammar generator or create manually. See [TextMate Grammar Guide](https://macromates.com/manual/en/language_grammars).

### Step 5: Create Snippets

**File**: `snippets/sruja.json`

Add common patterns (see examples above).

---

## Development Workflow

### Local Development

1. **Open extension folder**:
   ```bash
   code .vscode-extension
   ```

2. **Press F5** to launch Extension Development Host

3. **Test features**:
   - Open a `.sruja` file
   - Check syntax highlighting
   - Test snippets
   - Verify file association

### Testing

- Create test `.sruja` files
- Verify syntax highlighting works
- Test all snippets
- Check language configuration (comments, brackets)

---

## Integration with Go CLI

The extension can call the Go CLI tool for validation:

```json
{
  "tasks": {
    "tasks": [
      {
        "label": "sruja: compile",
        "type": "shell",
        "command": "sruja",
        "args": ["compile", "${file}"],
        "group": "build"
      },
      {
        "label": "sruja: validate",
        "type": "shell",
        "command": "sruja",
        "args": ["validate", "${file}"],
        "group": "test"
      }
    ]
  }
}
```

**Usage**:
- Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)
- Type "Tasks: Run Task"
- Select "sruja: validate" or "sruja: compile"

---

## Essential Features for Developer Productivity

All features are implemented via CLI integration (calling Go CLI tool), keeping the extension simple while maximizing productivity:

**Implementation Strategy**: VS Code extension calls Go CLI commands, which do all the parsing and analysis. This keeps the extension simple (TypeScript/Node.js) while leveraging the Go tooling.

---

## Optional Features: Difficulty Assessment

### 1. Hover Documentation (Easy - 2-4 hours)

**What it does**: Show element information when hovering over identifiers.

**Implementation**:
- Call Go CLI: `sruja hover <file> <line> <column>`
- CLI returns JSON with element info
- Display in VS Code hover

**Code**:
```typescript
// extension.ts
vscode.languages.registerHoverProvider('sruja', {
  async provideHover(document, position) {
    const { exec } = require('child_process');
    const cmd = `sruja hover "${document.fileName}" ${position.line + 1} ${position.character + 1}`;
    
    return new Promise((resolve) => {
      exec(cmd, (error, stdout) => {
        if (error) return resolve(null);
        const info = JSON.parse(stdout);
        resolve(new vscode.Hover(info.description));
      });
    });
  }
});
```

**Go CLI command needed**:
```go
// cmd/sruja/hover.go
func hoverCommand(file string, line, col int) {
    // Parse file
    // Find element at position
    // Return JSON with description, type, etc.
}
```

**Difficulty**: ⭐ Easy (2-4 hours)
- Simple CLI integration
- No complex parsing needed
- Just find element at position

---

### 2. Code Completion (Moderate - 1-2 days)

**What it does**: Suggest keywords, element names, and properties as you type.

**Implementation Options**:

#### Option A: Simple Keyword Completion (Easy - 2-4 hours)
```typescript
vscode.languages.registerCompletionItemProvider('sruja', {
  provideCompletionItems() {
    return [
      new vscode.CompletionItem('workspace', vscode.CompletionItemKind.Keyword),
      new vscode.CompletionItem('system', vscode.CompletionItemKind.Keyword),
      new vscode.CompletionItem('container', vscode.CompletionItemKind.Keyword),
      // ... more keywords
    ];
  }
});
```

**Difficulty**: ⭐ Easy (2-4 hours)
- Static keyword list
- No parsing needed

#### Option B: Context-Aware Completion (Moderate - 1-2 days)
```typescript
vscode.languages.registerCompletionItemProvider('sruja', {
  async provideCompletionItems(document, position) {
    // Call Go CLI to get context
    const cmd = `sruja complete "${document.fileName}" ${position.line + 1} ${position.character + 1}`;
    const suggestions = await execCLI(cmd);
    return suggestions.map(s => new vscode.CompletionItem(s.text, s.kind));
  }
});
```

**Go CLI command needed**:
```go
// cmd/sruja/complete.go
func completeCommand(file string, line, col int) {
    // Parse file up to cursor
    // Determine context (in workspace? in system? after ->?)
    // Return suggestions:
    //   - Keywords (if at start of statement)
    //   - Element names (if after -> or in reference)
    //   - Properties (if in metadata block)
}
```

**Difficulty**: ⭐⭐ Moderate (1-2 days)
- Need to parse partial file
- Context detection
- AST traversal to find available elements

---

### 3. Go-to-Definition (Moderate - 1-2 days)

**What it does**: Jump to element definition when clicking on references.

**Implementation**:
```typescript
vscode.languages.registerDefinitionProvider('sruja', {
  async provideDefinition(document, position) {
    const cmd = `sruja definition "${document.fileName}" ${position.line + 1} ${position.character + 1}`;
    const result = await execCLI(cmd);
    
    if (result.file && result.line) {
      return new vscode.Location(
        vscode.Uri.file(result.file),
        new vscode.Position(result.line - 1, result.column - 1)
      );
    }
    return null;
  }
});
```

**Go CLI command needed**:
```go
// cmd/sruja/definition.go
func definitionCommand(file string, line, col int) {
    // Parse file
    // Find identifier at position
    // Resolve reference (find definition)
    // Return file, line, column of definition
}
```

**Difficulty**: ⭐⭐ Moderate (1-2 days)
- Need to parse full file
- Reference resolution
- Cross-file support (if multi-file) adds complexity

---

## Implementation Summary

| Feature | Difficulty | Time | Notes |
|---------|-----------|------|-------|
| **Hover** | ⭐ Easy | 2-4 hours | Simple CLI integration |
| **Code Completion (keywords)** | ⭐ Easy | 2-4 hours | Static list |
| **Mermaid Preview** | ⭐⭐ Moderate | 4-6 hours | Webview panel with auto-refresh |
| **Code Completion (context-aware)** | ⭐⭐ Moderate | 1-2 days | Needs parsing |
| **Go-to-Definition** | ⭐⭐ Moderate | 1-2 days | Needs reference resolution |

**Total for all features**: ~4-5 days of work

## Implementation Priority

**All features are essential for developer productivity** - implement in this order:

1. ✅ **Hover** (easiest, high value) - Start here
2. ✅ **Keyword completion** (easy, good UX) - Quick win
3. ✅ **Context-aware completion** (moderate, high value) - Essential for productivity
4. ✅ **Go-to-definition** (moderate, high value) - Essential for navigation

**Total implementation time**: ~3-4 days for all features

**Why all features?**
- Developer productivity is critical for language adoption
- All features use CLI integration (simple to implement)
- High ROI - significantly improves development experience
- Makes the language feel professional and polished

---

## Implementation Guide

See [VS Code Extension Implementation Plan](./vscode-extension-implementation.md) for detailed implementation steps for all essential features.

## Future: Full LSP Extension (Optional)

If you need full LSP support later (for published extension):

1. **Option A**: Implement LSP server in Go, connect via VS Code LSP client
2. **Option B**: Implement LSP server in TypeScript (see `ui-future/lsp-architecture.md`)

For local development, CLI integration is sufficient and simpler.

---

## Quick Start

1. **Create extension folder**:
   ```bash
   mkdir -p .vscode-extension
   cd .vscode-extension
   ```

2. **Initialize**:
   ```bash
   npm init -y
   npm install --save-dev @types/vscode
   ```

3. **Create files**:
   - `package.json` (extension manifest)
   - `language-configuration.json`
   - `syntaxes/sruja.tmLanguage.json`
   - `snippets/sruja.json`

4. **Test**:
   - Press F5 in VS Code
   - Open a `.sruja` file
   - Verify syntax highlighting and snippets work

---

## Resources

- [VS Code Extension API](https://code.visualstudio.com/api)
- [TextMate Grammar Guide](https://macromates.com/manual/en/language_grammars)
- [VS Code Snippets](https://code.visualstudio.com/docs/editor/userdefinedsnippets)
- [Language Configuration](https://code.visualstudio.com/api/language-extensions/language-configuration-guide)

---

*This extension is for local development productivity only. Use the Go CLI tool for advanced features.*


---
title: "VS Code Extension"
weight: 15
summary: "VS Code support for Sruja DSL with bundled WASM validation, previews, and CLI-backed context engineering."
---

# VS Code Extension

The Sruja VS Code extension provides language support for `.sruja` files with **bundled WebAssembly** for validation, formatting, export, and preview flows. CLI-backed commands such as **refresh context**, **status**, **review**, **drift**, and **MCP registration** work when the Sruja CLI is installed and available on `PATH` or configured via `sruja.lsp.path`.

## Installation

### From VS Code Marketplace

**[Install directly](https://marketplace.visualstudio.com/items?itemName=SrujaAI.sruja)** (opens the extension page), or:

1. Open VS Code
2. Go to Extensions (`Cmd+Shift+X` or `Ctrl+Shift+X`)
3. Search for "Sruja"
4. Click **Install** on "Sruja"

### From VSIX File

1. Download the `.vsix` file from [GitHub Releases](https://github.com/sruja-ai/sruja/releases)
2. In VS Code, go to Extensions
3. Click the `...` menu → "Install from VSIX..."
4. Select the downloaded `.vsix` file

## Features

The extension provides a complete LSP implementation with the following features:

### ✨ Core Editing Features

#### 1. **Diagnostics** (Errors & Warnings)

- Real-time error detection as you type
- Syntax errors highlighted immediately
- Validation errors (duplicate IDs, invalid references, etc.)
- Hover over errors to see detailed messages

#### 2. **Hover Information**

- Hover over any symbol to see its definition
- Shows system/container/component information
- Displays labels and descriptions

**Usage**: Hover your mouse over any identifier

#### 3. **Autocomplete** (IntelliSense)

- Keyword suggestions (`architecture`, `system`, `container`, etc.)
- Symbol completion (suggests existing systems, containers, components)
- Context-aware suggestions

**Usage**: Type `Ctrl+Space` (or `Cmd+Space` on Mac) to trigger autocomplete

#### 4. **Go to Definition**

- Navigate to symbol definitions with `F12` or `Cmd+Click`
- Works with qualified names (e.g., `App.API`)
- Supports both root and child paths in FQNs

**Usage**:

- `F12` - Go to definition
- `Cmd+Click` (Mac) or `Ctrl+Click` (Windows/Linux) - Go to definition
- `Cmd+F12` - Go to implementation

#### 5. **Find All References**

- Find all places where a symbol is used
- Shows references in relations and qualified names
- Opens in the References panel

**Usage**:

- `Shift+F12` - Find all references
- Right-click → "Find All References"

#### 6. **Rename Symbol**

- Rename a symbol and all its references
- Updates both definitions and usages
- Handles qualified names correctly

**Usage**:

- `F2` - Rename symbol
- Right-click → "Rename Symbol"

#### 7. **Document Symbols** (Outline)

- View all symbols in the current file
- Navigate quickly through your architecture
- Shows hierarchy: systems → containers → components

**Usage**:

- `Cmd+Shift+O` (Mac) or `Ctrl+Shift+O` (Windows/Linux) - Go to symbol in file
- View → Outline

#### 8. **Workspace Symbols**

- Search for symbols across all open files
- Quick navigation to any symbol in your workspace

**Usage**:

- `Cmd+T` (Mac) or `Ctrl+T` (Windows/Linux) - Go to symbol in workspace

#### 9. **Code Formatting**

- Format your Sruja DSL code
- Consistent indentation and spacing

**Usage**:

- `Shift+Alt+F` (Windows/Linux) or `Shift+Option+F` (Mac) - Format document
- Right-click → "Format Document"

### 🎨 Additional Features

#### Syntax Highlighting

- Color-coded keywords, strings, and identifiers
- Makes code easier to read and understand

#### Architecture Previews

- **Open Diagram Preview** renders the active `.sruja` file as Mermaid in a side panel
- **Open Focused Diagram Preview** renders a sliced view around the selected element
- **Open Markdown Preview** opens the extension's custom markdown preview

**Usage**:

- Command palette: `Cmd+Shift+P` / `Ctrl+Shift+P`
- Run one of:
  - `Sruja: Open Diagram Preview`
  - `Sruja: Open Focused Diagram Preview`
  - `Sruja: Open Markdown Preview`

#### Register MCP Server (Cursor)

- Registers the Sruja MCP server with Cursor so Cursor can call Sruja tools (repomap, context, drift)
- Requires the Sruja CLI (configure `sruja.lsp.path` if needed)

**Usage**:

- Command palette: `Cmd+Shift+P` → "Sruja: Register MCP Server (Cursor)"

## Keyboard Shortcuts

| Feature                   | Mac                  | Windows/Linux         |
| ------------------------- | -------------------- | --------------------- |
| Go to Definition          | `F12` or `Cmd+Click` | `F12` or `Ctrl+Click` |
| Find All References       | `Shift+F12`          | `Shift+F12`           |
| Rename Symbol             | `F2`                 | `F2`                  |
| Go to Symbol in File      | `Cmd+Shift+O`        | `Ctrl+Shift+O`        |
| Go to Symbol in Workspace | `Cmd+T`              | `Ctrl+T`              |
| Format Document           | `Shift+Option+F`     | `Shift+Alt+F`         |
| Trigger Autocomplete      | `Cmd+Space`          | `Ctrl+Space`          |

## Configuration

The extension supports the following settings:

### Available Settings

- `sruja.lsp.path` - Optional absolute path to the Sruja CLI binary. This is used for CLI-backed workspace commands such as refresh context, status, review, drift, and MCP registration.
- `sruja.skills.path` - Optional path to a skills folder for AI rules. If unset, the extension uses the workspace `skills` folder or bundled skills when available.

To configure:

1. Open Settings (`Cmd+,` or `Ctrl+,`)
2. Search for "Sruja"
3. Adjust the formatting options as needed

## Debugging

If you encounter issues with the extension:

### Check Output Channel

1. Open Output panel: `View → Output`
2. Select **Sruja** from the dropdown
3. Check for command output and error messages

### Common Issues

**Extension not working?**

- Check the **Sruja** output channel for errors
- Ensure WASM files are present (should be installed automatically)
- Try reloading the window: `Cmd+Shift+P` → "Developer: Reload Window"

**LSP features not responding?**

- Check if the file has a `.sruja` extension
- Verify the language mode is set to "Sruja" (bottom-right of VS Code)
- Run **Sruja: Run validation** on the active file and check the Problems panel

**Rename not working?**

- Make sure you're clicking on a valid symbol (not a keyword)
- Check the output channel for error messages
- Confirm the file is valid first with **Sruja: Run validation**

## Architecture

The extension uses **WebAssembly (WASM)** for core language features, which means:

- ✅ **No CLI dependency for core language features** - Diagnostics, hover, completion, formatting, preview, and markdown export work without installing the Sruja CLI
- ✅ **Fast** - WASM provides near-native performance
- ✅ **Portable** - Same code runs in browser and VS Code
- ✅ **Self-contained** - All functionality bundled in the extension

The extension also bridges into the CLI for workspace-level context engineering: refresh repo context, review, drift, status, and MCP registration.

## What's Next?

- **Learn the DSL**: Check out the [Syntax Reference](docs/reference/syntax.md)
- **See Examples**: Browse [Example Architectures](docs/examples.md)
- **Get Help**: Join [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions)

## Contributing

Found a bug or have a feature request?

- **Report Issues**: [GitHub Issues](https://github.com/sruja-ai/sruja/issues)
- **Suggest Features**: [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions)
- **Contribute Code**: See [Contribution Guide](https://github.com/sruja-ai/sruja/blob/main/docs/CONTRIBUTING.md)

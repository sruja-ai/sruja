---
title: "VS Code Extension"
weight: 15
summary: "Full-featured Language Server Protocol (LSP) support for Sruja DSL in VS Code."
---

# VS Code Extension

The Sruja VS Code extension provides comprehensive language support for `.sruja` files with full LSP (Language Server Protocol) features powered by WebAssembly.

## Installation

### From VS Code Marketplace

1. Open VS Code
2. Go to Extensions (`Cmd+Shift+X` or `Ctrl+Shift+X`)
3. Search for "Sruja"
4. Click **Install** on "Sruja DSL Language Support"

### From VSIX File

1. Download the `.vsix` file from [GitHub Releases](https://github.com/sruja-ai/sruja/releases)
2. In VS Code, go to Extensions
3. Click the `...` menu → "Install from VSIX..."
4. Select the downloaded `.vsix` file

## Features

The extension provides a complete LSP implementation with the following features:

### ✨ Core LSP Features

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

#### Preview Architecture

- Generate visual previews of your architecture
- Opens in VS Code's markdown preview

**Usage**:

- Right-click on a `.sruja` file → "Preview Sruja Architecture"
- Or use the command palette: `Cmd+Shift+P` → "Sruja: Preview Architecture"

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

### Formatting Options

- `sruja.formatting.enabled` (default: `true`) - Enable automatic formatting
- `sruja.formatting.tabSize` (default: `2`) - Number of spaces per indentation level
- `sruja.formatting.insertSpaces` (default: `true`) - Use spaces instead of tabs

To configure:

1. Open Settings (`Cmd+,` or `Ctrl+,`)
2. Search for "Sruja"
3. Adjust the formatting options as needed

## Debugging

If you encounter issues with the extension:

### Check Output Channel

1. Open Output panel: `View → Output`
2. Select "Sruja WASM LSP" from the dropdown
3. Check for error messages or initialization issues

### Debug Command

1. Open Command Palette: `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Run: "Sruja: Debug WASM LSP"
3. This will test all LSP functions and show results in the output channel

### Common Issues

**Extension not working?**

- Check the "Sruja WASM LSP" output channel for errors
- Ensure WASM files are present (should be installed automatically)
- Try reloading the window: `Cmd+Shift+P` → "Developer: Reload Window"

**LSP features not responding?**

- Check if the file has a `.sruja` extension
- Verify the language mode is set to "Sruja" (bottom-right of VS Code)
- Run the debug command to see which functions are failing

**Rename not working?**

- Make sure you're clicking on a valid symbol (not a keyword)
- Check the output channel for error messages
- Try the debug command to test rename functionality

## Architecture

The extension uses **WebAssembly (WASM)** for all LSP functionality, which means:

- ✅ **No CLI dependency** - Works without installing the Sruja CLI
- ✅ **Fast** - WASM provides near-native performance
- ✅ **Portable** - Same code runs in browser and VS Code
- ✅ **Self-contained** - All functionality bundled in the extension

The LSP implementation is powered by the same Go codebase that runs the CLI, compiled to WASM for use in Node.js environments.

## What's Next?

- **Learn the DSL**: Check out the [Syntax Reference](/docs/reference/syntax)
- **See Examples**: Browse [Example Architectures](/docs/examples)
- **Get Help**: Join [Discord](https://discord.gg/VNrvHPV5) or [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions)

## Contributing

Found a bug or have a feature request?

- **Report Issues**: [GitHub Issues](https://github.com/sruja-ai/sruja/issues)
- **Suggest Features**: [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions)
- **Contribute Code**: See [Contribution Guide](https://github.com/sruja-ai/sruja/blob/main/docs/CONTRIBUTING.md)

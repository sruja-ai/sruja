# Sruja for VS Code

Professional language support for the Sruja architecture-as-code DSL, including comprehensive IntelliSense, real-time validation, and interactive documentation preview.

[![VS Code Version](https://img.shields.io/badge/VS%20Code-%3E%3D1.75.0-blue)](https://code.visualstudio.com/)
[![License](https://img.shields.io/badge/license-Apache%202.0-green)](LICENSE.txt)

## Features

### Core Language Features

- 🎨 **Syntax Highlighting** - Full syntax support for `.sruja` files with semantic tokens
- 🧠 **IntelliSense** - Auto-completion, hover information, and go-to-definition
- ✅ **Real-time Validation** - Instant error detection and diagnostics with debounced updates
- 📝 **Code Formatting** - Auto-format your Sruja code with configurable options
- 👁️ **Architecture Preview** - Preview architecture documentation with embedded C4 diagrams
- 🔍 **Symbol Navigation** - Outline view, workspace symbols, and find all references
- ✏️ **Refactoring** - Rename symbols across your architecture
- 🎯 **Code Actions** - Quick fixes and code suggestions

### Advanced Features

- 💡 **Inlay Hints** - See element types inline as you code
- 🔗 **Code Lenses** - View reference counts above symbols
- 📋 **Signature Help** - Parameter hints during typing
- 🎨 **Semantic Highlighting** - Enhanced syntax highlighting based on code meaning
- 📊 **Status Bar Integration** - See error/warning counts at a glance
- ⚡ **Performance Optimized** - Intelligent caching and debouncing for smooth experience

## Prerequisites

No prerequisites required! The extension uses WASM and works out of the box.

## Quick Start

1. **Install the extension** from the VS Code Marketplace
2. **Open a `.sruja` file** - Syntax highlighting and IntelliSense activate automatically
3. **Start coding** - The language server provides real-time feedback
4. **Preview your architecture** - Right-click on a `.sruja` file and select "Preview Sruja Architecture"

## Usage Examples

### Basic Architecture Definition

```sruja
architecture "E-Commerce Platform" {
  person Customer "Customer"

  system OrderService "Order Service" {
    container OrderAPI "Order API" {
      technology "Spring Boot"
    }
  }

  Customer -> OrderService.OrderAPI "places orders"
}
```

### Using IntelliSense

- **Auto-completion**: Type `sys` and press `Ctrl+Space` to see system-related completions
- **Hover**: Hover over any symbol to see its definition and documentation
- **Go-to-definition**: `F12` or `Ctrl+Click` to jump to symbol definitions
- **Find references**: `Shift+F12` to find all usages of a symbol
- **Rename**: `F2` to rename a symbol across your architecture

### Preview Architecture

1. Open a `.sruja` file
2. Right-click in the editor or file explorer
3. Select "Preview Sruja Architecture"
4. View the generated markdown with C4 diagrams

## Commands

- **`Sruja: Preview Architecture`** - Generate and preview markdown export with C4 diagrams
- **`Sruja: Debug WASM LSP`** - Debug and test WASM LSP functionality (for troubleshooting)

## Configuration

Configure the extension in VS Code settings:

```json
{
  "sruja.formatting.enabled": true,
  "sruja.formatting.tabSize": 2,
  "sruja.formatting.insertSpaces": true
}
```

## Troubleshooting

### Extension Not Working

1. **Check Status Bar**: Look for "Sruja: Ready" in the bottom-right status bar
2. **Check Output**: View → Output → Select "Sruja WASM LSP" from dropdown
3. **Reload Window**: `Ctrl+Shift+P` → "Developer: Reload Window"

### Preview Not Working

1. Ensure the `.sruja` file is saved (not untitled)
2. Check that the file contains valid Sruja DSL syntax
3. Check extension output for errors (View → Output → "Sruja WASM LSP")
4. Try the debug command: `Sruja: Debug WASM LSP`

### Performance Issues

- The extension uses intelligent caching (5 second TTL)
- Diagnostics are debounced (300ms delay) for better performance
- Large files may take a moment to parse initially

### Common Issues

**"WASM initialization failed"**

- Ensure WASM files are bundled with extension
- Try reloading the window
- Check the output channel for detailed error messages

**"No diagnostics showing"**

- Ensure your `.sruja` file has valid syntax
- Check that the language server initialized (status bar should show "Ready")
- Try saving the file to trigger diagnostics

**"IntelliSense not working"**

- Ensure the file is saved (not untitled)
- Check that the language server is running (status bar)
- Try reloading the window

## Learn More

- [Sruja Documentation](https://sruja.ai)
- [GitHub Repository](https://github.com/sruja-ai/sruja)
- [Examples](https://github.com/sruja-ai/sruja/tree/main/examples)

Apache 2.0

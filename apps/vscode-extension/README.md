# Sruja DSL Language Support for VS Code

Language support for the Sruja architecture-as-code DSL, including syntax highlighting, IntelliSense, validation, and markdown preview.

## Features

- 🎨 **Syntax Highlighting** - Full syntax support for `.sruja` files
- 🧠 **IntelliSense** - Auto-completion, hover information, and go-to-definition
- ✅ **Validation** - Real-time error detection and diagnostics
- 📝 **Formatting** - Auto-format your Sruja code
- 👁️ **Markdown Preview** - Preview architecture documentation with embedded diagrams

## Prerequisites

No prerequisites required! The extension uses WASM and works out of the box.

## Usage

1. **Open a `.sruja` file** - Syntax highlighting activates automatically
2. **Language Server** - Starts automatically when you open `.sruja` files
3. **Preview Architecture** - Right-click on a `.sruja` file and select "Preview Sruja Architecture"

## Commands

- `Sruja: Preview Architecture` - Generate and preview markdown export

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

### Preview Not Working

1. Ensure the `.sruja` file is saved
2. Check extension output for errors
3. Verify WASM files are bundled with the extension

## Learn More

- [Sruja Documentation](https://sruja.ai)
- [GitHub Repository](https://github.com/sruja-ai/sruja)
- [Examples](https://github.com/sruja-ai/sruja/tree/main/examples)

## License

MIT

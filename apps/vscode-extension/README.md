# Sruja DSL Language Support for VS Code

Language support for the Sruja architecture-as-code DSL, including syntax highlighting, IntelliSense, validation, and markdown preview.

## Features

- 🎨 **Syntax Highlighting** - Full syntax support for `.sruja` files
- 🧠 **IntelliSense** - Auto-completion, hover information, and go-to-definition
- ✅ **Validation** - Real-time error detection and diagnostics
- 📝 **Formatting** - Auto-format your Sruja code
- 👁️ **Markdown Preview** - Preview architecture documentation with embedded diagrams

## Prerequisites

This extension requires the `sruja` CLI tool to be installed on your system.

### Install Sruja CLI

**Automated Install (Recommended):**
```bash
curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
```

**Manual Download:**
Download from [GitHub Releases](https://github.com/sruja-ai/sruja/releases)

**From Source:**
```bash
go install github.com/sruja-ai/sruja/cmd/sruja@latest
```

Verify installation:
```bash
sruja --version
```

## Usage

1. **Open a `.sruja` file** - Syntax highlighting activates automatically
2. **Language Server** - Starts automatically when you open `.sruja` files
3. **Preview Architecture** - Right-click on a `.sruja` file and select "Preview Sruja Architecture"

## Commands

- `Sruja: Restart Language Server` - Restart the LSP server
- `Sruja: Show Language Server Output` - View server logs
- `Sruja: Preview Architecture` - Generate and preview markdown export

## Configuration

Configure the extension in VS Code settings:

```json
{
  "srujaLanguageServer.path": "sruja",  // Path to sruja executable (default: "sruja")
  "srujaLanguageServer.enableLogging": false,
  "srujaLanguageServer.logLevel": "info",
  "sruja.formatting.enabled": true,
  "sruja.formatting.tabSize": 2,
  "sruja.formatting.insertSpaces": true
}
```

## Troubleshooting

### Language Server Not Starting

1. Verify `sruja` is installed: `sruja --version`
2. Check if `sruja` is in your PATH
3. Set custom path in settings: `"srujaLanguageServer.path": "/path/to/sruja"`
4. Check output channel: `Sruja: Show Language Server Output`

### Preview Not Working

1. Ensure the `.sruja` file is saved
2. Verify `sruja export markdown` works from command line
3. Check extension output for errors

## Learn More

- [Sruja Documentation](https://sruja.ai)
- [GitHub Repository](https://github.com/sruja-ai/sruja)
- [Examples](https://github.com/sruja-ai/sruja/tree/main/examples)

## License

MIT

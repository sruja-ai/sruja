# Using Sruja LSP in Cursor

Cursor (based on VS Code) can use the Sruja Language Server Protocol (LSP) for features like:
- Syntax diagnostics (error highlighting)
- Code completion
- Hover information
- Go to definition

## Quick Setup

### Option 1: Use Local Binary (Recommended)

The extension automatically detects the `sruja-lsp` binary in your workspace root:

1. **Build the LSP binary** (if not already built):
```bash
cd /Users/dilipkola/Workspace/sruja
go build -o sruja-lsp ./cmd/sruja-lsp
```

2. **Install the VS Code extension in Cursor**:
   - Open Cursor
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Type "Extensions: Install from VSIX..." or "Install Extension"
   - Navigate to `.vscode-extension` folder
   - Or manually install:
     ```bash
     # In Cursor, open Command Palette (Cmd+Shift+P)
     # Select "Extensions: Install from VSIX..."
     # Choose: .vscode-extension/package.json (or build a VSIX)
     ```

3. **Alternative: Install extension from folder**:
   - In Cursor, open Command Palette (`Cmd+Shift+P`)
   - Run: `Developer: Install Extension from Location...`
   - Select the `.vscode-extension` folder

### Option 2: Install Binary Globally

If you want `sruja-lsp` available system-wide:

```bash
# Build the binary
go build -o sruja-lsp ./cmd/sruja-lsp

# Install to a location in your PATH
sudo cp sruja-lsp /usr/local/bin/
# Or for user-local installation:
mkdir -p ~/.local/bin
cp sruja-lsp ~/.local/bin/
# Add to PATH in ~/.zshrc or ~/.bashrc:
# export PATH="$HOME/.local/bin:$PATH"
```

## Verify Installation

1. **Open a `.sruja` file** in Cursor
2. **Check the status bar** - you should see "Sruja LSP" or similar
3. **Test features**:
   - Type some code - you should see syntax errors highlighted
   - Hover over identifiers - should show information
   - Type `system` or `container` - should see autocomplete

## Troubleshooting

### LSP Not Starting

1. **Check if binary exists**:
```bash
ls -la sruja-lsp
```

2. **Check Cursor output**:
   - Open Command Palette (`Cmd+Shift+P`)
   - Run: `Output: Show Output Channel`
   - Select "Sruja LSP" from dropdown
   - Look for error messages

3. **Test binary manually**:
```bash
# The LSP uses stdio, so test with:
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./sruja-lsp
```

### Extension Not Activating

1. **Check file association**: Make sure `.sruja` files are recognized
2. **Reload Cursor**: `Cmd+Shift+P` → "Developer: Reload Window"
3. **Check extension logs**: View → Output → Select "Sruja" channel

## Manual Configuration

If automatic detection doesn't work, you can manually configure the LSP path in Cursor settings:

1. Open Cursor Settings (`Cmd+,`)
2. Search for "sruja"
3. Or edit `.vscode/settings.json`:
```json
{
  "sruja.lsp.path": "/absolute/path/to/sruja-lsp"
}
```

## Features Available

- ✅ **Diagnostics**: Real-time syntax error detection
- ✅ **Completion**: Keyword and identifier suggestions
- ✅ **Hover**: Information about identifiers
- ✅ **Definition**: Go to definition (jump to where identifiers are defined)
- ✅ **Workspace Index**: Cross-file symbol resolution

## Development Mode

For development, the extension can also use `go run`:

```bash
# The extension will automatically use:
go run ./cmd/sruja-lsp/main.go
```

This is useful when actively developing the LSP server.





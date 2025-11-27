# Using Sruja LSP with Trae Editor

Trae editor supports the Language Server Protocol (LSP), allowing you to use `sruja-lsp` for enhanced editing of `.sruja` files.

## Prerequisites

1. **Build the LSP binary**:
```bash
cd /Users/dilipkola/Workspace/sruja
go build -o sruja-lsp ./cmd/sruja-lsp
```

2. **Verify the binary works**:
```bash
./sruja-lsp
# (LSP uses stdio, so it will wait for input - this is normal)
```

## Configuration Steps

### Step 1: Find Your Binary Path

Get the absolute path to your `sruja-lsp` binary:

```bash
# From your workspace root
pwd
# Output: /Users/dilipkola/Workspace/sruja

# Full path to binary:
/Users/dilipkola/Workspace/sruja/sruja-lsp
```

### Step 2: Configure Trae Editor

1. **Open Trae Settings**:
   - Go to Settings/Preferences (usually `Cmd+,` or `Ctrl+,`)
   - Navigate to "Language Servers" or "LSP" section

2. **Add Sruja Language Server**:
   - Click "Add Language Server" or similar
   - Configure with these settings:
     - **Name**: `sruja` or `Sruja LSP`
     - **Command/Executable**: `/Users/dilipkola/Workspace/sruja/sruja-lsp`
     - **Arguments**: (leave empty, or `[]`)
     - **File Patterns**: `*.sruja` or `**/*.sruja`
     - **Language ID**: `sruja` (if required)

3. **Transport Settings**:
   - **Transport**: `stdio` (standard input/output)
   - **Protocol**: `JSON-RPC 2.0` (usually automatic)

4. **Save and Restart**:
   - Save the configuration
   - Restart Trae editor

### Step 3: Verify Installation

1. **Open a `.sruja` file** in Trae
2. **Check for LSP features**:
   - Syntax errors should be highlighted
   - Hover over identifiers to see information
   - Type `system` or `container` to see autocomplete
   - Use "Go to Definition" on identifiers

## Alternative: Install Binary Globally

If you want the LSP available system-wide:

```bash
# Install to a location in your PATH
sudo cp sruja-lsp /usr/local/bin/
# Or for user-local installation:
mkdir -p ~/.local/bin
cp sruja-lsp ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"  # Add to ~/.zshrc or ~/.bashrc
```

Then use just `sruja-lsp` as the command in Trae settings.

## Troubleshooting

### LSP Not Starting

1. **Check binary permissions**:
```bash
chmod +x sruja-lsp
```

2. **Test binary manually**:
```bash
# The LSP communicates via stdio, so test with:
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}' | ./sruja-lsp
```

3. **Check Trae logs**:
   - Look for LSP-related errors in Trae's output/logs
   - Check if the binary path is correct

### Features Not Working

1. **Verify file association**: Make sure `.sruja` files are recognized
2. **Check LSP status**: Trae should show LSP connection status
3. **Restart LSP**: Some editors have a "Restart Language Server" command

## LSP Features Available

- ✅ **Diagnostics**: Real-time syntax error detection
- ✅ **Completion**: Keyword and identifier suggestions
- ✅ **Hover**: Information about identifiers (file location, line number)
- ✅ **Definition**: Go to definition (jump to where identifiers are defined)
- ✅ **Workspace Index**: Cross-file symbol resolution

## Configuration File Example

If Trae uses a configuration file (like `trae.json` or `.trae/config.json`), it might look like:

```json
{
  "languageServers": {
    "sruja": {
      "command": "/Users/dilipkola/Workspace/sruja/sruja-lsp",
      "args": [],
      "filePatterns": ["*.sruja"],
      "transport": "stdio"
    }
  }
}
```

## General LSP Setup (Any Editor)

Since `sruja-lsp` uses standard LSP over stdio, it works with any LSP-compatible editor:

### Key Configuration Values:
- **Command**: Path to `sruja-lsp` binary
- **Transport**: `stdio`
- **Protocol**: `JSON-RPC 2.0`
- **File Patterns**: `*.sruja`, `**/*.sruja`
- **Language ID**: `sruja`

### Supported Editors:
- ✅ Trae Editor
- ✅ VS Code / Cursor
- ✅ Neovim (with nvim-lspconfig)
- ✅ Emacs (with lsp-mode)
- ✅ Sublime Text (with LSP package)
- ✅ Any editor with LSP support


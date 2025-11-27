# Sruja LSP Setup for Any Editor

The `sruja-lsp` server uses standard Language Server Protocol (LSP) over stdio, making it compatible with any editor that supports LSP.

## Quick Reference

**Binary**: `sruja-lsp`  
**Transport**: `stdio` (standard input/output)  
**Protocol**: `JSON-RPC 2.0`  
**File Patterns**: `*.sruja`  
**Language ID**: `sruja`

## Build the LSP Binary

```bash
cd /Users/dilipkola/Workspace/sruja
go build -o sruja-lsp ./cmd/sruja-lsp
```

## Editor-Specific Setup

### Neovim

Using `nvim-lspconfig`:

```lua
-- In your Neovim config (init.lua or ~/.config/nvim/init.lua)
local lspconfig = require('lspconfig')

lspconfig.sruja = {
  cmd = { '/path/to/sruja-lsp' },
  filetypes = { 'sruja' },
  root_dir = lspconfig.util.root_pattern('.git', 'sruja.json'),
  settings = {}
}
```

Or using Mason:

```lua
require("mason-lspconfig").setup({
  ensure_installed = {},
})

require("lspconfig").sruja.setup({
  cmd = { "/path/to/sruja-lsp" },
})
```

### Emacs

Using `lsp-mode`:

```elisp
;; In your Emacs config
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration
             '(".*\\.sruja$" . "sruja"))

(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "/path/to/sruja-lsp")
                  :major-modes '(sruja-mode)
                  :server-id 'sruja))
```

### Sublime Text

Using LSP package:

1. Install "LSP" package via Package Control
2. Open Command Palette → "Preferences: LSP Settings"
3. Add:

```json
{
  "clients": {
    "sruja": {
      "command": ["/path/to/sruja-lsp"],
      "enabled": true,
      "languageId": "sruja",
      "scopes": ["source.sruja"],
      "syntaxes": ["Packages/User/Sruja.sublime-syntax"]
    }
  }
}
```

### Vim/Neovim (vim-lsp)

```vim
" In your .vimrc or init.vim
if executable('sruja-lsp')
    au User lsp_setup call lsp#register_server({
        \ 'name': 'sruja',
        \ 'cmd': {server_info->['sruja-lsp']},
        \ 'whitelist': ['sruja'],
        \ })
endif
```

### Helix Editor

In `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "sruja"
scope = "source.sruja"
file-types = ["sruja"]
language-server = { command = "/path/to/sruja-lsp" }
```

### Kate Editor

In `~/.config/kate/lsp.json`:

```json
{
  "servers": {
    "sruja": {
      "command": ["/path/to/sruja-lsp"],
      "highlightingModeRegex": "^.*\\.sruja$"
    }
  }
}
```

## Testing the LSP

Test the LSP server manually:

```bash
# Send an initialize request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"rootUri":"file:///tmp"}}' | ./sruja-lsp

# The server should respond with capabilities
```

## LSP Features

The `sruja-lsp` server provides:

- **textDocument/diagnostic**: Syntax error detection
- **textDocument/completion**: Code completion
- **textDocument/hover**: Hover information
- **textDocument/definition**: Go to definition
- **textDocument/didOpen**: Document opened
- **textDocument/didChange**: Document changed
- **textDocument/didSave**: Document saved

## Troubleshooting

### Binary Not Found

```bash
# Make sure binary is executable
chmod +x sruja-lsp

# Test if it runs
./sruja-lsp
# (It will wait for stdio input - this is normal)
```

### LSP Not Connecting

1. Check the binary path is absolute
2. Verify file associations match `*.sruja`
3. Check editor logs for LSP errors
4. Ensure stdio transport is used (not TCP/WebSocket)

### Features Not Working

1. Verify LSP is initialized (check editor status)
2. Open a valid `.sruja` file
3. Check editor's LSP logs
4. Try restarting the language server

## Configuration Template

Most editors need:

```json
{
  "command": "/absolute/path/to/sruja-lsp",
  "args": [],
  "fileTypes": ["sruja"],
  "transport": "stdio"
}
```

Or in YAML:

```yaml
command: /absolute/path/to/sruja-lsp
args: []
fileTypes:
  - sruja
transport: stdio
```





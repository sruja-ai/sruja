# Trae Editor: Alternative Setup (LSP Only)

Since Trae might not support VS Code extensions, use the LSP directly for error highlighting and diagnostics.

## Quick Setup

### Step 1: Build the LSP Binary

```bash
cd /Users/dilipkola/Workspace/sruja
go build -o sruja-lsp ./cmd/sruja-lsp
```

### Step 2: Configure LSP in Trae

1. **Open Trae Settings**
   - Go to Settings/Preferences
   - Navigate to "Language Servers" or "LSP" section

2. **Add Sruja Language Server**:
   - **Name**: `sruja` or `Sruja LSP`
   - **Command/Executable**: `/Users/dilipkola/Workspace/sruja/sruja-lsp`
   - **Arguments**: (leave empty)
   - **File Patterns**: `*.sruja` or `**/*.sruja`
   - **Language ID**: `sruja` (if required)
   - **Transport**: `stdio`
   - **Protocol**: `JSON-RPC 2.0`

3. **Save and Restart Trae**

### Step 3: Verify LSP is Working

1. Open a `.sruja` file
2. You should see:
   - Syntax errors highlighted with red squiggles
   - Validation errors shown inline
   - Hover information on identifiers

## For Diagram Preview (Without Extension)

Since the extension might not work in Trae, use the CLI directly:

### Option 1: Terminal Command

```bash
# In Trae's integrated terminal
sruja compile examples/person.sruja
```

This outputs Mermaid code that you can:
- Copy to a markdown file
- View in a Mermaid preview extension
- Use in documentation

### Option 2: Create a Trae Task

Create `.trae/tasks.json` (or similar):

```json
{
  "tasks": [
    {
      "label": "Sruja: Compile to Mermaid",
      "type": "shell",
      "command": "sruja",
      "args": ["compile", "${file}"],
      "problemMatcher": []
    }
  ]
}
```

Then use `Cmd+Shift+P` → "Run Task" → "Sruja: Compile to Mermaid"

### Option 3: Use External Tool

1. Install a Mermaid preview extension (if Trae supports it)
2. Run `sruja compile file.sruja > diagram.mmd`
3. Open `diagram.mmd` in the preview

## Why Extension Might Not Work

Trae might:
- Not support VS Code extensions at all
- Require extensions to be packaged differently
- Have extension support disabled
- Need extensions to be installed via a marketplace

## Check Trae Documentation

Look for:
- "Extension support" in Trae settings
- "VS Code compatibility" features
- Extension installation instructions
- LSP configuration guide

## Recommended Approach for Trae

**Use LSP for error highlighting** (works great!)
**Use CLI for diagram generation** (simple and reliable)

This gives you:
- ✅ Real-time error detection
- ✅ Syntax highlighting (via LSP)
- ✅ Diagram generation (via CLI)
- ✅ No extension compatibility issues


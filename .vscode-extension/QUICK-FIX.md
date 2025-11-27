# Quick Fix: Command Not Found

If you're seeing `command 'sruja.showPreview' not found`, follow these steps **in order**:

## Step 1: Reinstall Extension

```bash
cd .vscode-extension
npm run install:cursor    # For Cursor
# OR
npm run install:trae      # For Trae
# OR  
npm run install:antigravity  # For Antigravity
```

## Step 2: Completely Restart Editor

**Important**: Don't just reload the window - completely quit and restart:
- **Cursor**: Quit Cursor completely (Cmd+Q on Mac), then reopen
- **Trae**: Quit Trae completely, then reopen
- **Antigravity**: Quit Antigravity completely, then reopen

## Step 3: Check Extension is Activating

1. Open **Output** panel: `View` → `Output`
2. Select **"Log (Extension Host)"** from the dropdown
3. Look for these messages:
   - `Sruja extension activating...`
   - `✅ All Sruja commands registered successfully!`

If you see errors instead, share them.

## Step 4: Try the Command

1. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type: `Sruja: Show Diagram Preview`
3. Press Enter

## Step 5: Verify Installation

Run the verification script:

```bash
cd .vscode-extension
npm run verify
```

This will show you:
- Where the extension is installed
- If files are present
- Activation events configured

## Common Issues

### Extension Not Activating
- Check Output panel for errors
- Make sure `out/extension.js` exists and is recent
- Try reinstalling: `npm run install:cursor` (or trae/antigravity)

### Commands Still Not Found
- The extension might not support VS Code extensions
- Try using LSP instead (see `docs/development/trae-editor-lsp-setup.md`)
- Check if the editor has extension support enabled

### LSP Errors (Non-Critical)
- LSP errors won't prevent commands from working
- Commands should still be available even if LSP fails

## Still Not Working?

1. Check the exact error message in Output panel
2. Verify extension files exist: `ls -la ~/.cursor/extensions/sruja/out/`
3. Try manual installation:
   ```bash
   rm -rf ~/.cursor/extensions/sruja
   cd .vscode-extension
   npm run install:cursor
   ```
4. Restart editor completely


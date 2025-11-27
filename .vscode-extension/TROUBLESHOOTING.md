# Troubleshooting: Command Not Found

If you see `command 'sruja.showPreview' not found`, follow these steps:

## Step 1: Verify Extension is Installed

### For Cursor:
```bash
ls -la ~/.cursor/extensions/sruja
```

### For Trae:
```bash
ls -la ~/.trae/extensions/sruja
# Or
ls -la ~/.config/trae/extensions/sruja
```

You should see files like `package.json`, `out/extension.js`, etc.

## Step 2: Check Extension is Compiled

```bash
cd .vscode-extension
npm run compile
```

Make sure `out/extension.js` exists and was recently updated.

## Step 3: Reinstall Extension

### For Cursor:
```bash
cd .vscode-extension
npm run install:cursor
```

### For Trae:
```bash
cd .vscode-extension
npm run install:trae
```

## Step 4: Reload Editor

**For Cursor:**
1. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type: `Developer: Reload Window`
3. Press Enter

**For Trae:**
- Completely quit and restart Trae editor

## Step 5: Check Extension is Activating

1. Open Output panel: `View` → `Output`
2. Select "Log (Extension Host)" from the dropdown
3. Open a `.sruja` file
4. Look for these messages:
   - "Sruja extension activating..."
   - "✅ Sruja extension activated. Commands registered:"

If you don't see these messages, the extension isn't activating.

## Step 6: Verify Activation Events

The extension should activate when:
- A `.sruja` file is opened (`onLanguage:sruja`)
- OR when you try to run a command (`onCommand:sruja.showPreview`)

Make sure you have a `.sruja` file open, or try the command directly.

## Step 7: Check for Errors

In the Output panel, look for any error messages. Common issues:

1. **Missing dependencies**: Make sure `node_modules` exists
   ```bash
   cd .vscode-extension
   npm install
   ```

2. **Compilation errors**: Check TypeScript compilation
   ```bash
   cd .vscode-extension
   npm run compile
   ```

3. **Extension path issues**: Make sure the extension is in the correct location

## Step 8: Manual Command Test

Try running the command directly via the command palette:
1. Press `Cmd+Shift+P` (or `Ctrl+Shift+P`)
2. Type: `Sruja: Show Diagram Preview`
3. If it doesn't appear, the extension isn't loaded

## Step 9: Verify package.json

Check that the command is registered in `package.json`:

```bash
cat ~/.cursor/extensions/sruja/package.json | grep -A 10 "commands"
```

You should see:
```json
"commands": [
  {
    "command": "sruja.showPreview",
    "title": "Show Diagram Preview",
    ...
  }
]
```

## Step 10: Nuclear Option - Clean Install

If nothing works:

```bash
# Remove existing installation
rm -rf ~/.cursor/extensions/sruja
# Or for Trae:
rm -rf ~/.trae/extensions/sruja

# Reinstall
cd .vscode-extension
npm run install:cursor  # or install:trae

# Reload editor
```

## Still Not Working?

1. Check the editor's extension logs for detailed error messages
2. Verify the editor supports VS Code extensions (some editors only support LSP)
3. If using Trae and it doesn't support VS Code extensions, use LSP instead:
   - See `docs/development/trae-editor-lsp-setup.md`
   - Configure `sruja-lsp` binary in Trae's LSP settings


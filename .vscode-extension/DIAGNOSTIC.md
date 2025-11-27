# Diagnostic Steps for Command Issues

## For Cursor (Command Not Showing)

1. **Check Extension is Installed**:
   ```bash
   ls -la ~/.cursor/extensions/sruja/package.json
   ```

2. **Check Output Panel**:
   - Open: `View` → `Output`
   - Select: "Log (Extension Host)"
   - Look for: "Sruja extension activating..."
   - Look for: "✅ Sruja extension activated successfully"

3. **Check if Commands are Registered**:
   In the Output panel, you should see:
   ```
   Command 'sruja.showPreview' registered: true
   ```

4. **Try Direct Command Execution**:
   - Press `Cmd+Shift+P`
   - Type: `>Developer: Execute Command`
   - Type: `sruja.showPreview`
   - This bypasses the command palette

5. **Check Extension Activation**:
   - Press `Cmd+Shift+P`
   - Type: `>Developer: Show Running Extensions`
   - Look for "sruja" in the list
   - Check if it shows as "Activated" or has errors

## For Trae (Command Shows But Doesn't Work)

1. **Check if Command is Actually Registered**:
   - Open Output panel
   - Look for: "Command 'sruja.showPreview' registered: true"

2. **Check for Errors When Executing**:
   - Try the command
   - Check Output panel for errors
   - Look for: "sruja.showPreview command executed"

3. **Verify Extension Files**:
   ```bash
   ls -la ~/.trae/extensions/sruja/out/extension.js
   ```

4. **Check Trae Extension Support**:
   - Trae might show commands but not execute them if extension support is limited
   - Try using LSP instead (see `docs/development/trae-editor-lsp-setup.md`)

## Common Solutions

### Solution 1: Complete Reinstall
```bash
cd .vscode-extension
rm -rf ~/.cursor/extensions/sruja
rm -rf ~/.trae/extensions/sruja
npm run install:cursor
npm run install:trae
```
Then completely restart editors.

### Solution 2: Check Extension Host Logs
1. Open Output panel
2. Select "Log (Extension Host)"
3. Look for any red errors
4. Share the errors if found

### Solution 3: Manual Command Test
Try executing the command directly via Developer Tools:
1. `Cmd+Shift+P` → `Developer: Toggle Developer Tools`
2. In Console, type:
   ```javascript
   vscode.commands.executeCommand('sruja.showPreview')
   ```
3. Check for errors

### Solution 4: Verify package.json
```bash
cat ~/.cursor/extensions/sruja/package.json | grep -A 5 "commands"
```
Should show all 3 commands.

## If Nothing Works

The editors might not fully support VS Code extensions. In that case:
1. Use LSP for error highlighting (see `docs/development/trae-editor-lsp-setup.md`)
2. Use the CLI directly: `sruja compile file.sruja`
3. Check if the editors have extension support enabled in settings


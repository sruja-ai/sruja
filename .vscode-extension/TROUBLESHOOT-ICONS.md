# Troubleshooting: Sruja File Icons Not Showing

If "Sruja File Icons" doesn't appear in the File Icon Theme list, follow these steps:

## Step 1: Verify Files Exist

Check that all required files are present:

```bash
cd .vscode-extension
ls -la icons/
# Should show:
# - sruja.svg
# - sruja-16.png
# - sruja-32.png
# - sruja-icon-theme.json
```

## Step 2: Verify package.json Configuration

The `package.json` should have:

```json
{
  "contributes": {
    "iconThemes": [
      {
        "id": "sruja-icons",
        "label": "Sruja File Icons",
        "path": "./icons/sruja-icon-theme.json"
      }
    ]
  }
}
```

## Step 3: Verify Icon Theme File

The `icons/sruja-icon-theme.json` should have:

```json
{
  "iconDefinitions": {
    "_sruja_file": {
      "iconPath": "./sruja-16.png"
    }
  },
  "fileExtensions": {
    "sruja": "_sruja_file"
  }
}
```

**Important**: The `iconPath` is relative to the theme file location, not the extension root.

## Step 4: Reinstall Extension

1. **Remove old extension**:
   ```bash
   # For Cursor:
   rm -rf ~/.cursor/extensions/sruja
   # OR for VS Code:
   rm -rf ~/.vscode/extensions/sruja
   ```

2. **Copy extension again**:
   ```bash
   cp -r .vscode-extension ~/.cursor/extensions/sruja
   # OR for VS Code:
   cp -r .vscode-extension ~/.vscode/extensions/sruja
   ```

3. **Reload Cursor/VS Code**:
   - `Cmd+Shift+P` → "Developer: Reload Window"
   - OR quit and restart completely

## Step 5: Check Extension is Loaded

1. Open Extensions view (`Cmd+Shift+X`)
2. Search for "Sruja"
3. Verify "Sruja Language Support" is:
   - ✅ Installed
   - ✅ Enabled (not disabled)
   - No error messages

## Step 6: Check for Errors

1. Open Developer Tools:
   - `Cmd+Shift+P` → "Developer: Toggle Developer Tools"
2. Go to Console tab
3. Look for errors related to:
   - "icon theme"
   - "sruja-icons"
   - "sruja-icon-theme.json"
   - File path errors

## Step 7: Manual Activation

Try setting it directly in settings.json:

1. `Cmd+Shift+P` → "Preferences: Open User Settings (JSON)"
2. Add:
   ```json
   "workbench.iconTheme": "sruja-icons"
   ```
3. Save and check if it works

## Step 8: Verify Icon Theme ID

The ID in `package.json` must match what you use in settings:

- `package.json`: `"id": "sruja-icons"`
- `settings.json`: `"workbench.iconTheme": "sruja-icons"`

## Common Issues

### Issue: Icon theme not in dropdown

**Cause**: Extension not properly loaded or icon theme contribution not recognized

**Fix**: 
- Reinstall extension (Step 4)
- Check for errors in Developer Tools
- Verify `package.json` syntax is valid JSON

### Issue: Icon theme appears but icons don't show

**Cause**: Icon file path is wrong or files missing

**Fix**:
- Verify PNG files exist: `ls -la icons/*.png`
- Check icon path in theme file is correct
- Path should be relative to theme file: `./sruja-16.png`

### Issue: Extension shows error

**Cause**: Invalid JSON or missing files

**Fix**:
- Validate JSON: `cat package.json | python -m json.tool`
- Check all file paths exist
- Verify icon files are readable

## Still Not Working?

1. **Check Cursor/VS Code version**:
   - Icon themes require VS Code 1.10.0+
   - Update if needed

2. **Try a simpler test**:
   - Create a minimal icon theme with just one icon
   - Test if that works

3. **Check other extensions**:
   - Disable other icon theme extensions
   - They might conflict

4. **Check logs**:
   - View → Output → Select "Log (Window)"
   - Look for extension loading errors


# Icon Setup for Sruja Extension

The icon has been configured for the Sruja extension. Here's what was set up:

## What's Configured

1. **Extension Icon**: `icons/sruja.svg` - Shows in extension marketplace/details
2. **File Icons**: PNG versions created for file explorer icons
3. **Icon Theme**: Configured to show custom icon for `.sruja` files

## To See the Icon in Cursor/VS Code

### Step 1: Reinstall/Reload Extension

After updating the extension, you need to reload:

1. **If installed from folder**: 
   - Remove the old extension
   - Copy the updated `.vscode-extension` folder again
   - Reload Cursor/VS Code

2. **Reload Window**:
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Type "Developer: Reload Window"
   - Press Enter

### Step 2: Reinstall Extension (If Not Already Done)

**Important**: After updating the extension files, you MUST reinstall it:

1. **Remove old extension**:
   ```bash
   # For Cursor:
   rm -rf ~/.cursor/extensions/sruja
   # OR for VS Code:
   rm -rf ~/.vscode/extensions/sruja
   ```

2. **Copy extension again**:
   ```bash
   cd /Users/dilipkola/Workspace/sruja
   cp -r .vscode-extension ~/.cursor/extensions/sruja
   # OR for VS Code:
   cp -r .vscode-extension ~/.vscode/extensions/sruja
   ```

3. **Reload Cursor/VS Code**:
   - `Cmd+Shift+P` → "Developer: Reload Window"
   - OR quit and restart completely

### Step 3: Activate Icon Theme (Important!)

The icon theme needs to be manually activated. Here are **three ways** to do it:

#### Method 1: Via Command Palette (Easiest)

1. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type: `Preferences: File Icon Theme`
3. Press Enter
4. **Look for "Sruja File Icons"** in the dropdown list
   - If it's NOT there, go back to Step 2 (reinstall extension)
   - If it IS there, select it
5. Done! Your `.sruja` files should now show the custom icon

#### Method 2: Via Settings UI

1. Open Settings:
   - Press `Cmd+,` (Mac) or `Ctrl+,` (Windows/Linux)
   - OR: Click the gear icon ⚙️ → "Settings"
2. In the search box at the top, type: `file icon theme`
3. Look for the setting: **"Workbench: File Icon Theme"**
4. Click the dropdown next to it
5. Select **"Sruja File Icons"**
6. The setting will save automatically

#### Method 3: Via Settings JSON (Advanced)

1. Press `Cmd+Shift+P` / `Ctrl+Shift+P`
2. Type: `Preferences: Open User Settings (JSON)`
3. Add or update this line:
   ```json
   "workbench.iconTheme": "sruja-icons"
   ```
4. Save the file (`Cmd+S` / `Ctrl+S`)
5. The icon theme will activate immediately

### Step 3: Verify It Works

1. Open a folder with `.sruja` files
2. Look in the file explorer (sidebar)
3. `.sruja` files should now show your custom icon instead of the default file icon

## Troubleshooting

### Icon Still Not Showing

1. **Check extension is loaded**:
   - Open Extensions view (click the Extensions icon in sidebar, or `Cmd+Shift+X`)
   - Verify "Sruja Language Support" is installed and enabled
   - Look for a green checkmark or "Enabled" status

2. **Verify icon files exist**:
   ```bash
   ls -la .vscode-extension/icons/
   # Should show: sruja.svg, sruja-16.png, sruja-32.png, sruja-icon-theme.json
   ```

3. **Check icon theme is selected**:
   - Settings → Search "file icon theme"
   - Should show "Sruja File Icons" as selected
   - If it shows "None" or something else, select "Sruja File Icons"

4. **Restart Cursor/VS Code completely**:
   - Quit the application completely
   - Reopen it
   - Check if icons appear

### Icon Theme Not in List

If "Sruja File Icons" doesn't appear in the theme dropdown:

1. **Reinstall the extension** (most common fix):
   ```bash
   # Remove old
   rm -rf ~/.cursor/extensions/sruja
   # Copy fresh
   cd /Users/dilipkola/Workspace/sruja
   cp -r .vscode-extension ~/.cursor/extensions/sruja
   ```
   Then reload Cursor completely (quit and restart)

2. **Verify extension is loaded**:
   - Extensions view (`Cmd+Shift+X`)
   - Search for "Sruja"
   - Check "Sruja Language Support" is installed and enabled
   - Look for any error messages

3. **Check for errors in Developer Tools**:
   - `Cmd+Shift+P` → "Developer: Toggle Developer Tools"
   - Console tab → Look for errors about:
     - "icon theme"
     - "sruja-icons"
     - File path errors

4. **Verify files exist**:
   ```bash
   ls -la ~/.cursor/extensions/sruja/icons/
   # Should show: sruja-16.png, sruja-32.png, sruja-icon-theme.json
   ```

5. **Try manual activation in settings.json**:
   - `Cmd+Shift+P` → "Preferences: Open User Settings (JSON)"
   - Add: `"workbench.iconTheme": "sruja-icons"`
   - Save and check if it works

6. **See detailed troubleshooting**: Check `TROUBLESHOOT-ICONS.md` for more help

### Icon Shows But Wrong Size/Quality

- The PNG icons are 16x16 and 32x32 pixels
- If they look blurry, you may need higher resolution versions
- To regenerate: `rsvg-convert -w 32 -h 32 icons/sruja.svg -o icons/sruja-32.png`

## Files Created

- `icons/sruja.svg` - Original SVG icon
- `icons/sruja-16.png` - 16x16 PNG for file icons
- `icons/sruja-32.png` - 32x32 PNG for file icons  
- `icons/sruja-icon-theme.json` - Icon theme configuration

## Quick Reference

**Fastest way to activate:**
1. `Cmd+Shift+P` (or `Ctrl+Shift+P`)
2. Type: `file icon theme`
3. Select "Sruja File Icons"
4. Done!

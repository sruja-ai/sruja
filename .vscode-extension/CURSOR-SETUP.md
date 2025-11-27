# Setting Up Sruja Extension in Cursor

## Quick Setup for Cursor

### Step 1: Install the Extension

**Option A: Install from Folder (Easiest)**
1. Open Cursor
2. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
3. Type: `Extensions: Install Extension from Location...`
4. Navigate to and select the `.vscode-extension` folder in your workspace
5. Cursor will install the extension

**Option B: Manual Installation**
```bash
# Copy extension to Cursor's extensions folder
cp -r .vscode-extension ~/.cursor/extensions/sruja
```

### Step 2: Reload Cursor
1. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type: `Developer: Reload Window`
3. Press Enter

### Step 3: Verify Extension is Active
1. Open a `.sruja` file (e.g., `examples/full_features.sruja`)
2. Check the status bar - you should see "Sruja LSP" if the LSP is running
3. You should see syntax highlighting

### Step 4: Show the Diagram Preview

**Method 1: Command Palette (Recommended)**
1. With a `.sruja` file open, press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P`
2. Type: `Sruja: Show Diagram Preview`
3. Press Enter
4. The diagram should open in a side panel

**Method 2: Editor Toolbar**
- Look for a preview icon in the editor toolbar (top right)

**Method 3: Sidebar View**
- In the Explorer sidebar, look for "Sruja Diagram" section
- It appears when a `.sruja` file is active

## Troubleshooting

### Commands Don't Appear
1. **Check extension is installed**:
   - Press `Cmd+Shift+P`
   - Type: `Extensions: Show Installed Extensions`
   - Look for "Sruja Language Support"

2. **Check extension is activated**:
   - Open Output panel: `View` → `Output`
   - Select "Log (Extension Host)" from dropdown
   - Look for "Sruja extension activating..." message

3. **Reload Cursor again**:
   - `Cmd+Shift+P` → `Developer: Reload Window`

### Diagram Doesn't Show
1. **Check sruja binary exists**:
   ```bash
   ls -la sruja
   ```
   Should show the binary file

2. **Test compilation manually**:
   ```bash
   ./sruja compile examples/full_features.sruja
   ```
   Should output Mermaid code

3. **Check for errors**:
   - Open Output panel
   - Select "Sruja" channel
   - Look for error messages

### LSP Not Working
1. **Build the LSP binary** (if not already):
   ```bash
   go build -o sruja-lsp ./cmd/sruja-lsp
   ```

2. **Check LSP is running**:
   - Open Output panel
   - Select "Sruja LSP" channel
   - Should see connection messages

## Development Mode

If you're actively developing the extension:
1. Open `.vscode-extension` folder in Cursor
2. Press `F5` to launch Extension Development Host
3. In the new window, test your changes





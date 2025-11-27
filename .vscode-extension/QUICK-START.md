# Quick Start - Sruja VS Code Extension

## To See the Diagram Preview

### Step 1: Reload VS Code
The extension needs to be reloaded to pick up changes:
1. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type "Developer: Reload Window" and press Enter

### Step 2: Open a .sruja file
Open `examples/full_features.sruja` or any `.sruja` file

### Step 3: Show the Diagram
You have 3 options:

**Option A: Command Palette (Recommended)**
1. Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
2. Type "Sruja: Show Diagram Preview"
3. Press Enter

**Option B: Editor Toolbar**
- Look for the preview icon in the editor toolbar (top right of editor)

**Option C: Sidebar View**
- In the Explorer sidebar, look for "Sruja Diagram" section
- It should appear when a `.sruja` file is active

## Troubleshooting

If commands don't appear:
1. **Check extension is loaded**: Open Output panel (`View` → `Output`), select "Log (Extension Host)" and look for "Sruja extension activating..."
2. **Verify binary exists**: Make sure `sruja` binary exists in workspace root
3. **Check for errors**: Look in the Output panel for any error messages

## Development Mode

To run in development mode:
1. Open `.vscode-extension` folder in VS Code
2. Press `F5` to launch Extension Development Host
3. In the new window, open your `.sruja` file
4. Commands should be available





# Installing the Extension Locally

## Method 1: Install from VSIX (Recommended)

1. Build the VSIX:

   ```bash
   npm run build:vsix
   ```

2. Install in VS Code:
   - Open VS Code
   - Press `Cmd+Shift+X` (Mac) or `Ctrl+Shift+X` (Windows/Linux) to open Extensions
   - Click the `...` menu (top right)
   - Select "Install from VSIX..."
   - Navigate to `apps/vscode-extension/sruja-language-support.vsix`
   - Select it and click Install
   - Reload VS Code when prompted

## Method 2: Using Command Palette

1. Build the VSIX:

   ```bash
   npm run build:vsix
   ```

2. In VS Code:
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Type: "Extensions: Install from VSIX..."
   - Select: `apps/vscode-extension/sruja-language-support.vsix`
   - Reload VS Code

## Method 3: Development Mode (For Testing)

1. Open the `apps/vscode-extension` folder in VS Code
2. Press `F5` (or Run → Start Debugging)
3. A new "Extension Development Host" window opens
4. In that window, open your `.sruja` files

## Method 4: Manual CLI Install (if `code` command is available)

If you have the VS Code CLI in your PATH:

```bash
npm run build:vsix
code --install-extension sruja-language-support.vsix
```

To add VS Code CLI to PATH:

- **Mac**: Open VS Code → `Cmd+Shift+P` → "Shell Command: Install 'code' command in PATH"
- **Windows**: Usually installed automatically
- **Linux**: May need to add manually to PATH

## Note

The extension uses WASM and doesn't require any external CLI tools. Everything works out of the box!

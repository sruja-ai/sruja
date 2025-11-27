# Installing Sruja Extension

## Cursor Editor

### Quick Install

Run this command from the `.vscode-extension` directory:

```bash
npm run install:cursor
```

Or use the update command (same thing):

```bash
npm run update:cursor
```

This will:
1. Compile the TypeScript extension
2. Copy the extension to `~/.cursor/extensions/sruja`
3. Exclude unnecessary files (node_modules, .git, etc.)

### Manual Installation

If you prefer to install manually:

```bash
# From the workspace root
cp -r .vscode-extension ~/.cursor/extensions/sruja
```

## Trae Editor

### Quick Install

Run this command from the `.vscode-extension` directory:

```bash
npm run install:trae
```

Or use the update command (same thing):

```bash
npm run update:trae
```

This will:
1. Compile the TypeScript extension
2. Copy the extension to Trae's extensions folder (auto-detected)
3. Exclude unnecessary files (node_modules, .git, etc.)

### Manual Installation

If Trae supports VS Code extensions, you can install manually:

```bash
# From the workspace root
# Location depends on your OS and Trae configuration
cp -r .vscode-extension ~/.trae/extensions/sruja
# Or
cp -r .vscode-extension ~/.config/trae/extensions/sruja
```

**Note**: If Trae doesn't support VS Code extensions, you'll need to configure the LSP manually. See `docs/development/trae-editor-lsp-setup.md` for LSP setup instructions.

## Antigravity Editor

### Quick Install

Run this command from the `.vscode-extension` directory:

```bash
npm run install:antigravity
```

Or use the update command (same thing):

```bash
npm run update:antigravity
```

This will:
1. Compile the TypeScript extension
2. Copy the extension to Antigravity's extensions folder (auto-detected)
3. Exclude unnecessary files (node_modules, .git, etc.)

### Manual Installation

If Antigravity supports VS Code extensions, you can install manually:

```bash
# From the workspace root
# Location depends on your OS and Antigravity configuration
cp -r .vscode-extension ~/.antigravity/extensions/sruja
# Or
cp -r .vscode-extension ~/.config/antigravity/extensions/sruja
```

**Note**: If Antigravity doesn't support VS Code extensions, you'll need to configure the LSP manually. See `docs/development/trae-editor-lsp-setup.md` for LSP setup instructions (similar setup applies to Antigravity).

## After Installation

### For Cursor

1. **Reload Cursor**:
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Type: `Developer: Reload Window`
   - Press Enter

2. **Verify Installation**:
   - Open a `.sruja` file
   - Press `Cmd+Shift+P`
   - Type: `Sruja: Show Diagram Preview`
   - The command should appear and work

### For Trae

1. **Restart Trae editor** completely

2. **Verify Installation**:
   - Open a `.sruja` file
   - If Trae supports VS Code extensions, use command: `Cmd+Shift+P` → `Sruja: Show Diagram Preview`
   - If not, configure LSP manually (see `docs/development/trae-editor-lsp-setup.md`)

### For Antigravity

1. **Restart Antigravity editor** completely

2. **Verify Installation**:
   - Open a `.sruja` file
   - If Antigravity supports VS Code extensions, use command: `Cmd+Shift+P` → `Sruja: Show Diagram Preview`
   - If not, configure LSP manually (similar to Trae setup)

## Updating the Extension

### Cursor

Whenever you make changes to the extension:

```bash
cd .vscode-extension
npm run update:cursor
```

Then reload Cursor to see the changes.

### Trae

Whenever you make changes to the extension:

```bash
cd .vscode-extension
npm run update:trae
```

Then restart Trae editor to see the changes.

### Antigravity

Whenever you make changes to the extension:

```bash
cd .vscode-extension
npm run update:antigravity
```

Then restart Antigravity editor to see the changes.

## Troubleshooting

### Cursor

#### Extension Not Found
- Check that the extension was copied: `ls -la ~/.cursor/extensions/sruja`
- Verify the `package.json` exists: `cat ~/.cursor/extensions/sruja/package.json`

#### Commands Don't Appear
- Reload Cursor: `Cmd+Shift+P` → `Developer: Reload Window`
- Check Output panel: `View` → `Output` → Select "Log (Extension Host)"
- Look for "Sruja extension activating..." message

#### Errors in Extension
- Check the compiled output: `ls -la ~/.cursor/extensions/sruja/out/`
- Recompile: `cd .vscode-extension && npm run compile`
- Reinstall: `npm run install:cursor`

### Trae

#### Extension Not Found
- Check that the extension was copied: `ls -la ~/.trae/extensions/sruja`
- Or try: `ls -la ~/.config/trae/extensions/sruja`
- Verify the `package.json` exists: `cat ~/.trae/extensions/sruja/package.json`

#### Commands Don't Appear
- **If Trae supports VS Code extensions**: Restart Trae completely
- **If Trae doesn't support VS Code extensions**: You need to configure LSP manually
  - See: `docs/development/trae-editor-lsp-setup.md`
  - Configure the `sruja-lsp` binary in Trae's LSP settings

#### Errors in Extension
- Check the compiled output: `ls -la ~/.trae/extensions/sruja/out/`
- Recompile: `cd .vscode-extension && npm run compile`
- Reinstall: `npm run install:trae`

#### LSP Setup (If VS Code Extensions Not Supported)
If Trae doesn't support VS Code extensions, you can still use the LSP for error highlighting and other features:

1. Build the LSP binary:
   ```bash
   go build -o sruja-lsp ./cmd/sruja-lsp
   ```

2. Configure in Trae's LSP settings:
   - Command: `/path/to/sruja-lsp`
   - File patterns: `*.sruja`
   - Transport: `stdio`

See `docs/development/trae-editor-lsp-setup.md` for detailed instructions.

### Antigravity

#### Extension Not Found
- Check that the extension was copied: `ls -la ~/.antigravity/extensions/sruja`
- Or try: `ls -la ~/.config/antigravity/extensions/sruja`
- Verify the `package.json` exists: `cat ~/.antigravity/extensions/sruja/package.json`

#### Commands Don't Appear
- **If Antigravity supports VS Code extensions**: Restart Antigravity completely
- **If Antigravity doesn't support VS Code extensions**: You need to configure LSP manually
  - See: `docs/development/trae-editor-lsp-setup.md` (similar setup)
  - Configure the `sruja-lsp` binary in Antigravity's LSP settings

#### Errors in Extension
- Check the compiled output: `ls -la ~/.antigravity/extensions/sruja/out/`
- Recompile: `cd .vscode-extension && npm run compile`
- Reinstall: `npm run install:antigravity`

#### LSP Setup (If VS Code Extensions Not Supported)
If Antigravity doesn't support VS Code extensions, you can still use the LSP for error highlighting and other features:

1. Build the LSP binary:
   ```bash
   go build -o sruja-lsp ./cmd/sruja-lsp
   ```

2. Configure in Antigravity's LSP settings:
   - Command: `/path/to/sruja-lsp`
   - File patterns: `*.sruja`
   - Transport: `stdio`

The setup is similar to Trae editor. See `docs/development/trae-editor-lsp-setup.md` for reference.


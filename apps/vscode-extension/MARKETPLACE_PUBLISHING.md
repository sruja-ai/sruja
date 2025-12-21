# Publishing to VS Code Marketplace

## Overview

When published to the VS Code Marketplace, users can install the extension directly from VS Code. The extension uses WASM and works out of the box with no external dependencies.

## How It Works for End Users

### 1. Installation Flow

1. **User installs extension from marketplace:**
   - Search for "Sruja DSL Language Support" in VS Code Extensions
   - Click "Install"
   - Extension installs automatically
   - No additional setup required!

### 2. Features Available

Once installed, users get:

- **Syntax Highlighting**: `.sruja` files are highlighted
- **Language Server**:
  - Auto-completion
  - Error detection
  - Validation
  - Hover information
- **Commands**:
  - `sruja.previewArchitecture` - Preview markdown export
- **Context Menu**: Right-click on `.sruja` files to preview

### 3. Configuration

Users can configure the extension via VS Code settings:

```json
{
  "sruja.formatting.enabled": true,
  "sruja.formatting.tabSize": 2,
  "sruja.formatting.insertSpaces": true
}
```

## Publishing Process

### Prerequisites

1. **VS Code Marketplace Account:**
   - Create account at https://marketplace.visualstudio.com/manage
   - Create a publisher (e.g., "sruja-ai")
   - Get Personal Access Token (PAT)

2. **Update package.json:**
   - Remove `"private": true` (or set to `false`)
   - Ensure version follows semver (e.g., "0.1.0")
   - Add marketplace metadata if needed

3. **Install vsce:**
   ```bash
   npm install -g @vscode/vsce
   ```

### Publishing Steps

1. **Login to marketplace:**

   ```bash
   vsce login sruja-ai
   # Enter your Personal Access Token
   ```

2. **Build and publish:**

   ```bash
   cd apps/vscode-extension
   npm run build:vsix  # Test build first
   vsce publish
   ```

3. **Verify:**
   - Check https://marketplace.visualstudio.com/items?itemName=sruja-ai.sruja-language-support
   - Test installation in a clean VS Code instance

### Version Updates

To publish updates:

1. Update version in `package.json`:

   ```json
   "version": "0.1.1"
   ```

2. Publish:

   ```bash
   vsce publish
   ```

3. Or publish with specific version:
   ```bash
   vsce publish 0.1.1
   ```

## Important Considerations

### 1. WASM-Based (No CLI Dependency)

✅ **No external dependencies**: The extension uses WASM and works out of the box:

- All language features work without CLI
- WASM files are bundled with the extension
- No platform-specific installation needed

### 2. Platform Support

The extension works on all platforms (Windows, macOS, Linux):

- WASM is platform-agnostic
- No additional binaries required
- Works immediately after installation

### 3. Error Handling

The extension handles errors gracefully:

- Shows helpful error messages for WASM initialization failures
- Status bar shows extension status
- Preview errors are displayed in the preview pane

### 4. README Content

Update `README.md` to include:

- Quick start guide (no installation needed)
- Configuration options
- Troubleshooting section
- Links to documentation

## Post-Publishing Checklist

- [ ] Verify extension appears in marketplace
- [ ] Test installation on clean VS Code instance
- [ ] Verify all features work (WASM-based, no CLI needed)
- [ ] Test error handling for WASM initialization failures
- [ ] Update main repository README with extension link
- [ ] Add extension badge to main README
- [ ] Monitor marketplace reviews and issues

## Marketplace Badge

Add to main README.md:

```markdown
[![VS Code Extension](https://img.shields.io/badge/VS%20Code-Extension-blue)](https://marketplace.visualstudio.com/items?itemName=sruja-ai.sruja-language-support)
```

# Publishing to VS Code Marketplace

## Overview

When published to the VS Code Marketplace, users can install the extension directly from VS Code. However, the extension requires the `sruja` CLI tool to be installed separately.

## How It Works for End Users

### 1. Installation Flow

1. **User installs extension from marketplace:**
   - Search for "Sruja DSL Language Support" in VS Code Extensions
   - Click "Install"
   - Extension installs automatically

2. **User needs to install `sruja` CLI:**
   - The extension requires the `sruja` command-line tool to be available
   - Users must install it separately using one of these methods:
     ```bash
     # Automated install (recommended)
     curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
     
     # Or download from GitHub Releases
     # Or: go install github.com/sruja-ai/sruja/cmd/sruja@latest
     ```

3. **Extension automatically detects `sruja`:**
   - Extension looks for `sruja` in PATH
   - If not found, users can configure path in settings

### 2. Features Available

Once installed, users get:

- **Syntax Highlighting**: `.sruja` files are highlighted
- **Language Server**: 
  - Auto-completion
  - Error detection
  - Validation
  - Hover information
- **Commands**:
  - `sruja.restartServer` - Restart language server
  - `sruja.showOutput` - View server logs
  - `sruja.previewArchitecture` - Preview markdown export
- **Context Menu**: Right-click on `.sruja` files to preview

### 3. Configuration

Users can configure the extension via VS Code settings:

```json
{
  "srujaLanguageServer.path": "sruja",  // Path to sruja executable
  "srujaLanguageServer.enableLogging": false,
  "srujaLanguageServer.logLevel": "info",
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

### 1. CLI Dependency

⚠️ **Critical**: The extension requires the `sruja` CLI to be installed separately. Consider:

- Adding installation instructions in README
- Adding a post-install message or notification
- Creating a setup wizard (future enhancement)
- Bundling the CLI (complex, platform-specific)

### 2. Platform Support

The extension works on all platforms (Windows, macOS, Linux), but:
- Users must install the correct `sruja` binary for their platform
- The extension will detect the binary automatically if in PATH

### 3. Error Handling

The extension handles missing CLI gracefully:
- Shows helpful error messages
- Provides configuration option to set custom path
- Status bar shows connection status

### 4. README Content

Update `README.md` to include:
- Installation instructions for `sruja` CLI
- Quick start guide
- Configuration options
- Troubleshooting section
- Links to documentation

## Post-Publishing Checklist

- [ ] Verify extension appears in marketplace
- [ ] Test installation on clean VS Code instance
- [ ] Verify all features work with CLI installed
- [ ] Test error handling when CLI is missing
- [ ] Update main repository README with extension link
- [ ] Add extension badge to main README
- [ ] Monitor marketplace reviews and issues

## Marketplace Badge

Add to main README.md:

```markdown
[![VS Code Extension](https://img.shields.io/badge/VS%20Code-Extension-blue)](https://marketplace.visualstudio.com/items?itemName=sruja-ai.sruja-language-support)
```
















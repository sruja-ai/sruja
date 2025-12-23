# Publishing to VS Code Marketplace and Open VSX

## Overview

When published to the VS Code Marketplace and Open VSX Registry, users can install the extension directly from VS Code and VS Code alternatives (like VSCodium, Eclipse Theia, etc.). The extension uses WASM and works out of the box with no external dependencies.

### Marketplaces

- **VS Code Marketplace**: https://marketplace.visualstudio.com/items?itemName=srujaai.sruja
- **Open VSX Registry**: https://open-vsx.org/extension/srujaai/sruja

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
   - Create a publisher (e.g., "srujaai")
   - Get Personal Access Token (PAT)
   - Store as GitHub secret: `AZURE_DEVOPS_PAT`

2. **Open VSX Registry Account:**
   - Register for an Eclipse account and link it to your GitHub account
   - Sign the Eclipse Foundation Open VSX Publisher Agreement
   - **Create the namespace** (must match `publisher` field in `package.json`):
     ```bash
     npx ovsx create-namespace srujaai -p <your-open-vsx-token>
     ```
     - The namespace `srujaai` must match your `package.json` publisher field exactly
     - This only needs to be done once
   - Generate an access token on your Open VSX profile page: https://open-vsx.org/user-settings/tokens
   - Store as GitHub secret: `OPEN_VSX_TOKEN`
   - See: https://github.com/eclipse/openvsx/wiki/Publishing-Extensions

3. **Update package.json:**
   - Remove `"private": true` (or set to `false`)
   - Ensure version follows semver (e.g., "0.1.0")
   - Add marketplace metadata if needed

4. **Install publishing tools:**
   ```bash
   npm install -g @vscode/vsce
   # ovsx is installed via npx when needed
   ```

### Publishing Steps

#### Automated Publishing (Recommended)

The extension is automatically published to both marketplaces when a GitHub release is published:

1. **Create a GitHub Release:**
   - Go to **Releases** → **Draft a new release**
   - Create a new tag (e.g., `v0.1.1`)
   - Publish the release
   - The `deploy-prod.yml` workflow will automatically:
     - Publish to VS Code Marketplace
     - Publish to Open VSX Registry

#### Manual Publishing

1. **Login to VS Code Marketplace:**

   ```bash
   vsce login srujaai
   # Enter your Personal Access Token
   ```

2. **Login to Open VSX:**

   ```bash
   npx ovsx login
   # Enter your Open VSX access token
   ```

3. **Build and publish:**

   ```bash
   cd apps/vscode-extension
   npm run build:vsix  # Test build first
   
   # Publish to VS Code Marketplace
   npm run publish:marketplace
   
   # Publish to Open VSX Registry
   npm run publish:openvsx
   ```

4. **Verify:**
   - VS Code Marketplace: https://marketplace.visualstudio.com/items?itemName=srujaai.sruja
   - Open VSX Registry: https://open-vsx.org/extension/srujaai/sruja
   - Test installation in a clean VS Code instance

### Version Updates

To publish updates:

1. Update version in `package.json`:

   ```json
   "version": "0.1.1"
   ```

2. Publish (automated via GitHub release):
   - Create a GitHub release with the new version tag
   - Both marketplaces will be updated automatically

3. Or publish manually:
   ```bash
   # VS Code Marketplace
   npm run publish:marketplace
   
   # Open VSX Registry
   npm run publish:openvsx
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

- [ ] Verify extension appears in VS Code Marketplace
- [ ] Verify extension appears in Open VSX Registry
- [ ] Test installation on clean VS Code instance
- [ ] Test installation on VSCodium (Open VSX)
- [ ] Verify all features work (WASM-based, no CLI needed)
- [ ] Test error handling for WASM initialization failures
- [ ] Update main repository README with extension links
- [ ] Add extension badges to main README
- [ ] Monitor marketplace reviews and issues

## Marketplace Badges

Add to main README.md:

```markdown
[![VS Code Extension](https://img.shields.io/badge/VS%20Code-Extension-blue)](https://marketplace.visualstudio.com/items?itemName=srujaai.sruja)
[![Open VSX](https://img.shields.io/badge/Open%20VSX-Extension-orange)](https://open-vsx.org/extension/srujaai/sruja)
```

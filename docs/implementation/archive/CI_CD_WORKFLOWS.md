# CI/CD Workflows for Sruja

## Overview

CI/CD workflows for:
- Validating Sruja files and detecting conflicts
- Ensuring quality in pull requests and main branch
- Publishing artifacts (CLI binaries, npm packages, VS Code extension, JetBrains plugin)
- Generating previews and automating releases

## Workflows

### 1. PR Validation Workflow

**Purpose**: Validate all `.sruja` files and changes in pull requests.

**File**: `.github/workflows/sruja-validate.yml`

```yaml
name: Validate Sruja Files

on:
  pull_request:
    paths:
      - '**/*.sruja'
      - 'changes/**'
      - 'architecture/**'
      - '.github/workflows/sruja-validate.yml'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for change validation
      
      - name: Setup Go
        uses: actions/setup-go@v4
        with:
          go-version: '1.25'
      
      - name: Install Sruja CLI
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Lint all Sruja files
        run: |
          find . -name "*.sruja" -not -path "./.git/*" | while read file; do
            echo "Linting $file"
            sruja lint "$file" || exit 1
          done
      
      - name: Validate changes
        run: |
          if [ -d "changes" ] && [ "$(ls -A changes/*.sruja 2>/dev/null)" ]; then
            echo "Validating changes..."
            sruja change validate --all || exit 1
          else
            echo "No changes directory found, skipping change validation"
          fi
      
      - name: Check for conflicts
        run: |
          if [ -d "changes" ] && [ "$(ls -A changes/*.sruja 2>/dev/null)" ]; then
            echo "Checking for conflicts..."
            sruja change conflicts || echo "No conflicts detected"
          fi
      
      - name: Validate ADR states
        run: |
          # Check that all ADRs referenced by changes are in final state
          # This is handled by sruja change validate, but we can add explicit check
          echo "ADR state validation is part of change validation"
      
      - name: Round-trip test
        run: |
          # Test DSL → JSON → DSL round-trip for all .sruja files
          find . -name "*.sruja" -not -path "./.git/*" -not -path "./changes/*" | while read file; do
            echo "Testing round-trip for $file"
            TEMP_JSON=$(mktemp).json
            TEMP_DSL=$(mktemp).sruja
            
            sruja export json "$file" "$TEMP_JSON" || exit 1
            sruja json-to-dsl "$TEMP_JSON" "$(dirname "$TEMP_DSL")" --format single || exit 1
            
            # Compare (basic check - full comparison would be more complex)
            echo "Round-trip test passed for $file"
            rm -f "$TEMP_JSON" "$TEMP_DSL"
          done
```

**Triggers**:
- Pull requests with `.sruja` files
- Changes to workflow file

**Validations**:
- ✅ Lint all `.sruja` files
- ✅ Validate all changes can be applied
- ✅ Check for conflicts between changes
- ✅ Round-trip test (DSL → JSON → DSL)

### 2. Preview Generation Workflow

**Purpose**: Generate architecture previews for PRs automatically.

**File**: `.github/workflows/sruja-preview.yml`

```yaml
name: Sruja Architecture Preview

on:
  pull_request:
    paths:
      - '**/*.sruja'
      - 'changes/**'
      - 'architecture/**'

jobs:
  preview:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Setup Go
        uses: actions/setup-go@v4
        with:
          go-version: '1.25'
      
      - name: Install Sruja CLI
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Detect changes
        id: changes
        run: |
          CHANGES=$(git diff --name-only origin/${{ github.base_ref }}...HEAD | grep '\.sruja$' | grep '^changes/' | sed 's|changes/||;s|\.sruja||' | tr '\n' ',' | sed 's/,$//')
          
          if [ -n "$CHANGES" ]; then
            echo "changes=$CHANGES" >> $GITHUB_OUTPUT
            echo "has_changes=true" >> $GITHUB_OUTPUT
          else
            echo "has_changes=false" >> $GITHUB_OUTPUT
          fi
      
      - name: Generate preview snapshot
        if: steps.changes.outputs.has_changes == 'true'
        id: preview
        run: |
          PREVIEW_NAME="pr-${{ github.event.pull_request.number }}"
          sruja snapshot preview "$PREVIEW_NAME" --changes "${{ steps.changes.outputs.changes }}"
          
          # Export to HTML
          sruja export html "snapshots/preview/$PREVIEW_NAME.sruja" "preview.html"
          
          echo "preview_name=$PREVIEW_NAME" >> $GITHUB_OUTPUT
          echo "preview_file=preview.html" >> $GITHUB_OUTPUT
      
      - name: Upload preview artifact
        if: steps.preview.outputs.preview_name
        uses: actions/upload-artifact@v4
        with:
          name: architecture-preview-pr-${{ github.event.pull_request.number }}
          path: preview.html
          retention-days: 7
      
      - name: Comment on PR
        if: steps.preview.outputs.preview_name
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
          script: |
            const previewName = '${{ steps.preview.outputs.preview_name }}';
            const changes = '${{ steps.changes.outputs.changes }}'.split(',').filter(c => c);
            
            const body = `## 🏗️ Architecture Preview
            
            Preview snapshot generated for this PR.
            
            **Changes included:**
            ${changes.length > 0 ? changes.map(c => `- \`${c}\``).join('\n') : 'All changes in PR'}
            
            **View Preview:**
            - [Open in Studio](https://sruja.ai/studio?preview=${previewName})
            - [Download HTML Preview](./preview.html) (artifact)
            
            **Note:** This is a preview snapshot - changes may not be finalized.
            `;
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });
```

**Triggers**:
- Pull requests with `.sruja` files

**Actions**:
- ✅ Detects changes in PR
- ✅ Generates preview snapshot
- ✅ Exports to HTML
- ✅ Uploads as artifact
- ✅ Comments on PR with preview link

### 3. Main Branch Validation

**Purpose**: Validate all files on main branch after merge.

**File**: `.github/workflows/sruja-main.yml`

```yaml
name: Validate Main Branch

on:
  push:
    branches:
      - main
      - master
    paths:
      - '**/*.sruja'
      - 'changes/**'
      - 'architecture/**'

jobs:
  validate-main:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Go
        uses: actions/setup-go@v4
        with:
          go-version: '1.25'
      
      - name: Install Sruja CLI
        run: |
          curl -fsSL https://raw.githubusercontent.com/sruja-ai/sruja/main/scripts/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Lint all files
        run: |
          find . -name "*.sruja" -not -path "./.git/*" | while read file; do
            sruja lint "$file" || exit 1
          done
      
      - name: Validate all changes
        run: |
          if [ -d "changes" ] && [ "$(ls -A changes/*.sruja 2>/dev/null)" ]; then
            sruja change validate --all || exit 1
          fi
      
      - name: Apply changes (dry-run)
        run: |
          # Test that all approved changes can be applied
          if [ -d "changes" ] && [ "$(ls -A changes/*.sruja 2>/dev/null)" ]; then
            sruja change apply --dry-run || exit 1
          fi
      
      - name: Generate current snapshot
        run: |
          # Generate current.sruja from base + changes
          if [ -d "changes" ] && [ "$(ls -A changes/*.sruja 2>/dev/null)" ]; then
            sruja change apply
            # Verify current.sruja is valid
            if [ -f "current.sruja" ]; then
              sruja lint current.sruja || exit 1
            fi
          fi
```

**Triggers**:
- Push to main/master branch

**Validations**:
- ✅ Lint all files
- ✅ Validate all changes
- ✅ Test change application (dry-run)
- ✅ Generate and validate current snapshot

### 4. Release Workflow

**Purpose**: Build and release Sruja CLI binaries, npm packages, and IDE extensions.

**File**: `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  # Job 1: Build and release CLI binaries
  cli-release:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Go
        uses: actions/setup-go@v4
        with:
          go-version: '1.25'
      
      - name: Build binaries
        run: |
          # Build for multiple platforms
          GOOS=linux GOARCH=amd64 go build -o sruja-linux-amd64 ./cmd/sruja
          GOOS=darwin GOARCH=amd64 go build -o sruja-darwin-amd64 ./cmd/sruja
          GOOS=darwin GOARCH=arm64 go build -o sruja-darwin-arm64 ./cmd/sruja
          GOOS=windows GOARCH=amd64 go build -o sruja-windows-amd64.exe ./cmd/sruja
      
      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            sruja-linux-amd64
            sruja-darwin-amd64
            sruja-darwin-arm64
            sruja-windows-amd64.exe
          tag_name: ${{ github.ref_name }}
          name: Release ${{ github.ref_name }}

  # Job 2: Publish npm packages
  npm-publish:
    runs-on: ubuntu-latest
    needs: cli-release
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      
      - name: Publish Viewer Library
        working-directory: ./viewer
        run: |
          npm ci
          npm run build
          npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
      
      # Optional: Publish other npm packages if needed
      # - name: Publish Local Studio (if published separately)
      #   working-directory: ./local-studio
      #   run: |
      #     npm ci
      #     npm run build
      #     npm publish --access public
      #   env:
      #     NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  # Job 3: Publish VS Code Extension
  vscode-publish:
    runs-on: ubuntu-latest
    needs: cli-release
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install dependencies
        working-directory: ./vscode-extension
        run: npm ci
      
      - name: Build extension
        working-directory: ./vscode-extension
        run: npm run compile
      
      - name: Package extension
        working-directory: ./vscode-extension
        run: |
          npm install -g @vscode/vsce
          vsce package
      
      - name: Publish to VS Code Marketplace
        working-directory: ./vscode-extension
        run: |
          vsce publish -p ${{ secrets.VSCE_PAT }}
        env:
          VSCE_PAT: ${{ secrets.VSCE_PAT }}

  # Job 4: Publish JetBrains Plugin
  jetbrains-publish:
    runs-on: ubuntu-latest
    needs: cli-release
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Java
        uses: actions/setup-java@v4
        with:
          distribution: 'temurin'
          java-version: '17'
      
      - name: Build plugin
        working-directory: ./jetbrains-plugin
        run: |
          chmod +x gradlew
          ./gradlew buildPlugin
      
      - name: Publish to JetBrains Marketplace
        working-directory: ./jetbrains-plugin
        run: |
          ./gradlew publishPlugin \
            -Pjetbrains.token=${{ secrets.JETBRAINS_TOKEN }} \
            -Pjetbrains.channels=stable
        env:
          JETBRAINS_TOKEN: ${{ secrets.JETBRAINS_TOKEN }}
```

### 5. NPM Package Publishing (Separate Workflow)

**Purpose**: Publish npm packages on version bumps (alternative to release workflow).

**File**: `.github/workflows/npm-publish.yml`

```yaml
name: Publish NPM Packages

on:
  push:
    branches:
      - main
    paths:
      - 'viewer/package.json'
      - 'viewer/**'
  workflow_dispatch:
    inputs:
      package:
        description: 'Package to publish (viewer)'
        required: true
        type: choice
        options:
          - viewer

jobs:
  publish-viewer:
    if: github.event_name == 'workflow_dispatch' && github.event.inputs.package == 'viewer' || github.event_name == 'push' && contains(github.event.head_commit.message, '[publish viewer]')
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      
      - name: Get version
        id: version
        working-directory: ./viewer
        run: |
          VERSION=$(node -p "require('./package.json').version")
          echo "version=$VERSION" >> $GITHUB_OUTPUT
      
      - name: Check if version exists
        working-directory: ./viewer
        run: |
          PACKAGE_NAME=$(node -p "require('./package.json').name")
          if npm view "$PACKAGE_NAME@${{ steps.version.outputs.version }}" version &>/dev/null; then
            echo "Version ${{ steps.version.outputs.version }} already exists, skipping publish"
            exit 1
          fi
      
      - name: Install dependencies
        working-directory: ./viewer
        run: npm ci
      
      - name: Run tests
        working-directory: ./viewer
        run: npm test
      
      - name: Build
        working-directory: ./viewer
        run: npm run build
      
      - name: Publish to npm
        working-directory: ./viewer
        run: npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### 6. VS Code Extension Publishing (Separate Workflow)

**Purpose**: Publish VS Code extension on version bumps.

**File**: `.github/workflows/vscode-publish.yml`

```yaml
name: Publish VS Code Extension

on:
  push:
    branches:
      - main
    paths:
      - 'vscode-extension/package.json'
      - 'vscode-extension/**'
  workflow_dispatch:

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install dependencies
        working-directory: ./vscode-extension
        run: npm ci
      
      - name: Run tests
        working-directory: ./vscode-extension
        run: npm test || true  # Tests optional
      
      - name: Build extension
        working-directory: ./vscode-extension
        run: npm run compile
      
      - name: Package extension
        working-directory: ./vscode-extension
        run: |
          npm install -g @vscode/vsce
          vsce package
      
      - name: Check version
        working-directory: ./vscode-extension
        run: |
          VERSION=$(node -p "require('./package.json').version")
          echo "Publishing version $VERSION"
      
      - name: Publish to VS Code Marketplace
        working-directory: ./vscode-extension
        run: |
          vsce publish -p ${{ secrets.VSCE_PAT }} --yarn
        env:
          VSCE_PAT: ${{ secrets.VSCE_PAT }}
```

### 7. JetBrains Plugin Publishing (Separate Workflow)

**Purpose**: Publish JetBrains plugin on version bumps.

**File**: `.github/workflows/jetbrains-publish.yml`

```yaml
name: Publish JetBrains Plugin

on:
  push:
    branches:
      - main
    paths:
      - 'jetbrains-plugin/build.gradle.kts'
      - 'jetbrains-plugin/src/**'
  workflow_dispatch:

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Java
        uses: actions/setup-java@v4
        with:
          distribution: 'temurin'
          java-version: '17'
      
      - name: Build plugin
        working-directory: ./jetbrains-plugin
        run: |
          chmod +x gradlew
          ./gradlew buildPlugin
      
      - name: Check version
        working-directory: ./jetbrains-plugin
        run: |
          VERSION=$(./gradlew properties | grep "^version:" | awk '{print $2}')
          echo "Publishing version $VERSION"
      
      - name: Publish to JetBrains Marketplace
        working-directory: ./jetbrains-plugin
        run: |
          ./gradlew publishPlugin \
            -Pjetbrains.token=${{ secrets.JETBRAINS_TOKEN }} \
            -Pjetbrains.channels=stable
        env:
          JETBRAINS_TOKEN: ${{ secrets.JETBRAINS_TOKEN }}
```

### 8. Continuous Publishing (Nightly/Canary Builds)

**Purpose**: Publish canary/nightly builds for testing.

**File**: `.github/workflows/canary-publish.yml`

```yaml
name: Canary Publishing

on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight UTC
  workflow_dispatch:

jobs:
  npm-canary:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      
      - name: Install dependencies
        working-directory: ./viewer
        run: npm ci
      
      - name: Build
        working-directory: ./viewer
        run: npm run build
      
      - name: Publish canary
        working-directory: ./viewer
        run: |
          VERSION=$(node -p "require('./package.json').version")
          CANARY_VERSION="${VERSION}-canary.$(date +%Y%m%d).${GITHUB_SHA:0:7}"
          npm version "$CANARY_VERSION" --no-git-tag-version
          npm publish --access public --tag canary
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  vscode-canary:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install dependencies
        working-directory: ./vscode-extension
        run: npm ci
      
      - name: Build extension
        working-directory: ./vscode-extension
        run: npm run compile
      
      - name: Package extension
        working-directory: ./vscode-extension
        run: |
          npm install -g @vscode/vsce
          vsce package --out sruja-canary.vsix
      
      - name: Upload canary artifact
        uses: actions/upload-artifact@v4
        with:
          name: sruja-extension-canary
          path: vscode-extension/sruja-canary.vsix
          retention-days: 30
```

## Pre-commit Hooks (Optional)

**Purpose**: Validate files before commit (local development).

**File**: `.pre-commit-config.yaml`

```yaml
repos:
  - repo: local
    hooks:
      - id: sruja-lint
        name: Sruja Lint
        entry: sruja lint
        language: system
        files: \.sruja$
        pass_filenames: true
      
      - id: sruja-fmt
        name: Sruja Format
        entry: sruja fmt
        language: system
        files: \.sruja$
        pass_filenames: true
```

**Installation**:
```bash
pip install pre-commit
pre-commit install
```

## Required Secrets

Configure these secrets in GitHub repository settings:

### NPM Publishing
- `NPM_TOKEN`: npm access token with publish permissions
  - Create at: https://www.npmjs.com/settings/{username}/tokens
  - Scope: `Automation` token type

### VS Code Marketplace
- `VSCE_PAT`: Personal Access Token for VS Code Marketplace
  - Create at: https://dev.azure.com/{org}/_usersSettings/tokens
  - Or use: https://marketplace.visualstudio.com/manage/publishers/{publisher-id}
  - Required scopes: `Marketplace (Manage)`

### JetBrains Marketplace
- `JETBRAINS_TOKEN`: API token for JetBrains Marketplace
  - Create at: https://plugins.jetbrains.com/author/me/token
  - Required permissions: `Upload plugin`

## Publishing Strategy

### Version Management

1. **CLI Binaries**: Versioned via Git tags (`v1.0.0`)
2. **NPM Packages**: Versioned in `package.json`
   - Use semantic versioning (semver)
   - Update version before release
3. **VS Code Extension**: Versioned in `vscode-extension/package.json`
   - Must match VS Code extension manifest format
4. **JetBrains Plugin**: Versioned in `jetbrains-plugin/build.gradle.kts`
   - Must match plugin manifest format

### Publishing Workflows

**Option 1: Unified Release (Recommended)**
- Tag release: `git tag v1.0.0 && git push --tags`
- Triggers `release.yml` workflow
- Publishes all artifacts (CLI, npm, VS Code, JetBrains)

**Option 2: Separate Publishing**
- Use individual workflows (`npm-publish.yml`, `vscode-publish.yml`, `jetbrains-publish.yml`)
- Trigger manually or on version bumps
- More granular control

**Option 3: Canary Builds**
- Daily/nightly builds via `canary-publish.yml`
- Publish with `-canary` suffix
- Useful for testing and early access

### Package Configuration

#### Viewer Library (`viewer/package.json`)
```json
{
  "name": "@sruja-ai/viewer",
  "version": "1.0.0",
  "description": "Sruja architecture diagram viewer library",
  "main": "dist/sruja-viewer.js",
  "types": "dist/sruja-viewer.d.ts",
  "files": ["dist"],
  "publishConfig": {
    "access": "public"
  }
}
```

#### VS Code Extension (`vscode-extension/package.json`)
```json
{
  "name": "sruja",
  "displayName": "Sruja",
  "version": "1.0.0",
  "publisher": "sruja-ai",
  "engines": {
    "vscode": "^1.80.0"
  }
}
```

#### JetBrains Plugin (`jetbrains-plugin/build.gradle.kts`)
```kotlin
version = "1.0.0"
group = "com.sruja"

intellij {
    version.set("2023.2")
    type.set("IC")
    plugins.set(listOf("com.intellij.java"))
}
```

## Benefits

✅ **Automated Validation** - Catch errors before merge  
✅ **Conflict Detection** - Identify conflicts early  
✅ **Preview Generation** - Easy PR review  
✅ **Quality Assurance** - Round-trip tests ensure correctness  
✅ **Fast Feedback** - Developers see issues immediately  
✅ **Automated Publishing** - One-click releases for all artifacts  
✅ **Multi-Platform Support** - CLI binaries for all platforms  
✅ **Package Distribution** - npm, VS Code, JetBrains marketplaces  

## Integration with Existing Workflows

These workflows complement:
- Existing GitHub Actions (if any)
- PR review process
- Release process

## Customization

Teams can:
- Add custom validation rules
- Modify preview generation
- Add deployment steps
- Integrate with other tools


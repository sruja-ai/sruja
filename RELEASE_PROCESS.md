# Release Process

## Overview

We use a unified release workflow that handles:

1. **GitHub Release Creation** - Creates version tag and GitHub release
2. **VS Code Extension** - Publishes to VS Code Marketplace and Open VSX Registry
3. **Production Website** - Deploys to sruja-ai/prod-website
4. **Designer App** - Deploys to sruja-ai/prod-website/designer
5. **Go Binaries** - Creates release artifacts via GoReleaser

## How to Release

### Option 1: Manual Release (Recommended)

1. Go to **Actions** → **Unified Release** → **Run workflow**
2. **Leave version empty** (auto-detects from latest tag + commits)
   - Or optionally specify version: `v1.2.3` or `1.2.3`
   - Or choose bump type: `auto`, `patch`, `minor`, or `major`
3. Click **Run workflow**

The workflow will:

- **Auto-detect next version** from latest tag and commit messages
- Create git tag
- Create GitHub release
- Publish VS Code extension to VS Code Marketplace and Open VSX Registry
- Deploy to production website
- Deploy designer app
- Create Go release binaries

**Version Detection Logic:**

- **Major bump**: If commits contain `BREAKING CHANGE` or `!:`
- **Minor bump**: If commits contain `feat:` (new features)
- **Patch bump**: Otherwise (bug fixes, docs, etc.)

### Option 2: Tag-Based Release

1. Create and push a tag:

   ```bash
   git tag -a v1.2.3 -m "Release v1.2.3"
   git push origin v1.2.3
   ```

2. The workflow automatically triggers and performs all release steps

## Version Format

- Use semantic versioning: `v1.2.3` or `1.2.3`
- Format: `MAJOR.MINOR.PATCH`
- Examples: `v1.0.0`, `v1.2.3`, `v2.0.0`

## Release Verification

All release tags are GPG signed using the project signing key. To verify a release:

```bash
# Import the public key
gpg --import .github/gpg-public-key.asc

# Verify a tag
git tag -v v1.2.3
```

The public key is available at [`.github/gpg-public-key.asc`](.github/gpg-public-key.asc).

## What Gets Released

### VS Code Extension

- Version updated in `package.json`
- Published to VS Code Marketplace
- Published to Open VSX Registry (for VSCodium and other VS Code alternatives)
- Uses version from release tag

### Production Website

- Deployed to `sruja-ai/prod-website` repository
- Available at `https://sruja.ai`
- Includes all documentation and content

### Designer App

- Deployed to `sruja-ai/prod-website/designer/`
- Available at `https://sruja.ai/designer/`
- Includes all examples and assets

### Go Binaries

- Created via GoReleaser
- Attached to GitHub release
- Available for download

## Current Workflows

### Unified Release (`unified-release.yml`)

- **Trigger**: Tag push or manual dispatch
- **Does**: Everything in one workflow
- **Use this for**: Production releases

### Legacy Workflows (Still Active)

- `release.yml` - Go binaries only (tag-based)
- `publish-extension.yml` - Extension only (main branch)
- `deploy-astro-website.yml` - Website only (branch-based)

## Migration Plan

1. ✅ **Phase 1**: Create unified release workflow
2. ⏳ **Phase 2**: Test unified workflow
3. ⏳ **Phase 3**: Update documentation
4. ⏳ **Phase 4**: Deprecate individual workflows (optional)

## Benefits of Unified Release

- ✅ Single workflow for all releases
- ✅ Consistent versioning across all components
- ✅ Atomic releases (all or nothing)
- ✅ Clear release history
- ✅ Easy to trigger manually
- ✅ Version synchronization guaranteed

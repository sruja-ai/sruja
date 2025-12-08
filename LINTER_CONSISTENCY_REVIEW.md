# Linter Consistency Review

## Current State

### ✅ Go Linting (Consistent)

**Configuration:**
- **File**: `.golangci.yml` (root)
- **Version**: v2.6.2 (consistent in Makefile and CI)
- **Makefile**: `make lint` runs golangci-lint
- **CI**: Uses `golangci/golangci-lint-action@v9` with version v2.6.2

**Status:** ✅ **Consistent** - Single configuration, consistent version

### ⚠️ TypeScript/JavaScript Linting (Inconsistent)

**Root Configuration:**
- **File**: `.eslintrc.json` (root)
- **Root Script**: `npm run lint` → `turbo run lint`

**Package Lint Scripts:**

| Package/App | Has Lint Script? | Command | Extensions |
|------------|------------------|---------|------------|
| `packages/shared` | ✅ | `eslint . --ext ts` | ts |
| `packages/ui` | ✅ | `eslint . --ext ts,tsx` | ts,tsx |
| `packages/viewer` | ✅ | `eslint . --ext ts` | ts |
| `packages/html-viewer` | ✅ | `eslint . --ext ts` | ts |
| `apps/website` | ❌ | Missing | - |
| `apps/studio-core` | ❌ | Missing | - |
| `apps/viewer-core` | ❌ | Missing | - |
| `apps/vscode-extension` | ❌ | Missing | - |
| `apps/storybook` | ❌ | Missing | - |
| `apps/social-publish` | ❌ | Missing | - |

**ESLint Versions:**
- All packages use: `eslint@^8.57.0` ⚠️ (ESLint v9.39.1 is available - see [release notes](https://github.com/eslint/eslint/releases/tag/v9.39.1))
- All packages use: `@typescript-eslint/eslint-plugin@^7.0.0`
- All packages use: `@typescript-eslint/parser@^7.0.0`

**Note:** ESLint v9 introduced breaking changes with a new flat config format. Migration from v8 requires updating configuration files. Consider upgrading to v9 for better performance and new features, but this requires:
1. Migrating from `.eslintrc.json` to `eslint.config.js` (flat config)
2. Updating all ESLint-related dependencies
3. Testing across all packages and apps

**Status:** ⚠️ **Inconsistent** - Packages have lint scripts, but apps don't

## Issues Found

### 1. Missing Lint Scripts in Apps

**Problem:** Apps don't have lint scripts, so `npm run lint` at root won't lint them.

**Affected Apps:**
- `apps/website`
- `apps/studio-core`
- `apps/viewer-core`
- `apps/vscode-extension`
- `apps/storybook`
- `apps/social-publish`

### 2. ESLint Config Inheritance

**Current:** Root `.eslintrc.json` exists, but apps may not inherit it properly.

**Issue:** Apps without their own `.eslintrc.json` should inherit from root, but they may not be configured to do so.

### 3. Extension Mismatch

**Problem:** `packages/ui` uses `--ext ts,tsx` (correct for React), but other packages use only `--ext ts`. This is actually correct since they don't have TSX files, but it's inconsistent in format.

## Recommendations

### High Priority

1. **Add lint scripts to all apps**
   - Add `"lint": "eslint . --ext ts,tsx"` to apps with React
   - Add `"lint": "eslint . --ext ts"` to apps without React
   - Ensure apps inherit root ESLint config

2. **Verify ESLint config inheritance**
   - Ensure all apps/packages use root `.eslintrc.json`
   - Add explicit `"root": false` in app configs if they need overrides

### Medium Priority

3. **Standardize lint script format**
   - Use consistent format: `eslint . --ext ts,tsx` (even if only ts is needed)
   - Or document why extensions differ

4. **Add lint:fix scripts**
   - Add `"lint:fix": "eslint . --ext ts,tsx --fix"` to all packages/apps
   - Add to root: `"lint:fix": "turbo run lint:fix"`

### Low Priority

5. **Prettier integration**
   - Root has `prettier` in devDependencies
   - Consider adding `format` script to all packages/apps
   - Or use ESLint's formatting rules

## Summary

**Go Linting:** ✅ **Consistent**
- Single configuration file (`.golangci.yml`)
- Consistent version (v2.6.2) across Makefile and CI
- All Go code uses the same linter configuration

**TypeScript/JavaScript Linting:** ⚠️ **Inconsistent**
- ✅ Packages have lint scripts (shared, ui, viewer, html-viewer)
- ✅ ESLint versions are consistent across packages (`^8.57.0`, `^7.0.0`)
- ❌ Apps missing lint scripts (website, studio-core, viewer-core, vscode-extension, storybook, social-publish)
- ❌ Apps missing ESLint dependencies
- ⚠️ Config inheritance unclear (root `.eslintrc.json` exists but apps may not use it)

## ✅ **COMPLETED: ESLint v9 Upgrade & Shared Config**

**Status:** All tasks completed! ✅

1. ✅ **Upgraded to ESLint v9.39.1** across all packages and apps
2. ✅ **Created shared ESLint config** (`@sruja/eslint-config`)
   - Base config for TypeScript projects
   - React config for React/TSX projects
3. ✅ **Migrated from `.eslintrc.json`** to flat config (`eslint.config.js`)
4. ✅ **Updated all packages** to use shared config
5. ✅ **Added lint scripts** to all apps
6. ✅ **Added ESLint dependencies** to all apps

**New Structure:**
- `packages/eslint-config/` - Shared ESLint configuration package
- All packages/apps use `@sruja/eslint-config` or `@sruja/eslint-config/react`
- Consistent ESLint v9 flat config format across the monorepo
- Root `lint:fix` script added for auto-fixing

**Next Step:**
- Run `npm install` to install new dependencies
- Test with `npm run lint` at root


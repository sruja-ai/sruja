# Unused Code Detection - Complete Guide

## 🎯 Overview

This guide provides comprehensive methods to detect unused code in the Sruja codebase, supporting both TypeScript/JavaScript and Go.

## 🚀 Quick Start

### Run All Checks
```bash
# Comprehensive check (TypeScript + Go)
make check-unused

# Or use the script
./scripts/check-unused-code.sh
```

### TypeScript/JavaScript Only
```bash
npm run check:unused:all
```

### Go Only
```bash
staticcheck ./...
go vet ./...
```

## 📋 Detection Methods

### 1. Automated Tools

#### TypeScript/JavaScript

| Tool | Purpose | Command |
|------|---------|---------|
| **ts-prune** | Find unused exports | `npx ts-prune --project tsconfig.json` |
| **unimported** | Find unused files & deps | `npx unimported` |
| **depcheck** | Find unused npm packages | `npx depcheck` |
| **ESLint** | Find unused variables | Already in lint script |
| **TypeScript** | Built-in unused detection | `tsc --noUnusedLocals` |

#### Go

| Tool | Purpose | Command |
|------|---------|---------|
| **staticcheck** | Comprehensive unused code | `staticcheck ./...` |
| **go vet** | Unused variables/imports | `go vet ./...` |
| **golangci-lint** | Includes unused checks | `make lint` |

### 2. IDE Features

#### VS Code / Cursor

1. **Visual Indicators**:
   - Grayed-out imports = Unused
   - Wavy underlines = Unused variables
   - Dimmed code = Potentially unused

2. **Find References**:
   - Right-click symbol → "Find All References" (Shift+F12)
   - If no references found → likely unused

3. **Go to Definition**:
   - Check if exported symbols are imported elsewhere

### 3. Manual Techniques

#### Git Analysis
```bash
# Find files not modified in 6 months
git log --since="6 months ago" --name-only --pretty=format: | sort -u

# Find functions with no references
git grep -n "functionName" -- "*.ts" "*.tsx"
```

#### Code Search
```bash
# Check if component is imported
grep -r "import.*ComponentName" apps/studio-core/src

# Check if function is called
grep -r "functionName(" apps/studio-core/src
```

## 🔧 Setup & Configuration

### Installed Tools

✅ **ts-prune** - Unused exports detection
✅ **unimported** - Unused files & dependencies
✅ **depcheck** - Unused npm packages
✅ **staticcheck** - Go unused code (via Makefile)

### Configuration Files

- `.unimportedrc.json` - Configures unimported tool
- `tsconfig.json` - TypeScript config (enables unused detection)
- `.golangci.yml` - Go linter config

## 📊 Current Status

### Recent Cleanup

✅ **Builder Mode References Removed**:
- Deleted 6 unused BuilderMode components
- Removed BuilderModeStore
- Moved 5 components to proper locations
- Updated all imports

### Unused Exports Found (Sample)

From `ts-prune` scan:
- `ActivityBar` - Not imported
- `EmptyState` - Not imported  
- `ErrorBoundary` - Not imported
- `ExportDialog` - Not imported
- `InlineDocs` - Not imported
- `StepGuide` - Not imported
- `StudioSidebar` - Not imported
- `ViewerToolbar` - Not imported

**Note**: Some may be used dynamically or in tests. Verify before removing.

## 🎯 Best Practices

### When to Check

1. **Before Major Refactors** - Clean up unused code first
2. **After Feature Removal** - Remove orphaned code
3. **Monthly Audits** - Regular maintenance
4. **Before Releases** - Final cleanup

### How to Verify

1. **Check References** - Use IDE "Find All References"
2. **Check Tests** - May be used in test files
3. **Check Dynamic Usage** - Runtime imports, reflection
4. **Check Public APIs** - Exported for external use

### Safe to Remove

✅ Unused private functions
✅ Unused internal types
✅ Unused utility functions (if truly unused)
✅ Dead code paths
✅ Commented-out code

### Keep (Even if Appears Unused)

⚠️ Entry points (`main.tsx`, `index.ts`)
⚠️ Type-only exports (TypeScript)
⚠️ Public API exports
⚠️ Test utilities
⚠️ Dynamic imports
⚠️ Reflection-based code (Go)

## 🔄 Integration

### CI/CD Integration

Add to `.github/workflows/code-quality.yml`:

```yaml
- name: Check unused code
  run: |
    make check-unused
    npm run check:unused:all || true  # Don't fail CI, just report
```

### Pre-commit Hook (Optional)

```bash
# Add to .husky/pre-commit
npm run check:unused || echo "⚠️  Unused code detected (non-blocking)"
```

## 📈 Metrics

### Code Reduction Potential

Based on initial scan:
- **Unused Exports**: ~15-20 components/functions
- **Unused Files**: To be determined
- **Unused Dependencies**: To be determined

### Estimated Impact

- **Bundle Size**: Could reduce by 5-10% after cleanup
- **Maintainability**: Improved with less dead code
- **Onboarding**: Easier with cleaner codebase

## 🛠️ Troubleshooting

### False Positives

If tool reports unused but code is needed:

1. **Add to ignore list** (tool-specific)
2. **Document why** - Add comment explaining usage
3. **Verify usage** - Double-check with IDE search

### Tools Not Working

```bash
# Reinstall tools
npm install --save-dev ts-prune unimported depcheck

# Install Go tools
go install honnef.co/go/tools/cmd/staticcheck@latest
```

## 📚 Resources

- **ts-prune**: https://github.com/nadeesha/ts-prune
- **unimported**: https://github.com/smeijer/unimported
- **staticcheck**: https://staticcheck.io/
- **Full Guide**: See `UNUSED_CODE_DETECTION.md`

## ✅ Next Steps

1. ✅ Tools installed
2. ✅ Scripts configured
3. ✅ Makefile updated
4. ⏭️ Run initial full scan
5. ⏭️ Create cleanup plan
6. ⏭️ Integrate into CI/CD

# Unused Code Detection Guide

This document outlines tools and techniques to identify unused code in the Sruja codebase.

## Tools Overview

### TypeScript/JavaScript

1. **ts-prune** - Finds unused exports
2. **unimported** - Finds unused files, dependencies, and exports
3. **depcheck** - Finds unused dependencies
4. **TypeScript compiler** - Built-in unused variable detection
5. **ESLint** - `no-unused-vars` rule

### Go

1. **unused** - Finds unused code (functions, types, variables)
2. **deadcode** - Finds unreachable code
3. **go-tools** - Staticcheck includes unused code detection
4. **go vet** - Built-in unused variable detection

## Setup & Usage

### TypeScript/JavaScript Detection

#### 1. Install Tools

```bash
npm install --save-dev ts-prune unimported depcheck
```

#### 2. Add Scripts to package.json

```json
{
  "scripts": {
    "check:unused": "ts-prune",
    "check:unused:files": "unimported",
    "check:unused:deps": "depcheck",
    "check:unused:all": "npm run check:unused && npm run check:unused:files && npm run check:unused:deps"
  }
}
```

#### 3. Configure ts-prune

Create `tsconfig.prune.json`:
```json
{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "skipLibCheck": true
  }
}
```

#### 4. Configure unimported

Create `.unimportedrc.json`:
```json
{
  "ignorePatterns": [
    "**/*.test.ts",
    "**/*.test.tsx",
    "**/*.spec.ts",
    "**/*.spec.tsx",
    "**/index.ts",
    "**/index.tsx"
  ]
}
```

### Go Detection

#### 1. Install Tools

```bash
go install honnef.co/go/tools/cmd/staticcheck@latest
go install github.com/gordonklaus/ineffassign@latest
```

#### 2. Add to Makefile

```makefile
.PHONY: check-unused
check-unused:
	@echo "Checking for unused Go code..."
	@staticcheck ./...
	@go vet ./...
	@ineffassign ./...
```

## Automated Detection Scripts

### TypeScript/JavaScript

Run these commands to find unused code:

```bash
# Find unused exports
npm run check:unused

# Find unused files and dependencies
npm run check:unused:files

# Find unused npm packages
npm run check:unused:deps
```

### Go

```bash
# Run staticcheck (includes unused code detection)
staticcheck ./...

# Run go vet
go vet ./...

# Check for ineffectual assignments
ineffassign ./...
```

## Manual Detection Techniques

### 1. IDE Features

**VS Code:**
- Right-click on symbol → "Find All References"
- Grayed-out imports indicate unused
- TypeScript/Go extensions show unused warnings

**Cursor:**
- Similar to VS Code
- Use "Go to References" (Shift+F12)
- Check for "unused" warnings

### 2. Git Analysis

```bash
# Find files that haven't been modified recently
git log --since="6 months ago" --name-only --pretty=format: | sort -u

# Find functions that haven't been referenced
git grep -n "functionName" -- "*.ts" "*.tsx"
```

### 3. Code Search

```bash
# Search for imports/exports
grep -r "export.*ComponentName" apps/studio-core/src
grep -r "import.*ComponentName" apps/studio-core/src

# If import count is 0, component might be unused
```

## Common Patterns of Unused Code

1. **Unused Exports** - Exported but never imported
2. **Unused Imports** - Imported but never used
3. **Dead Functions** - Functions never called
4. **Unused Types** - Type definitions never referenced
5. **Unused Components** - React components never rendered
6. **Unused Constants** - Constants never referenced
7. **Commented Code** - Old code left in comments

## Integration with CI/CD

Add to `.github/workflows/code-quality.yml`:

```yaml
- name: Check for unused code
  run: |
    npm run check:unused:all
    make check-unused
```

## Best Practices

1. **Regular Audits** - Run unused code checks monthly
2. **Before Refactoring** - Check for unused code before major refactors
3. **After Merges** - Check after merging large features
4. **Pre-commit** - Consider adding to pre-commit hooks (optional)
5. **Documentation** - Document why code is kept if it appears unused but is needed

## False Positives

Some code may appear unused but is actually needed:

- **Entry points** - `main.tsx`, `index.ts`
- **Type definitions** - Used for type checking only
- **Test utilities** - Used in tests
- **Dynamic imports** - Code loaded at runtime
- **Reflection** - Go code accessed via reflection
- **Public APIs** - Exported for external use

## Tools Comparison

| Tool | Language | Detects | Speed | Accuracy |
|------|----------|---------|-------|----------|
| ts-prune | TS/JS | Unused exports | Fast | High |
| unimported | TS/JS | Files, deps, exports | Medium | High |
| depcheck | TS/JS | Unused packages | Fast | Medium |
| staticcheck | Go | Unused code | Fast | High |
| go vet | Go | Unused vars | Fast | High |

## Next Steps

1. Install detection tools
2. Run initial scan
3. Create baseline report
4. Set up automated checks
5. Create cleanup plan




# Quick Unused Code Detection

## Quick Commands

### All-in-One Check
```bash
# Run comprehensive unused code check
make check-unused

# Or use the script directly
./scripts/check-unused-code.sh
```

### TypeScript/JavaScript Specific

```bash
# Check for unused exports
npm run check:unused

# Check for unused files and dependencies  
npm run check:unused:files

# Check for unused npm packages
npm run check:unused:deps

# Run all TypeScript checks
npm run check:unused:all
```

### Go Specific

```bash
# Check with staticcheck (includes unused code)
staticcheck ./...

# Check with go vet
go vet ./...

# Or use Makefile
make lint  # Includes unused code detection
```

## IDE Integration

### VS Code / Cursor

1. **TypeScript**: Unused imports/variables show grayed out
2. **Go**: Unused code shows warnings
3. **Find References**: Right-click → "Find All References" (Shift+F12)
4. **Go to References**: Check if symbol is used elsewhere

### Quick Checks in IDE

- **Grayed imports** = Unused imports
- **Unused variable warnings** = Dead code
- **No references found** = Potentially unused

## Common Unused Code Patterns

### TypeScript/JavaScript
- ✅ **Unused exports** - `export function X()` but never imported
- ✅ **Unused imports** - `import { X }` but X never used
- ✅ **Unused files** - Files never imported anywhere
- ✅ **Unused dependencies** - Packages in package.json but never used

### Go
- ✅ **Unused functions** - Functions never called
- ✅ **Unused types** - Types never referenced
- ✅ **Unused variables** - Variables declared but never used
- ✅ **Unused imports** - Imports never used

## Automated Detection

The script `scripts/check-unused-code.sh` automatically:
1. Checks TypeScript/JavaScript for unused exports
2. Checks for unused files and dependencies
3. Checks Go code with staticcheck and go vet
4. Provides summary report

## Integration with CI/CD

Add to `.github/workflows/code-quality.yml`:

```yaml
- name: Check for unused code
  run: |
    make check-unused
    npm run check:unused:all
```

## Best Practices

1. **Run regularly** - Monthly or before major releases
2. **Before refactoring** - Clean up unused code first
3. **After feature removal** - Check for orphaned code
4. **Review carefully** - Some code may be used dynamically

## False Positives to Watch For

- Entry points (`main.tsx`, `index.ts`)
- Type-only imports (TypeScript)
- Dynamic imports (runtime loading)
- Reflection-based code (Go)
- Public API exports (intended for external use)
- Test utilities (used in tests)




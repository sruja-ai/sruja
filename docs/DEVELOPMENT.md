# Development Guide

This guide covers general development practices. For content creation, see [Content Contribution Guide](CONTENT_CONTRIBUTION_GUIDE.md).

## Quick Start

```bash
# Install dependencies
make install

# Build the CLI
make build

# Run tests
make test

# Format code
make fmt

# Lint code
make lint
```

## Git Hooks

### Pre-commit Hook

A pre-commit hook automatically tests Sruja code compilation when you commit changes to the `examples/` directory.

**What it does:**
- Runs compilation tests for playground examples
- Runs compilation tests for course code blocks
- Runs compilation tests for docs code blocks
- Prevents commits if any code fails to compile

**Setup:**

```bash
# Option 1: Use Makefile
make setup-hooks

# Option 2: Run script directly
./scripts/setup-git-hooks.sh
```

**How it works:**
- Only runs when you commit files in `examples/` directory
- Runs `go test` for compilation tests
- Blocks commit if tests fail
- Shows helpful error messages

**Bypass (not recommended):**
```bash
git commit --no-verify
```

**Manual test:**
```bash
# Test the hook manually
.git/hooks/pre-commit
```

## Running Tests

### Compilation Tests

Test that all playground examples compile correctly:

```bash
# Generate examples and test compilation
make generate-examples

# Or run tests directly
go test -v -run "TestPlaygroundExamples"
```

### All Tests

```bash
# Run all Go tests
make test

# Run with coverage
make test-coverage
```

## Test Coverage

Test coverage reporting is configured for TypeScript/JavaScript packages using Vitest and Codecov.

### Coverage Thresholds

**@sruja/shared:**
- Lines: 60%
- Functions: 60%
- Branches: 50%
- Statements: 60%

**@sruja/ui:**
- Lines: 50%
- Functions: 50%
- Branches: 40%
- Statements: 50%

### Running Coverage Locally

```bash
# Run coverage for all packages
npm run test:coverage

# Run coverage for specific package
npm run test:coverage --filter=@sruja/shared
npm run test:coverage --filter=@sruja/ui
```

### Viewing Coverage

- **Local**: Open `coverage/index.html` in browser after running `npm run test:coverage`
- **CI**: View in Codecov dashboard or PR comments
- **Badge**: Coverage badge in README shows current coverage

### Improving Coverage

1. Run `npm run test:coverage` locally
2. Open `coverage/index.html` to see uncovered lines
3. Add tests for uncovered code
4. Re-run to verify improvement

## Bundle Size Monitoring

Bundle size monitoring is configured using `size-limit` to track and prevent bundle bloat in TypeScript packages.

### Current Limits

**@sruja/shared:**
- Limit: 100 KB
- Purpose: Keep shared utilities lightweight

**@sruja/ui:**
- Limit: 500 KB
- Purpose: Keep UI component library reasonable

### Running Bundle Size Checks

```bash
# Check bundle sizes
npm run size

# See detailed breakdown
npm run size:why
```

### CI Integration

Bundle size checks run automatically in CI:
1. Packages are built
2. Size limits are checked
3. CI continues even if limits exceeded (for now)
4. Results are visible in CI logs

### Adjusting Limits

Edit `.size-limit.json` or package-specific `size-limit` config in `package.json`.

### Best Practices

1. **Monitor regularly**: Check sizes before major releases
2. **Set realistic limits**: Based on current size + 20% buffer
3. **Investigate increases**: Use `size:why` to understand what's growing
4. **Optimize imports**: Use tree-shaking friendly imports
5. **Code splitting**: Split large features into separate bundles


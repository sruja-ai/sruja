# Development Guide

This guide covers general development practices. For content creation, see [Content Contribution Guide](CONTENT_CONTRIBUTION_GUIDE.md).

## Git Hooks

### Pre-commit Hook

A pre-commit hook is set up to automatically test Sruja code compilation when you commit changes to the `examples/` directory.

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


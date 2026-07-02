# run-tests

## Why It Matters

Tests ensure code correctness and prevent regressions. Running tests early catches issues before they compound.

## When to Apply

- Before committing changes
- After adding new functionality
- After refactoring existing code
- Before creating a pull request

## Correct Approach

1. **Run all tests**:
   ```bash
   cargo test --workspace
   ```

2. **Run specific crate tests**:
   ```bash
   cargo test -p sruja-cli
   cargo test -p sruja-language
   ```

3. **Run single test**:
   ```bash
   cargo test test_name
   ```

4. **Run with coverage**:
   ```bash
   just test-coverage
   ```

5. **Run linting**:
   ```bash
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

## Incorrect Approach

- Skipping tests before committing
- Only running tests for changed code
- Ignoring clippy warnings

## Summary

**Run tests: all tests → specific crate → single test → coverage → lint.**

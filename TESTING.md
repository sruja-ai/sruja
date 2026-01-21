# Testing the Rust Migration

This document describes how to test the migrated Rust codebase.

## Prerequisites

1. **Install Rust toolchain**:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Verify installation**:
   ```bash
   cargo --version
   rustc --version
   ```

## Running Tests

### Quick Test Script

```bash
bash test_rust.sh
```

### Manual Testing

#### 1. Check Compilation

```bash
# Check all crates compile
cargo check --workspace

# Check specific crate
cargo check -p sruja-language
cargo check -p sruja-engine
cargo check -p sruja-export
cargo check -p sruja-lsp
cargo check -p sruja-cli
```

#### 2. Run All Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p sruja-language
cargo test -p sruja-engine
```

#### 3. Test CLI Commands

```bash
# Build the CLI
cargo build --release -p sruja-cli

# Test basic commands
./target/release/sruja --version
./target/release/sruja lint examples/simple.sruja
./target/release/sruja export json examples/simple.sruja
```

#### 4. Test LSP Server

```bash
# Build LSP server
cargo build --release -p sruja-lsp

# Test LSP (requires LSP client)
# Use with VS Code or other LSP client
```

## Test Coverage

### Language Parser Tests

- ✅ Simple system parsing
- ✅ Relation parsing
- ✅ Nested elements parsing
- ✅ Metadata parsing
- ✅ SLO blocks parsing

### Validation Engine Tests

- ✅ Unique ID rule
- ✅ Valid reference rule
- ✅ Cycle detection
- ✅ Orphan detection
- ✅ All 12 validation rules

### Export Tests

- ✅ JSON export
- ✅ Mermaid export
- ✅ Dot export
- ✅ Markdown export
- ✅ Context export
- ✅ DSL printer

### LSP Tests

- ✅ Hover functionality
- ✅ Completion
- ✅ Go to definition
- ✅ Find references
- ✅ Document symbols
- ✅ Formatting
- ✅ Rename

## Integration Tests

### Test with Example Files

```bash
# Test parsing example files
for file in examples/*.sruja; do
    echo "Testing $file"
    cargo run --bin sruja -- lint "$file"
done
```

### Compare Go vs Rust Output

```bash
# Run Go version
go run cmd/sruja/main.go export json examples/simple.sruja > go_output.json

# Run Rust version
cargo run --bin sruja -- export json examples/simple.sruja > rust_output.json

# Compare (should be similar)
diff go_output.json rust_output.json
```

## Performance Testing

```bash
# Benchmark parsing
cargo bench --bench parser

# Profile with perf (Linux)
perf record --call-graph dwarf cargo test
perf report
```

## Continuous Integration

Tests should run automatically in CI:

- On every push
- On pull requests
- Before merging to main

## Known Issues

- Some edge cases in parser may need additional test coverage
- LSP workspace symbols not yet fully tested
- Code actions need implementation and tests

## Adding New Tests

1. Add unit tests in the same file with `#[cfg(test)]`
2. Add integration tests in `tests/` directory
3. Follow Rust testing conventions
4. Test both success and failure cases

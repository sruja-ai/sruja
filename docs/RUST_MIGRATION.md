# Go to Rust Migration Guide

This branch contains the **pure Rust migration** of Sruja. All Go code is being migrated to Rust without maintaining backward compatibility layers.

## Overview

This is a **clean-slate Rust implementation** of Sruja. We're migrating components one at a time, ensuring feature parity with the original Go implementation.

## Architecture

### Current Structure

```
crates/
  sruja-diagnostics/    # Rust implementation of diagnostics
```

### Build System

- **Pure Rust**: All builds use `cargo`
- **No Go interop**: FFI bindings and Go compatibility layers have been removed
- **Rust-only tooling**: Uses standard Rust toolchain (cargo, clippy, rustfmt)

## Migration Status

### ✅ Completed

- [x] **Diagnostics Package** (`crates/sruja-diagnostics`)
  - Full Rust implementation
  - Types: Diagnostic, SourceLocation, Severity
  - Error codes and constants
  - Error reporter (BasicErrorReporter)
  - Formatted output matching Go implementation
  - Comprehensive test coverage

### 🔄 In Progress

- None currently

### 📋 Planned

- [ ] Language Parser (`crates/sruja-language`)
- [ ] Validation Engine (`crates/sruja-engine`)
- [ ] Exporters (`crates/sruja-export`)
- [ ] LSP Server (`crates/sruja-lsp`)
- [ ] CLI (`crates/sruja-cli`)

## Usage

### For Developers

#### Prerequisites

Install Rust if you haven't already:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Building

```bash
# Build all Rust crates
make build

# Or use cargo directly
cargo build --release
```

#### Testing

```bash
# Run all tests
make test

# Or use cargo directly
cargo test

# Run tests for a specific crate
cargo test -p sruja-diagnostics
```

#### Code Quality

```bash
# Format code
make fmt
# or
cargo fmt

# Lint code
make lint
# or
cargo clippy -- -D warnings

# Format check (CI)
cargo fmt --check
```

### For Contributors

When migrating a new component:

1. **Create Rust crate** in `crates/sruja-<component>/`
2. **Add to workspace** in root `Cargo.toml`
3. **Implement features** with tests
4. **Ensure feature parity** with Go implementation
5. **Update this document** with migration status

## Testing Strategy

### Unit Tests

Each crate has comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

### Integration Tests

Integration tests verify:

- Feature parity with Go implementation
- Correct behavior across component boundaries
- Performance characteristics

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p sruja-diagnostics

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

## Performance Considerations

### Benefits of Rust

- **Memory safety** - No GC pauses, zero-cost abstractions
- **Performance** - Compiled to native code
- **Concurrency** - Excellent parallel processing support
- **Type safety** - Compile-time guarantees

### Migration Impact

- **Smaller binaries** - No runtime overhead
- **Faster execution** - Native compilation
- **Better resource usage** - No garbage collection

## Troubleshooting

### Cargo Not Found

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build Errors

```bash
# Clean and rebuild
make clean
make build

# Or with cargo
cargo clean
cargo build
```

### Test Failures

```bash
# Run with more output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Project Structure

```
crates/
  sruja-diagnostics/      # Diagnostics system
    src/
      lib.rs              # Main library code
    Cargo.toml            # Crate configuration
    tests/                # Integration tests (if any)

Cargo.toml               # Workspace configuration
Makefile                 # Build automation
docs/
  RUST_MIGRATION.md      # This file
```

## Future Work

1. **Migrate parser** - Most critical component
2. **Migrate validator** - Core business logic
3. **Migrate exporters** - Various output formats
4. **Create CLI** - Command-line interface
5. **WASM support** - Browser compilation target

## References

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

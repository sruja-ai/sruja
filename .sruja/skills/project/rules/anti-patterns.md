# anti-patterns

## Why It Matters

Avoiding anti-patterns prevents architectural violations and maintains code quality over time.

## When to Apply

- Reviewing architecture changes
- Refactoring existing code
- Onboarding new team members

## Anti-Patterns to Avoid

### 1. Layer Violations

**Wrong:** Lower-tier crate depending on higher-tier crate
```rust
// In sruja-language (Core) depending on sruja-cli (Delivery)
use sruja_cli::some_function; // ❌
```

**Right:** Higher-tier crate depends on lower-tier
```rust
// In sruja-cli (Delivery) depending on sruja-language (Core)
use sruja_language::Parser; // ✅
```

### 2. sruja-cli Dependencies

**Wrong:** Other crates depending on sruja-cli
```toml
# In sruja-engine/Cargo.toml
[dependencies]
sruja-cli = { path = "../sruja-cli" } # ❌
```

**Right:** sruja-cli depends on other crates
```toml
# In sruja-cli/Cargo.toml
[dependencies]
sruja-engine = { path = "../sruja-engine" } # ✅
```

### 3. WASM Using Native APIs

**Wrong:** WASM crates using native-only APIs
```rust
// In sruja-wasm
use tree_sitter::Parser; // ❌ tree-sitter is native-only
```

**Right:** WASM crates use only WASM-compatible APIs
```rust
// In sruja-wasm
use sruja_language::Parser; // ✅ Use the language crate's parser
```

### 4. God Modules

**Wrong:** Single module doing everything
```rust
// lib.rs - 1000+ lines with all functionality
pub fn parse() { ... }
pub fn validate() { ... }
pub fn export() { ... }
pub fn scan() { ... }
```

**Right:** Separate modules with single responsibility
```rust
// lib.rs - Public API surface
pub mod parser;
pub mod validator;
pub mod exporter;
pub mod scanner;
```

### 5. Skipping Validation

**Wrong:** Committing without validation
```bash
git add . && git commit -m "changes" # ❌ No validation
```

**Right:** Validate before committing
```bash
cargo test --workspace
cargo clippy -- -D warnings
sruja lint repo.sruja
git add . && git commit -m "changes" # ✅
```

## Summary

**Anti-patterns: avoid layer violations, cli dependencies, WASM native APIs, god modules, skipping validation.**

# Sruja AI Agent Guide

This guide helps AI agents work effectively with the Sruja codebase.

## Build, Lint, and Test Commands

### Rust (Core)

```bash
# Build all crates
cargo build --release
make build

# Run all tests
cargo test --workspace
make test

# Run a single test
cargo test test_name

# Run tests in a specific crate
cargo test -p sruja-cli

# Run tests with coverage
make test-coverage
cargo llvm-cov

# Lint Rust code
cargo clippy -- -D warnings
make lint

# Format Rust code
cargo fmt
make fmt
```

### VS Code Extension (TypeScript)

```bash
cd extension

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Run tests
npm test
npm run test:vscode

# Lint (via tsc strict mode)
npm run compile
```

### WASM

```bash
# Run WASM tests
make test-wasm
cd crates/sruja-wasm && wasm-pack test --node

# Build WASM
make wasm
make wasm-nodejs
```

### E2E Tests

```bash
# Build and serve book first in another terminal
make book-serve

# Run Playwright E2E
make test-e2e
npm run e2e
```

## Code Style Guidelines

### Rust

**Naming Conventions:**
- Types/Enums: PascalCase (`NodeId`, `EdgeKind`)
- Functions/Variables: snake_case (`as_str`, `from_str`)
- Constants: SCREAMING_SNAKE_CASE
- Modules: lowercase (`mod severity`)

**Imports:**
- Group external imports first, then internal
- Use `use` with full paths where appropriate
- Use `pub use` to re-export commonly used types

**Code Organization:**
- Module-level docs with `//!` at top of files
- Function docs with `///`
- Use `#[cfg(test)]` for test modules
- Keep `lib.rs` as public API surface, impl details in other files

**Error Handling:**
- Use `Result<T, Box<dyn std::error::Error>>` for simple cases
- Use `thiserror` for custom error types
- Use `?` operator for propagation
- Provide context with `context()` or `.map_err()`

**Serde:**
- Use `#[serde(rename_all = "snake_case")]` for enums
- Use `#[derive(Serialize, Deserialize)]` for serializable types
- Consider `#[serde(skip_serializing_if = "Option::is_none")]`

**Testing:**
- Unit tests in `#[cfg(test)]` module
- Integration tests in `tests/` directory
- Use descriptive test names
- Test edge cases and error paths

### TypeScript (Extension)

**Naming Conventions:**
- Classes/Interfaces/Types: PascalCase (`DiagnosticCollection`)
- Functions/Variables: camelCase (`getSrujaPath`)
- Constants: UPPER_SNAKE_CASE (`DIAGNOSTIC_COLLECTION_ID`)

**Imports:**
- Use ES6 imports
- Group imports: Node stdlib, external libs, internal modules
- Use `import * as` for namespaces

**Code Style:**
- Use async/await for async operations
- Error handling with try/catch
- Use nullish coalescing `??` and optional chaining `?.`
- Type annotations for function parameters and returns
- Use `undefined` consistently, avoid `null` where possible

**VSCode API:**
- Use `vscode` namespace
- Register commands and providers in `activate()`
- Dispose resources properly
- Use OutputChannel for user-facing messages

### Sruja DSL (.sruja files)

**Structure:**
- Flat top-level declarations (no `architecture "Name" { }` wrapper)
- Define components before use in relationships
- PascalCase for element IDs
- Double quotes for all string values

**Components:**
- Every component must have a `description` field
- Containers must have `technology` field
- Use `person` for human actors only
- Use `system` for external software (APIs, SaaS)
- Use `database` (not `datastore`) for data stores

**Relationships:**
- Syntax: `source -> target "label"`
- Use specific, descriptive labels ("HTTPS", "REST API", "publishes events to")
- Reference nested components: `System.Container`
- Avoid circular dependencies
- No orphan components

**Validation:**
```bash
sruja lint file.sruja
```

## Formatting

All code uses:
- UTF-8 encoding
- LF line endings
- Trim trailing whitespace
- Insert final newline
- 2-space indentation (except Makefiles: tabs)

Run `cargo fmt` for Rust and follow EditorConfig rules.

## Architecture Patterns

The codebase follows a workspace pattern with multiple crates:
- **Core**: types, diagnostics, language, engine, export
- **Product**: graph, intent, report, lsp, wasm
- **CLI**: commands and user interface

WASM is used for browser and Node.js targets. LSP provides language server features.

## Key Commands for AI Agents

When working on Sruja:
1. Run `make lint` and `cargo test` before committing
2. For .sruja files, run `sruja lint file.sruja` after changes
3. Use `cargo clippy -- -D warnings` for strict linting
4. Build extension with `make build-extension`
5. Test CLI commands with `make test-cli-smoke`
6. For Rust coverage gaps (CLI handlers, LSP, WASM, tree-sitter) and infrastructure needs, see `docs/internal/TEST_COVERAGE_PLAN.md`

## Common Patterns

**Rust:**
```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyType {
    pub field: String,
}

#[derive(Error, Debug)]
pub enum MyError {
    #[error("not found: {0}")]
    NotFound(String),
}
```

**TypeScript:**
```typescript
import * as vscode from "vscode";

export async function myFunction(): Promise<void> {
    try {
        const result = await operation();
        return result;
    } catch (err) {
        vscode.window.showErrorMessage(`Error: ${err}`);
    }
}
```

**Sruja DSL:**
```sruja
MyComponent = container "My Component" {
  technology "Node.js"
  description "A deployable service"
}

MyComponent -> Database "SQL"
```

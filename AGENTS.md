This guide helps AI agents work effectively with the Sruja codebase.

## ⚡ Quick Reference (Top 5 Commands)

| Command | Purpose |
|---------|---------|
| `make setup` / `just setup` | **First-run setup** (install deps, hooks, build) |
| `make check` / `just check` | **Pre-commit check** (fmt + lint + test) |
| `make daily` / `just daily` | **Sync context** + check diagnostic drift |
| `sruja focus` | **Task briefing** (blast radius, decisions, AI info) |
| `sruja mcp -r .` | **Start MCP server** for deep context queries |

## Before Coding: Shared Understanding

**IMPORTANT**: From the talks, AI produces garbage without shared understanding. Re-running the compiler just produces more garbage.

### Required Process

Before any significant code change:

1. **Reach shared understanding first** - Use "grill me" or equivalent questioning
2. **Write the spec** - Document intent before code
3. **Verify against spec** - Test matches spec, not just compilation

### Anti-Patterns to Avoid

- ✗ Generating code without clear intent
- ✗ Running compiler repeatedly to fix AI errors
- ✗ Shipping code you can't explain in a post-mortem
- ✗ Merging AI output without verification

### Code Quality Gates

- Run `sruja drift -r .` before merging
- Track code churn with `./scripts/code-churn.sh`
- For Dependabot majors and MSRV-sensitive upgrades, follow [docs/MSRV_AND_DEPENDENCIES.md](docs/MSRV_AND_DEPENDENCIES.md)
- Review security with `./target/release/sruja lint` (if applicable)

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

# Run tests with coverage (excludes sruja-wasm — see docs/WASM_TESTING.md)
make test-coverage
# or: just test-coverage
# WASM bindings are tested separately: just test-coverage-wasm  (alias for wasm-pack tests)
# CI/script variant with extra llvm-cov flags:
#   bash scripts/coverage.sh

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

**Extension (VS Code test-electron):**

```bash
cd extension
# If WASM or language crate changed, copy fresh WASM first:
# (from repo root) bash extension/scripts/copy-assets.sh
npm run test:vscode
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
- **Define before use** – Every referenced component must be defined before use in relationships.
- **Pattern** – `Id = kind "Label" { ... }` (e.g. `API = container "API" { ... }`).
- **Nested syntax following C4 hierarchy**: Systems at top level, containers nested in systems, components nested in containers.
- **Flat top-level declarations only** – No `architecture "Name" { }` wrapper.
- Persons and external systems can be at top level.
- Define kinds at the top of file before using them.
- PascalCase for element IDs.
- Double quotes for all string values.

**Nesting Requirements:**
- `container`, `component`, `database`, `queue` MUST be nested inside a system.
- `component` MUST be nested inside a container (or system for edge cases).
- `system` and `person` can be at top level.
- Use dot notation to reference nested elements: `System.Container`, `System.Container.Component`.

**Components:**
- Every component must have a `description` field.
- Containers must have `technology` field (e.g. "PostgreSQL", "React").
- Use `person` for human actors only.
- Use `system` for external software (APIs, SaaS).
- Use `database` (not `datastore`) for data stores.
- **Unique IDs** – Component IDs must be unique.

**Relationships:**
- Syntax: `source -> target "label"`.
- Use specific, descriptive labels ("HTTPS", "REST API", "publishes events to").
- Reference nested components: `System.Container.Component`.
- Avoid circular dependencies between systems.
- **No orphans** – Every component must participate in at least one relationship.

**Validation:**
```bash
sruja lint file.sruja
```

## AI Editor UX: Validation

After **every** code iteration (each time you apply an edit to a `.sruja` file) in VS Code or Cursor:

1. **Run validation in the editor** – Invoke the command **Sruja: Run validation (check after AI/edit)**. This ensures the file stays valid.
2. **Or save the file** – Saving also triggers validation. For immediate feedback after an AI edit, prefer running the command.

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
1. **First Time Setup**: Run `make setup` to ensure all dependencies and git hooks are correctly installed.
2. **Dogfooding the Architecture**: Before proposing significant PRs, always respect `docs/architecture/*.sruja` as the "reviewed truth". If changing architecture, update those files. Also run `sruja doctor -r .`, `sruja daily -r .`, or `sruja drift -r . -a repo.sruja` to validate the baseline (`repo.sruja`).
3. Run `make check` before committing to ensure consistent formatting, linting, and passing tests.
4. For .sruja files, run `sruja lint file.sruja` after changes.
5. Use `cargo clippy -- -D warnings` for strict linting.
6. Build extension with `make build-extension`.
7. Test CLI commands with `make test-cli-smoke`.
8. For Rust coverage gaps (CLI handlers, LSP, WASM, tree-sitter) and infrastructure needs, see `docs/internal/TEST_COVERAGE_PLAN.md`.

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
MySystem = system "My System" {
  description "A deployable system"

  MyContainer = container "My Container" {
    technology "Node.js"
    description "A deployable service"
  }

  Database = database "Database" {
    technology "PostgreSQL"
    description "Data storage"
  }
}

MySystem.MyContainer -> MySystem.Database "SQL"
```

Sruja provides native integration for AI code editors (Cursor, Trae, Copilot, Cline, Windsurf, etc.) to give them deep context about the cross-repo architecture:

1. **Daily Context Sync**: Run `make daily` to check for architectural drift, build cross-repo context, and automatically update `.cursorrules`, `.copilot-instructions.md`, `CLAUDE.md`, and other editor-specific rules.
2. **Manual Sync**: Run `make context-sync` to force-update all editor context files without running tests/drift checks.
3. **MCP Server**: Configure your AI editor to use the Sruja Model Context Protocol (MCP) server.
   - **Command**: `sruja mcp -r .`
   - **Usage**: The MCP server exposes tools for the AI to query the architecture graph, resolve cross-repo dependencies, and check compliance on the fly.
4. **Cross-Repo Context**: Use `sruja ai-context -r repoA -r repoB` to dynamically build context payloads when working on multi-repo features.
5. **Public GitHub org layout** (product + Pages deploy targets): [docs/RELATED_REPOSITORIES.md](docs/RELATED_REPOSITORIES.md)

When using AI agents, leverage Sruja's context tools:
- **`sruja focus --file <path>`**: Generates a task-scoped briefing (blast radius, decisions, boundaries, AI instructions).
- **`sruja context-score`**: Quantifies the repository's AI-readiness (0-100 score across 5 dimensions).
- **`sruja ingest <doc>`**: Imports external documentation (ADRs, design docs) into `.sruja/context/`.
- **`sruja why <id>`**: Explains the rationale/logic behind a specific architectural component or relationship.
- **`sruja impact <id>`**: Analyzes the blast radius of changing a component.
- **`sruja intent check`**: Verifies if your code changes match your architectural intent.

## Editor-Specific Configs

Sruja provides specialized configs for different editors:
- **Cursor**: `.cursorrules` (auto-gen) and `.cursor/rules/*.mdc` (manual rules).
- **Claude Code**: `CLAUDE.md` and `.gemini/AGENTS.md` (shared with Gemini).
- **GitHub Copilot**: `.github/copilot-instructions.md`.
- **Windsurf**: `.windsurf/rules/`.
- **Cline**: `.clinerules`.

## Troubleshooting Agent Tasks

- **"Command Not Found"**: Ensure you've run `make build` and the `target/release` directory is populated.
- **"Invalid DSL"**: Run `sruja lint <file>` and paste the JSON error output to the assistant.
- **"Drift Detected"**: Run `sruja drift -r . --fix` (if available) or manually align `.sruja` with code.
- **"WASM Mismatch"**: If logic changed in `sruja-language` but extension behavior is old, run `make wasm-nodejs`.


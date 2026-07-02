---
name: sruja-project
version: "2026.07.1"
description: >
  Procedural workflows for working with the Sruja codebase.
  Teaches AI editors how to add components, validate changes, and follow patterns.
license: Apache-2.0
---

# Sruja Project Skill

Procedural workflows for working with the Sruja architecture-as-code platform.

## Project Overview

- **Type:** Rust workspace with 14 crates
- **Primary language:** Rust (core), TypeScript (extension)
- **Architecture:** Layered monolith with clear tier boundaries

## Workflows

### Adding a New Crate

1. **Identify the tier** for your crate:
   - **Core Engine** (sruja-diagnostics, sruja-language, sruja-engine, sruja-export, sruja-scan, sruja-graph-core)
   - **Extraction** (sruja-graph, sruja-extract)
   - **Delivery** (sruja-cli, sruja-wasm)
   - **Secondary** (sruja-diff, sruja-intent, sruja-agent, sruja-memory)

2. **Add to workspace** in root `Cargo.toml`:
   ```toml
   [workspace]
   members = ["crates/sruja-new-crate"]
   ```

3. **Add dependencies** in `crates/sruja-new-crate/Cargo.toml`

4. **Update CLI** if exposing new commands:
   - Add command definition in `src/cli/commands.rs`
   - Add handler in `src/cli/run.rs`
   - Add to `src/commands/mod.rs`

5. **Validate**:
   ```bash
   cargo build --release
   cargo test -p sruja-new-crate
   cargo clippy -- -D warnings
   ```

### Adding a New CLI Command

1. **Define the command** in `src/cli/commands.rs`:
   ```rust
   #[command(name = "my-command")]
   MyCommand {
       #[arg(long = "repo", short = 'r', default_value = ".")]
       repo: String,
   },
   ```

2. **Add handler** in `src/cli/run.rs`:
   ```rust
   Commands::MyCommand { repo } => {
       commands::my_module::my_command(&repo)
   }
   ```

3. **Implement the command** in `src/commands/my_module.rs`

4. **Export** in `src/commands/mod.rs`:
   ```rust
   pub use my_module::my_command;
   ```

5. **Test**:
   ```bash
   cargo test -p sruja-cli
   ./target/release/sruja my-command --help
   ```

### Validating Architecture Changes

1. **After any .sruja file change**:
   ```bash
   sruja lint repo.sruja
   ```

2. **Check for drift**:
   ```bash
   sruja sync -r .
   sruja drift -r .
   ```

3. **Verify no layer violations**:
   ```bash
   sruja classify -r .
   sruja drift -r . -a repo.sruja
   ```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p sruja-cli

# Single test
cargo test test_name

# With coverage
just test-coverage
```

## Layer Boundaries

Respect these tier dependencies:

| Tier | Crates | Can Depend On |
|------|--------|---------------|
| **Core Engine** | sruja-diagnostics, sruja-language, sruja-engine, sruja-export, sruja-scan, sruja-graph-core | Only core crates |
| **Extraction** | sruja-graph, sruja-extract | Core Engine |
| **Delivery** | sruja-cli, sruja-wasm | Core Engine, Extraction |
| **Secondary** | sruja-diff, sruja-intent, sruja-agent, sruja-memory | Core Engine, Extraction |

## Forbidden Patterns

1. **Lower-tier crates must not depend on higher-tier crates**
2. **sruja-cli is the top-level aggregator** — no other crate should depend on it
3. **WASM-only crates must not use native-only APIs** (tree-sitter, fastembed)

## Progressive Discovery

| Task | Load only |
|------|-----------|
| Add crate | `rules/add-crate.md` |
| Add CLI command | `rules/add-cli-command.md` |
| Validate changes | `rules/validate-changes.md` |
| Run tests | `rules/run-tests.md` |
| Common patterns | `rules/common-patterns.md` |
| Anti-patterns | `rules/anti-patterns.md` |

## Auto-Capturing Knowledge

When users share conventions, patterns, or workflows in conversation, automatically record them using the existing `sruja_record_learning` MCP tool.

**Detect these patterns:**
- "Always do X", "Never do Y", "We use Z for..."
- "When you see X, do Y", "For X, use Y"
- "The convention is...", "The pattern is..."

**Capture using MCP tool `sruja_record_learning`:**
```json
{
  "context": "user convention: [brief description]",
  "hypothesis": "the convention/pattern shared",
  "outcome": "success",
  "guardrail_advice": "how to apply this"
}
```

This creates an auto-learning loop without explicit user action.

## Quick Start

```
Use sruja-project skill. Help me add a new crate to the workspace.
```

## Versioning

Skills use CalVer: `YYYY.MM.MICRO`

- **YYYY** — Year
- **MM** — Month (01-12)
- **MICRO** — Patch increment within the month (1, 2, 3...)

Bump the version when:
- Adding new workflows or rules
- Changing existing behavior
- Fixing incorrect guidance

The version field in frontmatter is required for discoverability and compatibility checking.

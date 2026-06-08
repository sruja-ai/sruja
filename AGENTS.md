This guide helps AI agents work effectively with the Sruja codebase.

## Table of Contents

- [Quick Reference](#-quick-reference-top-5-commands)
- [Architecture Setup](#architecture-setup-ai-agent-workflow)
- [Before Coding: Shared Understanding](#before-coding-shared-understanding)
- [AI Agent Workflow](#ai-agent-workflow-multi-step-tasks)
- [Code-Embedded Architecture Metadata](#code-embedded-architecture-metadata)
- [Build, Lint, and Test Commands](#build-lint-and-test-commands)
- [Code Style Guidelines](#code-style-guidelines)
- [Editor-Specific Configs](#editor-specific-configs)
- [Troubleshooting](#troubleshooting-agent-tasks)

## ⚡ Quick Reference (Top 5 Commands)

| Command | Purpose |
|---------|---------|
| `make setup` / `just setup` | **First-run setup** (install deps, hooks, build) |
| `make check` / `just check` | **Pre-commit check** (fmt + lint + test) |
| `make daily` / `just daily` | **Sync context** + check diagnostic drift |
| `sruja focus` | **Task briefing** (blast radius, decisions, AI info) |
| `sruja mcp -r .` | **Start MCP server** for deep context queries |

## Architecture Setup (AI Agent Workflow)

Use this workflow to set up architecture enforcement for any repository. No separate API key needed — the AI agent provides the intelligence, sruja provides the tools.

### Step 1: Understand the codebase

```
sruja_list_architecture_index  → get structure
sruja_get_topology             → get dependencies
sruja_get_repomap              → get file overview
```

### Step 2: Classify the architecture

Call `sruja_classify` with a classification JSON that you (the AI agent) have determined:

```json
{
  "schema_version": "classification/v1",
  "project_type": "rust-workspace",
  "summary": { "crates": 14, "source_files": 500 },
  "layers": [
    { "name": "Core Engine", "members": ["sruja-language", "sruja-engine"] },
    { "name": "Delivery", "members": ["sruja-cli", "sruja-wasm"] }
  ],
  "boundaries": [
    { "from": "sruja-language", "to": "sruja-cli", "allowed": false, "reason": "Core should not depend on CLI" }
  ],
  "forbidden_patterns": [
    "Lower-tier crates must not depend on higher-tier crates"
  ]
}
```

Or run `sruja classify -r .` for heuristic classification (Rust workspaces only).

### Step 3: Generate IDE context

```
sruja_sync_ide_rules  → generates .cursorrules, copilot-instructions.md, llms-architecture.txt
```

### Step 4: Verify

```
sruja_check_drift  → verifies architecture rules
```

### MCP Tools Available

| Tool | Purpose |
|------|---------|
| `sruja_classify` | Generate or set classification (accepts JSON from agent) |
| `sruja_sync_ide_rules` | Generate IDE context from classification |
| `sruja_list_architecture_index` | List architecture elements |
| `sruja_get_topology` | Get upstream/downstream dependencies |
| `sruja_get_elements` | Get element details |
| `sruja_check_drift` | Check architecture enforcement |
| `sruja_check_violations` | Validate changes against boundaries |

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

- Run `sruja check -r .` before merging
- Track code churn with `./scripts/code-churn.sh`
- For Dependabot majors and MSRV-sensitive upgrades, follow [docs/MSRV_AND_DEPENDENCIES.md](docs/MSRV_AND_DEPENDENCIES.md)
- Review security with `./target/release/sruja lint` (if applicable)

## AI agent workflow (multi-step tasks)

Use this for non-trivial work (new features, multi-file refactors, tricky bugs). Skip for one-line fixes.

### Two layers of planning

1. **Project or scope layer** — What problem are we solving and what is in or out of scope? Sources: GitHub issue / Jira epic, ADR, `docs/architecture/*.sruja`, `repo.sruja`, product notes. Stay high level: user-visible behavior, boundaries, dependencies between initiatives. Do not pick implementation files yet.
2. **Task layer** — For a single issue or PR: which crates, modules, and tests change, in what order, and how will we validate? This is the implementation plan for *this* change only.

Keeping the layers separate avoids baking file-level guesses into scope docs and avoids treating a ticket as full product discovery.

### AI-DLC + workflow (full SDLC gates)

For AWS AI-DLC–style inception → construction → operations with Sruja gates, use `sruja workflow` (`init --with-aidlc`, `status --check`, `install-rules`, `design-review`). See [docs/AIDLC_INTEGRATION.md](docs/AIDLC_INTEGRATION.md) and `.cursor/commands/sruja-workflow-aidlc.md`.

### Artifact handoffs

Move work between humans and agents with **one primary artifact per stage**, so intent stays reviewable:

| Stage | Typical artifact |
|-------|------------------|
| Scope | Issue body, epic, or short doc under `docs/` |
| Architecture alignment | `docs/architecture/*.sruja` or `repo.sruja` updates when boundaries move |
| Task plan | Issue update, PR description section, or a `plan.md` in the branch (especially if multiple agents touch the same work) |

Review each artifact before the next stage. Passing tests alone is not a substitute for checking the artifact matches intent.

### Session hygiene

Long exploratory chats accumulate bias and stale assumptions. After you have a **written task plan** you are satisfied with:

- **Start a new agent conversation** for implementation, and paste or point the model only at the plan plus minimal pointers (issue link, file paths). The new session should implement from the plan, not re-derive scope from the entire old thread.

Trivial edits can stay in one session.

### Research without flooding context

Broad repo exploration produces large token dumps. Prefer:

- **MCP progressive disclosure** (in the IDE): `sruja_list_architecture_index` → `sruja_get_topology` → `sruja_get_elements` — each step returns `estimated_tokens` and `next_suggested_tool`
- **`sruja focus --file <path>`** (or the issue scope) for a compact briefing when MCP is unavailable
- **`sruja ai-context -f for-ai --cache-friendly`** for prompt-cache-ordered invariant/volatile JSON (single repo only)
- **Scoped reads**: search, then open only the files you need; summarize for the main thread
- **Isolated exploration**: use a read-only subagent or a short side thread for wide research, then bring back a short summary and file list—not raw dumps

Do **not** paste full `repo.sruja` or entire `sruja ai` briefs into chat when the ladder or focus briefing suffices.

Filling the context window “because we can” still hurts quality.

### Grounded harness and continual learning (host-owned)

Sruja is the **deterministic harness** (lint, drift, evidence, MCP, agent memory); the **editor or CI host** owns the LLM loop (Act / optional Reflect). There is no `--autonomous` CLI mode. Full guide: [docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md).

### CLI `agent` loop boundaries (Sruja as Plugin, Not Agent)

Sruja is a passive **developer plugin/harness**, NOT an orchestrator agent. The host environment (Cursor, Claude Code, Cline, Windsurf, etc.) owns the developer orchestrator runtime, transcript analysis, screenshot/video capture, and code-generation loops. 

`sruja agent run`, `agent plan`, and `agent apply` exist solely as **optional headless/CI convenience helpers** for architecture-bounded validation. They are not a general-purpose coding agent, web search, or a substitute for your editor’s orchestrator.

- **Do**: Use Sruja's passive gates (`lint`, `drift`, `intent check`, `focus`) and MCP tools within your host agent.
- **Do**: Use CLI `agent` commands for headless or CI-driven verification step pipelines.
- **Do not**: Build orchestration state machines or active reflection loops inside Sruja.
- **Do not**: Treat Sruja apply outputs as reviewed truth—always merge proposals into `repo.sruja` through manual review.
- **Kill rule**: if a workflow cannot name [define intent / understand context / detect drift / review change](docs/PRODUCT_FEATURE_ALIGNMENT_REPORT.md#canonical-workflows), keep it out of primary docs and automation until it can.

### Agentic memory utility (`.sruja/agent_memory.json`)

Learnings are curated skills, not append-only logs.

| Signal | Meaning |
|--------|---------|
| `retrieval_count` | Learning was **injected into context** (focus briefing, apply, or MCP fetch) |
| `task_success_after` / `task_total_after` | After injection, **`sruja agent run --mode apply`** finished with all verification steps `ok` or `skipped` |

**Rules:**

- Only learnings in **`surfaced_learning_ids`** (token-budget subset from focus) get counters — not every `find_relevant` match.
- **Plan-only** `agent run` does not bump retrievals; **apply** records retrievals at apply start and outcomes after verification.
- Standalone **`sruja focus`** records retrievals (no task outcome — no verification gate).
- **`sruja agent curate`** is suggest-only; use **`agent update`**, **`agent merge`**, **`agent delete --force`** to act. Merge resets task outcome counters (retrieval history is preserved).

### System evolution (outer loop)

When an agent ships wrong code, misleading edits, or misses a project convention, fix the **process layer** when the miss is repeatable—not only the code diff.

Before the next task, ask whether any of these should change:

- **`AGENTS.md`** — missing gate, unclear command, wrong default workflow
- **`.cursor/rules/*.mdc`** — file-type or domain-specific constraints the agent ignored
- **Skills** (e.g. under `.agents/skills/` or user skill paths) — recurring multi-step procedures belong in a skill, not a one-off wall of text
- **Cursor project commands** (`.cursor/commands/*.md`) — stable prompts you use more than a few times
- **Plan / issue templates** — acceptance criteria or validation steps the agent skipped

Commit those updates like code so the whole team benefits.

### Cursor commands in this repo

Reusable flows live under `.cursor/commands/` (e.g. prime, plan, implement-from-plan, evolve-rules). Prefer invoking them over retyping long procedures.

## Build, Lint, and Test Commands

### Rust (Core)

```bash
# Build all crates
cargo build --release
just build  # or: make build

# Run all tests
cargo test --workspace
just test   # or: make test

# Run a single test
cargo test test_name

# Run tests in a specific crate
cargo test -p sruja-cli

# Run tests with coverage (excludes sruja-wasm — see docs/WASM_TESTING.md)
just test-coverage  # or: make test-coverage
# WASM bindings are tested separately: just test-coverage-wasm  (alias for wasm-pack tests)
# CI/script variant with extra llvm-cov flags:
#   bash scripts/coverage.sh

# Lint Rust code
cargo clippy -- -D warnings
just lint   # or: make lint

# Format Rust code
cargo fmt
just fmt    # or: make fmt
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
just test-wasm   # or: make test-wasm
cd crates/sruja-wasm && wasm-pack test --node

# Build WASM
just wasm         # or: make wasm
just wasm-nodejs  # or: make wasm-nodejs
```

### E2E Tests

```bash
# Build and serve book first in another terminal
just book-serve  # or: make book-serve

# Run Playwright E2E
just test-e2e    # or: make test-e2e
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

## Code-Embedded Architecture Metadata

Link source code to architecture elements using module-level doc comments. This creates a bidirectional binding that is both human-readable and machine-queryable.

### Convention

Add `@element` and `@layer` annotations in `//!` doc comments at the top of each module's `lib.rs` or `mod.rs`:

```rust
//! @element Sruja.CLI
//! @layer Delivery
//! @boundary CLI must not depend on sruja-wasm internals
//!
//! Command-line interface for Sruja operations.
```

### Rules

- `@element` — The architecture element ID from `repo.sruja` (dot notation for nested: `Sruja.Context.Scan`)
- `@layer` — The layer name from classification (Core Engine, Extraction, Delivery, Secondary)
- `@boundary` — Optional: specific constraints for this module
- Keep annotations in the first 5 lines of the doc comment for discoverability
- Use `sruja focus --file <path>` to resolve file-to-element mappings at runtime

### Why

Source code cannot express **why** decisions were made. Code-embedded metadata bridges this gap by linking modules to Decision Records and architecture elements, giving AI agents the context they need without leaving the source file.

## Key Commands for AI Agents

For multi-step agent work, follow **AI agent workflow (multi-step tasks)** above and use `.cursor/commands/` when applicable.

When working on Sruja:
1. **First Time Setup**: Run `just setup` (or `make setup`) to ensure all dependencies and git hooks are correctly installed.
2. **Grounded architecture authoring**: For `repo.sruja` work, prefer `.sruja/author_evidence.json` (from `sruja sync` or `sruja author evidence`) or MCP `sruja_get_author_evidence`. Treat `.sruja/graph.json` as debug/export only—not default agent context. Synthesize proposals under `.sruja/proposals/` or `repo.sruja.working`; promote to `repo.sruja` only after human review. See [docs/plans/GROUNDED_ARCHITECTURE_AUTHORING_PLAN.md](docs/plans/GROUNDED_ARCHITECTURE_AUTHORING_PLAN.md).
3. **Dogfooding the Architecture**: Before proposing significant PRs, always respect `docs/architecture/*.sruja` as the "reviewed truth". If changing architecture, update those files. Also run `sruja status -r .`, `sruja review -r .`, or `sruja check -r . -a repo.sruja` to validate the baseline (`repo.sruja`).
4. Run `just check` (or `make check`) before committing to ensure consistent formatting, linting, and passing tests.
5. For .sruja files, run `sruja lint file.sruja` after changes.
6. Use `cargo clippy -- -D warnings` for strict linting.
7. Build extension with `just build-extension` (or `make build-extension`).
8. Test CLI commands with `just test-cli-smoke` (or `make test-cli-smoke`).
9. For Rust coverage gaps (CLI handlers, LSP, WASM, tree-sitter) and infrastructure needs, see `docs/internal/TEST_COVERAGE_PLAN.md`.

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

1. **Architecture Setup**: Run `sruja classify -r .` to generate `.sruja/classification.json`, then `sruja sync-ide-rules -r .` to generate IDE context files.
2. **Daily Context Sync**: Run `just daily` (or `make daily`) to check for architectural drift and update IDE context.
3. **Manual Sync**: Run `sruja sync-ide-rules -r .` to regenerate `.cursorrules`, copilot instructions, and `llms-architecture.txt`.
4. **MCP Server**: Configure your AI editor to use the Sruja Model Context Protocol (MCP) server.
   - **Command**: `sruja mcp -r .`
   - **Usage**: The MCP server exposes tools for the AI to query the architecture graph, classify repos, sync IDE rules, and check compliance on the fly.
5. **Cross-Repo Context**: Use `sruja ai-context -r repoA -r repoB` to dynamically build context payloads when working on multi-repo features.

When using AI agents, leverage Sruja's context tools:
- **`sruja focus --file <path>`**: Generates a task-scoped briefing (blast radius, decisions, boundaries, AI instructions).
- **`sruja context-score`**: Quantifies the repository's AI-readiness (0-100 score across 5 dimensions).
- **`sruja ingest <doc>`**: Imports external documentation (ADRs, design docs) into `.sruja/context/`.
- **`sruja why <id>`**: Explains the rationale/logic behind a specific architectural component or relationship.
- **`sruja impact <id>`**: Analyzes the blast radius of changing a component.
- **`sruja intent check`**: Verifies if your code changes match your architectural intent.

## Editor-Specific Configs

Sruja generates architecture data files. Tool-specific instructions are hand-written.

**Generated by `sruja sync-ide-rules`:**
- **Cursor**: `.cursorrules`
- **GitHub Copilot**: `.github/copilot-instructions.md`
- **All tools**: `llms-architecture.txt` (compact architecture brief)

**Hand-written (not generated by sruja):**
- **Claude Code**: `CLAUDE.md`
- **General agents**: `AGENTS.md`
- **Windsurf**: `.windsurf/rules/`
- **Cline**: `.clinerules`

## Troubleshooting Agent Tasks

- **"Command Not Found"**: Ensure you've run `just build` (or `make build`) and the `target/release` directory is populated.
- **"Invalid DSL"**: Run `sruja lint <file>` and paste the JSON error output to the assistant.
- **"Drift Detected"**: Run `sruja check -r . --fix` (if available) or manually align `.sruja` with code.
- **"WASM Mismatch"**: If logic changed in `sruja-language` but extension behavior is old, run `just wasm-nodejs` (or `make wasm-nodejs`).

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

When the user types `/graphify`, invoke the `skill` tool with `skill: "graphify"` before doing anything else.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- Dirty graphify-out/ files are expected after hooks or incremental updates; dirty graph files are not a reason to skip graphify. Only skip graphify if the task is about stale or incorrect graph output, or the user explicitly says not to use it.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).

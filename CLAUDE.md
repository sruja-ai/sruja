# Sruja - Claude Code Context

Sruja is an AI coding agent that helps you build better software by understanding your codebase architecture and verifying that changes align with design decisions.

## Quick Start

```bash
# Build
cargo build --release

# Test
cargo test --workspace

# Check architecture
sruja drift -r .
```

## Architecture

14 Rust crates in 4 tiers:

- **Core Engine** (6): sruja-language, sruja-engine, sruja-graph, sruja-graph-core, sruja-scan, sruja-diagnostics
- **Extraction** (2): sruja-extract, sruja-export
- **Delivery** (2): sruja-cli, sruja-wasm
- **Secondary** (4): sruja-diff, sruja-intent, sruja-agent, sruja-memory

Full architecture brief: `llms-architecture.txt`

## Key Rules

- Lower-tier crates must not depend on higher-tier crates
- sruja-cli is the top-level aggregator — no other crate should depend on it
- WASM-only crates must not use native-only APIs (tree-sitter, fastembed)
- Use `thiserror` for error types, never `anyhow`

## Commands

| Command | Purpose |
|---------|---------|
| `sruja classify -r .` | Generate architecture classification |
| `sruja sync-ide-rules -r .` | Generate IDE context files |
| `sruja drift -r .` | Check architecture enforcement |
| `sruja focus --file <path>` | Task-scoped context briefing |
| `sruja mcp -r .` | Start MCP server |

## Before Committing

```bash
just check  # fmt + lint + test + validate-book-dsl
```

## Full Guidelines

Read `AGENTS.md` for complete workflow, code style, and architecture patterns.

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).

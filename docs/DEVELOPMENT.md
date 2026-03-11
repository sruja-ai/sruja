# Development Guide

This guide covers development for the Rust-only Sruja repo (CLI, LSP, engine, export, WASM).

## Quick Start

```bash
# Install Rust (if needed): https://rustup.rs/
make install   # cargo fetch
make build     # cargo build --release
make test      # cargo test
make fmt       # cargo fmt
make lint      # cargo clippy
```

## First value (no .sruja)

```bash
cargo build --release -p sruja-cli
./target/release/sruja quickstart -r .
```

See [ARCHITECTURE_INTELLIGENCE.md](ARCHITECTURE_INTELLIGENCE.md) for full flow and current state.

## Validate .sruja files

```bash
./target/release/sruja lint examples/
./target/release/sruja export markdown path/to/file.sruja
```

## WASM build

```bash
make wasm        # web target → crates/sruja-wasm/pkg/
make wasm-nodejs # nodejs target (for future LSP/extension)
```

## Project layout

| Crate | Purpose |
|-------|---------|
| **sruja-cli** | CLI: lint, export, scan, why, drift |
| **sruja-language** | Parser and AST |
| **sruja-engine** | Validation rules |
| **sruja-export** | Markdown, Mermaid, JSON export |
| **sruja-lsp** | LSP server (VS Code extension) |
| **sruja-wasm** | WASM build for browser/Node |
| **sruja-diagnostics** | Diagnostic types |
| **sruja-app** | Desktop app (Dioxus) — architecture collaboration |
| **sruja-graph** | Knowledge graph, centrality, coupling |
| **sruja-scan** | Repo scanning (multi-language tree-sitter) |
| **sruja-diff** | Drift detection |
| **sruja-intent** | Intent vs. reality comparison |
| **book/** | mdBook documentation |

## Run sruja-app

```bash
export OPENROUTER_API_KEY="sk-or-v1-..."
cargo run -p sruja-app
```

## VS Code extension

The extension in `extension/` provides syntax highlighting and LSP integration for `.sruja` files.

## Skills and evaluation

- **Skills:** `skills/` — sruja-architecture, sruja-architecture-agent; install with `npx skills add ... --skill sruja-architecture-agent`.
- **Comparison (Mermaid vs Sruja):** `scripts/run_comparison_test.sh [project] [url]`; results in `evaluation/results/comparison_*`.
- **Improvement plan:** [docs/NEXT_STEPS_IMPROVEMENTS.md](NEXT_STEPS_IMPROVEMENTS.md) — next-phase plan for skill, testing, and docs.

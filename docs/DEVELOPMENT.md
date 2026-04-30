# Development Guide

This guide covers development for the Rust Sruja repo (CLI, engine, export, WASM) and the VS Code extension.

## Quick Start

```bash
# Install Rust (if needed): https://rustup.rs/
just install   # cargo fetch
just build     # cargo build --release
just test      # cargo test
just fmt       # cargo fmt
just lint      # cargo clippy
```

## First value (no .sruja)

```bash
cargo build --release -p sruja-cli
./target/release/sruja quickstart -r .
```

See `docs/RUN_GUIDE.md` and `demo/README.md` for the end-to-end demo flow.

## Validate .sruja files

```bash
./target/release/sruja lint book/valid-examples/*.sruja
./target/release/sruja export markdown path/to/file.sruja
```

## WASM build

```bash
just wasm        # web target → crates/sruja-wasm/pkg/
just wasm-nodejs # nodejs target
```

## Project layout

| Crate | Purpose |
|-------|---------|
| **sruja-cli** | CLI: lint, export, scan, why, drift, critique, propose, focus, context, agent, context-score, context-graph, compliance, ingest, federation |
| **sruja-language** | Parser, AST, and pluggable domain schemas |
| **sruja-engine** | Validation rules |
| **sruja-export** | Markdown, Mermaid, JSON, HTML/D3 export |
| **sruja-wasm** | WASM build for browser/Node |
| **sruja-diagnostics** | Diagnostic types |
| **sruja-graph** | Knowledge graph, centrality, coupling |
| **sruja-graph-core** | Core graph types and primitives |
| **sruja-scan** | Repo scanning (multi-language tree-sitter) |
| **sruja-diff** | Drift detection + proposal system |
| **sruja-intent** | Intent vs. reality comparison + adversarial critique engine |
| **sruja-extract** | Source code extraction utilities |
| **sruja-agent** | Agentic memory – persistent learning and guardrails for AI agents |
| **book/** | mdBook documentation |

## VS Code extension

The extension in `extension/` provides syntax highlighting and language features for `.sruja` files, powered by WASM.

## Skills and evaluation

- **Skills:** `skills/` — `sruja-architecture` is the single supported skill.
- **Comparison (Mermaid vs Sruja):** `scripts/run_comparison_test.sh [project] [url]`; results in `evaluation/results/comparison_*`.

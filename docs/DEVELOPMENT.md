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

## Validate .sruja files

```bash
cargo build --release -p sruja-cli
./target/release/sruja lint examples/
./target/release/sruja export markdown path/to/file.sruja
```

## WASM build

```bash
make wasm        # web target → crates/sruja-wasm/pkg/
make wasm-nodejs # nodejs target (for future LSP/extension)
```

## Project layout

- **crates/sruja-cli** – CLI (lint, export, etc.)
- **crates/sruja-language** – Parser and AST
- **crates/sruja-engine** – Validation rules
- **crates/sruja-export** – Markdown/Mermaid export
- **crates/sruja-lsp** – LSP server (for future VS Code extension)
- **crates/sruja-wasm** – WASM build for browser/Node
- **crates/sruja-diagnostics** – Diagnostic types
- **book/** – mdBook documentation

## VS Code extension (planned)

The VS Code extension will be reimplemented to integrate with the Rust LSP (`sruja-lsp`) and provide:

- Diagnostics, syntax highlighting, symbol navigation, go to definition
- Markdown export and other editor features

No Node/TypeScript extension code remains in this repo until that implementation.

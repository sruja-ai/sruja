# Sruja

**Architecture-as-code for the AI SDLC process.** Define architecture in `.sruja` files; validate, document, and keep it in sync with your workflow. We’re a tool for the lifecycle—not a diagramming product.

## Stack

- **Rust** – CLI, engine, LSP, WASM (single language for core)
- **VS Code extension** – Edit `.sruja`, diagnostics, optional diagram preview
- **Docs** – This book (mdBook, Rust-based; no TypeScript/Node)

> **New here?** Do [Quick start](getting-started.md) (about 5 min), then the [Beginner path](docs/beginner-path.md) (2–3 hours).

See [Quick start](getting-started.md) to install the CLI and create your first `.sruja` file. For a single entry point to **docs**, **tutorials**, and **courses**, use [Navigate](navigate.md). The **left sidebar** lists everything; press **`/`** or **`S`** to search.

> **Sruja "Show diagram" in code blocks:** Run `make wasm` from the repo root once, then run `make book-serve` (or `./serve.sh` from the book directory) so the WASM files are copied into the book output.

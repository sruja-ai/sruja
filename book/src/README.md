# Sruja

**Architecture-as-code for the AI SDLC process.** Define architecture in `.sruja` files; validate, and export to Markdown and Mermaid diagrams. We're a backend tool for the SDLC lifecycle—not a diagramming product.

## Why Sruja?

### The Problem

Most architecture tools make you choose:

- **Visual-only tools** (Draw.io) – no code, no version control, hard to maintain
- **Code-only tools** (Mermaid, PlantUML) – no validation, manual diagram updates
- **Stale diagrams** – architecture drifts from reality, documentation gets outdated

### Our Solution

Sruja gives you a code-first architecture tool:

| Feature                 | What you get                                                     |
| ----------------------- | ---------------------------------------------------------------- |
| **Markdown & Mermaid**  | Export architecture to clean Markdown docs and Mermaid diagrams  |
| **Version-controlled**  | `.sruja` files in Git, with proper code review workflows         |
| **Team-friendly**       | Developers work in code with familiar tools                      |
| **Built-in validation** | Catch architecture issues before they reach production           |
| **Multiple exports**    | JSON, Markdown, Mermaid – integrate into your existing toolchain |

### Who It's For

- **Engineering teams** who need architecture as part of their SDLC
- **Tech leads** who want to enforce architectural standards
- **Platform engineers** building guardrails for distributed teams
- **AI agents** that need to reason about system architecture

## How We Work

1. **Define** your architecture in `.sruja` files
2. **Validate** with built-in checks (cycles, orphans, unique IDs)
3. **Export** to JSON, Markdown, or Mermaid diagrams
4. **Integrate** into CI/CD, docs, and your IDE workflow

We're **ultra simple** – minimal surface area, no unnecessary apps or frameworks – and **highly functional** – what we ship works reliably for its scope.

## Stack

- **Rust** – CLI, engine, LSP, WASM (single language for core)
- **VS Code extension** – Edit `.sruja` files with syntax highlighting and diagnostics
- **Docs** – This book (mdBook, Rust-based; no TypeScript/Node)

> **New here?** Do [Quick start](getting-started.md) (about 5 min), then the [Beginner path](docs/beginner-path.md) (2–3 hours).

See [Quick start](getting-started.md) to install the CLI and create your first `.sruja` file. For a single entry point to **docs**, **tutorials**, and **courses**, use [Navigate](navigate.md). The **left sidebar** lists everything; press **`/`** or **`S`** to search.

> **Sruja "Show diagram" in code blocks:** Run `make wasm` from the repo root once, then run `make book-serve` (or `./serve.sh` from the book directory) so the WASM files are copied into the book output.

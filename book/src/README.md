# Sruja – Context engineering for the AI era.

<img src="sruja-logo.png" alt="Sruja" class="sruja-home-logo" width="160" />

**Context engineering for the AI era.** Use AI to generate and maintain architecture as code, backed by deterministic repo evidence. Validate the result, keep it in sync, and export to Markdown or Mermaid when you need docs or diagrams.

## Why Sruja?

### The Problem

Most architecture tools make you choose:

- **Visual-only tools** (Draw.io) – no code, no version control, hard to maintain
- **Code-only tools** (Mermaid, PlantUML) – no validation, manual diagram updates
- **Stale diagrams** – architecture drifts from reality, documentation gets outdated

### Our Solution

Sruja gives AI assistants and humans evidence-backed architecture context:

| Feature                 | What you get                                                     |
| ----------------------- | ---------------------------------------------------------------- |
| **AI skill first**      | Generate or update `repo.sruja` from real repo evidence          |
| **Evidence-backed**     | Discovery, sync, drift, and context commands ground the model    |
| **Version-controlled**  | `.sruja` files in Git, with proper code review workflows         |
| **Built-in validation** | Catch architecture issues before they reach production           |
| **Multiple exports**    | JSON, Markdown, Mermaid – outputs from reviewed truth            |

### Who It's For

- **Engineering teams** who need architecture as part of their SDLC
- **Tech leads** who want to enforce architectural standards
- **Platform engineers** building guardrails for distributed teams
- **AI agents** that need to reason about system architecture

## How We Work

1. **Install** the sruja-architecture skill in your AI editor
2. **Gather** deterministic repo evidence through Sruja commands
3. **Generate** or update `repo.sruja` with AI
4. **Validate** with lint, drift, and review workflows
5. **Reuse** the context in CI, docs, and IDE workflows

We're **ultra simple** – minimal surface area, no unnecessary apps or frameworks – and **highly functional** – what we ship works reliably for its scope.

## Stack

- **Rust** – CLI, engine, LSP, WASM (single language for core)
- **VS Code extension** – Edit `.sruja` files with syntax highlighting and diagnostics
- **Docs** – This book (mdBook, Rust-based; no TypeScript/Node)

> **New here?** Install the [sruja-architecture skill](getting-started.md) first (1 minute), then let your AI generate architecture for you. For a single entry point to **docs**, **tutorials**, and **courses**, use [Navigate](navigate.md). The **left sidebar** lists everything; press **`/`** or **`S`** to search.

See [Quick start](getting-started.md) to install the AI skill and create your first `.sruja` file. For a single entry point to **docs**, **tutorials**, and **courses**, use [Navigate](navigate.md). The **left sidebar** lists everything; press **`/`** or **`S`** to search.

> **Sruja "Show diagram" in code blocks:** Run `make wasm` from the repo root once, then run `make book-serve` (or `./serve.sh` from the book directory) so the WASM files are copied into the book output.

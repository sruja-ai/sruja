# Sruja — AI coding harness for repo structure

<img src="sruja-logo.png" alt="Sruja" class="sruja-home-logo" width="160" />

**Stop AI from breaking your repo structure.** Sruja scans code, builds a context graph, reports structural drift, and plugs into Cursor and other agents via MCP — `focus` before edits, `verify-task` after. Optional `repo.sruja` is reviewed CI intent, not the primary product.

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
| **Structural scan**     | Cycles, layer violations, god modules — file-level evidence      |
| **Agent harness**       | MCP + `focus` + `verify-task`; host keeps the LLM loop           |
| **Context graph**       | Repo-derived dependency/context graph for targeted retrieval      |
| **AI context export**   | `ai-context` outputs for host tools and prompt caching            |
| **Built-in gates**      | Drift, verification, CI-friendly outputs                          |
| **Optional reviewed intent** | `repo.sruja` when teams want strict CI gates                 |
| **Tier-2 exports**      | Markdown, Mermaid — derived views, not the product center          |

### Who It's For

- **Engineering teams** who need architecture as part of their SDLC
- **Tech leads** who want to enforce architectural standards
- **Platform engineers** building guardrails for distributed teams
- **AI agents** that need to reason about system architecture

## How We Work (Tier 1)

1. **Install CLI** — `curl -fsSL https://sruja.ai/install.sh | bash`
2. **Scan** — `sruja start -r .` then `sruja drift -r . --structural-only --advisory`
3. **Wire MCP** — profile `coding`; `focus` before edits, `verify-task` after
4. **Install harness skill** — `npx skills add … --skill sruja-harness`
5. **Optional** — `sruja-architecture` skill + `repo.sruja` for reviewed CI intent

We're **ultra simple** – minimal surface area, no unnecessary apps or frameworks – and **highly functional** – what we ship works reliably for its scope.

## Stack

- **Rust** – CLI, engine, LSP, WASM (single language for core)
- **VS Code extension** – Editor integration (briefing, validation, diagnostics)
- **Docs** – This book (mdBook, Rust-based; no TypeScript/Node)

> **New here?** [Quick start](getting-started.md): install CLI, run structural drift, wire MCP, install `sruja-harness`, run `verify-task` after agent edits. Add `sruja-architecture` only when you want `repo.sruja` in Git. For **docs** and **tutorials** (and optional learning), use [Navigate](navigate.md); press **`/`** or **`S`** to search.

> **Sruja "Show diagram" in code blocks:** Run `just wasm` (or `make wasm`) from the repo root once, then run `just book-serve` (or `make book-serve`, or `./serve.sh` from the book directory) so the WASM files are copied into the book output.

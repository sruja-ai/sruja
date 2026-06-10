# Sruja — context engineering for software changes

<img src="sruja-logo.png" alt="Sruja" class="sruja-home-logo" width="160" />

**Capture knowledge, retrieve context, verify changes.** Sruja scans code, links decision context, reports structural drift, and plugs into Cursor and other agents via MCP. Optional `repo.sruja` is reviewed intent in Git, not the primary product.

## Why Sruja?

### The Problem

Most architecture tools make you choose:

- **Visual-only tools** (Draw.io) – no code, no version control, hard to maintain
- **Code-only tools** (Mermaid, PlantUML) – no validation, manual diagram updates
- **Stale diagrams** – architecture drifts from reality, documentation gets outdated

### Our Solution

Sruja gives AI assistants and humans evidence-backed context for software changes:

| Feature                 | What you get                                                     |
| ----------------------- | ---------------------------------------------------------------- |
| **Structural scan**     | Cycles, layer violations, god modules — file-level evidence      |
| **Task context**        | `focus`, `ai`, and MCP for targeted retrieval                    |
| **Built-in gates**      | Drift, verification, CI-friendly outputs                         |
| **Optional reviewed intent** | `repo.sruja` when teams want durable intent in Git         |
| **Derived outputs**     | Markdown, Mermaid, and other exports when communication needs them |

### Who It's For

- **Engineering teams** who want shared knowledge and safer AI-assisted changes
- **Tech leads** who want decisions to stay visible at change time
- **Platform engineers** building guardrails for distributed teams
- **AI agents** that need grounded repo context before editing

## The Core Loop

1. **Install CLI** — `curl -fsSL https://sruja.ai/install.sh | bash`
2. **Scan** — `sruja start -r .` then `sruja drift -r . --structural-only --advisory`
3. **Retrieve context** — `focus`, `ai`, or MCP before edits
4. **Verify** — `verify-task` after edits
5. **Optional** — reviewed intent in Git with `repo.sruja`

We're **ultra simple** – minimal surface area, no unnecessary apps or frameworks – and **highly functional** – what we ship works reliably for its scope.

## Stack

- **Rust** – CLI, engine, LSP, WASM (single language for core)
- **VS Code extension** – Editor integration (briefing, validation, diagnostics)
- **Docs** – This book (mdBook, Rust-based; no TypeScript/Node)

> **New here?** [Quick start](getting-started.md): install CLI, scan current structure, retrieve context, and verify changes. Add reviewed intent only when you want `repo.sruja` in Git. Optional learning lives separately under [Navigate](navigate.md); press **`/`** or **`S`** to search.

> **Sruja "Show diagram" in code blocks:** Run `just wasm` (or `make wasm`) from the repo root once, then run `just book-serve` (or `make book-serve`, or `./serve.sh` from the book directory) so the WASM files are copied into the book output.

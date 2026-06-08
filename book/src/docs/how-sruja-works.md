---
title: "How Sruja Works"
weight: 3
---

# How Sruja Works

Sruja is built for **context engineering in the AI SDLC**: architecture as code backed by repo evidence, validation, drift checks, and editor/CI workflows. We are not a diagramming product; diagrams are exported from reviewed architecture truth.

## The Sruja Platform

The platform consists of several key components working together:

1.  **Parser & engine**: Rust crates for parsing, validation, and export (sruja-language, sruja-engine, sruja-export).
2.  **CLI**: Command-line interface for local development and CI/CD (sruja-cli).
3.  **WASM**: Rust core compiled to WebAssembly for the docs book and VS Code (sruja-wasm).
4.  **VS Code extension**: Editor integration powered by the WASM build (extension/ + sruja-wasm).
5.  **Docs**: This site—built with mdBook from the `book/` directory.

## How the pieces work together

Sruja’s core loop is **evidence → briefing → edit → gates**:

1.  The CLI scans real code into a dependency/evidence model (Tree-sitter–backed).
2.  `focus` turns that evidence into a task-scoped briefing (MCP or CLI output).
3.  The host agent edits code with boundaries in mind.
4.  `verify-task` runs deterministic checks so drift doesn’t silently accumulate.

When teams want strict enforcement, `repo.sruja` becomes reviewed intent in Git and `drift -a repo.sruja` becomes the CI gate.

## Context graph and AI context

Sruja is primarily a **context engineering** system. The core artifact is a **context graph** derived from the repo, not a hand-authored diagram.

Typical outputs:

- `.sruja/graph.json`: full dependency graph (evidence)
- `.sruja/context.json`: smaller, cache-friendly summary for agents
- `sruja ai-context -r .`: structured AI context export for host tools

MCP tools expose the same evidence and briefings in a tool-friendly form so the host agent can fetch exactly what it needs without dumping the whole repo into a prompt.

Architecture files (`repo.sruja`) are optional; they exist only if you want reviewed intent in Git and strict drift enforcement in CI.

## Key Components

### Core Engine (Rust)

The [`sruja-language`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-language) and [`sruja-engine`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-engine) crates form the foundation. They define the DSL grammar, parse input files into an AST (Abstract Syntax Tree), and run validation rules (like cycle detection and layer enforcement).

### WebAssembly (WASM)

The Rust core is compiled to WebAssembly ([`sruja-wasm`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-wasm)). The same parsing and validation logic runs in:

- **VS Code Extension**: For local preview without needing a CLI binary.
- **Documentation site**: For "Show diagram" in code blocks (like the one above).

### CLI & CI/CD

The `sruja` CLI ([`sruja-cli`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-cli)) is a static binary that wraps the core engine. It supports:

- **Local development**: `sruja fmt`, `sruja lint`, `sruja export`.
- **CI/CD**: Validate and export architecture in pipelines.
- **Export**: `sruja export json`, `sruja export mermaid`, `sruja export markdown`, `sruja export context`, `sruja export dsl`.

## Context Engineering

Sruja provides **context engineering** across four progressive layers:

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 4: Intent                                            │
│  "What did we intend vs what exists?"                       │
│  Commands: sruja drift -a, sruja intent check               │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Semantic                                          │
│  "What does this mean? (vocabulary, patterns)"              │
│  Commands: sruja analyze --semantic                         │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Structural                                        │
│  "What exists? (components, deps, metrics)"                 │
│  Commands: sruja scan, sruja quickstart, sruja discover    │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Syntactic                                         │
│  "Is the DSL valid?"                                        │
│  Commands: sruja lint                                       │
└─────────────────────────────────────────────────────────────┘
```

Each layer builds on the previous:

- **Syntactic**: Is the `.sruja` file valid? (lint)
- **Structural**: What components and dependencies exist? (scan, discover)
- **Semantic**: What patterns and relationships mean? (analyze)
- **Intent**: Does reality match declared architecture? (drift, intent check)

### AI Skill Multiplies Context

The **sruja-architecture skill** enhances all four layers:

| Layer | CLI Only | CLI + Skill |
|-------|----------|-------------|
| Syntactic | `sruja lint` | Pattern-aware DSL generation |
| Structural | `sruja scan` | Evidence-based discovery |
| Semantic | `sruja analyze` | Patterns and trade-offs |
| Intent | `sruja drift` | Multi-perspective review |

Install the skill to unlock AI-powered context engineering:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

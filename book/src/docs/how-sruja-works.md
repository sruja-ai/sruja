---
title: "How Sruja Works"
weight: 3
---

# How Sruja Works

Sruja is built for context engineering during software changes. The center is simple: gather repo evidence, retrieve the right context before editing, and verify the result after editing. Reviewed intent in Git is optional.

## The Sruja Platform

The platform consists of several key components working together:

1.  **Scan and evidence**: Rust crates gather structure and context from code.
2.  **CLI**: Local and CI workflows for retrieve and verify steps.
3.  **WASM**: Shared core for the docs book and VS Code.
4.  **VS Code extension**: Editor support for validation and previews.
5.  **Docs**: This site, built from the `book/` directory.

## How the pieces work together

Sruja’s core loop is **evidence → context → edit → verify**:

1.  The CLI scans real code into an evidence model.
2.  `focus`, `ai`, or MCP turns that evidence into task-scoped context.
3.  The host agent or developer edits code with that context in mind.
4.  `verify-task`, drift, and intent checks keep regressions from silently accumulating.

When teams want stricter governance, `repo.sruja` becomes reviewed intent in Git and `drift -a repo.sruja` becomes part of the CI gate.

## Context graph and AI context

Sruja is primarily a context engineering system. The core artifact is a repo-derived context graph, not a hand-authored diagram.

Typical outputs:

- `.sruja/graph.json`: full dependency graph (evidence)
- `.sruja/context.json`: smaller, cache-friendly summary for agents
- `sruja ai-context -r .`: structured AI context export for host tools

MCP tools expose the same evidence and briefings in a tool-friendly form so the host agent can fetch exactly what it needs without dumping the whole repo into a prompt.

Architecture files (`repo.sruja`) are optional; they exist only if you want reviewed intent in Git and stricter drift enforcement in CI.

## Key Components

### Core Engine (Rust)

The [`sruja-language`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-language) and [`sruja-engine`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-engine) crates form the foundation. They define the DSL grammar, parse input files into an AST (Abstract Syntax Tree), and run validation rules (like cycle detection and layer enforcement).

### WebAssembly (WASM)

The Rust core is compiled to WebAssembly ([`sruja-wasm`](https://github.com/sruja-ai/sruja/tree/main/crates/sruja-wasm)). The same parsing and validation logic runs in:

- **VS Code Extension**: For local preview without needing a CLI binary.
- **Documentation site**: For "Show diagram" in code blocks (like the one above).

### CLI & CI/CD

The `sruja` CLI wraps the core engine. The most important public workflows are:

- **Retrieve**: `focus`, `ai`, `mcp`
- **Verify**: `drift`, `intent check`, `verify-task`
- **Optional reviewed intent**: `lint`, `sync`, `drift -a repo.sruja`

## Context Engineering

Sruja provides context engineering across four progressive layers:

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

### Optional Reviewed-Intent Layer

The `sruja-architecture` skill is an optional accelerator for teams that want reviewed intent in Git:

| Layer | Core path | Optional reviewed-intent path |
|-------|-----------|-------------------------------|
| Syntactic | no file required | `sruja lint` |
| Structural | `sruja start`, `sruja drift` | evidence-backed modeling |
| Semantic | `focus`, `ai`, MCP retrieval | richer architecture interpretation |
| Intent | `verify-task`, `intent check` | `sruja drift -a repo.sruja` |

Install the skill only when you want that reviewed-intent workflow:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

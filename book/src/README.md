# Sruja — autonomous coding agent with deterministic gates

<img src="sruja-logo.png" alt="Sruja" class="sruja-home-logo" width="160" />

**Sruja is a CLI-first autonomous coding agent.** It scans your repo, grounds every edit in real topology, and verifies the result before you ship. Also works as a passive harness inside any editor (Cursor, Copilot, Claude, Windsurf) via MCP.

## The Problem

AI coding agents are fast, but they don't know your repo. They invent dependencies, break layer boundaries, and repeat mistakes. There's no built-in check between "generate" and "ship."

## The Solution

Sruja closes that gap with a **grounded agent loop**:

```text
focus / drift  →  agent edits code  →  verify-task  →  (critique or approve)
```

| Capability | What you get |
|------------|-------------|
| **Structural scan** | Cycles, layer violations, god modules — file-level evidence |
| **Autonomous agent** | `sruja agent loop` — plans, edits, verifies, critiques, replans |
| **Editor integration** | MCP `coding` profile — grounded context + post-edit verification |
| **Deterministic gates** | `verify-task`, `drift`, `intent check` — the actor never grades itself |
| **Optional reviewed intent** | `repo.sruja` when teams want durable boundaries in Git |

## Quick Start

```bash
# 1. Install
curl -fsSL https://sruja.ai/install.sh | bash

# 2. Scan your repo
sruja start -r .
sruja drift -r . --structural-only --advisory

# 3. Brief a change (paste into your agent)
sruja focus -r . --file path/to/file.rs

# 4. Verify after edits
sruja verify-task --profile coding -r .
```

No `repo.sruja` required on day one.

## Who It's For

- **Developers using AI agents** — verify generated changes before calling work done
- **Engineering teams** — shared knowledge and safer AI-assisted changes
- **Tech leads** — decisions stay visible at change time
- **Platform engineers** — guardrails for distributed teams using AI

## Stack

- **Rust** — CLI, engine, LSP, WASM
- **VS Code extension** — editor integration (briefing, validation, diagnostics)
- **Docs** — this book (mdBook)

> **New here?** [Quick start](getting-started.md): install, scan, focus, verify. Add reviewed intent only when you want `repo.sruja` in Git.

---
title: "CLI Guide"
weight: 50
summary: "The sruja command-line interface is your primary tool for interacting with Sruja projects."
---

# CLI Guide

The `sruja` command-line interface is your primary tool for interacting with Sruja. Start with the core loop first, then use the rest as supporting or advanced workflows.

<!--more-->

## Repository path

Most repo-scoped commands take **`-r`** with **`--repo`** as the long form (recommended). **`--path`** is accepted as an alias on many commands for older scripts. Positional paths (e.g. `sruja scan .`) remain valid where documented.

## Commands

## Start Here

If you are new, start with these commands:

```bash
sruja start -r .
sruja drift -r . --structural-only --advisory
sruja focus -r . --file path/to/file.rs
sruja verify-task --profile coding -r .
```

Use `sruja ai -r . --task "..."` or `sruja mcp -r .` when you want richer task context retrieval.

### `export`

Exports the architecture to various formats.

**Usage:**

```bash
sruja export <format> <file>
```

**Supported formats:**

- `json`: Structured JSON of the architecture.
- `mermaid`: Mermaid diagram code.
- `markdown`: Markdown docs with diagrams.
- `context`: Architecture context for AI tools (Cursor, Copilot).
- `dsl`: Pretty-printed DSL (canonical form).

**Options:**

- `--extended`: Include pre-computed views in JSON output.
- `--view-level <1|2|3>`: C4 view level for mermaid (1=context, 2=container, 3=component).
- `--target <id>`: Focus node ID for view levels 2/3.

**Examples:**

```bash
sruja export json architecture.sruja
sruja export json architecture.sruja --extended
sruja export mermaid architecture.sruja
sruja export markdown architecture.sruja
sruja export mermaid architecture.sruja --view-level 2 --target Shop.WebApp
```

### `tree`

Displays the architecture structure as a tree in the terminal.

**Usage:**

```bash
sruja tree <file>
```

**Example:**

```bash
sruja tree architecture.sruja
```

### `fmt`

Formats the Sruja file to a canonical style.

**Usage:**

```bash
sruja fmt [file]
```

### `lint`

Validates the Sruja file against rules and best practices.

**Usage:**

```bash
sruja lint [file]
# Machine-readable output for CI/agents:
sruja lint [file] --format json
```

For the JSON schema and diagnostic codes, see [LINT_JSON_OUTPUT.md](../../../docs/LINT_JSON_OUTPUT.md) in the repo (or [online](https://github.com/sruja-ai/sruja/blob/main/docs/LINT_JSON_OUTPUT.md)).

### Discovery (for skills and agents)

```bash
sruja discover --context -r .          # Human-readable repo context
sruja discover --context -r . --format json   # Machine-readable (components, areas, framework)
sruja discover                          # Question bank only
```

### Optional reviewed-intent and advanced commands

These commands are useful when you want reviewed intent in Git, richer automation, or advanced team workflows.

| Command | Purpose |
|---------|---------|
| **`sruja sync -r <path>`** | Refresh evidence files |
| **`sruja drift -r <path> -a repo.sruja`** | Compare reviewed intent to actual code |
| **`sruja ai-context -r <path>`** | Export structured context for host tools |
| **`sruja why "question" -r <path>`** | Investigate architecture questions with deterministic evidence |
| **`sruja drift --ci -r <path> -f github-actions`** | CI-focused drift check output |
| **`sruja baseline -r <path>`** | Snapshot current violations to ignore them in CI |
| **`sruja compliance -r <path> -a repo.sruja -i <intent_dir>`** | Structural drift + intent + policy violations |
| **`sruja publish -r <path> -o repo.bundle.json`** | Export repo truth + evidence for federation |
| **`sruja compose -i <bundle.json> -o system.index.json`** | Compose repo bundles into a system index |
| **`sruja mcp`** | Start MCP stdio server |
| **`sruja scan <path> --output graph.json`** | Raw graph JSON for scripting and advanced use |

**Examples (skill workflow / CI):**

```bash
sruja discover --context -r . --format json
sruja drift -r . -a repo.sruja
sruja impact Shop.WebApp -r .
sruja why "why did we choose PostgreSQL?" -r .
sruja ai-context -r . -f markdown -o .cursor/rules/architecture.md
```

### Older or supporting workflow aliases

These still exist, but they are not the main path to teach first.

```bash
# First-time setup
sruja start -r . --prompt

# First look (no files required)
sruja overview -r .

# Day-to-day review
sruja daily -r .

# Quick repo health check
sruja doctor -r .

# Keep feedback live while coding
sruja watch -r .
```

Aliases:

- `start` = `init`
- `daily` = `review`
- `overview` = `quickstart`
- `doctor` = `status`

### CI workflow

```bash
# Snapshot current violations (one-time)
sruja baseline -r . -o .sruja/violations.baseline.json

# CI check that ignores existing issues and reports only new ones
sruja drift --ci -r . --baseline .sruja/violations.baseline.json -f github-actions
```

### MCP (optional)

If your editor/tooling supports MCP, Sruja can run as an MCP stdio server:

```bash
sruja mcp
```

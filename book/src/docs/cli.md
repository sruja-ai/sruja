---
title: "CLI Guide"
weight: 50
summary: "The sruja command-line interface is your primary tool for interacting with Sruja projects."
---

# CLI Guide

The `sruja` command-line interface is your primary tool for interacting with Sruja projects.

<!--more-->

## Commands

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

### Commands that back the AI skill (and CI)

The **sruja-architecture skill** is the primary way to get value; it runs discovery and drift under the hood. These CLI commands power the skill and are also useful to run directly (CI, automation, power users).

| Command | Purpose |
|---------|---------|
| **`sruja discover --context -r <path> --format json`** | Evidence for the skill: repo structure, technologies, modules (skill runs this) |
| **`sruja sync -r <path>`** | Refresh `.sruja/context.json` (summary) and `.sruja/graph.json` (full graph); skill uses these for evidence |
| **`sruja drift -r <path> -a repo.sruja`** | Declared vs actual; skill uses for refinement; CI for gates |
| **`sruja context -r <path>`** | Export architecture context for AI tools |
| **`sruja quickstart -r <path>`** | Structural overview (optional; used by skill/CI, not promoted as primary) |
| **`sruja status -r <path>`** | Show baseline + health + truth status (reviewed / drifted / unknown) |
| **`sruja review -r <path>`** | Review workflow: refresh evidence, detect drift, propose updates/open questions |
| **`sruja impact <target> -r <path>`** | Blast radius analysis (upstream dependents + downstream dependencies) |
| **`sruja why "question" -r <path>`** | Ask architecture questions with deterministic evidence from the knowledge graph |
| **`sruja check -r <path> -f github-actions`** | CI-focused drift check output |
| **`sruja baseline -r <path>`** | Snapshot current violations to ignore them in CI (`sruja check --baseline ...`) |
| **`sruja intent check -r <path> -i <intent_dir>`** | Compare declared architectural intent vs actual implementation |
| **`sruja compliance -r <path> -a repo.sruja -i <intent_dir>`** | Structural drift + intent + policy violations (exit 1 if non-compliant) |
| **`sruja publish -r <path> -o repo.bundle.json`** | Export repo truth + evidence for federation |
| **`sruja compose -i <bundle.json> -o system.index.json`** | Compose repo bundles into a system index |
| **`sruja mcp`** | Start MCP stdio server (for tool-based clients) |
| **`sruja scan <path> --output graph.json`** | Raw graph JSON (scripting / advanced) |

**Examples (skill workflow / CI):**

```bash
sruja discover --context -r . --format json
sruja drift -r . -a repo.sruja
sruja impact Shop.WebApp -r .
sruja why "why did we choose PostgreSQL?" -r .
sruja context -r . -f markdown -o .cursor/rules/architecture.md
```

### Repo workflow (recommended)

Use these when you want “architecture health” and “keep it in sync” workflows in a repo.

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
sruja check -r . --baseline .sruja/violations.baseline.json -f github-actions
```

### MCP (optional)

If your editor/tooling supports MCP, Sruja can run as an MCP stdio server:

```bash
sruja mcp
```

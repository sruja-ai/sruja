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

### Architecture intelligence (no .sruja required)

These commands work on a repository path and do not require an existing `.sruja` file or API keys.

| Command | Purpose |
|---------|---------|
| **`sruja quickstart -r <path>`** | Scan repo; get inventory, health score, top findings, and next steps |
| **`sruja scan -r <path> -o graph.json`** | Infer architecture graph from code (outputs JSON) |
| **`sruja drift -r <path>`** | Detect drift: cycles, orphans, layer violations |
| **`sruja why "question" -r <path>`** | Answer "why" questions with evidence from the graph |
| **`sruja analyze -r <path>`** | Full analysis (structural, semantic, recommendations) |
| **`sruja context -r <path>`** | Export architecture context for AI tools (Cursor, Copilot, etc.) |

**Examples:**

```bash
sruja quickstart -r .
sruja scan -r . -o sruja.graph.json
sruja drift -r . -a architecture.sruja
sruja why "why did we choose PostgreSQL?" -r .
sruja context -r . -f markdown -o .cursor/rules/architecture.md
```

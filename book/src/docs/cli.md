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
sruja export [format] [file]
```

**Supported Formats:**

- `markdown`: Generates Markdown docs with diagrams.
- `mermaid`: Generates Mermaid diagram code.
- `svg`: Exports rendered SVG diagrams.
- `json`: Exports structured JSON of the architecture.
- `d2`: Generates D2 diagram code.

**Options:**

- `--view <name>`: Export a specific custom view (mermaid format only).
- `--all-views`: Export all custom views (mermaid and markdown formats).
- `--extended`: Include pre-computed views in JSON output.
- `--view-level <1|2|3>`: C4 view level (1=context, 2=container, 3=component).
- `--target <id>`: Focus node ID for view levels 2/3.

**Example:**

```bash
# Basic exports
sruja export markdown architecture.sruja
sruja export mermaid architecture.sruja
sruja export svg architecture.sruja
sruja export json architecture.sruja
sruja export d2 architecture.sruja

# Custom view exports
sruja export mermaid architecture.sruja --view api_focus
sruja export markdown architecture.sruja --all-views

# C4 view level exports
sruja export mermaid architecture.sruja --view-level 2 --target Shop.WebApp
```

### `tree`

Displays the architecture structure as a tree in the terminal.

**Usage:**

```bash
sruja tree --file [file]
```

**Options:**

- `--json`: Output as JSON.
- `--system [ID]`: Show tree for a specific system.

**Example:**

```bash
sruja tree --file architecture.sruja
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
```

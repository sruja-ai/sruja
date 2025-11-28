---
title: CLI Guide
weight: 4
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

-   `d2`: Generates D2 diagram code.

**Example:**

```bash
sruja export d2 architecture.sruja
```

### `tree`

Displays the architecture structure as a tree in the terminal.

**Usage:**

```bash
sruja tree --file [file]
```

**Options:**

-   `--json`: Output as JSON.
-   `--system [ID]`: Show tree for a specific system.

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

### `check`

Validates the Sruja file against rules and best practices.

**Usage:**

```bash
sruja check [file]
```

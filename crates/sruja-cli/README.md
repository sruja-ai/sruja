# sruja-cli

The primary command-line interface for Sruja, providing tools for architectural discovery, linting, drift detection, and export.

## Overview

`sruja-cli` serves as the user-facing entry point for the Sruja platform. it orchestrates the various Sruja crates to provide a seamless "Architecture as Code" experience.

## Key Features

- **Discovery:** `sruja discover` analyzes your codebase to infer architectural structure.
- **Linting:** `sruja lint` validates your `.sruja` files against the DSL rules.
- **Drift Detection:** `sruja drift` compares your declared architecture with the real-time code evidence.
- **Export:** `sruja export` generates Mermaid diagrams, Markdown documentation, and JSON models.
- **Agentic Memory:** `sruja agent` provides a history of architectural learnings and guardrails to guide AI agents.

## Usage

### Basic Commands

```bash
# Initialize Sruja in a repository
sruja init

# Scan the repository and see what Sruja detects
sruja quickstart

# Lint an architecture file
sruja lint repo.sruja

# Check for architectural drift
sruja drift -a repo.sruja
```

### Exporting Architecture

```bash
# Export to Mermaid for visualization
sruja export mermaid repo.sruja > diagram.mmd

# Export to Markdown for documentation
sruja export markdown repo.sruja > ARCHITECTURE.md
```

## Architecture

The CLI is built using `clap` for argument parsing and integrates with the following core crates:
- `sruja-engine`: Validation and rules.
- `sruja-language`: DSL parsing and AST management.
- `sruja-graph`: Knowledge graph operations.
- `sruja-agent`: Persistent memory for AI agents.

## Development

To build the CLI from source:

```bash
cargo build -p sruja-cli
```

To run tests:

```bash
cargo test -p sruja-cli
```

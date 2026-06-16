# Development Guide

This guide covers development for the Rust Sruja repo (CLI, engine, export, WASM) and the VS Code extension.

## Quick Start

```bash
# Install Rust (if needed): https://rustup.rs/
just install   # cargo fetch
just build     # cargo build --release
just test      # cargo test
just fmt       # cargo fmt
just lint      # cargo clippy
```

## First value (no .sruja)

```bash
cargo build --release -p sruja-cli
./target/release/sruja quickstart -r .
```

See `demo/README.md` for the end-to-end demo flow.

## Validate .sruja files

```bash
./target/release/sruja lint book/valid-examples/*.sruja
./target/release/sruja export markdown path/to/file.sruja
```

## WASM build

```bash
just wasm        # web target → crates/sruja-wasm/pkg/
just wasm-nodejs # nodejs target
```

## Project layout

| Crate | Purpose |
|-------|---------|
| **sruja-cli** | CLI: lint, export, scan, why, drift, critique, propose, focus, context, agent, context-score, context-graph, compliance, ingest, federation |
| **sruja-language** | Parser, AST, and pluggable domain schemas |
| **sruja-engine** | Validation rules |
| **sruja-export** | Markdown, Mermaid, JSON, HTML/D3 export |
| **sruja-wasm** | WASM build for browser/Node |
| **sruja-diagnostics** | Diagnostic types |
| **sruja-graph** | Knowledge graph, centrality, coupling |
| **sruja-graph-core** | Core graph types and primitives |
| **sruja-scan** | Repo scanning (multi-language tree-sitter) |
| **sruja-diff** | Drift detection + proposal system |
| **sruja-intent** | Intent vs. reality comparison + adversarial critique engine |
| **sruja-extract** | Source code extraction utilities |
| **sruja-agent** | Agentic memory – persistent learning and guardrails for AI agents |
| **book/** | mdBook documentation |

## VS Code extension

The extension in `extension/` provides syntax highlighting and language features for `.sruja` files, powered by WASM.

## Skills and evaluation

- **Skills:** `skills/` — `sruja-architecture` is the single supported skill.
- **Comparison (Mermaid vs Sruja):** `scripts/run_comparison_test.sh [project] [url]`; results in `evaluation/results/comparison_*`.

---

## Testing Features

### Testing `sruja drift --ci` (CI drift check)

#### Automated tests

Run the e2e tests (uses a temp repo, runs `sruja drift --ci` in JSON, text, and github-actions formats):

```bash
cargo test -p sruja-cli --test check_e2e
```

Or with output:

```bash
cargo test -p sruja-cli --test check_e2e -- --nocapture
```

#### Manual testing

From the repo root (or any path with a baseline and optional code):

```bash
# Build the CLI first if needed
cargo build --release -p sruja-cli

# Default: github-actions format (for CI)
sruja drift --ci -r .

# Text summary
sruja drift --ci -r . -f text

# JSON (for tooling)
sruja drift --ci -r . -f json
```

**What to verify:** Exit code is always 0. Output shows truth status, violation count, and (for `-f github-actions`) `::notice`/`::warning` annotations.

### Testing `sruja review` (evidence + drift + suggestions)

No automated e2e tests yet. Test manually.

```bash
# From repo root (uses . or -r path)
sruja review -r .

# JSON output
sruja review -r . -f json
```

**What to verify:** Output includes `truth_status`, `violations_count`, categorized lists (`new_components`, `missing_components`, `drifted_dependencies`), `open_questions`, and `suggestions`.

### Testing Federation: `sruja publish` and `sruja compose`

No dedicated e2e tests. Use manual runs and schema checks (see [FEDERATION.md](FEDERATION.md)).

#### Manual: publish (repo → bundle)

```bash
# Publish current repo to repo.bundle.json
sruja publish -r . -o repo.bundle.json

# Publish a subdirectory
sruja publish -r ./services/api -o /tmp/api.repo.bundle.json
```

**Verify:** `repo.bundle.json` exists and contains `schema_version`, `repo_id`, `context`, `truth_status`, and (if a baseline exists) `baseline_path` / `baseline_dsl`.

#### Manual: compose (bundles → system index)

```bash
# One bundle
sruja compose -i repo.bundle.json -o system.index.json

# Directory of bundles
sruja compose -i ./bundles -o system.index.json
```

**Verify:** `system.index.json` contains `schema_version`, `repos`, `nodes`, `edges`, and `conflicts` (may be empty).

### Quick Reference

| Feature   | Automated tests              | Manual test                          | CI |
|----------|------------------------------|--------------------------------------|----|
| **check**  | `cargo test -p sruja-cli --test check_e2e` | `sruja drift --ci -r . -f text` / `json` / `github-actions` | Sruja Check workflow on PR |
| **review** | —                            | `sruja review -r .` / `-f json`      | — |
| **publish**| —                            | `sruja publish -r . -o repo.bundle.json` | — |
| **compose**| —                            | `sruja compose -i repo.bundle.json -o system.index.json` | — |

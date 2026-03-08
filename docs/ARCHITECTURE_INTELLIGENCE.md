# Architecture Intelligence Layer

The Sruja architecture intelligence layer helps developers understand *why* their system is built the way it is. It provides **architecture memory + drift intelligence for real codebases**—with **no API keys required** for the first value.

## For Developers

### Use Cases

1. **"Why did we choose X?"** — Ask about technology decisions (Kafka, Redis, PostgreSQL, etc.)
2. **"What services do we have?"** — Query the architecture graph
3. **"How do components connect?"** — Inspect dependencies
4. **"Where is the drift?"** — Detect code vs. design mismatches with evidence

### Entry Points (CLI First)

| Tool | Purpose |
|------|---------|
| **sruja quickstart -r .** | Primary entry: inventory, drift summary, health score, actionable fixes — no API key |
| **sruja why** | Quick "why" queries against scanned repo; deterministic answers with evidence |
| **sruja drift** | Detect architecture drift (circular deps, orphans, layer violations, god modules) |
| **sruja scan** | Infer architecture graph from code |
| **sruja-app** (optional) | Desktop UI for repo + query — optional LLM key |

The CLI is the hero surface. No model key, desktop app, or .sruja files are required to get value.

### Quick Start (CLI — Zero Key)

```bash
# Ask "why" about your repo (scans on the fly, deterministic)
sruja why "why did we choose PostgreSQL?" -r .

# Or use a pre-generated graph
sruja scan . -o graph.json
sruja why "what services do we have?" --graph graph.json

# Full quickstart (inventory + drift + next steps)
sruja quickstart -r .
```

Output includes architecture inventory, drift findings, evidence references. No API key needed.

### Optional: Desktop App (LLM Enrichment)

```bash
# Set API key only if you want LLM extraction and agents
export OPENAI_API_KEY="sk-..."   # or OPENROUTER, ANTHROPIC, GEMINI
cargo run -p sruja-app
```

When a key exists, the app adds conversational extraction, decision drafts, richer natural-language synthesis. If the key is missing, deterministic CLI mode remains fully useful.

### Demos

| Demo | Command | What it shows |
|------|---------|----------------|
| **E2E value** | `make demo` or `cd evaluation/real-world-test && ./run_demo.sh` | Quickstart + drift on a real repo (Express); optional baseline. |
| **Architecture Intelligence** | `make demo-intel` or `cd demo && ./run_demo.sh` | Full flow: intent → scan → drift → analyze → why (deterministic). Use the Sruja skill in your editor for AI interpretation. |
| **Commit-to-commit drift** | `demo/run_commit_drift_demo.sh [REPO] [BASELINE] [HEAD]` | Architecture Intelligence at baseline commit, then drift report (new violations only) from baseline to head. See [demo/COMMIT_DRIFT_DEMO.md](../demo/COMMIT_DRIFT_DEMO.md). |

Run from the repo root after `make build`.

## Architecture

```
                    +-----------------------------+
                    |         Interfaces          |
                    |  CLI | MCP/HTTP | Slack*   |
                    +-------------+---------------+
                                  |
                                  v
                  +-------------------------------+
                  |  Architecture Intelligence     |
                  |  (single orchestration layer)  |
                  |  scan + query + drift + merge  |
                  +------+------------+------------+
                         |            |
                         v            v
                +-------------+   +-------------+
                | Knowledge   |   | Extraction  |
                | Graph       |   | (optional)  |
                +------+------+   +-------------+
                       |
                       v
                 +-----------+
                 | Language  |
                 | + Rules   |
                 +-----------+
```

`*` Slack is a later channel adapter, not a core dependency.

### MCP HTTP API (tool execution)

The MCP server exposes architecture intelligence as HTTP endpoints. In addition to fixed routes (`/architecture`, `/decisions`, `/query`, etc.), clients can list and execute tools dynamically:

- **GET /tools** — Returns the list of available tools (name + description). Response shape: `{ "success": true, "data": [ { "name": "run_analyze", "description": "..." }, ... ] }`.

- **POST /tools/execute** — Runs a single tool. Request body must be the JSON representation of one `SrujaTool` variant. Response shape: `{ "success": true, "data": { "tool": "run_analyze", "success": true, "result": { ... }, "error": null } }`.

**Request body examples:**

```json
{"SemanticAnalyze":{"repo_path":"/path/to/repo"}}
```

```json
{"RunAnalyze":{"repo_path":"/path/to/repo","traces_path":null,"intent_path":null}}
```

```json
{"Complexity":{"repo_path":"/path/to/repo","treewidth":true,"scc":true,"centrality":false,"coupling":false}}
```

```json
{"DetectDriftWithBaseline":{"repo_path":"/path/to/repo","architecture_path":"/path/to/arch.sruja"}}
```

Path parameters (e.g. `repo_path`, `architecture_path`) are validated against the server process current working directory; paths outside it are rejected. For long-running tools (e.g. `RunAnalyze` on large repos), consider using small or medium-sized repositories.

## Key Concepts

### Health score

The 0–100 health score is derived from structural violations (cycles, layer violations, god modules, orphans) with fixed weights and caps. It is **meaningful for comparing refs and spotting structural regression**, but it is not size-normalized by default and does not capture other dimensions (tests, docs, coupling strength). See [Health score](HEALTH_SCORE.md) for the formula, uses, and limitations. For when the findings are **really useful** vs. noisy (e.g. cycles vs. god modules in stories/vendor), see [Insights usefulness](INSIGHTS_USEFULNESS.md).

### Zero-Key Deterministic Mode

Without LLM, Sruja provides architecture intelligence via code evidence:

- **Inputs:** Source code + manifests (sruja-scan), optional baseline graph (sruja-diff)
- **Outputs:** Architecture inventory, dependency graph, drift/risk report (cycles, orphans, layer violations, god modules), evidence pointers
- **Why queries:** "X depends on Y because imports/calls were detected in A, B, C" — reproducible and inspectable

### LLM as Optional Enhancement

LLM is additive for:

- Conversational extraction from discussions
- Decision draft generation
- Richer natural-language synthesis

If key exists → enrich results.  
If key missing → deterministic mode remains fully useful.

### Query Types

- **Why** — Technology choices (e.g. "why Kafka?")
- **What** — Components by kind (services, databases, etc.)
- **How** — Dependencies and connectivity
- **Decisions** — List and explain ADRs (when extraction enabled)

### Configuration

| Variable | Purpose |
|----------|---------|
| `SRUJA_EXTRACTION_MODEL` | Model for extraction (default: openai/gpt-4o-mini) |
| `SRUJA_DATA_DIR` | Override persistence directory (default: `~/.sruja/data`) |
| `.env` | Loaded at app startup (see `.env.example`) |

## E2E Tests

```bash
# Why command flow (sruja-cli)
cargo test -p sruja-cli --test why_e2e

# All tests
make test
```

## Current state: where we're standing

Target flow: **two entry points (desktop app or CLI) → configure repos and docs → Sruja analyzes → architecture intelligence**. Status:

| Area | Implemented | Missing / planned |
|------|--------------|-------------------|
| **CLI** | **Configure:** repo via `-r`, intent via `-i` or `SRUJA_INTENT_PATH`, traces via `-t` or `SRUJA_TRACES_PATH`. **Analyze:** `quickstart`, `drift`, `why`, `scan`, `complexity`, `semantic`, `analyze`, `intent check`, `runtime analyze`. **Intelligence:** inventory, drift report, why evidence, health score, recommendations (text/JSON). | No `sruja.toml` / project config file yet (planned in two-dev plan Week 14). |
| **Desktop app** | **Configure:** repo path in sidebar; persisted in workspace (`repo_path`), auto-load on startup. **Analyze:** “Load Context” runs scan and merges into knowledge graph. **Intelligence:** “Ask” bar runs graph query (why-style), shows result in right panel; chat, sessions, extractions (with LLM). | No docs/intent path in workspace. No drift, quickstart, or full `analyze` in UI. Not yet “OpenCode/Codex-style” (e.g. open project first, then single place for all intelligence). |
| **Config** | CLI: flags and env vars. App: `WorkspaceState` has `repo_path` and `last_session_id` only. | `WorkspaceState` has no `intent_path` or `docs_path`. No shared `sruja.toml` or `sruja-config` crate. |
| **MCP** | Tools: `RunAnalyze`, `SemanticAnalyze`, `Complexity`, `DetectDriftWithBaseline` with `repo_path` (and optional `intent_path`, `traces_path`, `architecture_path`). | — |

**Summary:** CLI already supports the full flow (configure repos/docs via args or env → analyze → intelligence). The desktop app supports configure (repo only) → scan → query (why); adding docs/intent config, drift, and full analyze in the UI would align it with the same flow. A shared project config (e.g. `sruja.toml`) is not yet implemented.

## Strategy

For product direction, module decisions, and execution plan:  
**[architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md](../architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md)**

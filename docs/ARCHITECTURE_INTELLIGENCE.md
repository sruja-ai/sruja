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
| **sruja quickstart -r .** (planned) | Primary entry: inventory, drift summary, health score, actionable fixes — no API key |
| **sruja why** | Quick "why" queries against scanned repo; deterministic answers with evidence |
| **sruja drift** | Detect architecture drift (circular deps, orphans, layer violations, god modules) |
| **sruja scan** | Infer architecture graph from code |
| **sruja-app** (optional) | Desktop UI for chat, agents, extraction — requires LLM key |

The CLI is the hero surface. No model key, desktop app, or .sruja files are required to get value.

### Quick Start (CLI — Zero Key)

```bash
# Ask "why" about your repo (scans on the fly, deterministic)
sruja why "why did we choose PostgreSQL?" -r .

# Or use a pre-generated graph
sruja scan . -o graph.json
sruja why "what services do we have?" --graph graph.json

# Detect drift (planned: quickstart command)
# sruja quickstart -r .
```

Output includes architecture inventory, drift findings, evidence references. No API key needed.

### Optional: Desktop App (LLM Enrichment)

```bash
# Set API key only if you want LLM extraction and agents
export OPENROUTER_API_KEY="sk-or-v1-..."
cargo run -p sruja-app
```

When a key exists, the app adds conversational extraction, decision drafts, richer natural-language synthesis. If the key is missing, deterministic CLI mode remains fully useful.

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

## Key Concepts

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

### Configuration (Optional for LLM)

| Variable | Purpose |
|----------|---------|
| `OPENROUTER_API_KEY` | Optional — for LLM extraction and agents |
| `SRUJA_EXTRACTION_MODEL` | Model for extraction (default: openai/gpt-4o-mini) |
| `SRUJA_DATA_DIR` | Override persistence directory (default: `~/.sruja/data`) |
| `.env` | Loaded at app startup (see `.env.example`) |

## E2E Tests

```bash
# Architecture intelligence flow (sruja-chat)
cargo test -p sruja-chat --test architecture_intelligence_e2e

# Why command flow (sruja-cli)
cargo test -p sruja-cli --test why_e2e

# All tests
make test
```

## Strategy

For product direction, module decisions, and execution plan:  
**[architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md](../architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md)**

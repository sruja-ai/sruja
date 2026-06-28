# Sruja

[![Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg)](https://codecov.io/gh/sruja-ai/sruja)

**Sruja is a CLI-first autonomous coding agent where the actor never grades itself.** It scans your repo, grounds every edit in real topology, and verifies the result with an independent deterministic grader before you ship.

```text
sruja agent loop --goal "..."   →   comprehend · plan · execute · critique · replan
```

Sruja runs two ways:

- **Autonomous** — `sruja agent loop` owns the full observe → act → verify → critique → replan loop, bounded by budget, iterations, and file guards.
- **Passive harness** — inside any editor (Cursor, Copilot, Claude, Windsurf) via MCP; same deterministic gates, your LLM drives.

The deterministic layer (`drift`, `lint`, `verify-task`, `intent check`) is the **independent grader** — the model that wrote the code never judges it.

---

## The agent loop

The core product is a closed-loop agent that owns the full observe → act → verify → critique → replan cycle:

```bash
# Autonomous — Sruja plans, edits, verifies, critiques, and replans
sruja agent loop --goal "Refactor auth boundary without breaking tests"
```

Sruja's deterministic layer (drift, lint, verify-task, intent check) is the **independent grader** — the actor never grades itself.

### Editor-hosted loop (MCP)

When your editor drives the LLM, Sruja provides the grounding and verification gates:

```text
1. MCP: focus / drift state / boundary context
2. Host LLM edits code
3. sruja verify-task --profile coding -r .
4. (optional) sruja agent record -c "…" on failure
```

---

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

---

## How it works

### 1. Capture knowledge (optional)

Bring durable context into the repo — design docs, ADRs, decisions:

```bash
sruja ingest <path>
sruja decision add --title "Use async for I/O" --rationale "..."
```

Reviewed intent in Git (optional, for teams that want CI-enforced boundaries):

```bash
sruja lint repo.sruja
sruja sync -r .
sruja drift -r . -a repo.sruja
```

### 2. Retrieve task context

Before editing, get a file-scoped briefing with blast radius, dependencies, and relevant decisions:

```bash
sruja focus -r . --file path/to/file.rs    # structured briefing
sruja ai -r . --task "Refactor auth boundary"  # paste-ready brief
```

### 3. Verify alignment

After editing, deterministic checks confirm the result still matches structure and intent:

```bash
sruja verify-task --profile coding -r .   # full verification
sruja drift -r . --structural-only        # topology-only check
sruja intent check -r .                   # design-intent check
```

---

## Editor integration

Sruja integrates through MCP, giving your AI editor grounded context and post-edit verification.

Cursor template: [.cursor/mcp.json](.cursor/mcp.json)

```json
{
  "mcpServers": {
    "sruja": {
      "command": "sruja",
      "args": ["mcp", "-r", "."],
      "env": {
        "SRUJA_MCP_TOOL_PROFILE": "coding",
        "SRUJA_MCP_READONLY": "1"
      }
    }
  }
}
```

Use the default `coding` profile (15 tools: focus, drift, verify, memory search, hybrid query, critique, and more).

---

## How Sruja compares

| | Copilot / Cursor / Windsurf | Sruja |
|---|---------------------------|-------|
| **What it does** | Generates code from prompts | Generates code **and** verifies it against repo topology |
| **Grounding** | File context + embeddings | Structural scan (cycles, layers, god modules) + optional reviewed intent |
| **Verification** | None built-in | `verify-task`, `drift`, `intent check` — deterministic, post-edit |
| **Autonomous mode** | Host-managed agent loop | `sruja agent loop` — owns the full observe → act → verify → critique → replan |
| **Editor story** | Native agent UX | MCP `coding` profile — same gates, any editor |

**One line:** AI coding agents generate; Sruja generates **and** gates — before and after.

**Honest limits:** See [KNOWN_LIMITATIONS.md](docs/KNOWN_LIMITATIONS.md) — dynamic imports, reflection/DI, heuristic layers, orphan false positives on greenfield (use `drift --advisory`).

---

## Extensions

Everything else builds on the agent loop and deterministic gates above.

### Core-adjacent

- **Architecture authoring**: richer `repo.sruja` workflows, proposals, author evidence
- **Visualization**: Mermaid, Markdown, D2, GraphML, Neo4j, Obsidian exports
- **Team workflows**: review, drift in CI, compliance, critique, federation

### Advanced

- **Agent ops**: learnings, run snapshots, memory curation, continual learning
- **Workflow orchestration**: workflow and AI-DLC flows
- **Analytics**: dashboards, graph history, metrics, registry tooling

---

## Docs

- Website: https://sruja.ai
- Getting started: [book/src/getting-started.md](book/src/getting-started.md)
- Host/editor setup: [HOST_AGENT_INTEGRATION.md](docs/HOST_AGENT_INTEGRATION.md)
- Agent loop + continual learning: [GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md](docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md)
- Core vs extensions: [FEATURE_TIERS.md](docs/FEATURE_TIERS.md)
- MCP tools reference: [mcp_tools_reference.md](docs/mcp_tools_reference.md)

---

## Installation

**Install script**

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

**Cargo**

```bash
cargo install sruja-cli --git https://github.com/sruja-ai/sruja
```

**Build from source**

```bash
git clone https://github.com/sruja-ai/sruja.git
cd sruja
just build
```

### Optional skills

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

---

## Troubleshooting

**`sruja: command not found`**

```bash
export PATH="$HOME/.local/bin:$PATH"
```

**AI generates invalid reviewed intent**

```bash
sruja lint repo.sruja --format json
```

**Need to understand what Sruja found**

- `sruja focus` before a change
- `sruja ai` for a paste-ready task brief
- `sruja why` for investigation

## Contributing

- [Contributing Guide](docs/CONTRIBUTING.md)
- [Roadmap](ROADMAP.md)

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome!

## License

Apache 2.0 or MIT

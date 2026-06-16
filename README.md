# Sruja

[![Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg)](https://codecov.io/gh/sruja-ai/sruja)

Sruja is an AI coding agent that helps software teams build better code. It:

- captures important knowledge and decisions
- retrieves the right task context for developers
- verifies that code changes align with architectural intent

Sruja works as an intelligent coding partner that understands your codebase architecture and helps you make changes that respect your design decisions.

## Core Capabilities

### 1. Capture

Bring durable context into the repo:

- `sruja ingest <path>` for design docs, ADRs, and notes
- `sruja decision ...` for decision records and links
- optional `repo.sruja` when you want reviewed intent in Git

### 2. Retrieve

Give humans and AI the minimum useful context before editing:

- `sruja focus -r . --file path/to/file.rs`
- `sruja ai -r . --task "Refactor auth boundary"`
- `sruja mcp -r .` for editor and agent tooling

### 3. Verify

Check that implementation still matches reality and intent:

- `sruja drift -r . --structural-only --advisory`
- `sruja intent check -r .`
- `sruja verify-task --profile coding -r .`
- `sruja lint repo.sruja` when reviewed intent exists

## Quick Start

Install the CLI and start using Sruja as your AI coding agent:

```bash
curl -fsSL https://sruja.ai/install.sh | bash

# 1) detect current structure
sruja start -r .
sruja drift -r . --structural-only --advisory

# 2) brief a change
sruja focus -r . --file path/to/file.rs

# 3) verify the result
sruja verify-task --profile coding -r .
```

This works without a `repo.sruja` file. Add reviewed intent later if you want stricter decision tracking in Git.

## Optional Reviewed Intent

When you want durable, reviewable intent in version control:

```bash
sruja lint repo.sruja
sruja sync -r .
sruja drift -r . -a repo.sruja
```

Treat `repo.sruja` as reviewed truth, not as the day-one requirement.

## Editor Integration

Sruja integrates with your editor through MCP, providing AI coding agent capabilities directly in your development environment.

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

Use the default `coding` profile. Sruja provides grounded context and verification as your AI coding agent.

## Extensions

Everything else should build on the core foundations above.

### Core-adjacent extensions

- `architecture authoring`: richer `repo.sruja` workflows, proposals, author evidence
- `visualization`: Mermaid, Markdown, D2, GraphML, Neo4j, Obsidian exports
- `team workflows`: review, drift in CI, compliance, critique, federation

### Advanced extensions

- `agent ops`: plan/apply, learnings, run snapshots, memory curation
- `workflow orchestration`: workflow and AI-DLC flows
- `inspection and analytics`: dashboards, graph history, metrics, registry tooling

These remain useful, but they are not the product story to lead with.

## Product Contract

Every public feature should strengthen one of these jobs:

- `capture knowledge`
- `surface decisions`
- `retrieve relevant context`
- `verify alignment with intent`

If a feature does none of those, it belongs in an extension or should be hidden.

## How Sruja Compares

| | Structurizr / LikeC4 | Sruja |
|---|---------------------|-------|
| **Primary user** | Humans documenting architecture | Agents and CI enforcing topology from code |
| **Source of truth** | Authoritative DSL/workspace views | Scan-derived graph; optional `repo.sruja` overlay |
| **Day-one value** | Draw and share C4 views | `sruja drift -r . --structural-only` — violations with file paths |
| **Editor story** | Diagrams and workspace UX | MCP `coding` profile + `focus` / `verify-task` |
| **Exports** | Core product | Tier 2 — Mermaid/Markdown from snapshot |

**One line:** Structurizr documents architecture for humans; Sruja extracts topology from code and gates agents (MCP + CI) before and after generation.

**Not the same as SonarQube:** Sruja reports **structural** topology (cycles, layers, god modules), not style or security rule packs.

**Honest limits:** See [KNOWN_LIMITATIONS.md](docs/KNOWN_LIMITATIONS.md) — dynamic imports, reflection/DI, heuristic layers, orphan false positives on greenfield (use `drift --advisory`).

## Docs

- Website book: https://sruja.ai
- Getting started: [book/src/docs/getting-started.md](book/src/docs/getting-started.md)
- Host/editor setup: [HOST_AGENT_INTEGRATION.md](docs/HOST_AGENT_INTEGRATION.md)
- Core vs extensions: [FEATURE_TIERS.md](docs/FEATURE_TIERS.md)
- MCP tools reference: [mcp_tools_reference.md](docs/mcp_tools_reference.md)
- Context framing: [CONTEXT_ENGINEERING.md](docs/CONTEXT_ENGINEERING.md)

## Installation

### CLI

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

### Optional Skills

Harness skill:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
```

Architecture authoring skill:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

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

- Run `sruja focus` before a change
- Run `sruja ai` for a paste-ready task brief
- Run `sruja why` for investigation

## Contributing

- [Contributing Guide](docs/CONTRIBUTING.md)
- [Roadmap](ROADMAP.md)

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome!

## License

Apache 2.0 or MIT

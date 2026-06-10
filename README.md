# Sruja

[![Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg)](https://codecov.io/gh/sruja-ai/sruja)

Sruja is a context engineering tool for software teams. It helps you:

- capture important knowledge and decisions
- retrieve the right task context for developers and AI agents
- verify that code changes still align with those decisions

The product center is not diagrams or generic agent orchestration. The product center is a small loop:

1. capture knowledge and reviewed intent
2. brief the next change with grounded context
3. verify the result with deterministic checks

## Core Foundations

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

Install the CLI and run the core loop:

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

Sruja works best when your editor can ask for context directly through MCP.

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

Use the default `coding` profile. Host editors own the LLM loop; Sruja supplies grounded context and verification.

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

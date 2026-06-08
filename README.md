# Sruja — Stop AI from breaking your repo structure

[![Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg)](https://codecov.io/gh/sruja-ai/sruja)

**Sruja scans your codebase and reports structural drift** — circular dependencies, layer violations, god modules — with file-level evidence. No `.sruja` file required on day one. Use **MCP** or `focus` so Cursor and other agents stay inside boundaries; use **`verify-task`** as a host gate after edits.

Optional `.sruja` is a human-readable snapshot for diagrams and strict CI — not the primary product.

## Quick start (OSS)

```bash
curl -fsSL https://sruja.ai/install.sh | bash
sruja start -r .
sruja drift -r . --structural-only --advisory
```

You get a scan summary, actionable findings (file paths), and a **could not infer** section for scan limits. Pinned examples: [examples/oss-demo/](examples/oss-demo/).

## Core workflow (recommended)

1. **Scan & drift (from code)**: `sruja start -r .` then `sruja drift -r . --structural-only --advisory`
2. **Brief before edits**: `sruja focus -r . --file path/to/file.rs`
3. **Gate after edits**: `sruja verify-task --profile coding -r .`

## Editor integration (MCP)

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

## Skills (optional, but recommended in agent workflows)

Harness skill (runs gates and prevents “done” without verification):

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
```

Architecture skill (only when you want reviewed intent in Git):

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

## Optional: reviewed intent in Git (`repo.sruja`)

When you choose to version architecture intent:

```bash
sruja lint repo.sruja
sruja sync -r .
sruja drift -r . -a repo.sruja
```

Exports (Markdown/Mermaid) are derived artifacts. The harness and drift gates are the product center.

## Docs

- Website book: https://sruja.ai
- Host/editor setup: [HOST_AGENT_INTEGRATION.md](docs/HOST_AGENT_INTEGRATION.md)
- Feature tiers: [FEATURE_TIERS.md](docs/FEATURE_TIERS.md)
- MCP tools reference: [mcp_tools_reference.md](docs/mcp_tools_reference.md)

| | Mermaid | Sruja |
|---|--------|-------|
| **Purpose** | Draw diagrams (syntax for charts) | Define architecture as code; validate and keep in sync |
| **Value** | Diagrams | Single source of truth, lint, drift, compliance, versioned `.sruja` |
| **Export** | N/A (native format) | Export to Mermaid (and Markdown/JSON) when you need a diagram |

Mermaid is an **export target** for Sruja, not a competitor. Use Sruja for grounding, validation, and persistence; use the exported Mermaid for viewing or embedding diagrams.

**What if the AI makes a mistake?**

Run `sruja lint repo.sruja` to catch errors. Tell your AI: "Fix these lint errors" and paste the output.

---

## What can I do?

### Generate and maintain architecture (use the skill)

In your AI editor, use the **sruja-architecture skill**. It runs discovery and drift under the hood. You can also run these yourself for CI or scripting:

```bash
# Evidence for the skill (or CI)
sruja discover --context -r . --format json

# Plain-English explanation of what Sruja found and why
# Use --update (alias --incremental) to run instantly using AST caching
sruja discover --explain -r . --update

# Validate
sruja lint repo.sruja

# Drift (when you have a baseline; the skill uses this too)
sruja drift -r . -a repo.sruja
```

### Automatic Community Detection
When running `sruja discover --explain`, Sruja automatically partitions your architecture graph into highly cohesive modules using the Label Propagation Algorithm (LPA). It generates clear module labels, cohesion scores, and boundaries (flagging `Weakly Bounded` areas) to give you deep insights into your codebase's component groupings.

### Export for documentation

```bash
# Markdown (readable docs)
sruja export markdown repo.sruja > ARCHITECTURE.md

# Mermaid (diagrams)
sruja export mermaid repo.sruja > ARCHITECTURE.mmd

# JSON (machine-readable)
sruja export json repo.sruja > ARCHITECTURE.json

# GraphML (XML format for Gephi, Cytoscape)
sruja export graphml repo.sruja --output-dir ./exports

# Neo4j (Cypher statements for graph databases)
sruja export neo4j repo.sruja --output-dir ./exports

# Obsidian (interactive Markdown vault with dual-linked wiki-links and Mermaid graphs)
sruja export obsidian repo.sruja --output-dir ./obsidian_vault
```

### Catch Architectural Drift in CI/CD

Prevent architectural decay by running Sruja in your CI/CD pipeline. The `sruja-check` action provides inline Pull Request annotations for any architectural drift.

```yaml
# .github/workflows/sruja.yml
name: Sruja Architectural Check
on: [push, pull_request]

jobs:
  sruja:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Sruja Check
        uses: sruja-ai/sruja-check@v1
        with:
          architecture_file: "repo.sruja"
          violations_baseline: ".sruja/violations.baseline.json"
```

This ensures that any new, unmapped component or layer violation is flagged directly on the PR, keeping your architecture documentation and codebase in perfect sync.

---

## Installation

### CLI (required)

**Option A – Install script (recommended):**

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

Downloads the latest binary from GitHub Releases. Adds to `~/.local/bin` by default.

**Option B – Install via cargo (requires Rust):**

```bash
cargo install sruja-cli --git https://github.com/sruja-ai/sruja
```

**Option C – Build from source:**

```bash
git clone https://github.com/sruja-ai/sruja.git
cd sruja
just build
```

**Verify installation:**

```bash
sruja --version
```

### AI Skill (recommended but optional)

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

### VS Code Extension (optional)

Install from [VS Code Marketplace](https://marketplace.visualstudio.com/) for:
- Syntax highlighting
- Diagnostics
- Diagram preview
- Export commands

---

## Documentation

**Getting Started:**

- [Install as a Skill](docs/INSTALL_AS_SKILL.md) – Editor-specific setup
- [Getting Started with Skills](docs/GETTING_STARTED_SKILL.md) – Complete workflow guide
- [Skill Reference](skills/sruja-architecture/SKILL.md) – What the skill does

**Language & CLI:**

- [Language Specification](docs/LANGUAGE_SPECIFICATION.md) – Complete DSL reference
- [Run Guide](docs/RUN_GUIDE.md) – CLI commands and examples
- [Known Limitations](docs/KNOWN_LIMITATIONS.md) – What Sruja can/can't do

**Advanced Topics:**

- [Multi-Repo Federation Setup Guide](docs/FEDERATION_SETUP_GUIDE.md) – Step-by-step federation setup
- [Multi-Repo Federation](docs/FEDERATION.md) – Technical reference for federation
- [Design Philosophy](docs/DESIGN_PHILOSOPHY.md) – Why Sruja works this way

---

## Troubleshooting

**"sruja: command not found"**

The CLI isn't on your PATH. Try:

```bash
# Add to PATH (if using install script)
export PATH="$HOME/.local/bin:$PATH"

# Or re-open your terminal
```

**Skill isn't loading in my editor**

1. Make sure you ran: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`
2. Check your editor supports [skills.sh](https://skills.sh)
3. Restart your editor

**AI generates invalid Sruja code**

Run: `sruja lint repo.sruja --format json`

Copy the error output and paste it to your AI with: "Fix these lint errors."

**Discovery doesn't find my components**

- Check your language is supported (see table above)
- Make sure you're in the correct directory
- Run `sruja discover --explain -r .` to see what Sruja inferred and where confidence is lower
- Try: `sruja inspect overview -r .` to see what's being detected
- Open an issue if something obvious is missing

---

## Contributing

We welcome contributions!

- [Contributing Guide](docs/CONTRIBUTING.md)
- [Good First Issues](https://github.com/sruja-ai/sruja/labels/good%20first%20issue)

---

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind are welcome!

---

## License

Apache 2.0 or MIT (dual-licensed)

---

## Links

- **Website:** https://sruja.ai
- **GitHub:** https://github.com/sruja-ai/sruja
- **Discussions:** https://github.com/sruja-ai/sruja/discussions
- **Roadmap:** [ROADMAP.md](ROADMAP.md)
- **Contributing:** [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md)
- **Security:** [docs/SECURITY.md](docs/SECURITY.md)

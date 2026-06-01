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

## Step 2 — Editor integration (MCP)

Add to Cursor (template in [.cursor/mcp.json](.cursor/mcp.json)):

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

Before a task: `sruja focus -r . --file path/to/file.rs`. After edits: `sruja verify-task --profile coding -r .`.

### Post-AI-edit confidence report

After AI edits code, generate a confidence report showing what changed, what evidence was checked, and what risks remain:

```bash
sruja confidence -r . -f md
```

The report includes: verification results, changed files, intent/architecture alignment, blockers, and follow-up commands for the 3AM review. See [`.cursor/commands/sruja-confidence-report.md`](.cursor/commands/sruja-confidence-report.md).

## Step 3 — Skills (harness + optional architecture)

**Harness** (recommended for every agent workflow):

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
```

Teaches the editor to run `verify-task` before marking work done. Works without `repo.sruja`.

**Architecture** (optional — versioned intent in CI):

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

Ask your editor to promote scan evidence to `repo.sruja`, then `sruja lint repo.sruja` and `sruja drift -r . -a repo.sruja`.

Diagrams are Tier-2 exports (Mermaid/Markdown), not the hero.

---

## What is this?

**Problem:** AI editors introduce silent structural debt — wrong layers, new cycles, hub modules — while velocity stays high.

**Solution:** Deterministic scan + drift from code; MCP and `focus` feed agents bounded evidence; optional `repo.sruja` when teams want declared intent in CI.

**Not competing with:** Cursor, Windsurf, or Claude Code as your coding agent. Sruja is the **harness** (drift, focus, verify-task), not a second IDE.

Docs: [docs/MESSAGING.md](docs/MESSAGING.md) · [docs/FEATURE_TIERS.md](docs/FEATURE_TIERS.md) · [docs/HOST_AGENT_INTEGRATION.md](docs/HOST_AGENT_INTEGRATION.md) · [docs/examples/host-gates/](docs/examples/host-gates/) · [docs/STRUCTURIZR_VS_SRUJA.md](docs/STRUCTURIZR_VS_SRUJA.md)

---

## Quick Start (details)

### Option A — MCP + drift (recommended for OSS)

Use the commands in **Quick start** above, then register MCP. See [docs/mcp_tools_reference.md](docs/mcp_tools_reference.md).

### Option B — Skill for reviewed `repo.sruja`

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

In your AI editor:

```text
Use sruja-architecture. Run structural drift first, then promote evidence to repo.sruja and lint.
```

### Daily developer loop

Use the friendlier workflow aliases if you want Sruja to feel like a daily repo assistant instead of a long command list:

```bash
# First-time repo setup
sruja start -r . --prompt

# Install GitHub Actions workflows (check + onboarding brief)
sruja start -r . --ci

# Day-to-day review: refresh evidence and see what changed
sruja daily -r .

# Generate a paste-ready brief for your AI coding assistant
sruja ai -r . --task "Fix the parser error reporting"

# Keep feedback live while you code
sruja inspect watch -r .

# Quick repo health check
sruja doctor -r .
```

Aliases (top-level):
- `start` = `init`
- `daily` = `review`
- `doctor` = `status`

### Validate and export

```bash
# Check for errors
sruja lint repo.sruja

# Export a diagram (for docs, presentations, etc.)
sruja export mermaid repo.sruja > diagram.mmd
```

You can open `diagram.mmd` in [Mermaid Live Editor](https://mermaid.live) or use the VS Code extension for preview. **You can export to Mermaid whenever you need a diagram; Sruja’s job is to keep that architecture valid and in sync** (lint, drift, compliance), not to replace Mermaid—it complements it.

---

## Why use Sruja?

| Before Sruja | After Sruja |
|----------------|--------------|
| Diagrams drift from code | Architecture always in sync |
| Manual updates in drawing tools | AI generates from code |
| Can't validate architecture | Linting catches errors |
| Hard to see what changed | Version control shows everything |
| Scattered across tools | Single source of truth |

### Who is this for?

**Developers and Teams:**
- Keep architecture documentation accurate
- Catch architectural issues before they cause problems
- Onboard new team members faster

**Students and Learners:**
- Understand real-world architecture patterns
- See how production systems are designed
- Practice with actual codebases

**Software Architects:**
- Enforce standards across teams
- Detect drift automatically
- Scale evidence-backed architecture review

---

## Common Questions

**Do I need to learn the Sruja language?**

No. Your AI writes the `.sruja` files for you. You just need to know what to ask for, which we provide in prompts.

**What if I don't have an AI editor?**

You can still use the CLI. For evaluation, start with `sruja inspect overview -r . --generate-baseline` (structural draft), author `repo.sruja` with the skill or by hand, then `sruja lint repo.sruja`. Once reviewed truth exists, use `sruja sync -r .` and `sruja drift -r . -a repo.sruja` to keep declared architecture aligned with code. The skill makes this workflow smoother in AI editors, but the underlying commands work in CI and automation too.

**Can I use this with my existing project?**

Yes. Sruja supports many languages out of the box:

| Language | Support Level |
|----------|--------------|
| **JavaScript / TypeScript** | Excellent |
| **Python** | Excellent |
| **Go** | Excellent |
| **Rust** | Excellent (native) |
| **Java** | Good |
| **C#** | Good |
| **Ruby** | Good |
| **PHP** | Good |

Other languages may have partial support. The skill runs discovery for you; it will report what it detects in your codebase.

**How is this different from diagramming tools?**

Diagramming tools (Miro, LucidChart, Visio) are for drawing. Sruja is for defining architecture as code.

**Diagramming tools:**
- Manual updates required
- Drifts from reality
- No validation

**Sruja:**
- AI generates from code
- Always in sync
- Validates structure
- Version-controlled

You can still export diagrams from Sruja—just treat diagrams as output, not the source.

**Sruja vs Mermaid**

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

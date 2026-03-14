# Sruja (beta) – Architecture-as-Code Tool

[![TypeScript Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg?flag=typescript)](https://codecov.io/gh/sruja-ai/sruja)

**Architecture-as-code for the AI SDLC process** – define architecture in `.sruja` files; validate and export to Markdown and Mermaid diagrams.

> ⚠️ **Beta** – Sruja is under active development. APIs may change.

---

## Start here (about 2 minutes)

No config. No API keys. No `.sruja` file needed to start.

```bash
# 1. Install
curl -fsSL https://sruja.ai/install.sh | bash

# 2. Verify (if "command not found", add ~/.local/bin to PATH)
sruja --version

# 3. Run on your repo
sruja quickstart -r .
```

You get: architecture inventory, health score, top findings, and next steps.

**New to Sruja?** Run the demo first: `make demo` (or `cd evaluation/real-world-test && ./run_demo.sh`).  
For the full **Architecture Intelligence** flow (intent → scan → drift → analyze → AI ask): `make demo-intel` (or `cd demo && ./run_demo.sh`).

---

## Why Sruja?

### 🚀 **Zero-setup architecture intelligence first**

- **Quickstart** – Scan any repo; get health score, findings, and evidence (no keys)
- **Why** – Ask "why are we using Postgres?" with deterministic evidence from the graph
- **Drift** – Detect cycles, orphans, layer violations from scanned code

### 🔄 **Architecture-as-Code** (optional)

- Define architecture in `.sruja` files – version-controlled in Git
- Built-in validation – catch issues before they reach production
- Export to Markdown and Mermaid – integrate into your docs
- Works for developers and CI/CD pipelines

### 🤖 **AI-Ready Features**

- **Skills + CLI (no MCP)** – Editor integration uses **skills** (Cursor, Copilot, etc.) plus the **CLI** for deterministic output. There is no `sruja mcp` or MCP server in this repo.
- **Install as skill** – One command to add Sruja to Cursor (or other AI editors): [Install guide](docs/INSTALL_AS_SKILL.md). For architecture discovery, use the [recommended prompt](docs/INSTALL_AS_SKILL.md#recommended-prompt-architecture-discovery) (copy-paste in IDE chat).
- **Context for AI tools** – Export architecture context for Cursor, Copilot, and Claude
- **One-click baseline** – Generate `architecture.sruja` from your repo in seconds
- **PR-scoped drift** – Detect only NEW violations introduced in a PR

---

## Quick Start

### Install CLI

**Option A – install script (downloads binary from [GitHub Releases](https://github.com/sruja-ai/sruja/releases)):**

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

**Option B – from Git (requires Rust):**

```bash
cargo install sruja-cli --git https://github.com/sruja-ai/sruja
```

**Option C – build from source:**

```bash
git clone https://github.com/sruja-ai/sruja.git && cd sruja && make build
```

Then ensure the install directory is on your `PATH` (install script uses `~/.local/bin` by default; Option B uses `~/.cargo/bin`; Option C uses `target/release`).

**Verify install:** Run `sruja --version`. If you get "command not found", add the install directory to your PATH (e.g. `export PATH="$HOME/.local/bin:$PATH"`).

### Get Immediate Architecture Insights (Zero Setup)

**Scan any codebase and get architecture intelligence in seconds – no API keys, no configuration required:**

```bash
# Get immediate insights about your architecture
sruja quickstart

# Or specify a repository path
sruja quickstart -r /path/to/your/repo

# Get JSON output for programmatic use
sruja quickstart --format json
```

**What you get:**
- 📊 Architecture inventory (modules, services, databases, APIs)
- 💚 Health score with visual indicator
- 🔍 Top 3 critical findings with severity levels
- 🎯 Top 3 actionable fixes with priority and impact
- 📎 Evidence references from your code
- 🚀 Clear next steps

**Example output:**
```
══════════════════════════════════════════════════════════════════════
🚀 Sruja Quickstart - Architecture Intelligence
══════════════════════════════════════════════════════════════════════

📂 Scanning repository...
   ✓ Found 753 components

──────────────────────────────────────────────────────────────────────
📊 Architecture Inventory
──────────────────────────────────────────────────────────────────────
  Components detected:
    • 750 modules
    • 1 services
    • 2 databases
    • 1533 total dependencies

──────────────────────────────────────────────────────────────────────
💚 Architecture Health Score: 75/100
──────────────────────────────────────────────────────────────────────
  ███████████████░░░░░ ⚠ Fair
```

### Architecture analysis

**`sruja analyze`** provides structural architecture analysis and generates a CTO-level report with **health scores**, an **architecture completion score**, risks, and recommendations.

```bash
# Architecture analysis with health, completion score, and recommendations
sruja analyze -r .
sruja analyze -r . -f json
```

The JSON report from `sruja analyze -r . -f json` includes:

- `health_score` – overall structural health (0–100)
- `architecture_completion_score` – how well the current architecture model covers production concerns (0–100)
- `completion_breakdown` – per-dimension coverage:
  - `structural` – structural modeling and graph quality
  - `operational` – signals for operating and debugging in production (deployment, reliability, CI/CD)
  - `security` – basic attack-surface and vulnerability indicators

**Note:** Semantic, intent, and runtime analysis layers are in experimental preview. The current analyze command focuses on structural analysis.

**Command tiers:**

| Tier | Commands | When to use |
|------|----------|-------------|
| **First value** | `sruja quickstart -r .`, `sruja drift -r .` | Any repo, zero config. Start here. |
| **Deeper analysis** | `sruja analyze -r .`, `sruja complexity -r .` | Full picture: structural + intent. |
| **With a baseline** | `sruja drift -r . -a architecture.sruja`, `sruja lint`, `sruja export` | When you have (or create) a `.sruja` file. |
| **Optional** | `sruja intent check`, `sruja runtime analyze` | Trace files or intent dir. |

**Drift modes:** `sruja drift -r .` runs **scan-only** (no `.sruja` needed). Use `sruja drift -r . -a architecture.sruja` to **compare code to a declared baseline**; create one with the core Sruja skill (see `docs/INSTALL_AS_SKILL.md`) or manually.

**Optional environment variables** (defaults when flags are omitted):

- `SRUJA_INTENT_PATH` – Path to intent directory (ADRs, .sruja) for `sruja analyze` and `sruja intent check`. If unset, `sruja analyze` uses `repo/docs/architecture`.
- `SRUJA_TRACES_PATH` – Path to traces JSON file for `sruja analyze -t`. Only used when `-t` is not passed.
- No LLM API keys required for core CLI. For natural-language interpretation, use the Sruja skill in your editor (Cursor, Copilot, etc.).

**Add drift to CI** – Gate on architecture health. Exits with code 1 when cycles, orphans, or layer violations (Error severity) are found:

```yaml
# .github/workflows/drift.yml (GitHub Actions)
name: Architecture drift
on: [push, pull_request]
jobs:
  drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash
      - name: Check PATH
        run: echo "$HOME/.local/bin" >> $GITHUB_PATH && echo "$HOME/.local/bin" >> $GITHUB_ENV
      - name: Run drift check
        run: sruja drift -r .
        # Fails the job if structural violations (cycles, orphans, layer violations) exist
```

### AI-Ready Architecture

**Export context for AI coding assistants:**

```bash
# For Cursor IDE
sruja context -r . -f cursor-rules -o .cursorrules

# For GitHub Copilot
sruja context -r . -f copilot-instructions -o .github/copilot-instructions.md

# JSON format for custom tools
sruja context -r . -f json -o architecture-context.json
```

**Generate a baseline from your repo:**

```bash
# One-click baseline generation
sruja quickstart -r . --generate-baseline

# Then compare code vs baseline
sruja drift -r . -a architecture.sruja
```

### PR-Scoped Drift (Detect Only NEW Violations)

```bash
# Compare current branch against main
sruja drift-pr -r . --base origin/main

# GitHub Actions format for CI
sruja drift-pr -r . --base origin/main -f github-actions

# JSON output
sruja drift-pr -r . --base origin/main -f json
```

See `docs/RUN_GUIDE.md` for how to run the CLI and demos.

### AI-Assisted Discovery (Optional)

Use your AI assistant (Cursor, Claude, Copilot) to discover architecture from your codebase:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

Then ask: "Analyze the architecture of my repository." The AI will scan your code and generate `.sruja` files. See `docs/INSTALL_AS_SKILL.md` for the recommended prompt.

### Define Architecture Manually (Optional)

If you want to explicitly define your architecture in code, create `example.sruja`:

```sruja
person = kind "Person"
system = kind "System"
container = kind "Container"

user = person "User" {
  description "End user of the application"
}

app = system "My App" {
  web = container "Web Server" {
    technology "Node.js"
  }
}

user -> app.web "visits"
```

**Export & validate:**

```bash
sruja lint example.sruja
sruja export json example.sruja
sruja export markdown example.sruja
```

---

## Features

### 🎨 Code-First Architecture Tool

- **DSL**: Simple architecture definition language
- **Export/Import**: `.sruja` files work with Git

### ✅ Built-in Validation

- Cycle detection
- Orphan detection
- Unique ID enforcement
- Valid reference checking

### 📊 Multiple Outputs

- **JSON**: Full model with metadata
- **DSL**: Text format for Git
- **Markdown**: Documentation generation
- **Mermaid**: Diagram export

### 🔍 Developer Tools

- **Code Formatter**: `sruja fmt`
- **Tree View**: `sruja tree`
- **CLI**: Full command-line interface

### 🏗️ Architecture Intelligence (Beta)

- **CLI first, no key required:** `sruja quickstart`, `sruja why "question" -r .`, `sruja drift -r .` — deterministic evidence from scan and graph
- **Query:** "Why are we using X?" uses graph + scan evidence (deterministic); use the Sruja skill in your editor for AI interpretation

**Strategy (internal):** [docs/internal/architecture-lab/AI_FIRST_MODULE_ANALYSIS_FINAL.md](docs/internal/architecture-lab/AI_FIRST_MODULE_ANALYSIS_FINAL.md)

---

## Documentation

**Website**: https://sruja.ai

**Essential Guides:**

- [Install Sruja in your AI editor](docs/INSTALL_AS_SKILL.md) - One-page skill install for Cursor, Copilot, etc.
- [Contributing Guide](docs/CONTRIBUTING.md) - How to contribute
- [First Contribution](docs/FIRST_CONTRIBUTION.md) - Step-by-step guide
- [Language Specification](docs/LANGUAGE_SPECIFICATION.md) - Complete DSL reference
- [Design Philosophy](docs/DESIGN_PHILOSOPHY.md) - Language design principles
- [How to run Sruja](docs/RUN_GUIDE.md) - Clone, build, CLI, demos, extension

---

## Project Structure

```
sruja/
├── crates/               # Rust crates
│   ├── sruja-cli/        # CLI (lint, export, scan, quickstart, why, drift, analyze, context)
│   ├── sruja-language/   # Parser and AST
│   ├── sruja-engine/     # Validation rules
│   ├── sruja-export/     # Markdown, Mermaid, JSON export
│   ├── sruja-graph/      # Knowledge graph, centrality, coupling
│   ├── sruja-scan/       # Repo scanning (multi-language tree-sitter)
│   ├── sruja-diff/       # Drift detection (code vs. intent)
│   ├── sruja-intent/     # Intent vs. reality comparison
│   └── sruja-report/     # Report schema for analysis output
├── book/                 # mdBook documentation
└── book/valid-examples/  # Canonical example .sruja files (rendered in the book)
```

---

## Development

### Prerequisites

- **Rust >= 1.70**
- **Node.js >= 18**

### Setup

```bash
# Install dependencies
cargo fetch
npm install

# Setup git hooks (recommended)
make setup-hooks

# Build CLI
make build
```

### Testing

```bash
make test          # Run all tests
make test-rust     # Rust tests only
make test-coverage # With coverage report
```

---

## Contributing

We welcome contributions of all sizes!

### 🎯 New Contributors

**Start here**: [First Contribution Guide](docs/FIRST_CONTRIBUTION.md)

### Quick Links

- 🐛 [Good First Issues](https://github.com/sruja-ai/sruja/labels/good%20first%20issue)
- 📖 [Full Contribution Guide](docs/CONTRIBUTING.md)
- 💬 [Discord](https://discord.gg/VNrvHPV5) | [Discussions](https://github.com/sruja-ai/sruja/discussions)

### Ways to Contribute

**No code required:**

- Fix typos in docs
- Add examples
- Report bugs
- Write tutorials

**Code contributions:**

- Fix bugs
- Add features
- Improve tests
- Enhance tooling

### Pull Request Checklist

- ✅ Run `make test`, `make fmt`, `make lint`
- ✅ Add/update tests
- ✅ Keep changes focused
- ✅ Use Conventional Commits (`feat:`, `fix:`, `docs:`)

---

## License

Apache 2.0

---

## Links

- **Website**: https://sruja.ai
- **Discord**: https://discord.gg/VNrvHPV5
- **GitHub**: https://github.com/sruja-ai/sruja

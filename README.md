# Sruja (beta) – Architecture-as-Code Tool

[![TypeScript Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg?flag=typescript)](https://codecov.io/gh/sruja-ai/sruja)

**Architecture-as-code for the AI SDLC process** – define architecture in `.sruja` files; validate and export to Markdown and Mermaid diagrams.

> ⚠️ **Beta** – Sruja is under active development. APIs may change.

**No API key or config needed for first value** – run `sruja quickstart -r .`, `sruja why "..." -r .`, or `sruja drift -r .` on any repo.

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

### Architecture intelligence (four layers)

The main entrypoint for full architecture intelligence is **`sruja analyze`**. It runs structural, semantic, intent, and optional runtime analysis and outputs an overall health score and recommendations.

```bash
# Full analysis (structural + semantic + intent; optional runtime with -t)
sruja analyze -r .
sruja analyze -r . -t traces.json -i docs/architecture -f json
```

**Drill-down commands** (no API key required):

| Command | Purpose |
|--------|--------|
| `sruja quickstart -r .` | Fast health snapshot and top findings |
| `sruja drift -r .` | Structural drift (cycles, orphans, layers); use `-a path/to/arch.sruja` to compare against a baseline |
| `sruja complexity -r .` | Treewidth, SCC, centrality, coupling |
| `sruja semantic -r .` | Semantic coupling, bounded contexts, vocabulary leakage |
| `sruja intent check -r .` | Compare declared intent (ADRs, .sruja) vs scanned code |
| `sruja runtime analyze -t traces.json` | Runtime traces, emergent cycles, hotspots |

**Optional environment variables** (defaults when flags are omitted):

- `SRUJA_INTENT_PATH` – Path to intent directory (ADRs, .sruja) for `sruja analyze` and `sruja intent check`. If unset, `sruja analyze` uses `repo/docs/architecture`.
- `SRUJA_TRACES_PATH` – Path to traces JSON file for `sruja analyze -t`. Only used when `-t` is not passed.

See [Architecture Intelligence](docs/ARCHITECTURE_INTELLIGENCE.md) for details.

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

- **LSP Support**: VS Code extension with autocomplete
- **Code Formatter**: `sruja fmt`
- **Tree View**: `sruja tree`
- **CLI**: Full command-line interface

### 🏗️ Architecture Intelligence (Beta)

- **CLI first, no key required:** `sruja quickstart`, `sruja why "question" -r .`, `sruja drift -r .` — deterministic evidence from scan and graph
- **sruja-app** (optional): Desktop app for chat, agents, extraction — requires LLM key
- **Query:** "Why are we using X?" uses graph + scan evidence only; optional LLM enrichment when configured

**Strategy:** [architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md](architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md)

---

## Documentation

**Website**: https://sruja.ai

**Essential Guides:**

- [Contributing Guide](docs/CONTRIBUTING.md) - How to contribute
- [First Contribution](docs/FIRST_CONTRIBUTION.md) - Step-by-step guide
- [Language Specification](docs/LANGUAGE_SPECIFICATION.md) - Complete DSL reference
- [Design Philosophy](docs/DESIGN_PHILOSOPHY.md) - Language design principles
- [Architecture Intelligence](docs/ARCHITECTURE_INTELLIGENCE.md) - CLI-first drift/why, zero-key; [Strategy](architecture/AI_FIRST_MODULE_ANALYSIS_FINAL.md)

**Content Creation:**

- [Content Contribution](docs/CONTENT_CONTRIBUTION_GUIDE.md) - Creating courses & tutorials
- [Content Style Guide](docs/CONTENT_STYLE_GUIDE.md) - Writing best practices

---

## Project Structure

```
sruja/
├── crates/               # Rust crates
│   ├── sruja-cli/        # CLI (lint, export, scan, why, drift)
│   ├── sruja-language/   # Parser and AST
│   ├── sruja-engine/     # Validation rules
│   ├── sruja-export/     # Markdown, Mermaid, JSON export
│   ├── sruja-lsp/        # Language Server Protocol
│   ├── sruja-wasm/       # WebAssembly bindings
│   ├── sruja-app/        # Desktop app (Slack-style architecture collaboration)
│   ├── sruja-chat/       # Chat, agents, extraction (architecture intelligence)
│   ├── sruja-graph/      # Knowledge graph for decisions
│   ├── sruja-extract/    # LLM extraction (decisions, requirements)
│   ├── sruja-scan/       # Repo scanning (npm, cargo)
│   └── sruja-mcp/        # MCP server for AI tooling
├── extension/            # VS Code extension (syntax highlighting, LSP)
├── book/                 # mdBook documentation
└── examples/             # Example .sruja files
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

- 💡 [Contribution Ideas](docs/CONTRIBUTION_IDEAS.md)
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

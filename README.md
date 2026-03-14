# Sruja – Architecture-as-Code with AI Skills

[![TypeScript Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg?flag=typescript)](https://codecov.io/gh/sruja-ai/sruja)

**Architecture-as-code powered by AI skills** – define architecture in `.sruja` files; validate with deterministic CLI tools; use AI skills for discovery and modeling.

> ⚠️ **Beta** – Sruja is under active development. APIs may change.

---

## Quick Start (3 minutes)

### 1. Install the CLI

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

### 2. Install the core skill

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

### 3. Get architecture intelligence

Run instant structural analysis:

```bash
sruja quickstart -r .
```

Generate architecture with AI:

In your AI editor (Cursor, Copilot, Claude), run:

```
Use sruja-architecture. Run `sruja discover --context -r . --format json`, gather evidence, ask targeted questions if needed, generate architecture.sruja, then run `sruja lint` and fix until it passes.
```

Validate the generated architecture:

```bash
sruja lint architecture.sruja
```

---

## How Sruja Works

### Skill-First Architecture Discovery

**The core product is the AI skill**, not the CLI narrative reports. The workflow:

1. **Install skill** – Add `sruja-architecture` to your AI editor
2. **Collect evidence** – Run `sruja discover --context -r . --format json` for deterministic evidence
3. **Ask questions** – AI asks targeted questions only when scope is unclear
4. **Generate DSL** – AI produces minimal `architecture.sruja` based on evidence
5. **Validate** – Run `sruja lint` to catch errors
6. **Refine** – Use `sruja drift -r . -a architecture.sruja` to detect changes

### Deterministic CLI Primitives

The CLI provides deterministic, machine-readable outputs that skills depend on:

```bash
# Collect evidence (primary machine-facing contract)
sruja discover --context -r . --format json

# Validate DSL
sruja lint --format json architecture.sruja

# Format DSL
sruja fmt architecture.sruja

# Export for documentation
sruja export markdown architecture.sruja

# Detect drift
sruja drift --format json -r . -a architecture.sruja

# Check intent
sruja intent check --format json

# Export context
sruja context -r .
```

**No guessing, no heuristics** – The CLI returns what it actually finds in your code.

### Architecture-as-Code

Define your architecture in version-controlled `.sruja` files:

```sruja
import { * } from 'sruja.ai/stdlib'

user = person "User" {
  description "End user"
}

app = system "My App" {
  api = container "API" {
    technology "Node.js"
    description "REST API"
  }

  db = database "Database" {
    technology "PostgreSQL"
    description "Data storage"
  }
}

user -> app.api "HTTPS"
app.api -> app.db "SQL"
```

**Benefits:**
- Version-controlled architecture documentation
- Catch issues before production with `sruja lint`
- Detect drift with `sruja drift`
- Export to Markdown and Mermaid diagrams
- Works with Git, CI/CD, and AI tools

---

## Supported Languages

Sruja uses Tree-sitter parsers for precise code analysis. Language support varies between full AST parsing and minimal line-based extraction.

| Language | Parser Type | Notes |
|----------|-------------|-------|
| **TypeScript / JavaScript** | Full Tree-sitter | Best support for web frameworks and Node.js applications |
| **Python** | Full Tree-sitter | Strong support for Django, Flask, FastAPI applications |
| **Go** | Full Tree-sitter | Excellent for microservices and cloud-native applications |
| **Rust** | Full Tree-sitter | Native language; comprehensive support for all Rust patterns |
| **Java** | Full Tree-sitter | Good for enterprise applications and Spring Boot |
| **C#** | Full Tree-sitter | Support for .NET applications |
| **Ruby** | Full Tree-sitter | Rails and Ruby applications |
| **PHP** | Full Tree-sitter | PHP web applications |
| **Scala** | Full Tree-sitter | Scala applications |
| **C / C++** | Full Tree-sitter | Systems programming |
| **Kotlin** | Line-based extraction | Limited support due to Tree-sitter version compatibility; extracts imports and classes only |

**Best Results:** JavaScript/TypeScript, Go, Python, and Rust repos provide the most accurate architectural insights with minimal false positives.

**Known Limitations:**
- Dynamic imports and reflection-based patterns may be missed
- Entry points and test utilities may be flagged as orphans
- Kotlin support is minimal (line-based) compared to other languages
- Language-specific frameworks may require additional context

See [Known Limitations](docs/KNOWN_LIMITATIONS.md) for scanner and analysis caveats.

---

## Stable CLI Commands

These commands are the stable product surface:

| Command | Purpose | Format |
|---------|---------|--------|
| `lint` | Validate DSL | `--format json` |
| `fmt` | Format DSL | default |
| `export` | Export to formats | default |
| `discover --context` | Collect evidence | `--format json` |
| `drift` | Detect drift | `--format json` |
| `intent check` | Check intent | `--format json` |
| `context` | Export context | default |

### Quickstart

**Structural output only** – no framework/domain/security narratives unless backed by evidence:

```bash
sruja quickstart -r .
```

Returns:
- Architecture inventory
- Health score
- Top findings
- Evidence references
- Scan scope

### Example Workflow

```bash
# 1. Get instant structural analysis
sruja quickstart -r .

# 2. Use AI to generate architecture (in your editor)
# Paste this prompt:
# "Use sruja-architecture. Run `sruja discover --context -r . --format json`, gather evidence, ask targeted questions if needed, generate architecture.sruja, then run `sruja lint` and fix until it passes."

# 3. Validate the generated architecture
sruja lint architecture.sruja

# 4. Export documentation
sruja export markdown architecture.sruja > ARCHITECTURE.md

# 5. Add to CI
sruja drift -r . -a architecture.sruja --fail-on all
```

---

## Installation

### CLI

**Option A – install script (downloads binary from GitHub Releases):**

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

Verify install: `sruja --version`

### Skill

Install the core architecture skill:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

**For Cursor, Copilot, Claude, Continue.dev, or any AI editor with skills.sh support.**

---

## Documentation

**Primary Getting Started Guides:**

- [Install Sruja as a Skill](docs/INSTALL_AS_SKILL.md) – Skill installation and discovery workflow
- [Getting Started with Skill](docs/GETTING_STARTED_SKILL.md) – Core workflow with examples
- [Skill Reference](skills/sruja-architecture/SKILL.md) – Core skill orchestration guide
- [Skill Workflow Reference](skills/sruja-architecture/REFERENCE.md) – Detailed discovery and modeling guide
- [Prompt Patterns](skills/sruja-architecture/PROMPTS.md) – Reusable AI prompts

**CLI Engine Documentation:**

- [Language Specification](docs/LANGUAGE_SPECIFICATION.md) – DSL syntax and features
- [Run Guide](docs/RUN_GUIDE.md) – CLI commands and demos
- [Design Philosophy](docs/DESIGN_PHILOSOPHY.md) – Language design principles
- [Known Limitations](docs/KNOWN_LIMITATIONS.md) – Scanner limitations, false positives, and scope constraints
- [Support](docs/SUPPORT.md) – Community support options and enterprise considerations

---

## CI/CD Integration

### Lint and Drift Check

```yaml
# .github/workflows/architecture.yml
name: Architecture validation
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash
      - name: Lint
        run: find . -name '*.sruja' -exec sruja lint {} \;
      - name: Drift check
        run: sruja drift -r . -a architecture.sruja --fail-on all
```

### PR-Scoped Drift

```bash
# Detect only NEW violations in a PR
sruja drift-pr -r . --base origin/main -f github-actions
```

---

## Project Structure

```
sruja/
├── skills/sruja-architecture/  # Core AI skill
│   ├── SKILL.md               # Orchestration guide
│   ├── REFERENCE.md           # Discovery & modeling reference
│   ├── PROMPTS.md            # Prompt patterns
│   ├── AGENTS.md             # Compiled guide
│   └── agents/openai.yaml    # Skill/agent metadata (prompts, no UI)
├── crates/sruja-cli/         # CLI (engine primitives)
├── crates/sruja-language/    # Parser and AST
├── crates/sruja-engine/      # Validation rules
├── crates/sruja-scan/        # Repo scanning
└── crates/sruja-diff/        # Drift detection
```

---

## Development

```bash
# Setup
cargo fetch
npm install

# Build CLI
make build

# Run tests
make test
```

### VS Code Extension

The VS Code extension is **supported and actively maintained**. Core features (syntax highlighting, diagnostics, snippets, export, diagram preview) are stable. See [extension/README.md](extension/README.md) for installation and usage details.

To build and install the extension:
```bash
make build-extension
make install-extension
```

---

## Contributing

We welcome contributions!

- [Contributing Guide](docs/CONTRIBUTING.md)
- [Good First Issues](https://github.com/sruja-ai/sruja/labels/good%20first%20issue)

---

## License

Apache 2.0

---

## Links

- **Website**: https://sruja.ai
- **GitHub**: https://github.com/sruja-ai/sruja

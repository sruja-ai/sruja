# Sruja – Architecture intelligence for AI Era

[![TypeScript Coverage](https://codecov.io/gh/sruja-ai/sruja/branch/main/graph/badge.svg?flag=typescript)](https://codecov.io/gh/sruja-ai/sruja)

**Architecture intelligence for the AI era.** Use AI to generate and maintain architecture as code—so it stays in sync with your codebase.

---

## What is this?

**Problem:** Your code changes, but your architecture diagrams don't. They're in Miro, LucidChart, or old PDFs—drifting from reality.

**Solution:** Sruja uses AI to analyze your codebase and generate architecture as code (`.sruja` files). You can validate, version-control, and export it—keeping it always up-to-date.

**How it works:**
1. Run a command to analyze your code
2. Tell your AI editor to generate architecture
3. Validate it automatically
4. Export diagrams and docs when you need them

**You don't write `.sruja` files manually.** Your AI does it for you.

---

## Quick Start (3 minutes)

### Step 1: Install CLI

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

This installs the `sruja` command. You can check it worked:

```bash
sruja --version
```

### Step 2: Install the AI skill

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

This teaches your AI editor (Cursor, Copilot, Claude, etc.) how to generate Sruja architecture.

**Supported editors:** Cursor, GitHub Copilot, Claude, Continue.dev, and any editor with [skills.sh](https://skills.sh) support.

### Step 3: Get instant insights

Jump into your project folder and run:

```bash
cd your-project
sruja quickstart -r .
```

This shows you:
- What's in your codebase (services, databases, APIs)
- A health score
- Top issues to fix

**No AI required for this step**—it's just code analysis.

### Step 4: Generate architecture with AI

In your AI editor, paste this prompt:

```
Use sruja-architecture. Run `sruja discover --context -r . --format json`,
gather evidence from my code, ask targeted questions if needed,
generate repo.sruja, then run `sruja lint` and fix until it passes.
```

Your AI will:
1. Run the discovery command to understand your code
2. Ask you a few questions if anything is unclear
3. Generate a `repo.sruja` file
4. Fix any validation errors

### Step 5: Validate and export

```bash
# Check for errors
sruja lint repo.sruja

# Export a diagram (for docs, presentations, etc.)
sruja export mermaid repo.sruja > diagram.mmd
```

You can open `diagram.mmd` in [Mermaid Live Editor](https://mermaid.live) or use the VS Code extension for preview.

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
- Scale architecture governance

---

## Common Questions

**Do I need to learn the Sruja language?**

No. Your AI writes the `.sruja` files for you. You just need to know what to ask for, which we provide in prompts.

**What if I don't have an AI editor?**

You can still use Sruja! The CLI works standalone:
- `sruja quickstart` – Get architecture insights
- `sruja discover` – Export code structure
- `sruja lint` – Validate `.sruja` files
- `sruja export` – Generate diagrams and docs

However, an AI editor makes it much easier to generate and update architecture files.

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

Other languages may have partial support. Run `sruja quickstart -r .` to see what gets detected.

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

**What if the AI makes a mistake?**

Run `sruja lint repo.sruja` to catch errors. Tell your AI: "Fix these lint errors" and paste the output.

---

## What can I do?

### Get instant insights

```bash
sruja quickstart -r .
```

Shows architecture inventory, health score, and top findings—no `.sruja` file required.

### Generate and maintain architecture

```bash
# Let AI discover and generate
sruja discover --context -r . --format json

# Validate
sruja lint repo.sruja

# Detect changes over time
sruja drift -r .
```

### Export for documentation

```bash
# Markdown (readable docs)
sruja export markdown repo.sruja > ARCHITECTURE.md

# Mermaid (diagrams)
sruja export mermaid repo.sruja > ARCHITECTURE.mmd

# JSON (machine-readable)
sruja export json repo.sruja > ARCHITECTURE.json
```

### Use in CI/CD

```yaml
# .github/workflows/architecture.yml
name: Validate Architecture
on: [push, pull_request]

jobs:
  validate:
    steps:
      - uses: actions/checkout@v4
      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash
      - name: Lint
        run: sruja lint repo.sruja
```

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
make build
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

- [Multi-Repo Federation](docs/FEDERATION.md) – Managing multiple repos
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
- Try: `sruja quickstart -r .` to see what's being detected
- Open an issue if something obvious is missing

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

- **Website:** https://sruja.ai
- **GitHub:** https://github.com/sruja-ai/sruja
- **Discussions:** https://github.com/sruja-ai/sruja/discussions

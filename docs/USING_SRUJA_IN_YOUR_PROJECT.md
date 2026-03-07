# Using Sruja in Your Project

This guide is for **teams and organizations** that want to use Sruja in their own repositories to enhance their code: architecture-as-code, validation in CI, and AI-assisted generation with consistent rules.

## What you get

- **Architecture as code** – `.sruja` files in Git; no separate diagram tool to keep in sync.
- **Validation** – `sruja lint` catches undefined refs, circular dependencies, missing fields, orphans.
- **AI-friendly** – Rules and skills so Cursor, Copilot, etc. generate valid Sruja and better architecture.
- **CI** – Fail PRs when architecture is invalid; optional export to Markdown/JSON/Mermaid for docs.

## 1. Install (your machine and/or CI)

### CLI

**Option A – install script (downloads from [GitHub Releases](https://github.com/sruja-ai/sruja/releases)):**

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

Ensure the install directory is on your `PATH` (install script uses `~/.local/bin` by default; Option B uses `~/.cargo/bin`; Option C uses `target/release`).

**Check:**

```bash
sruja --help
sruja quickstart --help
```

### VS Code extension

Install **Sruja Language Support** from the [VS Code Marketplace](https://marketplace.visualstudio.com/) (or [Open VSX](https://open-vsx.org/)). You get syntax highlighting, LSP diagnostics, and optional diagram preview for `.sruja` files.

---

## 0. Try Architecture Intelligence (no .sruja required)

Get architecture insights in seconds—no API keys, no `.sruja` files, no configuration:

```bash
sruja quickstart -r .
```

You get: architecture inventory, health score, top findings, actionable fixes, and evidence references. From there:

- `sruja why "why did we choose PostgreSQL?" -r .` — Ask questions with deterministic evidence
- `sruja drift -r .` — Detect drift (circular deps, orphans, layer violations)

See [ARCHITECTURE_INTELLIGENCE.md](ARCHITECTURE_INTELLIGENCE.md) for details.

---

## 2. Add Sruja to your repo (5 minutes)

If you want to define architecture explicitly, add `.sruja` files:

### Step 1: Create or add architecture

Create `architecture.sruja` (or `docs/architecture.sruja`) and define your systems/containers/relationships. See [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md) and [examples](https://github.com/sruja-ai/sruja/tree/main/examples).

### Step 2: AI editor integration (so AI-generated code follows rules)

Copy into your project root:

- **`.cursorrules`** – Cursor uses this for Sruja DSL rules (see repo root or [skills/sruja-architecture](../skills/sruja-architecture/)).
- **`.copilot-instructions.md`** – GitHub Copilot uses this.
- **`.architecture-skill.md`** – Short pointer; optional full skill: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`.

Commit these so everyone (and CI) has the same setup. See [AI_EDITOR_INTEGRATION.md](AI_EDITOR_INTEGRATION.md) for details.

### Step 3: Validate in CI

In **your** repo you don’t have the Sruja monorepo, so install the CLI in CI from Git, then run lint.

**GitHub Actions example:**

```yaml
name: Validate Sruja

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
  paths:
    - '**/*.sruja'

jobs:
  sruja:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Sruja CLI
        run: cargo install sruja-cli --git https://github.com/sruja-ai/sruja --locked

      - name: Lint all .sruja files
        run: |
          find . -name '*.sruja' -not -path './target/*' | while read f; do
            echo "Linting $f"
            sruja lint "$f"
          done
```

Use `--locked` so the install matches the lockfile in the Sruja repo for reproducible CI.

**Optional – architecture drift in CI:**

```yaml
      - name: Architecture drift check
        run: sruja quickstart -r . -f json > sruja-report.json || true
      - name: Upload drift report
        uses: actions/upload-artifact@v4
        with:
          name: sruja-drift-report
          path: sruja-report.json
```

**Optional – export docs in CI:**

```yaml
      - name: Export architecture to Markdown
        run: |
          for f in $(find . -name '*.sruja' -not -path './target/*'); do
            out="${f%.sruja}.md"
            sruja export markdown "$f" > "$out" || true
          done
      - name: Upload architecture docs
        uses: actions/upload-artifact@v4
        with:
          name: architecture-docs
          path: '**/*.sruja.md'
```

---

## 3. How this enhances your code

| Practice | How Sruja helps |
|----------|------------------|
| **PR reviews** | CI fails if `.sruja` is invalid; reviewers see architecture changes in the diff. |
| **Onboarding** | New devs read `.sruja` and exported docs instead of hunting for “the” diagram. |
| **AI-generated code** | `.cursorrules` and Copilot instructions steer AI to valid DSL; `sruja lint` catches mistakes. |
| **Compliance / governance** | Policies and constraints in the DSL; lint enforces structure; export for auditors. |
| **Multi-repo** | Each repo can have its own `architecture.sruja` (or one per service); same CLI and CI pattern. |

---

## 4. Using Sruja across multiple repos

- **Per-repo** – Each repository that owns a service or app can have its own `.sruja` file(s). Add the same CI job (install CLI from Git + `sruja lint`) and the same AI files (e.g. copy `.cursorrules` and `.copilot-instructions.md` from a template or run `sruja init` once and commit).
- **Central docs repo** – Some teams keep a single “docs” or “architecture” repo with one or more `.sruja` files and run Sruja CI there; link to exported Markdown/JSON from other repos. Other repos don’t need the CLI unless they also own architecture files.
- **Shared rules** – Use the same [sruja-architecture skill](https://github.com/sruja-ai/sruja/tree/main/skills/sruja-architecture) (`npx skills add sruja-ai/sruja --skill sruja-architecture`) across repos so AI and humans share the same patterns and trade-offs.

---

## 5. Where to go next

- **Architecture Intelligence** – [ARCHITECTURE_INTELLIGENCE.md](ARCHITECTURE_INTELLIGENCE.md) – quickstart, why, drift (no .sruja required)
- **DSL reference** – [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md)
- **AI editors and catching bugs** – [AI_EDITOR_INTEGRATION.md](AI_EDITOR_INTEGRATION.md)
- **Reviewing AI-generated code** – [REVIEWING_AI_GENERATED_CODE.md](REVIEWING_AI_GENERATED_CODE.md)
- **Adoption and rollout** – [Adoption Guide](https://sruja.ai/docs/adoption-guide) and [Adoption Playbook](https://sruja.ai/docs/adoption-playbook) on the website
- **Examples** – [examples/](https://github.com/sruja-ai/sruja/tree/main/examples) in the Sruja repo

Sruja is open source. To report issues or suggest improvements, use [GitHub Issues](https://github.com/sruja-ai/sruja/issues) or [Discussions](https://github.com/sruja-ai/sruja/discussions).

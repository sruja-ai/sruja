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
sruja lint --help
```

### VS Code extension

Install **Sruja Language Support** from the [VS Code Marketplace](https://marketplace.visualstudio.com/) (or [Open VSX](https://open-vsx.org/)). You get syntax highlighting, LSP diagnostics, and optional diagram preview for `.sruja` files.

---

## 2. Add Sruja to your repo (5 minutes)

### Step 1: Create or add architecture

```bash
# From your repo root (recommended pilot path)
sruja quickstart -r . --generate-baseline   # repo.sruja.draft (structural evidence)
# Author repo.sruja with the sruja-architecture skill, then:
sruja lint repo.sruja
sruja sync -r .
```

`--generate-baseline` writes a workspace map draft—not reviewed architecture. Promote to `repo.sruja` via the skill, lint, then refresh evidence under `.sruja/` for drift workflows.

### Step 2: AI editor integration (so AI-generated code follows rules)

Install the skill in your AI editor:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

Optional: if you want helper files for Cursor/Copilot prompts, generate them once and commit:

```bash
sruja start -r . --prompt
```

See [AI editor integration](https://github.com/sruja-ai/sruja/blob/main/docs/AI_EDITOR_INTEGRATION.md) in the repo for details.

### Step 3: Validate in CI

In CI, keep it simple: lint the baseline and check for drift against `repo.sruja`.

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

      - name: Install Sruja CLI
        run: curl -fsSL https://sruja.ai/install.sh | bash

      - name: Lint baseline
        run: sruja lint repo.sruja

      - name: Drift check (declared vs actual)
        run: sruja drift -r . -a repo.sruja
```

If you prefer PR annotations, use the `sruja-check` action (see the main README).

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
| **Onboarding** | New devs read `.sruja` and exported docs instead of hunting for "the" diagram. |
| **AI-generated code** | The skill and editor integrations steer AI to valid DSL; `sruja lint` catches mistakes. |
| **Compliance / governance** | Policies and constraints in the DSL; lint enforces structure; export for auditors. |
| **Multi-repo** | Each repo can have its own `architecture.sruja` (or one per service); same CLI and CI pattern. |

---

## 4. Using Sruja across multiple repos

- **Per-repo** – Each repository that owns a service or app can have its own `.sruja` file(s). Add the same CI job and the same AI setup (e.g. copy `.cursorrules` and `.copilot-instructions.md` from a template, install the skill, or run `sruja start -r . --prompt` once and commit).
- **Central docs repo** – Some teams keep a single "docs" or "architecture" repo with one or more `.sruja` files and run Sruja CI there; link to exported Markdown/JSON from other repos. Other repos don't need the CLI unless they also own architecture files.
- **Shared rules** – Use the same [sruja-architecture skill](https://github.com/sruja-ai/sruja/tree/main/skills/sruja-architecture) (`npx skills add sruja-ai/sruja --skill sruja-architecture`) across repos so AI and humans share the same patterns and trade-offs.

---

## 5. Where to go next

- **Language specification** – [Language specification](../reference/language-spec.md) (full DSL reference in this book)
- **AI editors and catching bugs** – [AI editor integration](https://github.com/sruja-ai/sruja/blob/main/docs/AI_EDITOR_INTEGRATION.md) in the repo
- **Reviewing AI-generated code** – [Reviewing AI-generated code](https://github.com/sruja-ai/sruja/blob/main/docs/REVIEWING_AI_GENERATED_CODE.md) in the repo
- **Adoption and rollout** – [Adoption guide](adoption-guide.md) and [Adoption playbook](adoption-playbook.md) in this book
- **Examples** – [Examples Gallery](../examples/index.md) (canonical book-backed examples)

Sruja is open source. To report issues or suggest improvements, use [GitHub Issues](https://github.com/sruja-ai/sruja/issues) or [Discussions](https://github.com/sruja-ai/sruja/discussions).

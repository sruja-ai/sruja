# Using Sruja in Your Project

This guide is for **teams and organizations** that want AI assistants and humans to work from the same repo-grounded architecture context: skill-led generation, architecture-as-code, validation in CI, and reusable evidence.

## What you get

- **AI skill first** – Cursor, Copilot, Claude, etc. can generate and maintain `repo.sruja` from repo evidence.
- **Architecture as code** – `.sruja` files in Git; diagrams and docs are exported from reviewed truth.
- **Validation** – `sruja lint` catches undefined refs, circular dependencies, missing fields, orphans.
- **Fresh context** – `sruja sync`, `focus`, `ai`, and MCP keep agents grounded before coding.
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
git clone https://github.com/sruja-ai/sruja.git && cd sruja && just build
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

## 0. Start with the AI Skill

Install the primary Sruja interface first. The skill guides the AI editor to gather evidence, generate or update `repo.sruja`, and validate the result.

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

Then ask your AI editor:

```text
Use sruja-architecture. Gather evidence from this repo, generate or update repo.sruja,
then run sruja lint and fix until it passes.
```

For deterministic CLI evaluation without an AI editor, run:

```bash
sruja quickstart -r .
```

You get: architecture inventory, health score, top findings, actionable fixes, and evidence references. From there:

- `sruja sync -r .` — Refresh evidence for the skill, MCP, drift, and review workflows
- `sruja why "why did we choose PostgreSQL?" -r .` — Ask questions with deterministic evidence
- `sruja status -r .` — Check repo health and truth status

See [RUN_GUIDE.md](RUN_GUIDE.md) for running the CLI and demos.

---

## 2. Add Sruja to your repo (5 minutes)

Use the canonical pilot path to create a single reviewed baseline and keep evidence fresh:

### Step 1: Create or add architecture

Ask the skill to generate or update `repo.sruja`. If you are evaluating from the CLI only:

```bash
sruja quickstart -r . --generate-baseline   # repo.sruja.draft (structural evidence)
# Author reviewed intent in repo.sruja (sruja-architecture skill), then:
sruja lint repo.sruja
sruja sync -r .
```

`--generate-baseline` writes a capped workspace map draft, not reviewed architecture. Promote to `repo.sruja` via the skill (or manual edit), lint, then refresh evidence under `.sruja/` for drift workflows.

### Step 2: AI editor integration (so AI-generated code follows evidence)

Use the skill in your AI editor for ongoing updates. See **[GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md)**.

Optional: generate helper files (Cursor/Copilot prompts) once and commit:

```bash
sruja start -r . --prompt
```

Commit these so everyone (and CI) has the same setup. See **[GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md)** for editor setup.

### Step 3: Validate in CI

In CI, lint the baseline and check drift against `repo.sruja`.

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

**Optional – architecture drift in CI:**

```yaml
      - name: Architecture drift check
        run: sruja drift -r . -a repo.sruja -f json > sruja-drift-report.json || true
      - name: Upload drift report
        uses: actions/upload-artifact@v4
        with:
          name: sruja-drift-report
          path: sruja-drift-report.json
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
| **AI-assisted work** | The skill and editor integrations feed AI current repo evidence; `sruja lint` catches mistakes. |
| **Policy guardrails** | Policies and constraints in the DSL; lint enforces structure; export for auditors when needed. |
| **Multi-repo** | Each repo can have its own `repo.sruja` (or one per service; `architecture.sruja` is also supported); same CLI and CI pattern. |

---

## 4. Using Sruja across multiple repos

- **Per-repo** – Each repository that owns a service or app can have its own `.sruja` file(s). Add the same CI job and the same AI files (e.g. copy `.cursorrules` and `.copilot-instructions.md` from a template or run `sruja start -r . --prompt` once and commit).
- **Central docs repo** – Some teams keep a single “docs” or “architecture” repo with one or more `.sruja` files and run Sruja CI there; link to exported Markdown/JSON from other repos. Other repos don’t need the CLI unless they also own architecture files.
- **Shared rules** – Use the same [sruja-architecture skill](https://github.com/sruja-ai/sruja/tree/main/skills/sruja-architecture) (`npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture`) across repos so AI and humans share the same patterns and trade-offs.

---

## 5. Where to go next

- **DSL reference** – [LANGUAGE_SPECIFICATION.md](LANGUAGE_SPECIFICATION.md)
- **AI editors and catching bugs** – [GETTING_STARTED_SKILL.md](GETTING_STARTED_SKILL.md)
- **Adoption and rollout** – [Adoption Guide](../book/src/docs/adoption-guide.md) and [Adoption Playbook](../book/src/docs/adoption-playbook.md)
- **Examples** – The canonical examples live in `book/valid-examples/` and are rendered in the mdBook “Examples Gallery” at `sruja.ai`.

Sruja is open source. To report issues or suggest improvements, use [GitHub Issues](https://github.com/sruja-ai/sruja/issues) or [Discussions](https://github.com/sruja-ai/sruja/discussions).

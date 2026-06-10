# Core And Extensions In Your Project

This guide is for teams that want to adopt Sruja without letting advanced features take over the product story. Start with the core loop, then add extensions only when they clearly help.

## Start with the core

These are the defaults to adopt first:

- **Capture** – ingest docs and record decisions you want future humans and AI to reuse.
- **Retrieve** – `focus`, `ai`, and MCP before editing.
- **Verify** – `drift`, `intent check`, and `verify-task` after editing.
- **Optional reviewed intent** – `repo.sruja` only when you want durable, reviewable intent in Git.

## Then add extensions deliberately

Only add these after the core loop is working:

- **Reviewed intent workflows** – richer `repo.sruja` authoring and stricter CI gates
- **Visualization** – Markdown, Mermaid, and other exports
- **Team review flows** – critique, PR drift checks, policy-oriented enforcement
- **Federation and multi-repo** – cross-repo packaging and retrieval
- **Advanced agent ops** – plans, learnings, and memory workflows

---

## What you get from the core path

- **Small default workflow** – easier to explain and adopt
- **Repo-grounded context** – no need to start with modeling everything
- **Deterministic verification** – changes get checked after editing
- **Safer expansion** – extensions stay available without cluttering onboarding
- **MCP integration** – tool-based editor and agent workflows when needed

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
git clone https://github.com/sruja-ai/sruja.git && cd sruja && just build  # or: make build
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

### Step 1: Start with the core loop

```bash
# From your repo root
sruja start -r .
sruja drift -r . --structural-only --advisory
```

This gives you immediate value without committing any new architecture files.

### Step 2: Retrieve context before editing

If you use an AI editor/agent, use Sruja as the guardrail:

```bash
sruja focus -r . --file path/to/file.rs
sruja ai -r . --task "Refactor auth boundary"
sruja verify-task --profile coding -r .
```

To enable tool-based integration, start MCP:

```bash
sruja mcp -r .
```

See [Host agent integration](https://github.com/sruja-ai/sruja/blob/main/docs/HOST_AGENT_INTEGRATION.md) for editor setup.

### Step 3 (optional): Add reviewed intent in Git

When the team is ready to review architecture intent like code, generate `repo.sruja` using the architecture skill:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

Then validate and compare intent to reality:

```bash
sruja lint repo.sruja
sruja sync -r .
sruja drift -r . -a repo.sruja
```

Optional: generate prompt/helper files for Cursor/Copilot once and commit:

```bash
sruja start -r . --prompt
```

See [AI editor integration](https://github.com/sruja-ai/sruja/blob/main/docs/AI_EDITOR_INTEGRATION.md) for details.

### Step 4: Validate in CI

In CI, keep it simple: lint reviewed intent and check drift. For incremental adoption, use a baseline so existing issues don’t block you.

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

## 3. Where extensions fit

Once the core path works, add extensions based on a real need:

| Need | Add |
|------|-----|
| Reviewable intent in Git | `repo.sruja`, `lint`, `sync`, `drift -a repo.sruja` |
| Better documentation outputs | `export` commands |
| Stricter review flows | CI drift checks, critique, policy-oriented docs |
| Cross-repo understanding | federation docs and tooling |
| Advanced agent workflows | plans, learnings, memory, and other advanced surfaces |

---

## 4. How this enhances your code

| Practice | How Sruja helps |
|----------|------------------|
| **PR reviews** | CI fails if `.sruja` is invalid; reviewers see architecture changes in the diff. |
| **Onboarding** | New devs read `.sruja` and exported docs instead of hunting for "the" diagram. |
| **AI-generated code** | The skill and editor integrations steer AI to valid DSL; `sruja lint` catches mistakes. |
| **Compliance / governance** | Policies and constraints in the DSL; lint enforces structure; export for auditors. |
| **Multi-repo** | Each repo can have its own `architecture.sruja` (or one per service); same CLI and CI pattern. |

---

## 5. Using Sruja across multiple repos

- **Per-repo** – Each repository that owns a service or app can have its own `.sruja` file(s). Add the same CI job and the same AI setup (e.g. copy `.cursorrules` and `.copilot-instructions.md` from a template, install the skill, or run `sruja start -r . --prompt` once and commit).
- **Central docs repo** – Some teams keep a single "docs" or "architecture" repo with one or more `.sruja` files and run Sruja CI there; link to exported Markdown/JSON from other repos. Other repos don't need the CLI unless they also own architecture files.
- **Shared rules** – Use the same [sruja-architecture skill](https://github.com/sruja-ai/sruja/tree/main/skills/sruja-architecture) (`npx skills add sruja-ai/sruja --skill sruja-architecture`) across repos so AI and humans share the same patterns and trade-offs.

---

## 6. Where to go next

- **Core workflow** – [Getting started](getting-started.md)
- **Language specification** – [Language specification](../reference/language-spec.md) (full DSL reference in this book)
- **AI editors and catching bugs** – [AI editor integration](https://github.com/sruja-ai/sruja/blob/main/docs/AI_EDITOR_INTEGRATION.md) in the repo
- **Reviewing AI-generated code** – [Reviewing AI-generated code](https://github.com/sruja-ai/sruja/blob/main/docs/REVIEWING_AI_GENERATED_CODE.md) in the repo
- **Adoption and rollout** – [Adoption guide](adoption-guide.md) and [Adoption playbook](adoption-playbook.md) in this book
- **Examples** – [Examples Gallery](../examples/index.md) (canonical book-backed examples)

Sruja is open source. To report issues or suggest improvements, use [GitHub Issues](https://github.com/sruja-ai/sruja/issues) or [Discussions](https://github.com/sruja-ai/sruja/discussions).

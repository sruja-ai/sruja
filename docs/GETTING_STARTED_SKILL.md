# Getting Started with Sruja Skills

**AI coding harness + optional architecture authoring.**

Sruja is not a second coding agent. Install the **harness** skill so any agent runs `verify-task` before done; add **sruja-architecture** when you want reviewed `repo.sruja` in Git.

See [INSTALL_AS_SKILL.md](INSTALL_AS_SKILL.md) and [COMMUNITY_SKILLS_STACK.md](COMMUNITY_SKILLS_STACK.md).

---

## What You'll Need

1. **Sruja CLI** – Scan, drift, focus, verify-task
2. **AI editor** – Cursor, Copilot, Claude, etc. (owns the LLM loop)
3. **Skills** – `sruja-harness` (required for gates); `sruja-architecture` (optional)

---

## Tier 1 workflow (harness)

```
focus / drift  →  host agent edits code  →  verify-task  →  (optional) agent record on failure
```

No `repo.sruja` required for structural scan and verify gates.

---

## Tier 1b workflow (architecture skill)

```
You → Tell AI to analyze your code
  ↓
AI → Runs discover / sync evidence
  ↓
AI → Generates repo.sruja
  ↓
AI → lint + drift against repo.sruja
```

---

## Quick Start (Copy These Steps)

### Step 1: Install CLI + harness skill

```bash
curl -fsSL https://sruja.ai/install.sh | bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness
```

### Step 2: Scan and wire MCP

```bash
sruja start -r .
sruja drift -r . --structural-only --advisory
```

Register MCP in Cursor (see [.cursor/mcp.json](../.cursor/mcp.json)) or extension **Register MCP Server**.

### Step 3: Agent loop

```bash
sruja focus -r . --file path/to/file.rs
# … host agent edits …
sruja verify-task --profile coding -r .
```

### Step 4 (optional): Architecture skill

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

### Step 5 (optional): Generate architecture

In your AI editor, run:

```
Use sruja-architecture. Gather evidence (it prefers .sruja/context.json when present;
when missing, it runs discover for you—no need to run a command first),
ask targeted questions if needed,
generate repo.sruja (architecture.sruja is also supported),
then run `sruja lint` and fix.
```

### Step 6: Validate reviewed truth

```bash
sruja lint repo.sruja
```

### Step 7: Export (optional)

```bash
# For documentation
sruja export markdown repo.sruja > ARCHITECTURE.md

# For diagrams
sruja export mermaid repo.sruja > ARCHITECTURE.mmd
```

---

## Understanding the Commands

### sruja quickstart

**What it does:** Analyzes your code and shows you a summary.

**Why use it:** Get instant insights without generating any files.

**Output:**
- What services, databases, APIs are in your code
- A health score (0-100)
- Top issues to fix

**When to use it:**
- First time seeing a codebase
- Checking if Sruja understands your project
- Getting an architecture health check

### sruja discover

**What it does:** Detailed analysis in JSON format.

**Why use it:** The AI needs this to generate architecture.

**Output:** Code structure, technologies, dependencies, boundaries

**When to use it:**
- When generating `repo.sruja` files (via AI)
- When checking what evidence is available

### sruja lint

**What it does:** Validates a `repo.sruja` file.

**Why use it:** Catches errors before they become problems.

**Output:** Errors or "All checks passed"

**Common errors:**
- `E204: Circular dependency` – Service A depends on B, B depends on A
- `E205: Orphan element` – Something with no connections
- `E201: Invalid kind` – Unknown element type

**When to use it:**
- After AI generates a file
- After you make changes
- In CI/CD pipelines

### sruja drift

**What it does:** Compares your code to your `repo.sruja` file.

**Why use it:** Keep architecture in sync as code changes.

**Output:** What's new, missing, or different

**When to use it:**
- After making code changes
- Before a release
- Regular maintenance checks

### sruja impact

**What it does:** Computes blast radius from the scanned code dependency graph.

**Why use it:** Estimate change risk before refactors by seeing:
- Upstream dependents (what relies on your target)
- Downstream dependencies (what your target relies on)
- Centrality metrics (useful signal for “critical” nodes)

**Examples:**

```bash
sruja impact <target> -r . --depth 3
sruja impact <target> -r . --depth 3 -f json
```

---

## Example Session

**You:** "I want to document my Node.js API project."

**AI:** "Sure! Running `sruja discover --context -r . --format json`..."

*(analyzes code)*

**AI:** "I found:
- Express API server
- PostgreSQL database
- Redis cache
- 3 modules (auth, orders, users)

A few questions:
1. What's the external system that calls this API? (You: 'Mobile app')
2. Do you have any message queues? (You: 'No')

Generating `repo.sruja`..."

*(creates file)*

**AI:** "Running `sruja lint repo.sruja`..."

*(validates)*

**AI:** "✅ All checks passed! Here's your architecture:
[shows diagram]

I also exported a Markdown document you can share with your team."

---

## Common Patterns

### "Add a component to my architecture"

```
Use sruja-architecture. Read repo.sruja and add a [Payment Service]
container to handle Stripe integration. Connect it to the existing API container.
Then run sruja lint and fix any errors.
```

### "My code changed—update architecture"

```
Use sruja-architecture. Run `sruja drift -r . --format json`,
analyze what changed, and update repo.sruja to match the current code.
```

### "Explain this architecture"

```
Read repo.sruja and explain:
1. What systems are defined?
2. How do they connect?
3. What technologies are used?
```

---

## Tips for Success

**Be specific:** Instead of "Improve architecture," try "Add error handling to the API container."

**Validate often:** Run `sruja lint` after each AI edit—catch mistakes early.

**Start simple:** Get context + container levels working first, add components later if needed.

**Ask questions:** If you don't understand something, ask the AI "Why did you model it this way?"

**Trust the evidence:** If `sruja discover` doesn't find something, tell your AI—don't let it guess.

---

## Troubleshooting

| Problem | Solution |
|----------|----------|
| `sruja: command not found` | Add to PATH: `export PATH="$HOME/.local/bin:$PATH"` |
| Skill not loading | Restart your editor after installing |
| AI generates invalid code | Run `sruja lint repo.sruja` and paste errors to AI |
| Discovery misses components | Check language support at `sruja.ai` |

---

## Using Sruja in your project (single repo, monorepo, multi-repo)

Same skill-first workflow for every setup. Pick the one that matches you.

| Setup | What it means | What you do |
|-------|----------------|-------------|
| **Single repo** | One repository, one codebase, one architecture boundary. | One `repo.sruja`, one CI job. Default flow below. |
| **Monorepo** | One repository with multiple packages, apps, or services. | One `repo.sruja` for the whole repo (typical), or one per area if you want separate boundaries. Same CI as single repo. |
| **Multi-repo** | Many repositories (e.g. one repo per service or app). | Each repo has its own `repo.sruja` and CI. Optional: [federation](FEDERATION.md) to build a system-wide index. |

---

### Single repo

One codebase, one architecture. This is the default.

1. **Install the skill** (see [Quick Start](#quick-start-copy-these-steps) above). Use your AI to generate `repo.sruja` at the repo root.
2. **Commit** `repo.sruja` and your skill setup (e.g. `.cursorrules` or `npx skills add ...`) so the team shares the same rules. See [Install as a Skill](INSTALL_AS_SKILL.md).
3. **Add CI** to lint `.sruja` on every PR. Example (GitHub Actions):

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

**Optional:** `sruja drift -r . -a repo.sruja -f json` for machine-readable drift reports; `sruja export markdown` / `sruja export mermaid` for docs.

---

### Monorepo

One repo, many packages or apps (e.g. `packages/api`, `packages/web`, `apps/mobile`). Sruja treats it as one repo: discovery scans the whole tree from the root.

- **Typical:** One `repo.sruja` at the repo root that describes all systems, containers, and boundaries. Same workflow as single repo: skill, commit, CI.
- **Optional:** One `.sruja` per area (e.g. `packages/api/api.sruja`) if you want separate architecture files per package. CI: lint all `*.sruja` (same `find` as above).
- **Discovery:** Run from repo root: `sruja discover -r .` (or let the skill run it). The CLI scans the entire repo; you can scope later in the DSL by system/container.

No extra tooling. Same skill and CLI as single repo.

---

### Multi-repo

Many repos (e.g. one repo per microservice or app). Each repo is independent.

1. **In each repo:** Same as single repo — install the skill, generate and commit `repo.sruja`, add the same CI job. Use the same [sruja-architecture skill](https://github.com/sruja-ai/sruja/tree/main/skills/sruja-architecture) everywhere so AI and humans share rules.
2. **Optional – system-wide view:** To compose architecture across repos (one graph, canonical IDs, conflict reporting), use **federation**: each repo runs `sruja publish -r . -o repo.bundle.json`; a central job or script runs `sruja compose -i <bundles-dir> -o system.index.json`. See [FEDERATION_SETUP_GUIDE.md](FEDERATION_SETUP_GUIDE.md) for step-by-step setup or [FEDERATION.md](FEDERATION.md) for technical reference.

**Patterns:** Per-repo ownership (each repo owns its `.sruja`); or a central “docs” / “architecture” repo that holds `.sruja` files and Sruja CI while other repos use the skill locally.

---

### How this enhances your code

| Practice | How Sruja helps |
|----------|------------------|
| **AI-generated architecture** | Skill uses real code evidence; lint and drift keep output valid and in sync. |
| **Onboarding** | New devs and AI assistants read `.sruja` plus exported docs from the same reviewed truth. |
| **PR reviews** | CI fails if `.sruja` is invalid; reviewers see architecture changes in the diff. |
| **Policy guardrails** | Policies in the DSL; lint enforces structure; export for auditors when needed. |
| **Multi-repo** | Each repo has its own `repo.sruja` and CI; optional federation for system-wide view. |

---

## What's Next?

- **Deep dive:** [Skill Reference](../skills/sruja-architecture/SKILL.md)
- **Prompt patterns:** [Prompt Library](../skills/sruja-architecture/PROMPTS.md)
- **Complete guide:** [Skill Workflow Reference](../skills/sruja-architecture/REFERENCE.md)
- **Adoption:** [Adoption Guide](../book/src/docs/adoption-guide.md) (evaluate fit, plan rollout)

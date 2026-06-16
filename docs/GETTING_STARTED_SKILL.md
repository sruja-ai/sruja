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

---

## Recommended Skill Stack

| Order | Skill | Purpose |
|-------|-------|---------|
| 1 | `sruja-harness` | Run `verify-task` before marking any task done |
| 2 | `sruja-architecture` | Optional: promote scan evidence to reviewed `repo.sruja` |
| 3 | Community skills | Your coding/debug/review skill (Addy, skills.sh, etc.) |

```bash
# Harness first (works without repo.sruja)
npx skills add https://github.com/sruja-ai/sruja --skill sruja-harness

# Optional: reviewed architecture in Git
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

Install the **CLI** when skills need it: `curl -fsSL https://sruja.ai/install.sh | bash`. Register MCP in Cursor via [.cursor/mcp.json](../.cursor/mcp.json) or **Sruja: Register MCP Server**.

See [COMMUNITY_SKILLS_STACK.md](COMMUNITY_SKILLS_STACK.md) and [HOST_AGENT_INTEGRATION.md](HOST_AGENT_INTEGRATION.md).

---

## What Architecture Skill Does

| Without Skill | With Skill |
|---------------|-------------|
| You write `.sruja` by hand | AI generates it from code |
| You must learn the language | You just know what to ask for |
| Easy to make syntax errors | Validation catches mistakes |
| Manual updates | AI keeps it in sync |

---

## Why Install the Skill?

**Faster generation:** AI writes `.sruja` files in seconds vs manual work.

**Fewer errors:** AI knows the syntax and best practices.

**Better patterns:** The skill includes architecture patterns and trade-offs.

**Continuous updates:** As code changes, AI can update architecture automatically.

---

## Harness Loop (Any Skill)

```text
1. sruja focus -r . --file <path>     # before edit
2. Host agent edits code
3. sruja verify-task --profile coding -r .
4. sruja agent record …               # optional, on failure
```

Do **not** use `sruja agent run` as the primary loop — the host owns Act; Sruja owns Verify.

---

## AI-Assisted Development Playbook

This section turns common "AI assisted development" advice into an **enforceable, repeatable workflow** using Sruja's deterministic harness: architecture evidence, explicit boundaries, and local verification gates.

### Goal

Enable fast iteration with AI coding assistants **without** accumulating silent structural debt (layer violations, circular dependencies, "god modules", diagram drift).

**In scope**
- Bounded code generation and refactors inside known architectural constraints
- Local "verify-task" loop before commit / PR
- Shared skills + editor rules for consistent outcomes across a team
- Architecture diagrams as exported artifacts (not hand-maintained drawings)

**Out of scope**
- Autonomous "AI engineer" workflows (Sruja is a harness; the editor/host owns the agent loop)
- Replacing code review (the harness reduces risk; humans still review intent and product correctness)

### Daily workflow (recommended)

#### 1) Put a harness on the assistant (grounded context)

**What you want:** The assistant should *pull* bounded, machine-readable evidence instead of you pasting architecture rules into chat.

- **MCP setup**: follow `docs/mcp_setup.md`
- **Tool profile**: keep `SRUJA_MCP_TOOL_PROFILE=coding` for day-to-day tasks
- **Read-only mode (recommended)**: `SRUJA_MCP_READONLY=1` so an assistant can't mutate proposals or write scratchpads unintentionally

**Editor rules as a stable "floor"**

Run this whenever architecture or dependency rules change:

```bash
sruja sync-ide-rules -r .
```

This keeps files like `.cursorrules`, `CLAUDE.md`, `.gemini/AGENTS.md`, and `llms-architecture.txt` aligned with the repo's current architecture context.

#### 2) Shift validation left (verify locally)

Treat this as the "adult supervision" loop: generate → verify → iterate → only then commit.

```bash
# Features / refactors
sruja verify-task --profile coding -r .

# Bug fixes (tight scope; include a target file)
sruja verify-task --profile bugfix --file <path> -r .

# Pre-merge hardening
sruja verify-task --profile review -r .
```

The goal is to catch architecture drift, broken boundaries, and intent mismatches **before** a reviewer sees the diff.

#### 3) Prefer architecture-as-code over "prompted rules"

Instead of repeating "don't import X from Y" in every conversation:

- Maintain a reviewed baseline: `repo.sruja`
- Use deterministic enforcement:
  - `sruja lint repo.sruja`
  - `sruja drift -r . -a repo.sruja`

This makes structural constraints **versioned and reviewable**, and lets tools enforce them consistently across humans and assistants.

#### 4) Standardize skills across the team

Use skills to make the assistant behave consistently across developers.

Recommended baseline for teams:
- A "task prime" skill (how to use `sruja focus`, the MCP ladder, and how to keep diffs small)
- A "verify-task before done" skill (always end with `sruja verify-task`)
- A "no drive-by refactors" skill (explicitly defer incidental cleanup)

### CI envelope (minimal)

Add a PR gate that runs the same checks you run locally:
- `sruja verify-task` in CI

Examples/templates:
- `.github/workflows/sruja-verify-task.yml`
- `templates/github-actions/sruja-verify-task-pr.yml`
- `docs/examples/host-gates/verify-task-pr.yml`

### Practical guardrails (what breaks first)

- **Large diffs**: AI is most dangerous when it changes too much at once. Keep PRs small; validate after each slice.
- **Boundary erosion**: the harness can block forbidden dependencies, but it won't invent missing architectural intent—write/maintain `repo.sruja` as reviewed truth.
- **"Looks right" bugs**: structural checks don't prove product behavior; continue to require tests + review.

### Multi-agent verification (high-stakes)

Single-agent outputs are best treated as **untrusted drafts**: they can be articulate, confident, and still wrong. For higher-stakes domains (security, compliance, finance, medical-ish, production safety), upgrade your workflow from "one agent + verify-task" to **multi-agent + go/no-go**.

**Roles (split incentives):**
- **Draft agent (generator)**: produce a fast first-pass answer/change, plus explicit claims.
- **Verifier agent (fact-checker)**: confirm claims against evidence (code, docs, tests, `sruja` outputs). It should try to falsify, not "polish".
- **Adversary agent (red team)**: look for edge cases, unsafe assumptions, boundary violations, and ways the change could mislead users or regress behavior.

**Go / No-go protocol (earned confidence):**
1. **Evidence**: every critical claim must be backed by a source of truth.
2. **Verifier "GO"**: `sruja verify-task` passes and claims are cross-checked.
3. **Adversary "GO"**: explicit list of failure modes considered.
4. **Any single "NO-GO" pauses the ship**: gather more evidence or escalate to human reviewer.

---

## Building from Source

### Prerequisites

- **Rust ≥ 1.70** – [rustup.rs](https://rustup.rs/)
- **Node.js ≥ 18** – Only needed for the VS Code extension and (optionally) npm-based tooling

Verify:

```bash
rustc --version   # e.g. 1.70+
node --version    # optional; only for extension
```

### Step 1: Clone and enter the repo

```bash
git clone https://github.com/sruja-ai/sruja.git
cd sruja
```

### Step 2: Install dependencies and build

```bash
# Fetch Rust dependencies
just install
# or: cargo fetch

# Build release binary (CLI)
just build
# or: cargo build --release
```

The CLI binary is at **`target/release/sruja`**.

### Step 3: Put the CLI on your PATH (optional but recommended)

**Option A – Use the built binary directly**

```bash
./target/release/sruja --version
```

**Option B – Install into Cargo's bin (so `sruja` works anywhere)**

```bash
cargo install --path crates/sruja-cli
# Then: sruja --version
# Binary is in ~/.cargo/bin (ensure that's on your PATH)
```

**Option C – Symlink**

```bash
sudo ln -sf "$(pwd)/target/release/sruja" /usr/local/bin/sruja
# or: ln -sf "$(pwd)/target/release/sruja" ~/.local/bin/sruja
```

### Step 4: First value (no config, no API keys)

Run context engineering on the repo itself:

```bash
# If you used Option B or C:
sruja start -r .
sruja drift -r . --structural-only --advisory

# Or with the built binary:
./target/release/sruja start -r .
./target/release/sruja drift -r . --structural-only --advisory
```

You should see structural findings with file-level evidence. No `.sruja` file or API keys required.

Other useful commands:

```bash
sruja focus -r . --file crates/sruja-cli/src/main.rs
sruja ai -r . --task "Fix auth bug"
sruja verify-task --profile coding -r .
sruja mcp -r .
sruja ingest docs/adr/ --category adr
```

Optional reviewed intent in Git:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
sruja lint repo.sruja
sruja sync -r .
sruja drift -r . -a repo.sruja
```

---

## Running the E2E Demo (Optional, ~2 min)

The demo clones Express.js (if needed), runs quickstart + drift, and optionally baseline/LLM eval.

```bash
cd evaluation/real-world-test
./run_demo.sh
```

- **No flags** – Fast path only (quickstart + drift). No config.
- **`--baseline`** – Also compare to an example architecture.
- **`--llm`** – Add LLM evaluation (requires an API key in `.env`; see Step 7).
- **`--all`** – Baseline + LLM.

If the script says "sruja CLI not found", ensure `sruja` is on PATH or build from repo root first (`just build`) and add `target/release` to PATH.

### Optional: Context Engineering microservices demo (~2 min)

This demo walks through the full flow: **intent (rulebook) → scan → drift → analyze → AI ask**, using the small Python microservices in `demo/`.

```bash
just demo-intel
# or: cd demo && ./run_demo.sh
```

- **No API key** – Steps 1–4 run; step 5 (AI ask) is skipped with a hint, and `sruja why` is run as a deterministic fallback when possible.
- **With API key** – Set `OPENROUTER_API_KEY` or `OPENAI_API_KEY` in repo root `.env` to enable the full AI ask step.

See `demo/README.md` for details.

---

## LLM / API Keys (Optional)

Only needed for:

- `sruja eval <path>`
- `./run_demo.sh --llm` or `./evaluate_architecture.sh <repo> --llm`

**Quick setup for evaluation/demo:**

```bash
cd evaluation/real-world-test
cp .env.example .env
# Edit .env and set one key, e.g.:
#   OPENROUTER_API_KEY=sk-or-v1-...
#   OPENAI_API_KEY=sk-...
#   ANTHROPIC_API_KEY=sk-ant-...
#   GEMINI_API_KEY=...
# Or local: SRUJA_LLM_PROVIDER=ollama
```

Then:

```bash
./run_demo.sh --llm
```

---

## VS Code Extension (Optional)

For syntax highlighting and LSP (e.g. validation, autocomplete) for `.sruja` files:

```bash
cd extension
npm install
npm run compile
```

Then in VS Code/Cursor: **Run → Start Debugging** (F5) with "Extension" launch config to open a new window with the extension loaded.  
Or build a VSIX and install it: `npm run package` then install the generated `.vsix` from the Extensions view.

---

## Running Tests

```bash
just test
# or: cargo test --workspace
```

Other targets:

```bash
just fmt          # Format Rust code
just lint         # Clippy
cargo test -p sruja-cli --test why_e2e   # Why command E2E (optional)
```

**Coverage (optional):** Host Rust coverage excludes the WASM crate (it is tested with `wasm-pack`). Run:

```bash
just test-coverage        # llvm-cov for workspace (excludes sruja-wasm)
just test-coverage-wasm   # wasm-bindgen tests for sruja-wasm
```

See [WASM_TESTING.md](WASM_TESTING.md) for rationale and CI alignment.

---

## Book (mdBook Docs, Optional)

```bash
just book-deps    # Install mdbook, mdbook-mermaid (one-time)
just wasm         # Needed for in-book WASM diagrams
just book         # Build book
just book-serve   # Serve at http://localhost:3000 (live reload)
```

---

## Summary: Minimal Path to "Running" Sruja

| Step | Command | Purpose |
|------|---------|--------|
| 1 | `git clone ... && cd sruja` | Get repo |
| 2 | `just install && just build` | Dependencies + CLI binary |
| 3 | `./target/release/sruja --version` | Verify CLI |
| 4 | `./target/release/sruja quickstart -r .` | First value (no config) |
| 5a | `cd evaluation/real-world-test && ./run_demo.sh` | Optional: E2E demo (quickstart + drift) |
| 5b | `just demo-intel` | Optional: Context Engineering demo (intent → scan → drift → analyze → AI) |

**Troubleshooting**

- **"sruja: command not found"** – Use full path `./target/release/sruja` or add it to PATH (Step 3).
- **"Cargo not found"** – Install Rust: https://rustup.rs/
- **Demo fails** – Ensure CLI is built and on PATH; for `--llm`, set one LLM key in `evaluation/real-world-test/.env`.

For contribution workflow (hooks, lint, test), see [CONTRIBUTING.md](CONTRIBUTING.md) and [DEVELOPMENT.md](DEVELOPMENT.md).

---

## Installation Options

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

## How Sruja Enhances Your Code

| Practice | How Sruja helps |
|----------|------------------|
| **PR reviews** | CI fails if `.sruja` is invalid; reviewers see architecture changes in the diff. |
| **Onboarding** | New devs read `.sruja` and exported docs instead of hunting for "the" diagram. |
| **AI-assisted work** | The skill and editor integrations feed AI current repo evidence; `sruja lint` catches mistakes. |
| **Policy guardrails** | Policies and constraints in the DSL; lint enforces structure; export for auditors when needed. |
| **Multi-repo** | Each repo can have its own `repo.sruja` (or one per service; `architecture.sruja` is also supported); same CLI and CI pattern. |

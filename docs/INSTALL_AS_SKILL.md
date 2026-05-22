# Install Sruja as a Skill

**Harness + optional architecture authoring for AI editors.**

Sruja is a **deterministic harness** for coding agents (drift, focus, `verify-task`) — not a second IDE agent. Skills teach your editor when to run Sruja gates; the CLI and MCP do the work.

## Recommended stack

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

## sruja-architecture (optional Tier 1b)

**Teach your AI editor to generate architecture—no DSL learning required.**

The `sruja-architecture` skill analyzes your code and generates `repo.sruja` files. Your AI handles the syntax, you just guide the process.

---

## What architecture skill does

| Without Skill | With Skill |
|---------------|-------------|
| You write `.sruja` by hand | AI generates it from code |
| You must learn the language | You just know what to ask for |
| Easy to make syntax errors | Validation catches mistakes |
| Manual updates | AI keeps it in sync |

---

## Installation (architecture skill only)

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

For agent gates without `repo.sruja`, install `sruja-harness` first (see **Recommended stack** above).

**Supported editors:** Cursor, GitHub Copilot, Claude, Continue.dev, and any editor with [skills.sh](https://skills.sh) support.

---

## Verify It Works

1. Restart your AI editor (if it's open)
2. Open your project folder
3. In the AI chat, type: "Use sruja-architecture"

If the skill loaded, your AI should acknowledge it and be ready to generate architecture.

---

## Your First Use

Now try it! In your project folder:

```bash
cd your-project
```

In your AI editor:

```
Use sruja-architecture. Run `sruja discover --context -r . --format json`,
gather evidence from my code, ask targeted questions if needed,
generate repo.sruja, then run `sruja lint` and fix until it passes.
```

**What happens:**

1. AI runs `sruja discover` to analyze your code
2. AI asks you 2-3 questions if anything is unclear
3. AI generates `repo.sruja` with your architecture
4. AI validates it with `sruja lint`
5. You review and make changes if needed

**Result:** A `repo.sruja` file in your project root!

---

## What the Skill Needs

When you use the skill, it runs Sruja CLI commands. If the CLI isn’t installed yet, the skill (or your AI) will guide you to install it (`curl -fsSL https://sruja.ai/install.sh | bash`). Install the **skill** first; install the **CLI** when the skill needs it.

The skill relies on these commands:

### Collect evidence

```bash
sruja discover --context -r . --format json
```

**What this does:** Analyzes your code and returns a detailed JSON.

**When it's used:** Every time you generate or update architecture.

### Validate

```bash
sruja lint repo.sruja
```

**What this does:** Checks your `repo.sruja` file for errors.

**When it's used:** After the AI generates or edits a file.

### Detect drift

```bash
sruja drift -r . -a repo.sruja
```

**What this does:** Compares your code to your `repo.sruja` file.

**When it's used:** When you've changed code and want to update architecture.

### Impact analysis (optional)

```bash
sruja impact <target> -r . --depth 3
```

**What this does:** Computes blast radius from the scanned code dependency graph (upstream dependents + downstream dependencies), including centrality metrics when available.

**When it's used:** Before refactors to estimate change risk and identify critical nodes.

---

## Common Workflows

### New project: Generate from scratch

```
Use sruja-architecture. Analyze this codebase with
`sruja discover --context -r . --format json`,
identify the main systems, containers, and their relationships,
generate repo.sruja using C4 context and container levels,
then run `sruja lint` and fix until it passes.
```

### Update existing: Keep in sync

```
Use sruja-architecture. I've made code changes.
Run `sruja drift -r . --format json`,
analyze what changed, and update repo.sruja to match.
```

### Add feature: Extend architecture

```
Use sruja-architecture. Read repo.sruja and add a
[Payment Gateway] container for processing Stripe webhooks.
Connect it to the existing API service and database.
Run `sruja lint` and fix any errors.
```

---

## Troubleshooting

| Issue | Solution |
|--------|----------|
| **"Skill not found"** | Make sure you ran the install command and restarted your editor |
| **"sruja: command not found"** | Install CLI: `curl -fsSL https://sruja.ai/install.sh | bash` |
| **AI makes syntax errors** | Run `sruja lint repo.sruja` and paste errors to AI |
| **Editor doesn't support skills** | Use Cursor or install [skills.sh](https://skills.sh) to add support |

---

## Why Install the Skill?

**Faster generation:** AI writes `.sruja` files in seconds vs manual work.

**Fewer errors:** AI knows the syntax and best practices.

**Better patterns:** The skill includes architecture patterns and trade-offs.

**Continuous updates:** As code changes, AI can update architecture automatically.

---

## Harness loop (any skill)

```text
1. sruja focus -r . --file <path>     # before edit
2. Host agent edits code
3. sruja verify-task --profile coding -r .
4. sruja agent record …               # optional, on failure
```

Do **not** use `sruja agent run` as the primary loop — the host owns Act; Sruja owns Verify.

## Next Steps

- **Harness skill:** [skills/sruja-harness/SKILL.md](../skills/sruja-harness/SKILL.md)
- **Community stack:** [COMMUNITY_SKILLS_STACK.md](COMMUNITY_SKILLS_STACK.md)
- **Complete guide:** [Getting Started](GETTING_STARTED_SKILL.md) – Full walkthrough
- **Skill reference:** [Skill Documentation](../skills/sruja-architecture/SKILL.md) – What the architecture skill knows
- **Prompt examples:** [Prompt Library](../skills/sruja-architecture/PROMPTS.md) – Reusable prompts

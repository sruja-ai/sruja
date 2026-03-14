# Install Sruja as a Skill

**Teach your AI editor to generate architecture—no DSL learning required.**

The `sruja-architecture` skill analyzes your code and generates `repo.sruja` files. Your AI handles the syntax, you just guide the process.

---

## What This Does

| Without Skill | With Skill |
|---------------|-------------|
| You write `.sruja` by hand | AI generates it from code |
| You must learn the language | You just know what to ask for |
| Easy to make syntax errors | Validation catches mistakes |
| Manual updates | AI keeps it in sync |

---

## Installation (One Command)

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

That's it! Your AI editor now knows how to generate Sruja architecture.

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

The skill relies on these commands from the CLI:

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

## Next Steps

- **Complete guide:** [Getting Started](GETTING_STARTED_SKILL.md) – Full walkthrough
- **Skill reference:** [Skill Documentation](../skills/sruja-architecture/SKILL.md) – What the skill knows
- **Prompt examples:** [Prompt Library](../skills/sruja-architecture/PROMPTS.md) – Reusable prompts

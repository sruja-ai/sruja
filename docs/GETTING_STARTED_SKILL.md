# Getting Started with Sruja Skills

**Architecture intelligence for the AI era—no DSL learning required.**

Your AI analyzes your code, generates architecture files, and keeps them in sync. You just need to know what to ask for.

---

## What You'll Need

1. **Sruja CLI** – Analyzes code and validates files
2. **AI editor with skill** – Generates architecture (Cursor, Copilot, Claude, etc.)
3. **A codebase** – Any project you want to document

---

## The Workflow (Plain English)

Here's what happens, step by step:

```
You → Tell AI to analyze your code
  ↓
AI → Runs sruja commands to understand your project
  ↓
AI → Asks you 2-3 questions if anything is unclear
  ↓
AI → Generates a repo.sruja file
  ↓
AI → Validates it automatically
  ↓
You → Review the result (make changes if needed)
  ↓
CLI → Keeps it in sync as your code changes
```

**You don't write syntax. You guide the process.**

---

## Quick Start (Copy These Steps)

### Step 1: Install the skill

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

### Step 2: Install the CLI when the skill needs it

When you use the skill, it runs `sruja discover`, `sruja lint`, and `sruja drift`. If the CLI isn’t installed, the skill will guide you. You can also install now:

```bash
curl -fsSL https://sruja.ai/install.sh | bash
```

### Step 3: Generate architecture

In your AI editor, run:

```
Use sruja-architecture. Gather evidence (it prefers .sruja/context.json when present;
when missing, it runs discover for you—no need to run a command first),
ask targeted questions if needed,
generate repo.sruja (architecture.sruja is also supported),
then run `sruja lint` and fix.
```

### Step 4: Validate

```bash
sruja lint repo.sruja
```

### Step 5: Export (optional)

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

## What's Next?

- **Deep dive:** [Skill Reference](../skills/sruja-architecture/SKILL.md)
- **Prompt patterns:** [Prompt Library](../skills/sruja-architecture/PROMPTS.md)
- **Complete guide:** [Skill Workflow Reference](../skills/sruja-architecture/REFERENCE.md)

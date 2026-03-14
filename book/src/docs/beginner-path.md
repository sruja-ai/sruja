---
title: "Beginner Path"
weight: 2
summary: "7 steps to use AI skills for architecture."
difficulty: "beginner"
estimatedTime: "~2–3 hours total"
---

# Beginner Path: Use AI Skills for Architecture

**Don't learn a language—use AI to generate and maintain architecture.**

If you just did [Quick Start](../getting-started.md), you have the CLI and skill installed. These 7 steps show how to use AI to maintain your architecture effectively.

> [!TIP]
> **Track your progress:** Check off each step as you complete it. Takes ~2–3 hours total.

---

## Step 1: Generate Your First Architecture ⏱️ 20–30 min

**What you'll do:** Let AI analyze your code and create `repo.sruja`

**Instructions:**

1. Open your project in your AI editor
2. Run this prompt:

```
Use sruja-architecture. Run `sruja discover --context -r . --format json`,
gather evidence, identify systems and containers,
generate repo.sruja with C4 context and container levels,
then run `sruja lint` and fix until it passes.
```

3. Wait for AI to ask 2-3 questions
4. Answer the questions (be specific: "Production database is PostgreSQL")
5. Review the generated `repo.sruja` file

**Success check:** You have a `repo.sruja` file that passes `sruja lint`

**What you learned:**
- How AI analyzes your codebase
- What questions it asks
- What a valid `repo.sruja` looks like

---

## Step 2: Understand What Was Generated ⏱️ 15–20 min

**What you'll do:** Read and understand your AI-generated architecture

**Instructions:**

1. Open `repo.sruja` in your editor
2. Read it section by section:
   - `person` entries = external actors
   - `system` entries = your major boundaries
   - `container` entries = deployable units
   - `->` lines = relationships
3. Run `sruja tree repo.sruja` to see structure visually
4. Ask AI to explain:

```
Read repo.sruja and explain:
1. What systems are defined?
2. How do containers connect?
3. What external actors use this system?
```

**Success check:** You can explain your architecture in plain English

**What you learned:**
- C4 structure (person → system → container)
- How to read relationships
- What each component does

---

## Step 3: Validate and Fix Errors ⏱️ 15–20 min

**What you'll do:** Learn to validate and fix issues with AI help

**Instructions:**

1. Run `sruja lint repo.sruja`
2. If you see errors, copy them to clipboard
3. Ask AI:

```
I got these lint errors. Fix them in repo.sruja:
[paste errors here]
```

**Common errors:**

| Error | What It Means | How to Fix |
|--------|----------------|--------------|
| `E204: Circular dependency` | A → B → A | Remove one edge in the cycle |
| `E205: Orphan element` | Something with no connections | Add relationships or remove it |
| `E201: Invalid kind` | Unknown type | Use person, system, container, database |

4. Re-run `sruja lint` until clean

**Success check:** `sruja lint repo.sruja` shows "All checks passed"

**What you learned:**
- How validation works
- Common architecture errors
- How to ask AI to fix issues

---

## Step 4: Make a Specific Change ⏱️ 20–30 min

**What you'll do:** Direct AI to add or modify something specific

**Instructions:**

Think of a change you want. Examples:

- **Add component:** "Add a logging service container"
- **Add relationship:** "Connect the new logger to all existing containers"
- **Modify description:** "Update the API container description to say it handles webhooks"

Then ask AI:

```
Use sruja-architecture. Read repo.sruja and:
[your change here]
Then run sruja lint and fix any errors.
```

**Success check:** Your change is added and `sruja lint` passes

**What you learned:**
- How to make targeted updates
- To validate after each change
- That iteration is normal (AI can refine it)

---

## Step 5: Export for Your Team ⏱️ 10–15 min

**What you'll do:** Share architecture with stakeholders

**Instructions:**

1. Export Markdown:

```bash
sruja export markdown repo.sruja > ARCHITECTURE.md
```

2. Export diagram:

```bash
sruja export mermaid repo.sruja > ARCHITECTURE.mmd
```

3. Open `ARCHITECTURE.md` and review
4. Open `ARCHITECTURE.mmd` in [Mermaid Live Editor](https://mermaid.live)

**Success check:** You can send `ARCHITECTURE.md` to your team and they understand it

**What you learned:**
- Different export formats
- How to share architecture
- That diagrams are output, not the source

---

## Step 6: Detect and Fix Drift ⏱️ 20–30 min

**What you'll do:** Keep architecture in sync as code changes

**Instructions:**

1. Make a small change to your actual code
   - Add a new function
   - Rename a file
   - Move a module

2. Run drift detection:

```bash
sruja drift -r . --format json
```

3. Ask AI:

```
Use sruja-architecture. Here's drift output:
[paste JSON]
Analyze what changed and update repo.sruja to match current code.
```

4. Review the changes AI made
5. Run `sruja lint repo.sruja`

**Success check:** `sruja drift` shows no issues (or you've addressed them)

**What you learned:**
- How to detect changes over time
- To let AI update architecture automatically
- That architecture stays in sync with code

---

## Step 7: Explore Patterns with AI ⏱️ 15–20 min

**What you'll do:** Use AI as an architecture advisor

**Instructions:**

Pick a pattern and ask AI to explain it:

**Monolith:**

```
Use sruja-architecture. Explain the monolith pattern:
When to use it, pros/cons, and show an example in Sruja syntax.
```

**Microservices:**

```
Use sruja-architecture. Explain microservices:
When to split, trade-offs, and generate an example architecture.
```

**Event-driven:**

```
Use sruja-architecture. Show me an event-driven architecture
with Kafka, producers, and consumers. Explain trade-offs.
```

**Success check:** You understand different patterns and when to use them

**What you learned:**
- Architectural patterns
- When each pattern makes sense
- That AI can be your advisor, not just generator

---

## Tips for Success

| Practice | Why It Helps |
|-----------|---------------|
| **Be specific** | "Add logging to API container" works better than "Improve architecture" |
| **Validate often** | Run `sruja lint` after each AI edit—catch mistakes early |
| **Ask questions** | "Why did you model it this way?" helps you learn |
| **Start simple** | Get context + container levels right, add detail later |
| **Trust evidence** | If `sruja discover` doesn't find something, tell your AI—don't let it guess |
| **Iterate** | First attempt won't be perfect—refine with feedback |

---

## Common Mistakes to Avoid

❌ **Don't:** "Generate full architecture for everything"

✅ **Do:** Start with context + container levels, add components when needed

❌ **Don't:** Guess at missing information

✅ **Do:** Ask targeted questions, list what's unknown

❌ **Don't:** Skip validation

✅ **Do:** Always run `sruja lint` after AI edits

❌ **Don't:** Model for completeness

✅ **Do:** Model what evidence supports—minimal is better

---

## What's Next?

**Go deeper:**

- [System Design 101 – Module 1](../courses/system-design-101/module-1-fundamentals/module-overview.md) – Learn architecture concepts
- [Skill Reference](../../skills/sruja-architecture/SKILL.md) – What the skill knows

**Practice:**

- Try different codebases (your own or open-source examples)
- Practice making specific changes
- Compare different patterns with AI

**Connect:**

- [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions) – Ask questions, share patterns
- [Community](../../community.md) – See how others use Sruja

---

## Quick Reference

| Want to | How |
|----------|------|
| **Generate architecture** | Prompt AI from Step 1 |
| **Validate** | `sruja lint repo.sruja` |
| **Export docs** | `sruja export markdown repo.sruja > doc.md` |
| **Export diagram** | `sruja export mermaid repo.sruja > diagram.mmd` |
| **Check drift** | `sruja drift -r . --format json` |
| **Fix errors** | Paste lint output to AI and say "fix these" |
| **Explain pattern** | "Explain [pattern] with pros/cons and example" |

---

**Remember:** You're not learning a language. You're learning how to:
1. Guide AI with clear requests
2. Validate results
3. Iterate and refine
4. Use AI as an advisor

The skill handles the syntax—you handle the thinking.

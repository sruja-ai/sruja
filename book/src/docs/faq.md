---
title: "FAQ"
weight: 100
summary: "Common questions about getting started with Sruja."
---

# Frequently Asked Questions

**New to Sruja? Start here.**

---

## Installation

### Do I need to know programming to use Sruja?

**Not really.** You need:
- Basic comfort with terminals (running commands)
- A codebase to document (any language)
- An AI editor (Cursor, Copilot, Claude, etc.)

You don't write the `.sruja` files manually—your AI does it.

### What editors work with Sruja?

**AI editors with skill support:**
- Cursor ✅ (built-in support)
- GitHub Copilot ✅ (requires [skills.sh](https://skills.sh))
- Claude ✅ (requires [skills.sh](https://skills.sh))
- Continue.dev ✅ (requires [skills.sh](https://skills.sh))
- Any editor with skills.sh support

**Manual use:**
- VS Code with Sruja extension (syntax highlighting, preview)
- Terminal (run commands manually, write `.sruja` by hand)

### What's the difference between the CLI and the skill?

| CLI | Skill |
|------|--------|
| **Analyzes code** | Generates `.sruja` files |
| **Validates files** | Knows syntax and patterns |
| **Exports diagrams** | Interprets and refines |
| **No AI required** | Requires AI editor |

You need both: CLI for validation, skill for generation.

### Can I use Sruja without an AI editor?

**Yes, but it's more work.**

You can:
- Run `sruja discover` and manually create `.sruja` files
- Use the VS Code extension for syntax highlighting
- Write `.sruja` by hand using the [language reference](../reference/language-spec.md)

However, AI makes it 10x easier. Consider using Cursor or installing skills.sh for your current editor.

---

## Workflow

### How long does it take to generate architecture?

**First time:** 5-10 minutes
- AI analyzes code
- AI asks 2-3 questions
- AI generates and validates

**Updates:** 2-5 minutes
- AI reads existing file
- AI makes your change
- AI validates

**With practice:** Most updates are 1-2 minutes

### What information does AI need from me?

Usually just 2-3 questions:

1. **System boundaries:** "What are the main systems?"
2. **External actors:** "Who or what calls your system?"
3. **Deployment:** "How is this deployed?"

The AI gets everything else from your code.

### What if I don't know the answer to a question?

**That's fine!** Tell your AI:

```
I don't know the answer to "What's the deployment model?"
Please assume [state your best guess or leave as TODO].
```

The AI can generate a TODO or make reasonable assumptions, which you can refine later.

---

## Files and Syntax

### What's the difference between `repo.sruja` and `architecture.sruja`?

They're the same thing, just different names.

- **`repo.sruja`** – Recommended for new projects
- **`architecture.sruja`** – Supported for backward compatibility

Use whichever you like—Sruja detects both.

### Where should I put `repo.sruja`?

**Repository root** (same folder as `package.json`, `pom.xml`, etc.)

Example structure:
```
my-project/
├── src/
├── package.json
├── README.md
└── repo.sruja          ← Put it here
```

### Can I have multiple `.sruja` files?

**Yes.** Common patterns:

- One file per system: `user-service.sruja`, `order-service.sruja`
- One per module: `auth.sruja`, `api.sruja`, `db.sruja`
- Nested: `arch/repo.sruja`, `arch/services.sruja`

Sruja will detect any `.sruja` file when you run commands.

### Do I need to import from `stdlib`?

**Recommended: yes.**

`import { * } from 'sruja.ai/stdlib'` gives you standard types:
- `person`, `system`, `container`, `database`, `queue`
- Without needing to define them manually

You can still define types manually (see [Quick Start](../getting-started.md)), but imports save time.

---

## Validation

### What does `sruja lint` check?

| Check | What It Finds | Severity |
|--------|----------------|----------|
| **Syntax errors** | Invalid keywords, bad structure | High |
| **Circular dependencies** | A → B → A loops | Medium |
| **Orphan elements** | Components with no connections | Low |
| **Missing fields** | Required description/technology missing | Medium |

Run `sruja lint repo.sruja` before committing changes.

### What if I see errors?

**Copy and paste to AI:**

```
Fix these lint errors in repo.sruja:
[paste errors]
```

**Common fixes:**

| Error | AI Prompt |
|--------|------------|
| Circular | "Remove one relationship in the cycle" |
| Orphan | "Add a relationship to this component or remove it" |
| Missing field | "Add a description field to this container" |

### Can I ignore lint errors?

**Not recommended.** Lint errors indicate real issues:
- Your architecture might not match reality
- Team members might misunderstand the design
- Documentation might be incomplete

Fix errors or add a comment explaining why it's okay.

---

## Using with AI

### How specific should I be with prompts?

**More specific = better results.**

❌ **Too vague:**
```
Improve my architecture.
```

✅ **Specific:**
```
Add error handling to the API container.
Update the description to mention rate limiting.
```

### What if AI generates something wrong?

**Don't panic.** This is normal.

1. Tell AI: "This isn't quite right. Here's what I want: [describe]"
2. Or: "Undo that change. Try again with these constraints: [list]"
3. Or: Make the change yourself, then ask AI to validate

Validation catches mistakes, and iteration is expected.

### Can AI handle large codebases?

**Yes.** Sruja scales:

- **Small (10-100 files):** Full analysis in seconds
- **Medium (100-1000 files):** Full analysis in 1-2 minutes
- **Large (1000+ files):** May take several minutes

AI can handle any size—it just processes more data for larger projects.

---

## Comparison to Other Tools

### How is Sruja different from diagramming tools (Miro, LucidChart)?

| Diagramming Tools | Sruja |
|-----------------|--------|
| Manual drawing | AI generates from code |
| Drifts from reality | Always in sync |
| Can't validate | Linting catches errors |
| Hard to version control | Git diff shows changes |
| Output, not source | Single source of truth |

**Think of it this way:** Diagrams are like screenshots—great for viewing, but not the source.

### How is Sruja different from architecture frameworks (Spring, DDD)?

**Sruja doesn't replace frameworks—it documents them.**

- **Frameworks** (Spring, DDD, Clean Architecture) – Guide how to write code
- **Sruja** – Documents what your code actually does

You can use both: Sruja documents your Spring-based microservices architecture.

---

## Language Support

### What programming languages does Sruja support?

| Language | Support | Notes |
|-----------|----------|--------|
| **JavaScript / TypeScript** | ✅ Excellent | Best support |
| **Python** | ✅ Excellent | Strong framework detection (Django, Flask, FastAPI) |
| **Go** | ✅ Excellent | Great for microservices |
| **Rust** | ✅ Excellent | Native language support |
| **Java** | ✅ Good | Spring Boot, Jakarta EE |
| **C#** | ✅ Good | .NET applications |
| **Ruby** | ✅ Good | Rails and Ruby apps |
| **PHP** | ✅ Good | PHP frameworks |
| **Scala** | ⚠️ Limited | Partial support |

### What if my language isn't listed?

Try it! Run:
```bash
sruja discover -r . --format json
```

If you see reasonable output, Sruja can analyze it. If not, please open an issue.

---

## Troubleshooting

### "sruja: command not found"

**The CLI isn't on your PATH.**

Try:
```bash
# Check if installed
which sruja

# Add to PATH (if using install script)
export PATH="$HOME/.local/bin:$PATH"

# Or restart your terminal
```

### Skill not loading in my editor

**Common causes:**

1. Didn't restart editor after installing
2. Editor doesn't support skills.sh
3. Typed wrong skill name

**Fixes:**
- Restart your editor
- Try: `npx skills list` to see installed skills
- Check editor has skills.sh support

### "Command fails: directory not found"

**You're in the wrong folder.**

Make sure:
```bash
# You should be IN your project
cd your-project

# Not AT the Sruja repo
# ❌ Wrong: cd ~/sruja
# ✅ Right: cd ~/my-project
```

### Discovery shows nothing

**Possible causes:**

1. **No supported files** – Empty directory or wrong language
2. **Wrong directory** – Not in the project root
3. **Language limitation** – Partial support for that language

**Try:**
```bash
# See what's detected
sruja discover -r . --format json

# Try quickstart for better visibility
sruja quickstart -r .
```

---

## Best Practices

### Start small

✅ **Do:**
- Get person + system + container levels working first
- Add components only when needed
- Model what evidence supports

❌ **Don't:**
- Model everything you can think of
- Add components "for completeness"
- Over-complicate simple systems

### Validate often

✅ **Do:**
- Run `sruja lint` after each AI edit
- Check before committing
- Fix errors before calling it "done"

❌ **Don't:**
- Skip validation to "save time"
- Commit known errors with a TODO

### Iterate with AI

✅ **Do:**
- Treat first attempt as draft, not final
- Provide feedback: "Good, but add X"
- Ask AI to refine specific parts

❌ **Don't:**
- Expect perfection on first try
- Give up if it's not exactly right
- Start over from scratch instead of refining

### Document decisions

✅ **Do:**
- Add descriptions to components
- Note why you chose a pattern
- Explain trade-offs

❌ **Don't:**
- Leave components unlabeled
- Assume everyone knows your intent

---

## Getting Help

### Where can I ask questions?

- **GitHub Discussions:** https://github.com/sruja-ai/sruja/discussions
- **Documentation:** [Getting Started](./getting-started.md) or [Beginner Path](./beginner-path.md)
- **Examples:** [Examples Gallery](../examples/index.md)

### How do I report a bug?

1. Check if it's already reported: https://github.com/sruja-ai/sruja/issues
2. If not, create a new issue with:
   - Steps to reproduce
   - What you expected
   - What actually happened
   - Your environment (OS, editor, language)

### How do I contribute?

See [Contributing Guide](../../docs/CONTRIBUTING.md) for guidelines.

---

## Quick Command Reference

| Command | When to Use | Example |
|----------|----------------|----------|
| `sruja quickstart -r .` | First look at a codebase | Instant overview |
| `sruja discover -r .` | When generating with AI | Gather evidence |
| `sruja lint repo.sruja` | After AI edits | Validate file |
| `sruja export mermaid` | Need a diagram | Generate for docs |
| `sruja export markdown` | Need readable docs | Export ARCHITECTURE.md |
| `sruja drift -r .` | After code changes | Detect what's different |

---

## Key Takeaways

1. **You don't learn a language**—you guide AI
2. **Validation is your friend**—run lint often
3. **Start minimal**—add detail only when needed
4. **Iterate**—refine with feedback
5. **Use AI as advisor**—ask about patterns and trade-offs

**The skill handles syntax. You handle thinking.**

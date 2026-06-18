---
title: "Introduction"
weight: 0
---

# What is Sruja?

**Sruja is a CLI-first autonomous coding agent with deterministic gates.**

It scans your repo, grounds every edit in real topology, and verifies the result before you ship. Sruja also works as a passive harness inside any editor (Cursor, Copilot, Claude, Windsurf) via MCP.

## The Problem

AI coding agents are fast, but they don't know your repo. They invent dependencies, break layer boundaries, and repeat mistakes. There's no built-in check between "generate" and "ship."

| Your approach | Problems |
|--------------|----------|
| **Raw model context** | Easy to miss boundaries, invent dependencies, or forget prior decisions |
| **Manual review** | Slow, inconsistent, doesn't scale with AI-generated volume |
| **CI-only checks** | Catches issues after merge, not before |

## The Solution

**A grounded agent loop with an independent grader.**

```text
focus / drift  →  agent edits code  →  verify-task  →  (critique or approve)
```

Sruja has two modes:

### Autonomous mode

```bash
sruja agent loop --goal "Refactor auth boundary without breaking tests"
```

Sruja owns the full observe → act → verify → critique → replan cycle. The deterministic layer (drift, lint, verify-task, intent check) is the **independent grader** — the actor never grades itself.

### Editor-hosted mode (MCP)

When your editor drives the LLM, Sruja provides the grounding and verification gates:

1. MCP: focus / drift state / boundary context
2. Host LLM edits code
3. `sruja verify-task --profile coding -r .`
4. (optional) `sruja agent record -c "..."` on failure

## How Sruja helps

| Before Sruja | With Sruja |
|--------------|-----------|
| AI guesses from partial context | AI works from repo evidence and linked decisions |
| No check between generate and ship | Deterministic gates verify every change |
| Mistakes found in code review | Structural issues caught before review |
| Decisions forgotten or scattered | Captured and retrievable at change time |
| Diagrams become stale truth | Diagrams are optional exports, not the product |

---

## Start Here

- [Quick start](../getting-started.md): install, scan, focus, verify
- [CLI guide](cli.md): daily commands, CI-friendly outputs, and workflows
- [How Sruja works](how-sruja-works.md): the agent loop in detail
- [VS Code extension](vscode-extension.md): editor commands, diagnostics, and previews

---

## Who is Sruja For?

### Developers using AI agents

- **Verify generated changes** before calling work done
- **Brief agents before edits** with the right task context
- **Reduce risky guesses** from incomplete repo understanding

### Engineering teams

- **Shared knowledge** — capture docs and decisions in a durable, reviewable way
- **CI guardrails** — alignment checked continuously
- **Preserved reasoning** — future maintainers and agents do not start cold

### Platform engineers

- **Guardrails for AI** — structural gates that scale across teams
- **MCP integration** — grounded context for any AI editor
- **Optional reviewed intent** — `repo.sruja` when teams want durable boundaries in Git

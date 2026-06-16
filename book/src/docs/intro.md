---
title: "Introduction"
weight: 0
---

# Introduction

**Context engineering for knowledge and decisions.**

Sruja helps teams capture important engineering knowledge, retrieve the right task context for developers and AI agents, and verify that changes still align with those decisions. Optional `repo.sruja` is reviewed intent in Git, not the day-one requirement.

> **New here?** Start with [Quick start](../getting-started.md) to learn the core loop: capture, retrieve, verify. Add `repo.sruja` only when you want reviewed intent in Git.

## The Problem

How does your AI know the real architecture today?

| Your approach | Problems |
|--------------|------------|
| **Raw model context** | Easy to miss boundaries, invent dependencies, or forget prior decisions |
| **Drawings in Miro/LucidChart** | Manual updates, easy to forget, drifts from code |
| **Wiki pages** | Inconsistent, hard to maintain, no validation |

Sound familiar? You're not alone. Most teams struggle with this.

## The Solution

**A small context loop with optional reviewed intent.**

With Sruja:

- Capture knowledge and decisions with docs, decision records, and optional reviewed intent
- Retrieve grounded context with `focus`, `ai`, and MCP before the host agent edits
- Verify the result with drift, intent, and `verify-task`
- Add reviewed intent in Git (`repo.sruja`) only when the team wants stricter governance

Sruja is **not** a replacement for Cursor or Copilot — it is the guardrail layer beside them.

## How This Helps

| Before Sruja | With Sruja |
|----------------|-------------|
| AI guesses from partial context | AI works from repo evidence and linked decisions |
| Knowledge lives in stale docs or chat history | Important context is captured and retrievable |
| Hard to catch generated mistakes | Drift, intent, and verification gates catch regressions |
| Hard to brief agents consistently | Task-scoped context is reusable |
| Diagrams become the truth | Diagrams are optional outputs, not the product center |

---

## Start Here

- [Quick start](../getting-started.md): core capture, retrieve, verify loop
- [Getting started](getting-started.md): core workflow plus optional reviewed intent
- [CLI guide](cli.md): daily commands, CI-friendly outputs, and workflows
- [VS Code extension](vscode-extension.md): editor commands, diagnostics, and previews

---

## What Sruja Optimizes For

- **Evidence over guesses**: context starts from what exists in code and linked docs today
- **Small surface area**: a few core workflows used consistently
- **Explicit trade-offs**: reviewed intent is optional until the team needs it
- **CLI-first agent**: `sruja agent loop` owns the full closed loop, or use MCP from any editor
- **Knowledge and decisions**: context engineering, not a feature catalog

## Who is Sruja For?

### Software Architects

- **Review changes** against evidence and intent
- **Prevent drift** through automated gates
- **Document decisions** with [ADRs (Architecture Decision Records)](concepts/adr.md)
- **Keep reviewed truth** in Git when needed

### Developers Using AI

- **Brief agents before edits** with the right task context
- **Reduce risky guesses** from incomplete repo understanding
- **Verify generated changes** before calling work done
- **Keep prior decisions visible** at change time

### Teams Maintaining Shared Knowledge

- **Capture docs and decisions** in a durable, reviewable way
- **Integrate into CI/CD** so alignment is checked continuously
- **Preserve reasoning** so future maintainers and agents do not start cold
- **Export artifacts** only when communication requires them

## Next Steps

- **New to Sruja?** Start with [Quick start](../getting-started.md)
- **Need workflows?** Read the [CLI guide](cli.md) and [How Sruja works](how-sruja-works.md)
- **Ready for reviewed intent?** Follow [Using Sruja in your project](using-sruja-in-your-project.md)

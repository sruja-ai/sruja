---
title: "Introduction"
weight: 0
---

# Introduction

**AI coding harness for repo structure.**

Sruja scans your codebase, reports structural problems AI edits tend to introduce, and gives agents bounded context via MCP (`focus`, drift state, `verify-task`). Optional `repo.sruja` is reviewed CI intent — not required on day one.

> **New here?** Install the CLI, register MCP, install `sruja-harness` — [Quick start](../getting-started.md) (~5 min). Add `sruja-architecture` only when you want versioned architecture in Git.

## The Problem

How does your AI know the real architecture today?

| Your approach | Problems |
|--------------|------------|
| **Raw model context** | Easy to miss boundaries, invent dependencies, or forget prior decisions |
| **Drawings in Miro/LucidChart** | Manual updates, easy to forget, drifts from code |
| **Wiki pages** | Inconsistent, hard to maintain, no validation |

Sound familiar? You're not alone. Most teams struggle with this.

## The Solution

**Deterministic harness plus optional architecture-as-code.**

With Sruja (Tier 1):

- Structural scan and drift — no `.sruja` required
- `focus` briefings before the host agent edits
- `verify-task` gates after edits (lint/tests/drift based on repo profile)
- MCP integration so the host agent can query Sruja deterministically
- Context graph + AI context exports for host agents and CI
- Optional: reviewed intent in Git (`repo.sruja`) when teams want strict CI gates

Sruja is **not** a replacement for Cursor or Copilot — it is the guardrail layer beside them.

## How This Helps

| Before Sruja | With Sruja |
|----------------|-------------|
| AI guesses from partial context | AI works from repo evidence |
| Architecture lives in stale diagrams | Architecture lives in versioned `repo.sruja` |
| Hard to catch generated mistakes | Validation catches syntax, drift, and structural issues |
| Hard to brief agents consistently | Task-scoped context is reusable |
| Diagrams become the truth | Diagrams are exports (when you choose to generate them) |

---

## Start Here

- [Quick start](../getting-started.md): harness-first workflow
- [Getting started (full)](getting-started.md): Tier 1 + Tier 2 in one page
- [CLI guide](cli.md): daily commands, CI-friendly outputs, and workflows
- [VS Code extension](vscode-extension.md): editor commands, diagnostics, diagram preview

---

## What Sruja Optimizes For

- **Evidence over guesses**: the harness starts from what exists in code today
- **Small surface area**: a few commands used consistently (`focus`, `verify-task`, drift)
- **Explicit trade-offs**: optional reviewed intent when the team is ready
- **Host-owned LLM loop**: Sruja never replaces your editor/agent
- **Context engineering**: context graphs + MCP tools, not "learn a DSL"

## Who is Sruja For?

### Students & Learners

- **Understand system design** through production-ready examples from fintech, healthcare, and e-commerce
- **Use AI skills** to generate architecture and explore patterns without manual DSL writing
- **Real-world scenarios** that prepare you for interviews and real projects

### Software Architects

- **Review architecture changes** against evidence and intent
- **Prevent architectural drift** through automated gates
- **Scale guardrails** across multiple teams without turning every review into archaeology
- **Document decisions** with [ADRs (Architecture Decision Records)](concepts/adr.md)

### Product Teams

- **Link requirements to architecture** - see how features map to technical components
- **Track SLOs and metrics** alongside your architecture
- **Align technical decisions** with business goals and user needs
- **Communicate architecture** to stakeholders (export to Markdown/Mermaid when needed)

### DevOps Engineers

- **Integrate into CI/CD** - validate architecture on every commit
- **Automate documentation** generation from architecture files
- **Model deployments** - Blue/Green, Canary, multi-region strategies
- **Refresh evidence** so AI assistants and reviewers see current repo context

## Next Steps

- **New to Sruja?** Start with [Quick start](../getting-started.md)
- **Need workflows?** Read the [CLI guide](cli.md) and [How Sruja works](how-sruja-works.md)
- **Ready for reviewed intent?** Follow [Using Sruja in your project](using-sruja-in-your-project.md)

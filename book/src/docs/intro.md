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
- `focus` / MCP briefings before the host agent edits
- `verify-task` after edits (lint, tests, drift when applicable)
- Optional: `sruja-architecture` skill → reviewed `repo.sruja` in Git
- Diagrams and docs are Tier-2 exports, not the source of truth

Sruja is **not** a replacement for Cursor or Copilot — it is the guardrail layer beside them.

## How This Helps

| Before Sruja | With Sruja |
|----------------|-------------|
| AI guesses from partial context | AI works from repo evidence |
| Architecture lives in stale diagrams | Architecture lives in versioned `repo.sruja` |
| Hard to catch generated mistakes | Validation catches syntax, drift, and structural issues |
| Hard to brief agents consistently | Task-scoped context is reusable |
| Diagrams become the truth | Diagrams are exported from reviewed truth |

---

## Key Concepts

**Architecture as Code:** Instead of drawing boxes, you define structure in code. AI writes it, you validate it, and everyone uses the same source.

**Validation:** Like `lint` for code, `sruja lint` checks for:
- Circular dependencies
- Orphaned components
- Missing connections
- Rule violations

**C4 Model:** Sruja uses the C4 approach, which organizes architecture into levels:
- **Person:** Users, external systems
- **System:** Major boundaries (e.g., "Order System")
- **Container:** Deployable units (e.g., "API Service")
- **Component:** Internal parts (e.g., "Payment Module")

This hierarchy makes architecture clear and understandable.

## Who is Sruja For?

### Students & Learners

- **Understand system design** through production-ready examples from fintech, healthcare, and e-commerce
- **Use AI skills** to generate architecture and explore patterns without manual DSL writing
- **Real-world scenarios** that prepare you for interviews and real projects

### Software Architects

- **Review architecture changes** against evidence and intent
- **Prevent architectural drift** through automated validation
- **Scale guardrails** across multiple teams without turning every review into archaeology
- **Document decisions** with [ADRs (Architecture Decision Records)](docs/concepts/adr.md)

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

## Example

Here's a simple example to get you started:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

App = system "My App" {
    Web = container "Web Server"
    DB = database "Database"
}

User = person "User"

User -> App.Web "Visits"
App.Web -> App.DB "Reads/Writes"

view index {
    include *
}
```

For **production-ready examples** with real-world patterns, see our [Examples](docs/examples.md) page featuring:

- Banking systems (fintech)
- E-commerce platforms
- Healthcare platforms (HIPAA-compliant)
- Multi-tenant SaaS platforms

## Next Steps

- **New to Sruja?** Start with [Getting Started](docs/getting-started.md)
- **Use AI:** Install the skill in your editor and let AI generate architecture from your codebase
- **Need examples?** Check out [Real-World Examples](docs/examples.md)
- **Ready to build?** Use the [VS Code extension](../vscode.md) for diagram preview

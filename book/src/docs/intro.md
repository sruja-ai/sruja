---
title: "Introduction"
weight: 0
---

# Introduction

**Context engineering for the AI era.**

Sruja helps AI-assisted teams generate and maintain architecture as code from deterministic repo evidence. The result is a reviewable `repo.sruja` truth file that feeds better context to editors, CI, and coding agents.

> **New here?** Do [Quick start](../getting-started.md) (about 5 min), then the [Beginner path](beginner-path.md) (2–3 hours). Start with the AI skill; you don't write `.sruja` files manually.

## The Problem

How does your AI know the real architecture today?

| Your approach | Problems |
|--------------|------------|
| **Raw model context** | Easy to miss boundaries, invent dependencies, or forget prior decisions |
| **Drawings in Miro/LucidChart** | Manual updates, easy to forget, drifts from code |
| **Wiki pages** | Inconsistent, hard to maintain, no validation |

Sound familiar? You're not alone. Most teams struggle with this.

## The Solution

**Architecture as code plus evidence.**

With Sruja:

- The sruja-architecture skill gathers deterministic repo evidence
- Your AI generates or updates `repo.sruja`
- Lint, drift, and intent checks validate the result
- Editors and agents reuse that context before coding
- Diagrams and docs are exported when needed, not treated as the source

**You don't learn a new language first.** You guide the AI, review the output, and let validation catch mistakes.

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

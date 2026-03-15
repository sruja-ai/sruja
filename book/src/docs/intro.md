---
title: "Introduction"
weight: 0
---

# Introduction

**Architecture intelligence for the AI era.**

Sruja uses AI to analyze your code and generate architecture as code—so it never drifts from reality.

> **New here?** Do [Quick start](../getting-started.md) (about 5 min), then the [Beginner path](beginner-path.md) (2–3 hours). You don't write `.sruja` files manually—your AI does it for you.

## The Problem

How do you document architecture today?

| Your approach | Problems |
|--------------|------------|
| **Drawings in Miro/LucidChart** | Manual updates, easy to forget, drifts from code |
| **Wiki pages** | Inconsistent, hard to maintain, no validation |
| **PNG/PDF diagrams** | Can't version control diff, outdated quickly |

Sound familiar? You're not alone. Most teams struggle with this.

## The Solution

**Architecture as code.**

With Sruja:

- AI analyzes your codebase automatically
- You get a `repo.sruja` file (architecture definition)
- Validate it automatically in CI/CD
- Export diagrams when needed (not as the source)

**You don't learn a new language.** You ask your AI to generate the file, and it handles the syntax.

## How This Helps

| Before Sruja | With Sruja |
|----------------|-------------|
| Update diagrams manually | AI generates from code |
| Diagram drifts from reality | Always in sync |
| Can't catch errors | Validation catches issues |
| Hard to review changes | Git diff shows everything |
| Scattered tools | Single source of truth |

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

- **Enforce architectural standards** with policy-as-code
- **Prevent architectural drift** through automated validation
- **Scale governance** across multiple teams without manual reviews
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
- **Track infrastructure** - map logical architecture to physical deployment

## Example

Here's a simple example to get you started:

```sruja
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

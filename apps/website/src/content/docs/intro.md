---
title: "Introduction"
weight: 0
---

# Introduction

Sruja is an **open source** architecture-as-code tool. It helps teams define, validate, and evolve their software architecture using a text-based language (DSL).

## Why Sruja?

Most teams document architecture in static diagrams (Miro, LucidChart, Visio) or inconsistent Wiki pages. These suffer from:
1.  **Drift:** The code changes, but the diagram doesn't.
2.  **Inconsistency:** Every architect draws "boxes and arrows" differently.
3.  **No Validation:** You can't "test" a PNG image for broken dependencies.

**Sruja treats Architecture like Code:**
*   **Version Control:** Commit your architecture to Git.
*   **Validation:** CI/CD checks for circular dependencies and rule violations.
*   **Consistency:** Based on the **[C4 Model](/docs/concepts/c4-model)** for clear, hierarchical abstractions.

## Who is Sruja For?

### Students & Learners
- **Learn system design** with production-ready examples from fintech, healthcare, and e-commerce
- **Hands-on courses** covering fundamentals to advanced patterns
- **Real-world scenarios** that prepare you for interviews and real projects

### Software Architects
- **Enforce architectural standards** with policy-as-code
- **Prevent architectural drift** through automated validation
- **Scale governance** across multiple teams without manual reviews
- **Document decisions** with ADRs (Architecture Decision Records)

### Product Teams
- **Link requirements to architecture** - see how features map to technical components
- **Track SLOs and metrics** alongside your architecture
- **Align technical decisions** with business goals and user needs
- **Communicate architecture** to stakeholders with clear, visual diagrams

### DevOps Engineers
- **Integrate into CI/CD** - validate architecture on every commit
- **Automate documentation** generation from architecture files
- **Model deployments** - Blue/Green, Canary, multi-region strategies
- **Track infrastructure** - map logical architecture to physical deployment

## Example

Here's a simple example to get you started:

```sruja
specification {
  element system
  element container
  element datastore
  element person
}

model {
  App = system "My App" {
    Web = container "Web Server"
    DB = datastore "Database"
  }
  
  User = person "User"
  
  User -> App.Web "Visits"
  App.Web -> App.DB "Reads/Writes"
}

views {
  view index {
    include *
  }
}
```

For **production-ready examples** with real-world patterns, see our [Examples](/docs/examples) page featuring:
- Banking systems (fintech)
- E-commerce platforms
- Healthcare platforms (HIPAA-compliant)
- Multi-tenant SaaS platforms

## Next Steps

- **New to Sruja?** Start with [Getting Started](/docs/getting-started)
- **Want to learn?** Explore our [Courses](/courses) and [Tutorials](/tutorials)
- **Need examples?** Check out [Real-World Examples](/docs/examples)
- **Ready to build?** Try the [Sruja Designer](/designer) for interactive visualization

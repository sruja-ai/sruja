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

## Example

```sruja
system App "My App" {
  container Web "Web Server"
  datastore DB "Database"
}
person User "User"
User -> App.Web "Visits"
App.Web -> App.DB "Reads/Writes"
```

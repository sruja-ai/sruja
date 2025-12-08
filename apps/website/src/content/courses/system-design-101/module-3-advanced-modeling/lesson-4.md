---
title: "Lesson 4: Architectural Perspectives"
weight: 4
summary: "Understanding context, containers, and components without special DSL keywords."
---

# Lesson 4: Architectural Perspectives

As your system grows, a single diagram becomes too cluttered. You need different "maps" for different audiences:
*   **Executives:** Need a high-level overview (Context).
*   **Architects:** Need to see service boundaries (Containers).
*   **Developers:** Need to see internal details (Components).

Sruja models naturally support multiple perspectives without special keywords. Use the built‑in elements, and tooling presents the right level of detail.

## One Model, Multiple Perspectives

```sruja
architecture "E-Commerce" {
  person Customer
  system Shop {
    container WebApp "Web Application"
    container API "API Service"
    datastore DB "Database"
  }

  // Context perspective: Customer ↔ Shop
  Customer -> WebApp "Uses"
  WebApp -> API "Calls"
  API -> DB "Reads/Writes"
}
```

- Context: Persons and systems
- Container: Systems and their containers
- Component: Containers and their internals

Use the Studio/Viewer to switch between perspectives derived from the same model. No `view` keyword required.

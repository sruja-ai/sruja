---
title: "Content Style Guide"
weight: 1
summary: "Standards for writing Sruja DSL examples in docs, tutorials, and courses."
---

# Content Style Guide

Use these conventions for all Sruja examples to keep content consistent and compilable.

## Core Rules

- Prefer `architecture`, `system`, `container`, `component`, `datastore`, `person`, and `scenario`.
- Do not use deprecated keywords: `module`, `context`, `boundedContext`, `data`, `api`, `policy`, `flow`, or bare `external`.
- Use fully qualified names when referring to nested elements outside local scope (e.g., `System.Container`, `System.Container.Component`).
- Place `tags [...]` inside element blocks (e.g., in a `container`), not at the system root.
- Use `metadata { key "value" }` syntax (no colon).
- Mark external boundaries with `tags ["external"]` inside the element.
- Keep scenario step labels short and action‑oriented.

## Example Template

```sruja
architecture "Sample" {
  person User
  system App {
    container WebApp
    container API
    datastore DB
  }

  // Valid relations
  User -> App.WebApp "Uses"
  App.WebApp -> App.API "Calls"
  App.API -> App.DB "Reads/Writes"

  // Scenario
  scenario Checkout "User Checkout" {
    User -> App.WebApp "adds items"
    App.WebApp -> App.API "submits"
    App.API -> App.DB "reserves"
    App.API -> App.WebApp "confirms"
    App.WebApp -> User "shows result"
  }
}
```

## Linking

When introducing a concept, link to related pages:
- Scenario → Layering, Validation
- Relations → Scenario, Validation
- Metadata & Tags → Validation

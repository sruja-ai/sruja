---
title: "Architecture files (legacy, optional)"
weight: 12
summary: "Optional modeling primitives for reviewed intent (repo.sruja). Not required for the harness."
---

# Architecture files (legacy, optional)

This section is for teams that choose to store **reviewed intent in Git** (`repo.sruja`). If you’re using Sruja primarily as an **AI coding harness** (MCP + context graph + focus/verify gates), you can ignore this section.

## What you model (when you choose Tier 2)

- **People and systems**: who uses what, and where the boundary is.
- **Containers and components**: deployable units and meaningful internal sub-parts.
- **Relations**: explicit connections with protocols and intent.
- **Views**: what you want to render/export for a given audience.
- **Validation**: making sure the model is consistent and useful.

## Where to go next

- [Architecture](architecture.md)
- [C4 model](c4-model.md)
- [System](system.md)
- [Container](container.md)
- [Component](component.md)
- [Person](person.md)
- [Relations](relations.md)
- [Views](views.md)
- [Validation](validation.md)
- [Deployment](deployment.md)
- [Requirements](requirements.md)
- [Scenario](scenario.md)
- [ADR](adr.md)
- [Policy](policy.md)

---

## The `overview { ... }` block (optional)

Use `overview` to add a short, human-readable summary shown in exports.

Use `overview` to provide a concise system description shown in docs/exports.

## Syntax

```sruja
import { * } from 'sruja.ai/stdlib'


overview {
title "E‑Commerce Platform"
summary "Web, API, and DB supporting browse, cart, and checkout"
}

view index {
include *
}
```

## Guidance

- Keep summary short and practical; avoid marketing language.
- Use `overview` at architecture root; prefer `description` inside elements for details.

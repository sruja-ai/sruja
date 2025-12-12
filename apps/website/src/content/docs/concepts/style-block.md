---
title: "Style Block"
weight: 42
summary: "Define global rendering styles for elements and relations."
---

# Style Block

Use `style` to set global visual defaults across diagrams and exports.

## Syntax

```sruja
architecture "Shop" {
  style {
    element "Datastore" { shape cylinder color "#22c55e" }
    element "API" { color "#0ea5e9" }
    relation "Calls" { color "#ef4444" }
  }

  // ... rest of architecture
}
```

## Guidance
- Use a small, consistent palette; prefer semantic colors.
- Override globally here; adjust per‑view in `views { styles { ... } }` when needed.

## Related
- `views` for per‑view styling
- `relations` and core element types


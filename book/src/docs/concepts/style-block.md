---
title: "Style Block"
weight: 42
summary: "Define global rendering styles for elements and relations."
---

# Style Block

Use `style` to set global visual defaults across diagrams and exports.

## Syntax

```sruja
system = kind "System"
container = kind "Container"
database = kind "Database"

style {
  element "Database" { shape cylinder color "#22c55e" }
  element "Container" { color "#0ea5e9" }
  relationship "Calls" { color "#ef4444" }
}

App = system "App" {
  API = container "API"
  DB = database "DB"
}

API -> DB "Calls"

view index {
  include *
}
```

## Guidance

- Use a small, consistent palette; prefer semantic colors.
- Override globally here; adjust per‑view in `view SOME_VIEW { styles { ... } }` when needed.

## Related

- `views` for per‑view styling
- `relations` and core element types

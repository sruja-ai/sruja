---
title: "Views"
weight: 36
summary: "Create focused visualizations using includes, excludes, and per‑view styles."
---

# Views

Define `views` to customize what elements appear and how they render.

## Syntax

```sruja
person = kind "Person"
system = kind "System"
container = kind "Container"
database = kind "Database"

App = system "Application" {
  WebApp = container "Web App"
  API = container "API"
  DB = database "Database"
}

User = person "User"

User -> App.WebApp "Uses"
App.WebApp -> App.API "Calls"
App.API -> App.DB "Reads/Writes"

view api_focus of App {
  title "API Focus"
  include App.API App.DB
  exclude App.WebApp
}

styles {
  element "Database" { shape "cylinder" color "#3b82f6" }
  relationship "Calls" { color "#ef4444" }
}

view index {
  include *
}
```

## Guidance

- Use `include` to spotlight critical paths; use `exclude` to reduce noise.
- Keep view names descriptive (e.g., "API Focus", "Data Flow").
- Use view `styles` for legibility: color important relations, reshape data stores.

## Related

- `relations` for edges
- `style` block for global defaults

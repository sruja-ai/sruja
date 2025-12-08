---
title: "Layering"
weight: 7
summary: "Keep dependencies flowing downward: Web → API → DB."
---

# Layering

Layering keeps your architecture modular. Higher layers depend on lower ones, not the other way around.

## Allowed Direction

```sruja
architecture "Shop" {
  system App {
    container WebApp
    container API
    datastore DB
  }

  App.WebApp -> App.API "Calls"
  App.API -> App.DB "Reads/Writes"
}
```

## Violation Example

```sruja
architecture "Shop" {
  system App {
    container WebApp
    container API
  }

  App.API -> App.WebApp "Returns UI"
}
```

This causes a layer violation. Fix by:
- Inverting the dependency (WebApp depends on an interface exposed by API)
- Using events/messages instead of direct upward calls

## See Also

- [Scenario](/docs/concepts/scenario)
- [Validation](/docs/concepts/validation)

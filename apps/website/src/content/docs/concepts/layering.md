---
title: "Layering"
weight: 30
summary: "Keep dependencies flowing downward: Web → API → DB."
---

# Layering

Layering keeps your architecture modular. Higher layers depend on lower ones, not the other way around.

## Allowed Direction

```sruja
element person
element system
element container
element component
element datastore
element queue

system App {
container WebApp
container API
datastore DB
}

App.WebApp -> App.API "Calls"
App.API -> App.DB "Reads/Writes"

view index {
include *
}
```

## Violation Example

```sruja
element person
element system
element container
element component
element datastore
element queue

system App {
WebApp = container
API = container
}

App.API -> App.WebApp "Returns UI"

```

This causes a layer violation. Fix by:

- Inverting the dependency (WebApp depends on an interface exposed by API)
- Using events/messages instead of direct upward calls

## See Also

- [Scenario](/docs/concepts/scenario)
- [Validation](/docs/concepts/validation)

---
title: "Properties"
weight: 40
summary: "Attach arbitrary key‑value metadata to elements using `properties`."
---

# Properties

Use `properties` for structured metadata beyond `metadata` labels.

## Syntax

```sruja
architecture "Shop" {
  system App {
    container API {
      properties {
        owner "platform-team"
        repo "github.com/org/app-api"
        language "go"
      }
    }
  }
}
```

## Guidance
- Prefer simple strings; use clear keys like `owner`, `repo`, `language`.
- Keep sensitive values out of source; use references or IDs.


---
title: "Properties"
weight: 40
summary: "Attach arbitrary key‑value metadata to elements using `properties`."
---

# Properties

Use `properties` for structured metadata beyond `metadata` labels.

## Syntax

```sruja
import { * } from 'sruja.ai/stdlib'


App = system "App" {
  API = container "API" {
    properties {
      owner "platform-team"
      repo "github.com/org/app-api"
      language "go"
    }
  }
}

view index {
include *
}
```

## Guidance

- Prefer simple strings; use clear keys like `owner`, `repo`, `language`.
- Keep sensitive values out of source; use references or IDs.

---
title: "Validation"
weight: 35
summary: "Automatic checks: IDs, references, cycles, layering, externals."
---

# Validation

Sruja validates your model to catch issues early.

## Common Checks

- Unique IDs within scope
- Valid references (relations connect existing elements)
- Cycles (informational; feedback loops are valid)
- Layering violations (dependencies must flow downward)
- External boundary checks
- Simplicity guidance (non‑blocking)

## Example

```sruja
architecture "Shop" {
  person User
  system App {
    container WebApp
    container API
    datastore DB
  }

  // Valid relations (qualified cross-scope)
  User -> App.WebApp "Uses"
  App.WebApp -> App.API "Calls"
  App.API -> App.DB "Reads/Writes"
}
```

Run `sruja validate` locally or in CI to enforce these rules.

## See Also

- [Layering](/docs/concepts/layering)
- [Scenario](/docs/concepts/scenario)

---
title: "Requirements"
weight: 31
summary: "Model functional and non‑functional requirements directly in Sruja DSL."
---

# Requirements

Use `requirement` to capture functional, performance, security, and constraint requirements. Requirements are declared at the architecture root only.

## Syntax

```sruja
specification {
  element person
  element system
  element container
  element component
  element datastore
  element queue
}

model {
  requirement R1 functional "Support 10k concurrent users"
  requirement R2 performance "p95 < 200ms for /checkout"
  requirement R3 security "PII encrypted at rest"
  requirement R4 constraint "Only PostgreSQL managed service"
}

views {
  view index {
    include *
  }
}
```

## Guidance
- Keep requirement titles concise and testable.
- Reference requirements in ADRs and scenarios where relevant.
- Validate with `sruja lint` to surface unmet or conflicting requirements.
- Declarations at system/container/component level are deprecated and ignored by exporters and UI.

## Related
- `scenario` for behavior walkthroughs
- `slo` for targets and windows
- `adr` for decision records

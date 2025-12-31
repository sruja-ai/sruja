---
title: "Requirements"
weight: 31
summary: "Model functional and non‑functional requirements directly in Sruja DSL."
---

# Requirements

Use `requirement` to capture functional, performance, security, and constraint requirements. Requirements are declared at the architecture root only.

## Syntax

```sruja
person = kind "Person"
system = kind "System"
container = kind "Container"

// Requirements using flat syntax
R1 = requirement functional "Support 10k concurrent users"
R2 = requirement performance "p95 < 200ms for /checkout"
R3 = requirement security "PII encrypted at rest"
R4 = requirement constraint "Only PostgreSQL managed service"
R5 = requirement nonfunctional "System must be maintainable"

view index {
  include *
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

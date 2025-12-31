---
title: "Constraints and Conventions"
weight: 43
summary: "Specify architectural constraints and shared conventions."
---

# Constraints and Conventions

Use these blocks to formalize architectural limits and team agreements.

## Syntax

Sruja uses **flat syntax** - all declarations are top-level, no wrapper blocks needed:

```sruja
person = kind "Person"
system = kind "System"
container = kind "Container"

// Constraints using flat syntax
constraints {
  "No direct WebApp -> DB writes"
  "Only managed Postgres for relational data"
}

// Conventions using flat syntax
conventions {
  "kebab-case for services"
  "W3C trace context propagated across services"
}
```

## Guidance

- Use `constraints` for firm boundaries (security, compliance, architecture).
- Capture team `conventions` to improve consistency across services.

## Related

- `policy` for governance documents
- `adr` for rationale behind constraints

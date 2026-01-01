---
title: "Lesson 2: Consistency via Constraints & Conventions"
weight: 2
summary: "Use constraints and conventions to manage consistency trade‑offs."
---

# Lesson 2: Consistency via Constraints & Conventions

## Why Constraints?

They document trade‑offs and prevent accidental coupling across services.

## Sruja: Guardrails for Consistency

```sruja
import { * } from 'sruja.ai/stdlib'


constraints {
rule "No cross‑service transactions"
rule "Idempotent event handlers"
}
conventions {
naming "kebab-case"
retries "Exponential backoff (max 3)"
}

view index {
include *
}
```

## Practice

- Add constraints that support your chosen consistency model.
- Capture conventions for retries, idempotency, and naming.

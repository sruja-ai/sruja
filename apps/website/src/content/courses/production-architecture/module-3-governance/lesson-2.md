---
title: "Lesson 2: Policies, Constraints, Conventions"
weight: 2
summary: "Codify guardrails and agreements; enforce consistency."
---

# Lesson 2: Policies, Constraints, Conventions

## Why Governance?
Governance ensures systems remain secure, maintainable, and consistent as they evolve.

## Sruja: Codify Guardrails

```sruja
architecture "Platform" {
  policy Security {
    description "Security posture for services"
  }

  constraints {
    rule "No PII in logs"
    rule "Only managed Postgres for relational data"
  }

  conventions {
    naming "kebab-case for services"
    tracing "W3C trace context propagated"
  }
}
```

## Practice
- Add a policy describing your security posture.
- Capture 2–3 constraints and conventions used by your team.


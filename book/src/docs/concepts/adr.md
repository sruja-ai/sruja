---
title: "Architecture Decision Records (ADR)"
weight: 51
summary: "Capture architecture decisions directly in your model."
---

# Architecture Decision Records (ADR)

Sruja allows you to capture **Architecture Decision Records (ADRs)** directly within your architecture model. This keeps the "why" close to the "what".

## When to write an ADR

- A decision is hard to reverse (database choice, major dependency, core integration pattern)
- The team might ask "why did we do this?" in 3–6 months
- There are meaningful trade-offs (security, performance, cost, operability)
- A policy, constraint, or convention depends on the decision

## Syntax

### Defining an ADR

You can define an ADR with a full body describing the context, decision, and consequences.

```sruja
import { * } from 'sruja.ai/stdlib'


ADR001 = adr "Use PostgreSQL" {
status "Accepted"
context "We need a relational database with strong consistency guarantees."
decision "We will use PostgreSQL 15."
consequences "Good ecosystem support, but requires managing migrations."
}
```

You can also use the non-assignment form:

```sruja
import { * } from 'sruja.ai/stdlib'

adr ADR001 "Use PostgreSQL" {
  status "Accepted"
  context "We need a relational database with strong consistency guarantees."
  decision "We will use PostgreSQL 15."
  consequences "Good ecosystem support, but requires managing migrations."
}
```

## Writing strong ADRs

- Make the title a concrete choice, not a topic ("Use PostgreSQL", not "Database")
- Write context as a problem statement + constraints + decision drivers
- Write decision as a specific commitment (what you will do, and what you will not do)
- Write consequences as concrete outcomes (positive and negative), including follow-up work

### Status lifecycle

Recommended statuses:

- `Proposed`
- `Accepted`
- `Rejected`
- `Superseded`
- `Deprecated`

## Connecting ADRs to policies and scenarios

- If the decision must be enforced, encode it as a [Policy](docs/concepts/policy.md) rule.
- If the decision changes behavior, document it as a [Scenario](docs/concepts/scenario.md) or `flow` so reviewers can see the impact on interactions.

### Optional title

The title is optional; if omitted, Sruja uses the ADR id as the title.

- **ID**: Unique identifier (e.g., `ADR001`).
- **Title**: Short summary of the decision.
- **Status**: Current status (e.g., `Proposed`, `Accepted`, `Deprecated`).
- **Context**: The problem statement and background.
- **Decision**: The choice made.
- **Consequences**: The pros, cons, and implications of the decision.
- **Decision**: The choice made.
- **Consequences**: The pros, cons, and implications of the decision.

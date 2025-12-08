---
title: "Architecture Decision Records (ADR)"
weight: 10
summary: "Capture architecture decisions directly in your model."
---

# Architecture Decision Records (ADR)

Sruja allows you to capture **Architecture Decision Records (ADRs)** directly within your architecture model. This keeps the "why" close to the "what".

## Syntax

### Defining an ADR

You can define an ADR with a full body describing the context, decision, and consequences.

```sruja
adr ADR001 "Use PostgreSQL" {
    status "Accepted"
    context "We need a relational database with strong consistency guarantees."
    decision "We will use PostgreSQL 15."
    consequences "Good ecosystem support, but requires managing migrations."
}
```

### Linking ADRs

You can link an ADR to the elements it affects (System, Container, Component) by referencing its ID inside the element's block.

```sruja
system Backend "Backend API" {
    // Link to the ADR
    adr ADR001
}
```

### Optional Title

The title is optional if you are just referencing an ADR or if you want to define it later.

```sruja
adr ADR002
```

## Fields

-   **ID**: Unique identifier (e.g., `ADR001`).
-   **Title**: Short summary of the decision.
-   **Status**: Current status (e.g., `Proposed`, `Accepted`, `Deprecated`).
-   **Context**: The problem statement and background.
-   **Decision**: The choice made.
-   **Consequences**: The pros, cons, and implications of the decision.

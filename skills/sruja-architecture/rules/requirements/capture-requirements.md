# capture-requirements

## Why It Matters

Requirements in the DSL connect business and compliance needs to the architecture. Capturing them in repo.sruja enables traceability, export to docs, and later linking to elements and code.

## When to Apply

- When generating or refining repo.sruja from a requirements document or user story set
- When compliance or governance requires documented requirements (functional, non-functional, constraint, security, performance)
- When the team wants "why" documented next to "what" (architecture)

## Correct Approach

Use the Sruja requirement syntax in repo.sruja:

```sruja
R1 = requirement functional "Users must be able to log in"
R2 = requirement nonfunctional "API must respond in under 200ms p95"
R3 = requirement constraint "Must use PostgreSQL for primary store"
R4 = requirement security "All secrets in vault; no keys in code"
R5 = requirement performance "Support 10k concurrent connections"
```

- Use a stable **ID** (e.g. R1, REQ-AUTH-01) for traceability.
- Keep the **description** clear and testable where possible.
- Use requirement **types** (functional, nonfunctional, constraint, security, performance, etc.) as defined by the DSL.

Capture only requirements that affect or constrain the architecture. Do not duplicate a full backlog; focus on architecturally relevant requirements.

## Linking to Elements

After capture, link requirements to elements via tags or references (see link-requirements.md) so traceability queries ("which components implement R1?") can be answered.

## Incorrect Approach

- Inventing requirements not stated by the user or not in evidence
- Using vague descriptions that cannot be verified
- Capturing hundreds of low-level requirements that do not drive architecture

## Summary

**Capture architecturally relevant requirements in repo.sruja with clear IDs and types; link to elements for traceability.**

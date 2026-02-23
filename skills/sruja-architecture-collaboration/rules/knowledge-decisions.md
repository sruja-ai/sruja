# Knowledge: Decision Registry (ADRs)

## Description

Architecture Decision Records for capturing and tracing decisions.

## ADR Structure

```markdown
# ADR-[NUMBER]: [TITLE]

## Status
[Proposed | Accepted | Deprecated | Superseded by ADR-XXX]

## Context
[What is the issue that motivates this decision?]

## Decision
[What is the change that we're proposing/have made?]

## Consequences

### Positive
- [Benefit 1]
- [Benefit 2]

### Negative
- [Drawback 1]
- [Drawback 2]

### Risks
- [Risk 1] - Mitigation: [How addressed]

## Related
- Requirement: [REQ-ID]
- Pattern: [pattern-name]
- Component: [component-name]
- Supersedes: [ADR-XXX] (if applicable)
```

## ADR Lifecycle

```
1. PROPOSED
   - Draft created
   - Under discussion

2. ACCEPTED
   - Approved by stakeholders
   - Implemented in architecture

3. DEPRECATED
   - No longer recommended
   - Still in use but planned removal

4. SUPERSEDED
   - Replaced by newer ADR
   - Link to replacement
```

## When to Create ADR

Create an ADR when:
- Making a significant architectural choice
- Choosing between multiple options
- Decision affects multiple components
- Decision has important trade-offs
- Future team members need context

## ADR Numbering

```
ADR-001: First architecture decision
ADR-002: Second decision
ADR-003: Third decision
...

Keep sequential, don't skip numbers.
```

## Linking in .sruja

```sruja
system "My Platform" {
  metadata {
    decisions [
      "ADR-001: Microservices architecture",
      "ADR-002: Use PostgreSQL for primary data",
      "ADR-003: API Gateway pattern"
    ]
  }
}
```

## Decision Traceability

```
Requirement → ADR → Component → Pattern

Example:
FR-001 (OAuth) 
  → ADR-002 (Choose Auth0)
    → api-gateway component
      → api-gateway pattern
```

## Best Practices

- Write for future readers
- Include context that may be forgotten
- Document rejected alternatives
- Update status when changed
- Link to related ADRs

## Anti-Patterns

- ❌ No ADRs at all
- ❌ ADRs without context
- ❌ Not updating status
- ❌ Orphan ADRs (not linked)
- ❌ Too detailed or too vague

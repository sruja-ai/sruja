# Knowledge: Traceability Matrix

## Description

Links requirements to decisions to components for full traceability.

## Traceability Chain

```
Stakeholder Need
    ↓
Requirement (FR-001)
    ↓
Decision (ADR-002)
    ↓
Component (api-gateway)
    ↓
Pattern (API Gateway)
    ↓
Implementation
```

## Requirements Traceability

| Requirement | Type | Decision | Component | Status |
|-------------|------|----------|-----------|--------|
| FR-001 | Functional | ADR-002 | api-gateway | ✅ |
| FR-002 | Functional | ADR-003 | ws-server | ✅ |
| NFR-001 | Non-functional | ADR-004 | all | ⚠️ |

## Decision Traceability

| Decision | Made On | By | Status | Related |
|----------|---------|-----|--------|---------|
| ADR-001 | 2025-01-10 | architect | Accepted | FR-001, FR-002 |
| ADR-002 | 2025-01-11 | architect | Accepted | NFR-001 |
| ADR-003 | 2025-01-12 | team | Proposed | FR-003 |

## Component Traceability

| Component | Addresses | Pattern | Dependencies |
|-----------|-----------|---------|--------------|
| api-gateway | FR-001, NFR-002 | api-gateway | auth0 |
| user-service | FR-001, FR-004 | microservice | user-db |
| order-service | FR-002, FR-003 | microservice | order-db |

## Embedding in .sruja

```sruja
system "My Platform" {
  api = container "API Gateway" {
    metadata {
      addresses ["FR-001", "NFR-002"]
      decision "ADR-002"
      pattern "api-gateway"
    }
  }
}
```

## Coverage Report

```
Requirements Coverage:
- Functional: 5/5 (100%)
- Non-functional: 2/3 (67%)

Gaps:
- NFR-003: Not addressed by any component
```

## Best Practices

- Link everything bidirectionally
- Check coverage at each milestone
- Highlight gaps immediately
- Update as architecture evolves

## Anti-Patterns

- ❌ No traceability
- ❌ One-way links
- ❌ Not checking coverage
- ❌ Stale traceability

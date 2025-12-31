# Sruja DSL Roadmap

**Version:** 1.0  
**Date:** 2025-12-31  
**Status:** Active

## What Sruja Is

Sruja is an **Architecture Description Language (ADL)** for modeling software architecture. It is NOT a general-purpose programming language or data modeling language.

**Key Principle**: Keep Sruja simple and focused. If a feature belongs in OpenAPI, GraphQL, SQL, or application code, it doesn't belong in Sruja.

---

## Current Strengths

| Capability                                                            | Status                  |
| --------------------------------------------------------------------- | ----------------------- |
| Core elements (person, system, container, component, database, queue) | ✅ Solid                |
| Relationships with implied parent inference                           | ✅ Solid                |
| Views (system context, container, component, deployment)              | ✅ Solid                |
| Governance (requirements, ADRs, policies, scenarios, flows)           | ✅ **Unique advantage** |
| SLOs and scale blocks                                                 | ✅ **Industry-first**   |
| LSP support (completion, hover, go-to-definition, diagnostics)        | ✅ Solid                |
| Formatter, Validator, Multiple exports                                | ✅ Solid                |

---

## Competitive Advantage vs Structurizr DSL

| Feature                          | Structurizr   | Sruja         |
| -------------------------------- | ------------- | ------------- |
| Requirements tracking            | ❌            | ✅            |
| Policies with enforcement levels | ❌            | ✅            |
| SLOs                             | ❌            | ✅            |
| Scale blocks                     | ❌            | ✅            |
| Everything else                  | ✅ Equivalent | ✅ Equivalent |

---

## What Belongs in Sruja

✅ **Yes — Architectural concerns**:

- Components and their relationships
- System boundaries
- Deployment architecture
- Data flow (not data structure)
- Cross-cutting concerns (security, availability, scaling)
- Architecture decisions (ADRs)
- Requirements traceability

❌ **No — Use other tools**:

- API schemas → OpenAPI/Swagger
- Database schemas → SQL migrations, Prisma
- Data structures → TypeScript, Go, Java
- Validation rules → JSON Schema, application code
- Enums and types → Programming languages

---

## Next Improvements (In Priority Order)

### 1. Better Error Messages

**Effort:** 1-2 weeks | **Impact:** High

Enhance parser and validator errors with actionable guidance:

```
Error: Undefined element 'unknownService' in relation
File: shop.sruja:15:5
Help: Check if the element exists. Use `sruja tree` to see all elements.

Fix:
  User -> knownService "uses"  // Correct
```

**Why:** Critical for beginner onboarding and reduces debugging time.

---

### 2. Simplified Metadata Syntax

**Effort:** 1 week | **Impact:** Medium

Allow shorthand for common metadata:

```sruja
// Before
shop = system "Shop API" {
    metadata {
        team "platform"
        owner "platform@example.com"
    }
}

// After
shop = system "Shop API" {
    team "platform"
    owner "platform@example.com"
}
```

**Why:** Reduces nesting and improves readability.

---

### 3. Documentation Blocks

**Effort:** 1-2 weeks | **Impact:** Medium

Add inline documentation to elements:

```sruja
shop = system "Shop API" {
    doc """
    The Shop API handles all e-commerce operations.

    Owned by: Platform Team
    Tech Stack: Go, Gin, PostgreSQL
    """
}
```

**Why:** Keeps documentation with code (single source of truth).

---

## Explicitly Out of Scope

> [!CAUTION]
> Do NOT implement these features. They violate Sruja's architecture-first philosophy.

| Feature                            | Why Not                                 |
| ---------------------------------- | --------------------------------------- |
| Data fields (id, name, etc.)       | Use OpenAPI or GraphQL                  |
| Enums                              | Use programming language                |
| Optional fields (`?`)              | Use OpenAPI                             |
| Array types (`[]`)                 | Use OpenAPI                             |
| Validation constraints             | Use JSON Schema                         |
| Default values                     | Use SQL or application code             |
| Computed fields                    | Application logic, not architecture     |
| Generics                           | Use programming language                |
| Relationship cardinality (`[1:*]`) | ER diagram concern, not C4 architecture |

---

## Deferred / Not Now

These features are interesting but not currently aligned with priorities:

| Feature                   | Reason to Defer                                            |
| ------------------------- | ---------------------------------------------------------- |
| Import/Include patterns   | High complexity, most users have single-file architectures |
| Custom element kinds      | Already partially works, document when needed              |
| Tags on relationships     | Nice-to-have, not blocking anyone                          |
| Element groups/namespaces | Hierarchy already exists via systems/containers            |

---

## References

- [Language Specification](LANGUAGE_SPECIFICATION.md)
- [Design Philosophy](DESIGN_PHILOSOPHY.md)
- [Examples](../examples/)

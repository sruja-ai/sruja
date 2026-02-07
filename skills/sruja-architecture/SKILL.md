---
name: sruja-architecture
description: Architecture-as-code with Sruja DSL. Use when generating or refactoring Sruja .sruja files, designing system architecture, or making trade-off decisions (monolith vs microservices, sync vs async). Covers C4-style components, relationships, patterns, and anti-patterns.
license: MIT
---

# Sruja Architecture DSL

Comprehensive guide for software architecture design using Sruja DSL. Contains 50+ rules across 6 categories covering architectural principles, patterns, components, relationships, anti-patterns, and trade-offs. Designed to help AI assistants generate correct, well-architected Sruja DSL files.

## When to Apply

Reference these guidelines when:

- Generating Sruja architecture DSL from requirements
- Refactoring existing Sruja architectures
- Designing new software architectures
- Reviewing Sruja DSL files
- Making architectural decisions

## Rule Categories by Priority

| Priority | Category                 | Impact   | Prefix          |
| -------- | ------------------------ | -------- | --------------- |
| 1        | Architectural Principles | CRITICAL | `principle-`    |
| 2        | Component Types          | CRITICAL | `component-`    |
| 3        | Architectural Patterns   | HIGH     | `pattern-`      |
| 4        | Relationship Guidelines  | HIGH     | `relationship-` |
| 5        | Anti-Patterns            | MEDIUM   | `anti-`         |
| 6        | Trade-offs & Decisions   | MEDIUM   | `tradeoff-`     |

## Quick Reference

### 1. Architectural Principles (CRITICAL)

- `principle-separation` - Split systems by responsibility, avoid mixing concerns
- `principle-layered` - Use Presentation → Application → Domain → Infrastructure layers
- `principle-bounded-contexts` - Group related functionality into distinct contexts
- `principle-dependency-rule` - Dependencies point inward, use dependency inversion
- `principle-cohesion-coupling` - High cohesion within components, low coupling between

### 2. Component Types (CRITICAL)

- `component-person` - Use for external actors (users, systems, services)
- `component-system` - Use for major domain boundaries and subsystems
- `component-container` - Use for deployable units (APIs, workers, apps)
- `component-datastore` - Use for persistent storage and caches (DBs, message queues)

### 3. Architectural Patterns (HIGH)

- `pattern-monolith` - Single deployable unit for small teams, rapid development
- `pattern-microservices` - Independent services for multiple teams, complex domains
- `pattern-event-driven` - Event producers and consumers for real-time, loose coupling
- `pattern-cqrs` - Separate read/write models for complex queries, high throughput
- `pattern-hexagonal` - Ports and adapters for testability, clean architecture

### 4. Relationship Guidelines (HIGH)

- `relationship-synchronous` - Use for requests/responses requiring immediate feedback
- `relationship-asynchronous` - Use for events, no immediate response needed
- `relationship-labels` - Be specific: include protocols (HTTPS, gRPC) and purpose
- `relationship-direction` - Clear data flow with labels like "reads from", "writes to"

### 5. Anti-Patterns (MEDIUM)

- `anti-god-component` - Avoid single container doing everything
- `anti-direct-db-access` - Frontend should not access database directly
- `anti-circular-deps` - Extract common functionality to break cycles
- `anti-tight-coupling` - Use interfaces and abstraction layers
- `anti-orphan-components` - Every component should have relationships

### 6. Trade-offs & Decisions (MEDIUM)

- `tradeoff-monolith-vs-microservices` - Evaluate team size, domain complexity, scalability
- `tradeoff-sync-vs-async` - Consider real-time requirements, failure tolerance
- `tradeoff-security-layers` - Apply network, API, application, and data security
- `tradeoff-scaling-strategies` - Choose horizontal, vertical, or database scaling

## How to Use

Read individual rule files for detailed explanations and Sruja DSL examples:

```text
rules/principle-separation.md
rules/component-person.md
rules/pattern-monolith.md
rules/relationship-labels.md
rules/anti-god-component.md
rules/tradeoff-monolith-vs-microservices.md
```

Each rule file contains:

- Brief explanation of the principle or pattern
- When to apply it
- Sruja DSL code examples
- Common mistakes to avoid
- Related rules

## Full Compiled Document

For complete guide with all rules expanded: `AGENTS.md`

## Installation

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

## Related Skills

- `sruja-rust-best-practices` - Rust coding guidelines for Sruja core
- `sruja-validation` - Architecture validation rules and checks

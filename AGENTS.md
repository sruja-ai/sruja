---
name: sruja-architecture
description: >
  Architecture-as-code with Sruja DSL. Use when generating or refactoring Sruja
  .sruja files, designing system architecture, or making trade-off decisions.
  Covers C4-style components, relationships, patterns, and anti-patterns.
license: MIT
metadata:
  author: sruja-ai
  version: "1.0.0"
---

# Sruja Architecture DSL

Sruja focuses on **architecture and system design**—not general language coding guidelines. Use this when generating or modifying Sruja architecture DSL (.sruja files).

## When to Apply

- Generating Sruja architecture DSL from requirements
- Refactoring existing Sruja architectures
- Designing new software architectures
- Making architectural trade-off decisions (monolith vs microservices, sync vs async)

## Rule Categories

| Priority | Category                 | Impact   |
| -------- | ------------------------ | -------- |
| 1        | Architectural Principles | CRITICAL |
| 2        | Component Types          | CRITICAL |
| 3        | Architectural Patterns   | HIGH     |
| 4        | Relationship Guidelines  | HIGH     |
| 5        | Anti-Patterns            | MEDIUM   |
| 6        | Trade-offs & Decisions   | MEDIUM   |

## Full Guide

For the complete guide with all rules expanded, examples, and Sruja DSL patterns:

**`skills/sruja-architecture/AGENTS.md`**

## Quick Reference

- `principle-separation` - Split systems by responsibility
- `component-person` - External actors (users, systems)
- `component-container` - Deployable units (APIs, services)
- `component-datastore` - Databases, caches, queues
- `pattern-monolith` - Single deployable for small teams
- `pattern-microservices` - Independent services for scale
- `relationship-labels` - Be specific: protocols and purpose
- `anti-god-component` - Avoid single container doing everything
- `tradeoff-monolith-vs-microservices` - Evaluate team size, domain complexity

## Installation for Projects

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture
```

## Additional Skills

### sruja-architecture-agent

For AI-powered architecture discovery from codebases:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent
```

### sruja-architecture-collaboration

For multi-agent collaborative architecture design:

```bash
npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-collaboration
```

Enables:
- Multi-agent team roles (Analyst, Architect, Reviewer, Validator)
- Live architecture sessions
- Pattern library and ADR management
- CI/CD architecture review integration

See also: `.cursorrules`, `docs/LANGUAGE_SPECIFICATION.md`, `examples/`.

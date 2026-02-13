# Lesson 1: What is Systems Thinking?

## Learning Goals

- Understand what systems thinking is
- Recognize systems in everyday life and software
- See the connection between systems thinking and software architecture

## What is Systems Thinking?

Systems thinking is a holistic approach to understanding how components interact as part of a whole. Instead of looking at individual parts in isolation, systems thinking focuses on the relationships, patterns, and emergent behaviors that arise when components work together.

### A Simple Example

Consider a coffee shop:

**Isolated view (reductionist):**

- Coffee machine
- Barista
- Cups
- Beans
- Customers

**Systems thinking view:**

- Customer orders coffee → Barista uses machine → Machine produces coffee → Customer receives coffee → Customer might return
- The coffee machine needs beans (supply chain)
- The barista needs training (human systems)
- The shop needs location (infrastructure)
- Customer satisfaction affects future visits (feedback loop)

## Systems in Software Architecture

Every software system is a system of systems:

```
User Interface
    ↓
Application Logic
    ↓
Data Layer
    ↓
Infrastructure
```

But there's more:

- **Dependencies**: External APIs, libraries, services
- **People**: Users, developers, stakeholders
- **Processes**: Development, deployment, operations
- **Data**: Information flows, state, transactions
- **Feedback**: Monitoring, logs, user behavior

## The Iceberg Model

Systems thinking uses the "iceberg model" to understand systems:

```
Events (what you see)
    ↓
Patterns (what's happening over time)
    ↓
Structures (what's causing the patterns)
    ↓
Mental Models (what's shaping the structures)
```

In software architecture:

- **Events**: A user reports a bug
- **Patterns**: Similar bugs occur repeatedly
- **Structures**: Tightly coupled components, lack of testing
- **Mental Models**: "We need to ship fast, quality can wait"

## Why Systems Thinking in Architecture?

Traditional architecture often focuses on:

- Components and their functions
- Technology choices
- Implementation details

Systems thinking adds:

- Relationships and interactions
- Emergent behaviors
- Feedback loops
- Context and boundaries
- Flow of information

This leads to architectures that:

- Adapt to change more easily
- Handle edge cases better
- Scale more naturally
- Align with business goals

## Sruja and Systems Thinking

Sruja is built on systems thinking principles:

- **Elements as parts**: person, system, container, component
- **Relationships as interactions**: clear, labeled connections
- **Scenarios as flows**: data and information movement
- **Views as perspectives**: different angles on the same system
- **Validation as feedback**: catch problems early

## Key Takeaways

1. **Systems thinking looks at the whole**, not just parts
2. **Relationships matter as much as components**
3. **Patterns and emergent behaviors** are key insights
4. **Context is critical** - nothing exists in isolation
5. **Sruja supports systems thinking** through its language and features

## Exercise

Think of a system you work with daily (could be a codebase, a service, or a team). Identify:

- The main components (parts)
- How they interact (relationships)
- Who depends on it (context)
- Any feedback loops (how information flows back)

## Next Lesson

In [Lesson 2](lesson-2.md), we'll explore the five core systems thinking concepts you'll master in this course.

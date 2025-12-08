---
title: "Validation & Linting"
weight: 30
summary: "Use Sruja’s validator to catch cycles, orphan elements, and bad references."
tags: ["validation", "linting"]
---

# Validation & Linting

Sruja ships with a validation engine that helps keep architectures healthy.

## Linting

```bash
sruja lint architecture.sruja
```

Common checks:
- Unique IDs
- Valid references (relations must connect existing elements)
- Cycle detection (informational - cycles are valid for feedback loops, event-driven patterns, and mutual dependencies)
- Orphan detection (elements not used by any relation)
- Simplicity guidance (suggests simpler syntax when appropriate)

## Fixing Typical Errors

```sruja
system Shop {
  container Web
}
// Missing relation: add a person or downstream usage
person User
User -> Shop.Web "Visits"
```

## Understanding Validation Messages

### Cycles Are Valid

Sruja detects cycles but **doesn't block them** - cycles are valid architectural patterns:

- **Feedback loops**: User ↔ System interactions
- **Event-driven**: Service A ↔ Service B via events
- **Mutual dependencies**: Microservices that call each other
- **Bidirectional flows**: API ↔ Database (read/write)

```sruja
// This is valid - a feedback loop
User -> System "Requests"
System -> User "Responds"

// This is valid - event-driven pattern
ServiceA -> ServiceB "Sends Event"
ServiceB -> ServiceA "Responds with Event"
```

The validator will inform you about cycles but won't prevent compilation, as they're often intentional.

### Simplicity Guidance

Sruja helps you choose the right perspective:

- Use `system`, `container`, `component`, `datastore`, and `person` appropriately for your modeling goal.

These are informational messages to guide you toward the right tool for your modeling goal.

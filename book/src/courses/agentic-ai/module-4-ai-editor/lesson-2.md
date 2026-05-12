---
title: "Lesson 2: Context-Driven AI Assistance"
weight: 2
summary: "Build and optimize architecture context for AI-assisted development."
---

# Lesson 2: Context-Driven AI Assistance

## Why Context Matters

AI editors work best with relevant context. Without architecture context:
- AI makes assumptions about your system
- AI doesn't know your constraints
- AI suggests generic patterns that may not fit

With architecture context:
- AI understands your system's structure
- AI knows which components are critical
- AI provides relevant suggestions

## Building Architecture Context

### Basic Context

```bash
# Build context for current repo
sruja context -r .

# Output context as JSON
sruja context -r . --format json
```

### Cross-Repo Context

```bash
# Build context across multiple repos
sruja context -r repoA -r repoB -r repoC

# Useful for monorepos or microservices
```

### Focus Mode

For specific tasks, use focus mode to get a task-scoped briefing:

```bash
# Get context focused on a specific file/component
sruja focus --file src/api/main.rs

# Get context for a specific component
sruja focus --component PaymentService
```

Focus output includes:
- **Blast radius**: What depends on this
- **Decisions**: Why this was designed this way
- **Boundaries**: What this component owns
- **AI instructions**: How to work with this component

## Context Score

Measure how "AI-ready" your architecture is:

```bash
# Get AI-readiness score (0-100)
sruja context-score -r .
```

The score evaluates 5 dimensions:
| Dimension | What It Measures |
|-----------|-----------------|
| **Documentation** | Are components documented? |
| **Relationships** | Are dependencies explicit? |
| **Constraints** | Are policies defined? |
| **Coverage** | How much code is mapped? |
| **Freshness** | Is context up-to-date? |

## Optimizing Context for AI

### 1. Add Descriptions

```sruja
container PaymentService {
  description "Handles payment processing and fraud detection"
  // AI now understands what this does
}
```

### 2. Link Requirements

```sruja
container PaymentService {
  requirement "Must comply with PCI-DSS"
  requirement "Must process 1000 TPS"
}
```

### 3. Add Intent

```sruja
intent "Payment Goals" {
  outcome "All payments processed within 5 seconds"
}
```

### 4. Document Decisions

```sruja
PaymentService = container "Payment Service" {
  description "Handles payments"

  adr "ADR-0012: Why we use Stripe instead of PayPal"
  adr "ADR-0015: Why payments are synchronous"
}
```

## Daily Context Sync

For AI editors, run daily sync to keep context fresh:

```bash
# Quick sync
sruja sync -r .

# Full with drift check
sruja daily -r .
```

This updates:
- `.sruja/context.json`
- `.cursorrules`
- `.copilot-instructions.md`
- `CLAUDE.md`

## Next Steps

Lesson 3 covers patterns for effective AI pair programming with architecture.
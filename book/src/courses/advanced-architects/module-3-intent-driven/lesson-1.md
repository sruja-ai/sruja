---
title: "Lesson 1: Writing Architectural Intent"
weight: 1
summary: "Express desired outcomes and validate them with Sruja intent commands."
---

# Lesson 1: Writing Architectural Intent

## What Is Intent?

**Intent** is a formal declaration of what you *want* your architecture to achieve—not how to implement it, but what outcomes matter.

| Type | Example |
|------|---------|
| **Outcome** | "API latency must be < 200ms p99" |
| **Constraint** | "All data must be encrypted at rest" |
| **Convention** | "Services use event-driven communication" |

## Why Intent-First?

Traditional development:
1. Design architecture
2. Implement code
3. *Hope* it matches design

Intent-first development:
1. Write intent (what we want)
2. Design architecture to achieve intent
3. Validate that code matches intent
4. *Automatically* catch drift

## Expressing Intent with Requirements

In Sruja, you express intent using **requirements**:

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

R1 = requirement performance "API latency p99 must be less than 200ms"
R2 = requirement constraint "All data must be encrypted at rest"
R3 = requirement functional "Users must be able to authenticate via OAuth 2.0"
```

## Expressing Intent with Policies

**Policies** enforce constraints on your architecture:

```sruja
// partial
policy "Encryption Required" {
  category "security"
  enforcement "required"
  description "All databases must have encryption enabled"

  rule require metadata on { kind "database" } key "encryption" value "enabled"
}

policy "API Security" {
  category "security"
  enforcement "required"

  rule deny edge from { kind "external" } to { kind "database" }
}
```

## Intent Commands in Sruja

Sruja provides CLI commands to work with intent:

```bash
# Check intent compliance
sruja intent check -r .

# Generate intent report
sruja intent check -r . --format json > intent-report.json
```

## Linking Intent to Evidence

Map requirements to implementation evidence:

```sruja
// partial
R_Availability = requirement availability "99.9% uptime" {
  metadata {
    measured_by "health_checks.sh"
    current_value "99.95%"
    last_checked "2024-05-01"
  }
}

policy "Availability Policy" {
  description "System must maintain 99.9% uptime"

  rule require slo on { criticality "critical" }
}
```

## Intent Drift Detection

```bash
# Detect drift between code and architecture
sruja drift -r . -a repo.sruja

# Check compliance with policies
sruja compliance -r . -a repo.sruja
```

## Proposing Intent Changes

```bash
# Propose new architectural change
sruja propose create --description "Add caching layer to improve latency"

# View pending proposals
sruja propose list

# Approve a proposal
sruja propose approve --id <proposal_id>
```

## Hands-On: Define Intent for a Service

```sruja
// partial
import { * } from 'sruja.ai/stdlib'

R1 = requirement performance "API response time p99 < 150ms"
R2 = requirement availability "System uptime > 99.9%"
R3 = requirement constraint "All PII must be encrypted"
R4 = requirement functional "Support OAuth 2.0 authentication"

policy "Performance Policy" {
  category "performance"
  enforcement "required"

  rule require metadata on { kind "container" } key "latency_target"
}

policy "Security Policy" {
  category "security"
  enforcement "required"

  rule deny edge from { tag "public" } to { kind "database" } except { kind "container" tag "trusted" }
}
```

## Learning Outcomes

- ✅ Understand what architectural intent is and why it matters
- ✅ Express intent using requirements and policies in Sruja DSL
- ✅ Use `sruja intent check` and `sruja compliance` to validate intent
- ✅ Detect drift using `sruja drift`

## Quiz: Test Your Understanding

### Q1: What is architectural intent?

A) A detailed implementation plan
B) A formal declaration of what you want your architecture to achieve
C) A list of all components
D) A network diagram

### Q2: What command checks intent compliance?

A) `sruja lint`
B) `sruja intent check`
C) `sruja drift`
D) `sruja validate`

### Q3: What does `sruja drift` help detect?

A) Git conflicts
B) Changes between code and architecture
C) Network failures
D) Database issues

## Next Steps

Lesson 2 covers how to let AI challenge your intent and architecture decisions.
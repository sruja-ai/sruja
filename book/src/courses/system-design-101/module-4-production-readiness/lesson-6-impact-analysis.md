---
title: "Lesson 6 - Impact Analysis"
weight: 6
summary: "Analyze the blast radius of architectural changes using Sruja impact analysis."
---

# Lesson 6 - Impact Analysis

**Analyze the blast radius of changes before you make them.**

## Overview

This lesson teaches impact analysis—understanding what will break before you change something. This is a critical skill for production readiness: knowing the consequences of your changes.

## The Problem: Change Without Understanding

How many times has this happened?

- You changed a "simple" function
- 3 other services broke
- Incident opened, blamed on your change
- "Why didn't anyone know this was connected?"

## Solution: Impact Analysis

Before changing anything, run:

```bash
# What's affected by changing UserService?
sruja impact UserService -r . --depth 3
```

Output shows:
- **Downstream**: What depends on UserService
- **Upstream**: What UserService depends on
- **Blast radius**: How many components are affected

## Using Impact Analysis

### Basic Impact Query

```bash
# Show what depends on UserService
sruja impact UserService -r . --depth 3

# Output:
# Component: UserService
# ├── Downstream (3)
# │   ├── OrderProcessor → calls UserService
# │   ├── ProfileUI → uses UserService.getProfile()
# │   └── NotificationService → subscribes to user.* events
# └── Blast Radius: HIGH (3 critical paths)
```

### Before Making Changes

```bash
# Planning to remove LegacyAuth
sruja impact LegacyAuth -r . --depth 5

# Shows:
# - All components that call LegacyAuth
# - All tests that depend on it
# - Migration steps needed
```

### Using with Code Review

```bash
# In PR, comment with impact
sruja impact --diff --file changed-file.rs

# "This change affects:
# - PaymentService (critical)
# - 12 downstream tests
# Consider: updating PaymentContract first"
```

## Impact Analysis in Sruja DSL

### Tag Components for Tracking

```sruja
<!-- partial -->
PaymentService = container "Payment Service" {
  impact {
    tier "critical"  # Tiers: critical, high, medium, low
    blast_radius "high"
    change_frequency "low"
  }
}
```

### Define Impact Policies

```sruja
<!-- partial -->
policy "Impact-Aware Changes" {
  description "Changes to critical components require extra review"

  constraint "Critical components need blast radius analysis" {
    rule "changes to tier='critical' require impact report"
  }

  constraint "High blast radius requires approval" {
    rule "changes with blast_radius='high' require architect review"
  }
}
```

## Real-World Example

You're asked to: *"Refactor UserService to use new auth library"*

Without impact analysis:
1. Make the change
2. Push to CI
3. 3 services fail
4. Rollback, fix, incident postmortem

With impact analysis:
```bash
$ sruja impact UserService -r . --depth 3

Impact Report: UserService Refactor
===================================
Downstream Dependencies:
  - order-service::OrderProcessor
  - profile-service::ProfileAPI
  - notification-service::UserEventHandler

Estimated Blast Radius:
  - 5 services affected
  - 47 tests may need updates
  - 2 API contracts may break

Migration Steps:
  1. Update PaymentService to use new auth interface
  2. Update order-service contract
  3. Run integration tests
  4. Deploy in order: profile → notification → order
```

Now you know:
- What to update before touching UserService
- What tests to run
- What order to deploy

## CI/CD Integration

```yaml
# .github/workflows/prevent-incidents.yml
- name: Impact Analysis
  run: |
    # Get list of changed components
    CHANGED=$(git diff --name-only)

    for component in $CHANGED; do
      echo "Analyzing impact of $component..."
      sruja impact --file $component -r . --json > impact-$component.json

      # Fail if critical impact
      if grep -q '"blast_radius": "critical"' impact-$component.json; then
        echo "ERROR: Critical blast radius detected for $component"
        exit 1
      fi
    done
```

## Advanced: Dependency Graph Analysis

```bash
# Show full dependency path
sruja impact UserService -r . --graph

# Find circular dependencies
sruja impact --find-cycles

# Show all paths to critical service
sruja impact PaymentService -r . --upstream --depth 10
```

## Learning Outcomes

- ✅ Use `sruja impact` to analyze blast radius
- ✅ Understand downstream vs upstream dependencies
- ✅ Integrate impact analysis into change workflow
- ✅ Use impact data for risk assessment

## When to Run Impact Analysis

| Situation | Run Impact? |
|-----------|-------------|
| Before major refactor | ✅ Always |
| Before removing a component | ✅ Always |
| Before changing an API contract | ✅ Always |
| After incident | ✅ Understand dependencies |
| Code review | ✅ With `--diff` flag |
| Pre-commit | ✅ Quick check |

## Next Steps

This lesson completes the Production Readiness module. You've learned:
- ✅ Documenting decisions (ADRs)
- ✅ Health check endpoints
- ✅ Metrics and monitoring
- ✅ SLOs and error budgets
- ✅ Chaos engineering
- ✅ Impact analysis

## Raw Notes for Enhancement

### v0.20.0 Feature:
- `sruja impact <component>` - impact analysis command
- `--depth` flag for traversal depth
- Blast radius visualization

### Could be added as:
- **System Design 101**: Module 4 - Production Readiness, Lesson 6
- **System Design 201**: Could be a standalone lesson in Module 4 (Consistency)
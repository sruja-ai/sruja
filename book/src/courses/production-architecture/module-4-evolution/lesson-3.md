---
title: "Lesson 3: Continuous Evolution Workflow"
weight: 3
summary: "Integrating architectural health into development with maturation phases and autonomous optimization."
---

# Lesson 3: Continuous Evolution Workflow

## The Evolution Loop

From v0.42.0, Sruja provides an **evolutionary workflow** that continuously monitors and improves architecture:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Instrumentation│ → │ Context Hints│ → │Infra Discovery│
└─────────────┘    └─────────────┘    └─────────────┘
        ↓                ↓                  ↓
   Measure health    Suggest improvements  Auto-optimize
```

## Maturation Phases

### Phase 1: Instrumentation
Start by measuring what you have:

```bash
sruja discover -r . --context
```

This builds the initial architecture graph.

### Phase 2: Context Hints
Add human knowledge to guide evolution:

```bash
sruja intent propose -r . --from-diff
```

### Phase 3: Infrastructure Discovery
Let Sruja find infrastructure patterns:

```bash
sruja scan -r . --infra
```

## The Evolution Command

```bash
# Run evolution analysis
sruja evolution -r .

# Show recommended changes
sruja evolution -r . --propose

# Apply recommended changes
sruja evolution -r . --fix
```

## Drift Detection and Resolution

```bash
# Detect drift
sruja drift -r .

# Auto-fix where possible
sruja drift -r . --fix

# Show blast radius of changes
sruja impact <component> -r . --depth 3
```

## Agentic Memory (v0.37.0)

From v0.37.0, Sruja includes **agentic memory**—autonomous optimization that learns from past decisions:

```bash
# Enable autonomous mode
sruja agent run -r . --autonomous

# Review agent suggestions
sruja agent review -r .
```

The agent:
- Remembers past architectural decisions
- Learns from team preferences
- Proposes optimizations based on patterns
- Evaluates fitness of changes before applying

## Integrating into CI/CD

```yaml
# .github/workflows/evolution.yml
- name: Check Architecture Health
  run: |
    sruja health -r . --format json > health.json
    sruja drift -r . --fail-on-drift

- name: Evolution Check (daily)
  if: github.event_name == 'schedule'
  run: sruja evolution -r . --propose
```

## Summary

The continuous evolution workflow:
1. **Instrument**: Build architecture graph
2. **Measure**: Run health checks
3. **Detect**: Find drift with `sruja drift`
4. **Propose**: Get evolution suggestions
5. **Apply**: Use `sruja evolution --fix`
6. **Learn**: Agentic memory improves over time

## Module Complete!

You've completed the Evolutionary Architecture module. You now understand:
- ✅ Fitness functions and how to define them
- ✅ Health scoring and community detection
- ✅ Continuous evolution workflow with maturation phases
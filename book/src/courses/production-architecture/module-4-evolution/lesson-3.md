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
sruja intent history -r .

# Show recommended changes
sruja intent history -r . --propose

# Apply recommended changes
sruja intent history -r . --fix
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

## Agentic memory and bounded agent loop

Sruja records **architecture-bounded learnings** in `.sruja/agent_memory.json` and supports a **review-before-apply** agent loop. Sruja is a harness—not a full autonomous coding agent; your editor or CI hosts the LLM.

```bash
# Emit a reviewable plan grounded in repo evidence
sruja agent plan -r . --goal "Reduce drift on API layer" --file src/api/main.rs --print

# After human or CI review, apply the plan
sruja agent apply -r . --plan docs/plans/<run-id>.json

# Optional: parallel sandbox trajectories (MaTTS) when worktrees are available
sruja agent run -r . --goal "..." --file src/api/main.rs --mode apply --trajectories 3

# Inspect and curate learnings
sruja agent history -r .
sruja agent curate -r .
```

Enable automatic learning capture in `.sruja/config.toml`:

```toml
[agent]
auto_record_learnings = true
```

See [Grounded harness and continual learning](../../../../docs/GROUNDED_HARNESS_AND_CONTINUAL_LEARNING.md) for the full host-vs-harness model.

## Integrating into CI/CD

```yaml
# .github/workflows/evolution.yml
- name: Check Architecture Health
  run: |
    sruja health -r . --format json > health.json
    sruja drift -r . --fail-on-drift

- name: Evolution Check (daily)
  if: github.event_name == 'schedule'
  run: sruja intent history -r . --propose
```

## Summary

The continuous evolution workflow:
1. **Instrument**: Build architecture graph
2. **Measure**: Run health checks
3. **Detect**: Find drift with `sruja drift`
4. **Propose**: Get evolution suggestions
5. **Apply**: Use `sruja intent history --fix`
6. **Learn**: Agentic memory improves over time

## Module Complete!

You've completed the Evolutionary Architecture module. You now understand:
- ✅ Fitness functions and how to define them
- ✅ Health scoring and community detection
- ✅ Continuous evolution workflow with maturation phases
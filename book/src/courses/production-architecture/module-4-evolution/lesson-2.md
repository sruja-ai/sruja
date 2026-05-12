---
title: "Lesson 2: Architectural Health Scoring"
weight: 2
summary: "Quantifying and interpreting system health using Sruja health metrics."
---

# Lesson 2: Architectural Health Scoring

## Understanding Health Scores

Sruja calculates a composite **health score** (0-100) based on multiple architectural dimensions:

| Component | Weight | What It Measures |
|-----------|--------|------------------|
| **Complexity** | 25% | Cyclomatic complexity, nesting depth |
| **Coupling** | 25% | Dependencies between components |
| **Coverage** | 20% | Architecture documentation completeness |
| **Governance** | 15% | Policy compliance, constraint violations |
| **Evolution** | 15% | Drift from intended architecture |

## Running Health Check

```bash
# Check overall health
sruja health -r .

# Check with detailed output
sruja health -r . --verbose

# Check specific area
sruja health -r . --focus payments
```

## Interpreting the Dashboard

The health dashboard shows:

1. **Overall Score**: Big number (e.g., 87/100)
2. **Component Breakdown**: How each dimension contributes
3. **Trend Arrow**: Is health improving or declining?
4. **Top Issues**: Priority list of problems to fix
5. **Community Graph**: Visual of tightly coupled components

## Community Detection

From v0.41.0, Sruja includes community detection—automatically identifying groups of tightly coupled components:

```bash
sruja health -r . --show-communities
```

This helps answer:
- Which services should be deployed together?
- Where are the natural boundaries?
- What changed in the coupling pattern?

## Configuring Thresholds

```yaml
# .sruja/config.yaml
health:
  thresholds:
    overall:
      warning: 70
      critical: 50
    complexity:
      warning: 60
      critical: 40
    coupling:
      warning: 65
      critical: 45
```

## Exercise: Analyze Health of Your System

1. Run `sruja health -r .`
2. Note your overall score
3. Identify the lowest-scoring dimension
4. Run `sruja drift -r .` to see what's causing drift
5. Create a plan to improve the lowest area

## Next Steps

Lesson 3 covers integrating health monitoring into your continuous evolution workflow.
---
title: "Draft: Module 3 - Intent-Driven Development"
weight: 3
summary: "Write intent, use critique engine, and perform adversarial architectural review."
---

# Draft: Module 3 - Intent-Driven Development

**Writing architectural intent, adversarial critique, and evidence-based validation.**

## Overview

This module teaches intent-driven development with Sruja—expressing what you *want* the architecture to achieve, then validating that reality matches intent.

## Lessons

### Lesson 1: [Writing Architectural Intent](lesson-1.md)
_Expressing desired outcomes as formal intent_

- Intent syntax and structure
- Outcome-driven requirements
- Linking intent to evidence
- Intent provenance tracking

### Lesson 2: [Critique Engine & Adversarial Review](lesson-2.md)
_Let AI challenge your architecture decisions_

- The "Angry Agent" critique engine
- Running adversarial review
- Interpreting critique results
- Responding to critique

### Lesson 3: [Evidence Mapping & Compliance](lesson-3.md)
_Prove your architecture meets intent_

- Evidence collection
- Intent validation against code
- Compliance reporting
- Drift detection and resolution

## Learning Outcomes

- ✅ Write formal architectural intent
- ✅ Use critique engine to challenge decisions
- ✅ Map evidence from code to intent
- ✅ Validate compliance in CI/CD

## Prerequisites

- Completed Advanced Architects Modules 1-2
- Familiarity with Sruja policies and contracts
- Understanding of CI/CD workflows

## Estimated Time

2-3 hours

---

## Raw Thoughts from Analysis

### v0.34.0, v0.35.0 Features to Cover:
- `intent` command and domain
- Phase 4: Adversarial Critique Engine ("Angry Agent")
- Phase 3: Dynamic intent check and evidence mapping
- `sruja propose` command
- `sruja review` command

### Related Concepts:
- Intent-first development
- Architectural rationale (ADR)
- Critique-driven iteration
- Evidence-based validation
- Compliance and audit trails

### Key Commands:
- `sruja intent propose -r . --from-diff`
- `sruja intent check -r .`
- `sruja critique -r .`
- `sruja review -r .`
- `sruja propose -r .`

### Lesson Ideas:

1. **Writing Architectural Intent**
   - Intent syntax: what you want, not how
   - Outcome vs constraint intent
   - Linking to ADR/rationale
   - Example: "API latency must be < 200ms" vs "We use caching"

2. **Critique Engine**
   - The adversarial agent ("Angry Agent")
   - What it challenges: assumptions, trade-offs, risks
   - Running `sruja critique`
   - Interpreting critique report

3. **Evidence Mapping**
   - Collecting evidence from code
   - Mapping to intent requirements
   - Validation reporting
   - Drift detection
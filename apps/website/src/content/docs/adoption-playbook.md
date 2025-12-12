---
title: "Adoption Playbook"
weight: 22
summary: "Practical steps to roll out Sruja across teams and CI."
---

# Adoption Playbook

## Week 1: Baseline & CI
- Create a minimal `architecture.sruja` covering core systems.
- Add `sruja fmt` and `sruja lint` to CI; fail on violations.
- Export docs: `sruja export markdown architecture.sruja`.

## Week 2: Targets & Guardrails
- Add `slo` and `scale` for critical paths.
- Encode `constraints` and `conventions`; publish to teams.
- Introduce `views` for API/Data/Auth focus.

## Week 3: Governance & Evolution
- Add `policy` pages for security/operability.
- Track `change` and `snapshot` for versioned evolution.
- Wire linting to PR checks; require green builds.

## CI Example (GitHub Actions)
```yaml
name: sruja
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Lint DSL
        run: sruja lint architecture.sruja
      - name: Export Docs
        run: sruja export markdown architecture.sruja
```

## Success Metrics
- Review cycle time ↓
- Incident rate for architecture errors ↓
- Consistency across services ↑

---

**Note**: Sruja is **free and open source** (MIT licensed). Need help with implementation? Professional consulting services are available. Contact the team through [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions) to learn more.


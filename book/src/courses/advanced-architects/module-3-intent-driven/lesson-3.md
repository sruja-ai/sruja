---
title: "Lesson 3: Evidence Mapping & Compliance"
weight: 3
summary: "Prove architecture meets intent through evidence-based validation."
---

# Lesson 3: Evidence Mapping & Compliance

## Intent vs Reality

Writing intent is the *easy* part. The hard part is proving that reality matches intent.

**Evidence mapping** connects code/artifacts to intent requirements, creating a traceable chain:

```
Intent → Requirement → Implementation → Evidence
```

## The Evidence Mapping Workflow

```bash
# 1. Check intent compliance
sruja intent check -r .

# 2. Generate evidence report
sruja intent check -r . --format json > evidence.json

# 3. View gaps
sruja intent check -r . --show-gaps
```

## Evidence Collection

Evidence can come from multiple sources:

| Source | Example |
|--------|---------|
| **Code** | `grep -r "encryption" --include="*.py"` |
| **Config** | `database.encryption = true` |
| **Tests** | `test_payment_latency.py` |
| **Metrics** | APM dashboards |
| **Docs** | Architecture decision records |

## Automated Evidence Collection

```bash
# Collect from code
sruja scan -r . --evidence

# Collect from infrastructure
sruja scan -r . --infra --evidence

# Collect from tests
sruja scan -r . --tests --evidence
```

## Mapping Evidence to Intent

```sruja
intent "Data Privacy" {
  description "Customer PII must be protected"

  requirement encryption_at_rest {
    description "All databases must encrypt data"
    evidence {
      source "config/database.yml"
      assertion "encryption == true"
    }
  }

  requirement pii_masking {
    description "PII must be masked in logs"
    evidence {
      source "config/logging.yml"
      assertion "mask_pii == true"
      source "tests/pii_masking_test.py"
      assertion "test_passes"
    }
  }
}
```

## Compliance Reporting

```bash
# Generate compliance report
sruja intent check -r . --report

# Show compliance for specific intent
sruja intent check -r . --intent "Data Privacy" --report

# Export for audit
sruja intent check -r . --format json --output compliance-audit-2024-05-12.json
```

## Drift Detection

```bash
# Check for drift
sruja drift -r .

# Drift between intent and code
sruja drift -r . --intent

# Auto-fix drift
sruja drift -r . --fix
```

## CI/CD Integration

```yaml
# .github/workflows/compliance.yml
- name: Intent Compliance Check
  run: |
    sruja intent check -r . --fail-on-gaps

- name: Evidence Collection
  run: |
    sruja scan -r . --evidence --output evidence.json

- name: Drift Detection
  run: |
    sruja drift -r . --fail-on-drift
```

## Module Complete!

You've completed the Intent-Driven Development module. You now understand:
- ✅ Writing formal architectural intent
- ✅ Using critique engine for adversarial review
- ✅ Mapping evidence from code to requirements
- ✅ Proving compliance through automated validation

This module completes the Advanced Architects course with intent-first development skills.
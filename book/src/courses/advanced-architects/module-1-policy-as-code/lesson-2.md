---
title: "Lesson 2: Enforcing Policies in CI/CD"
weight: 2
summary: "Integrate architectural validation into your CI/CD pipeline to prevent violations automatically."
---

# Lesson 2: Enforcing Policies in CI/CD

## The Goal: Automatic Enforcement

Policies are useless if they're not enforced. This lesson shows you how to integrate Sruja validation into CI/CD so violations are caught **before** they reach production.

## Basic CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/architecture.yml
name: Architecture Validation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Sruja
        run: cargo install sruja-cli --git https://github.com/sruja-ai/sruja --locked
      
      - name: Validate Architecture
        run: |
          sruja fmt architecture.sruja
          sruja lint architecture.sruja
      
      - name: Check Constraints
        run: sruja validate architecture.sruja
      
      - name: Export Documentation
        run: sruja export markdown architecture.sruja > architecture.md
```

### GitLab CI

```yaml
# .gitlab-ci.yml
architecture-validation:
  image: alpine:latest
  before_script:
    - apk add --no-cache rust cargo
    - cargo install sruja-cli --git https://github.com/sruja-ai/sruja --locked
    - export PATH="$HOME/.cargo/bin:$PATH"
  script:
    - sruja fmt architecture.sruja
    - sruja lint architecture.sruja
    - sruja validate architecture.sruja
  only:
    - merge_requests
    - main
```

## Advanced: Policy Violation Reporting

Generate compliance reports in CI/CD:

```yaml
- name: Generate Compliance Report
  run: |
    sruja validate architecture.sruja --format-json > violations.json
    sruja compliance -r . -a architecture.sruja -f json > compliance.json
  
- name: Upload Reports
  uses: actions/upload-artifact@v3
  with:
    name: architecture-reports
    path: |
      violations.json
      compliance.json
      architecture.md
```

## Multi-Repository Governance

For organizations with multiple repositories, create a shared policy file:

```yaml
# .github/workflows/architecture.yml
- name: Validate Against Shared Policies
  run: |
    # Fetch shared policies from central repo
    git clone https://github.com/your-org/architecture-policies.git /tmp/policies
    
    # Validate architecture and optional external constraint files
    sruja validate architecture.sruja -c /tmp/policies/global-constraints.sruja
```

## Pre-commit Hooks

Catch violations before they're committed:

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Install Sruja if not available
if ! command -v sruja &> /dev/null; then
  cargo install sruja-cli --git https://github.com/sruja-ai/sruja --locked
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Validate architecture
sruja lint architecture.sruja
if [ $? -ne 0 ]; then
  echo "❌ Architecture validation failed. Fix errors before committing."
  exit 1
fi

sruja validate architecture.sruja
if [ $? -ne 0 ]; then
  echo "❌ Constraint violations found. Fix before committing."
  exit 1
fi

echo "✅ Architecture validation passed"
exit 0
```

## Integration with PR Reviews

Add architecture validation as a required check:

```yaml
- name: Architecture Gate
  run: |
    sruja validate architecture.sruja --fail-on-violations
```

**Result:** PRs can't be merged until architecture is valid.

## Monitoring Compliance

Track compliance over time:

```yaml
- name: Track Compliance Metrics
  run: |
    sruja compliance -r . -a architecture.sruja -f json > compliance-metrics.json
    
    # Send to monitoring system
    curl -X POST https://your-monitoring-system/api/metrics \
      -H "Content-Type: application/json" \
      -d @compliance-metrics.json
```

## Key Takeaways

1. **Integrate early** — Validate in CI/CD, not manually
2. **Fail fast** — Block merges on violations
3. **Report compliance** — Track metrics over time
4. **Share policies** — Use central policy files for multi-repo orgs
5. **Pre-commit hooks** — Catch issues before they're committed

## Real-World Pattern

**Large organization pattern:**

```yaml
# Central policy repository
architecture-policies/
  ├── global-constraints.sruja    # Organization-wide rules
  ├── team-payment.sruja          # Team-specific rules
  └── compliance-hipaa.sruja      # Compliance requirements

# Each service repository
service-repo/
  ├── architecture.sruja          # Service architecture
  └── .github/workflows/
      └── architecture.yml        # Validates against shared policies
```

## Next Steps

- Set up CI/CD validation for your architecture
- Create shared policy files for your organization
- Add pre-commit hooks for faster feedback
- Track compliance metrics over time

**You now know how to enforce policies automatically. Governance at scale! 🚀**

---
title: "Lesson 3: Governance Automation"
weight: 3
summary: "Automate governance with CI/CD pipelines."
---

# Lesson 3: Governance Automation

## CI/CD Integration

Automate governance checks in your pipeline:

```yaml
# .github/workflows/governance.yml
name: Architecture Governance

on:
  push:
    branches: [main]
  pull_request:
    types: [opened, synchronize]

jobs:
  governance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash

      - name: Validate Policies
        run: |
          sruja validate -r . \
            --policies ../federation/policies/global.sruja

      - name: Check Compliance
        run: |
          sruja compliance -r . \
            --output compliance.json

      - name: Fail on Critical Violations
        if: contains(steps.compliance.outputs.level, 'critical')
        run: |
          echo "Critical policy violations found"
          exit 1
```

## Automated Remediation

```bash
# Auto-fix where possible
sruja drift -r . --fix

# For policy violations that can be auto-fixed
sruja policy -r . --auto-fix

# Review auto-fixes before committing
sruja policy -r . --auto-fix --dry-run
```

## Governance Metrics

Track governance health over time:

```bash
# Track governance score
sruja metrics --governance --output metrics.json

# Dashboard
sruja metrics --governance --dashboard
```

## Gatekeeping

Prevent non-compliant repos from publishing:

```yaml
# .github/workflows/publish-bundle.yml
jobs:
  publish:
    needs: governance
    runs-on: ubuntu-latest
    steps:
      - name: Publish Bundle
        if: needs.governance.outputs.compliant == 'true'
        run: sruja publish -r . -o bundle.json

      - name: Block Non-Compliant
        if: needs.governance.outputs.compliant != 'true'
        run: |
          echo "Cannot publish: governance failures"
          exit 1
```

## Policy Distribution

```bash
# Push policies to repos
sruja policy push --policy global.sruja \
  --to ./repos/*/

# Pull policy updates
sruja policy pull --from ../federation/policies/
```

## Governance Dashboard

```bash
# View all repos governance status
sruja governance --dashboard --bundles ./bundles/

# Shows:
# - Repo compliance matrix
# - Policy violation trends
# - Team health scores
# - Action items
```

## Course Complete!

You've completed the Federated Architecture course. You now understand:
- ✅ Federation fundamentals and bundle publishing
- ✅ Cross-repo relationship modeling
- ✅ Conflict detection and resolution
- ✅ Federated governance and automation

## Next Steps

- Apply federation to your organization's repos
- Set up automated governance in CI/CD
- Train teams on federated architecture concepts
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

      - name: Check Compliance
        run: sruja compliance -r . -a repo.sruja

      - name: Check for Drift
        run: sruja drift --ci -r . -a repo.sruja
```

## Automated Remediation

```bash
# Check drift with fix suggestions
sruja drift -r . -a repo.sruja

# For auto-fix capabilities, update the architecture to match code
# sruja init --auto  # writes repo.sruja.draft; author repo.sruja with skill
```

## Governance Metrics

Track governance health over time:

```bash
# Check health
sruja health -r .

# With JSON output for tracking
sruja health -r . --format json

# Context score for AI-readiness
sruja context-score -r .
```

## Gatekeeping

Prevent drift from accumulating:

```yaml
# .github/workflows/quality-gate.yml
jobs:
  quality-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Architecture Check
        run: |
          # Lint the DSL
          sruja lint repo.sruja

          # Check drift
          sruja drift --ci -r . -a repo.sruja
```

## Policy Distribution

Sruja policies are defined in `repo.sruja` or separate `.sruja` files. To share policies across repos:

1. **Define policies in a central repo:**
   ```sruja
   // In federation-policies repo
   policy "Global Security" {
     constraint "All external endpoints must use HTTPS" {
       applies_to: container
     }
   }
   ```

2. **Reference from individual repos:**
   ```sruja
   import { Global Security } from "../federation-policies/policies.sruja"
   ```

## Governance Dashboard

```bash
# View overall health
sruja health -r .

# Generate AI context for review
sruja ai-context -r .
```

## Hands-On: Automate Governance

1. **Add to your CI/CD pipeline:**
   ```yaml
   - name: Architecture Governance
     run: |
       sruja lint repo.sruja
       sruja drift --ci -r . -a repo.sruja
   ```

2. **Run health check:**
   ```bash
   sruja health -r .
   ```

3. **Generate compliance report:**
   ```bash
   sruja compliance -r . -a repo.sruja
   ```

4. **Check context score:**
   ```bash
   sruja context-score -r .
   ```

## Learning Outcomes

- ✅ Integrate governance checks into CI/CD pipelines
- ✅ Use `sruja drift --ci` for automated drift detection
- ✅ Track governance health with `sruja health`
- ✅ Share policies across repos using imports

## Quiz: Test Your Understanding

### Q1: What command runs architecture checks in CI mode?

A) `sruja check`
B) `sruja drift --ci`
C) `sruja validate`
D) `sruja test`

### Q2: How can you share policies across multiple repos?

A) Copy-paste the policy code
B) Use import statements to reference external `.sruja` files
C) Store policies in a database
D) Use environment variables

### Q3: What does `sruja context-score` measure?

A) Network latency
B) Code coverage
C) AI-readiness of the codebase (0-100 score)
D) Server uptime

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
- Use `sruja ai-context` to provide architecture context to AI editors
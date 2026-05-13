---
title: "Lesson 2: Compliance Reporting"
weight: 2
summary: "Generate cross-repo compliance reports."
---

# Lesson 2: Compliance Reporting

## Building Compliance Reports

```bash
# Generate compliance report
sruja compliance -r . -a repo.sruja

# With JSON output for automation
sruja compliance -r . -a repo.sruja --format json
```

## Report Output

When you run `sruja compliance`, it shows:
- **Policy violations** found in your architecture
- **Structural drift** between code and architecture
- **Intent violations** from ADRs and documented intent
- **Overall compliance status** (pass/fail)

## Per-Repo Compliance

```bash
# Check compliance for specific repo
sruja compliance -r ./user-service -a user-service/repo.sruja

# Check with output format
sruja compliance -r . -a repo.sruja --format json
```

## Drift Reporting

```bash
# Show drift between code and architecture
sruja drift -r . -a repo.sruja

# In CI mode (GitHub Actions format)
sruja drift --ci -r . -a repo.sruja

# Show only violations
sruja drift -r . -a repo.sruja --violations-only
```

## Health Reporting

```bash
# Check overall health
sruja health -r .

# With architecture file
sruja health -r . -a repo.sruja

# JSON format for dashboards
sruja health -r . -a repo.sruja --format json
```

## Scheduled Reports

```yaml
# .github/workflows/architecture-report.yml
on:
  schedule:
    - cron: '0 8 * * 1'  # Weekly Monday 8am

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check Compliance
        run: sruja compliance -r . -a repo.sruja --format json > compliance.json

      - name: Check Health
        run: sruja health -r . -a repo.sruja --format json > health.json

      - name: Upload Reports
        uses: actions/upload-artifact@v4
        with:
          name: weekly-architecture-reports
          path: |
            compliance.json
            health.json
```

## Hands-On: Generate Compliance Reports

1. **Run compliance check:**
   ```bash
   sruja compliance -r . -a repo.sruja
   ```

2. **Run health check:**
   ```bash
   sruja health -r .
   ```

3. **Check for drift:**
   ```bash
   sruja drift -r . -a repo.sruja
   ```

4. **View in CI mode:**
   ```bash
   sruja drift --ci -r . -a repo.sruja
   ```

## Learning Outcomes

- ✅ Generate compliance reports using `sruja compliance`
- ✅ Check health with `sruja health`
- ✅ Detect drift using `sruja drift`
- ✅ Integrate reports into CI/CD pipelines

## Quiz: Test Your Understanding

### Q1: What command generates a compliance report?

A) `sruja report`
B) `sruja compliance`
C) `sruja audit`
D) `sruja check`

### Q2: What does `sruja drift --ci` do?

A) Runs in CI mode with GitHub Actions format output
B) Checks Git CI/CD pipeline status
C) Creates a new CI/CD workflow
D) Uploads to continuous integration service

### Q3: What does `sruja health` show?

A) Server health metrics
B) Overall architectural health and graph quality
C) Network connectivity status
D) Database health

## Module Complete!

You've completed Federated Governance. You now understand:
- ✅ Federated policy definition
- ✅ Policy inheritance and overrides
- ✅ Compliance reporting
- ✅ Governance automation

Course complete!
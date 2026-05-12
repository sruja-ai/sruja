---
title: "Lesson 2: Compliance Reporting"
weight: 2
summary: "Generate cross-repo compliance reports."
---

# Lesson 2: Compliance Reporting

## Building Compliance Reports

```bash
# Generate report for all repos
sruja compliance --bundles ./bundles/ \
  --output compliance-report.json

# Or use system index
sruja compliance --index system.index.json \
  --output compliance-report.json
```

## Report Structure

```json
{
  "generated_at": "2024-05-12T10:00:00Z",
  "scope": "all-repos",
  "summary": {
    "total_repos": 25,
    "compliant": 22,
    "partial": 2,
    "non_compliant": 1
  },
  "by_policy": [
    {
      "policy": "Global Security",
      "compliant": 24,
      "violations": 1,
      "repos": ["order-service"]
    }
  ],
  "by_repo": [
    {
      "repo_id": "user-service",
      "status": "compliant",
      "violations": [],
      "last_ checked": "2024-05-12"
    },
    {
      "repo_id": "order-service",
      "status": "non_compliant",
      "violations": [
        {
          "policy": "Global Security",
          "constraint": "All databases must have encryption",
          "element": "order-service::OrderDB",
          "severity": "high"
        }
      ]
    }
  ]
}
```

## Per-Team Reports

```bash
# Report for specific team
sruja compliance --team platform-team \
  --output team-compliance.json

# Report by domain
sruja compliance --domain payments \
  --output payments-compliance.json
```

## Drift Reporting

```bash
# Show drift across repos
sruja drift --bundles ./bundles/ \
  --output drift-report.json
```

## Dashboard Generation

```bash
# Generate HTML dashboard
sruja compliance --bundles ./bundles/ \
  --dashboard \
  --output compliance-dashboard.html
```

## Scheduled Reports

```yaml
# .github/workflows/compliance-report.yml
on:
  schedule:
    - cron: '0 8 * * 1'  # Weekly Monday 8am

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - name: Fetch bundles
        run: sruja fetch-bundles -i ./bundles/

      - name: Generate report
        run: sruja compliance --index system.index.json \
          --output compliance-$(date +%Y-%m-%d).json

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: weekly-compliance
          path: compliance-*.json

      - name: Notify
        if: failure()
        run: |
          sruja alert --slack "#architecture" \
            --message "Compliance report failed"
```

## Module Complete!

You've completed Federated Governance. You now understand:
- ✅ Federated policy definition
- ✅ Policy inheritance and overrides
- ✅ Compliance reporting
- ✅ Governance automation

Course complete!
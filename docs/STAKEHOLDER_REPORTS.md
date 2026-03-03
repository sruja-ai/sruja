# Sruja - Stakeholder Reports

Sruja provides tailored architecture analysis for different stakeholders in your organization using configurable views.

## Commands

You can access built-in reports for different roles using the `analyze` command and the `--view` flag:

```bash
# CTO Executive Summary
sruja analyze -r /path/to/repo --view cto

# SRE Reliability Report
sruja analyze -r /path/to/repo --view sre

# DevOps Deployment Readiness
sruja analyze -r /path/to/repo --view devops

# Security Analysis
sruja analyze -r /path/to/repo --view security

# Product Feature Dependencies
sruja analyze -r /path/to/repo --view product
```

<table>
 <tr><th valign="top">Stakeholder</th><th valign="top">Purpose</th><th valign="top">Built-in View Name</th></tr>
<tr><td valign="top">CTO</td><td>Executive architecture summary, tech stack, debt assessment, and risks</td><td>cto</td></tr>
<tr><td valign="top">SRE</td><td>Reliability metrics, SPOF identification, resilience recommendations</td><td>sre</td></tr>
<tr><td valign="top">DevOps</td><td>CI/CD indicators, deployment blockers, infrastructure readiness</td><td>devops</td></tr>
<tr><td valign="top">Security</td><td>Attack surface, vulnerabilities, security recommendations</td><td>security</td></tr>
<tr><td valign="top">Product</td><td>Feature overview, critical dependencies, shared components, impact analysis</td><td>product</td></tr>
</table>

## Configurable Views

You can customize these reports or create entirely new views by defining them in a `.sruja.yaml` file in the root of your repository. 

Custom views allow you to:
- **Extend existing views**: Inherit settings from built-in views like `sre` or `cto`.
- **Select specific sections**: Include or exclude analytical sections (e.g., `tech_debt`, `infrastructure`).
- **Override thresholds**: Set specific thresholds for coupling, orphans, and complexity.
- **Customize terminology**: Standardize names (e.g., replacing "service" with "microservice").

### Example: `.sruja.yaml` Custom View Configuration

```yaml
views:
  platform-engineer:
    extends: sre  # Inherit from the built-in SRE view
    name: "Platform Engineer Report"
    sections:
      - infrastructure
      - reliability
      - cost_optimization
    thresholds:
      max_coupling: 8
      max_orphans: 3
    terminology:
      service: "microservice"
      database: "data store"
  
  tech-lead:
    extends: cto
    name: "Tech Lead Summary"
    sections:
      - tech_debt
      - team_impact
      - recommendations
    exclude:
      - executive_summary
```

Once defined, your team can use these custom views right away:

```bash
sruja analyze -r . --view platform-engineer
sruja analyze -r . --view tech-lead
```

## Advanced Examples

```bash
# CTO report with JSON output
sruja analyze -r . --view cto --format json

# SRE report focusing on single points of failure
sruja analyze -r . --view sre --format json | jq '.sections.single_points_of_failure'

# Security assessment piped to file
sruja analyze -r . --view security --format json > security-report.json
```

# Sruja - Stakeholder Reports

Sruja provides tailored architecture analysis for different stakeholders in your organization.

## Commands

```bash
# CTO Executive Summary
sruja cto -r /path/to/repo

# SRE Reliability Report
sruja sre -r /path/to/repo

# DevOps Deployment Readiness
sruja devops -r /path/to/repo

# Security Analysis
sruja security -r /path/to/repo

# Product Feature Dependencies
sruja product -r /path/to/repo
```

```



 <td valign="top">Stakeholder</td><td valign="top">Purpose</td><td valign="top">Command</td></tr>
<tr><td valign="top">CTO</td><td>Executive architecture summary, tech stack, debt assessment, and risks</td></tr>
<tr><td valign="top">SRE</td><td>Reliability metrics, SPOF identification, resilience recommendations</td></tr>
<tr><td valign="top">DevOps</td><td>CI/CD indicators, deployment blockers, infrastructure readiness</td></tr>
<tr><td valign="top">Security</td><td>Attack surface, vulnerabilities, security recommendations</td></tr>
<tr><td valign="top">Product</td><td>Feature overview, critical dependencies, shared components, impact analysis</td></tr>
</table>

## Examples

```bash
# CTO report with JSON output
sruja cto -r . -f json

# SRE report focused on single points of failure
sruja sre -r . --format json | jq '.single_points_of_failure[] | .length -gt 0'

# DevOps readiness check
sruja devops -r . --format json

# Security assessment with high-risk components highlighted
sruja security -r . --format json

# Product feature dependencies
sruja product -r . --format json
```

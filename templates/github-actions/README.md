# Sruja GitHub Actions Templates

This directory contains reusable GitHub Actions workflows for Sruja.

## Available Workflows

| Workflow | Purpose | Triggers |
|----------|---------|----------|
| `sruja-drift.yml` | Architecture drift detection | Push, PR |
| `sruja-stakeholder.yml` | Multi-stakeholder reports | Manual, Schedule |
| `sruja-security.yml` | Security analysis | Push, Schedule |
| `sruja-pr.yml` | PR-scoped drift detection | Pull Request |
| `sruja-architecture-pr.yml` | PR gate: drift + blueprint lint | Pull Request |
| `sruja-release.yml` | Pre-release checks | Release |

## Quick Start

Copy the desired workflow to `.github/workflows/`:

```bash
# Copy drift detection
cp templates/github-actions/sruja-drift.yml .github/workflows/

# Copy security scanning (runs daily at 6 AM UTC)
cp templates/github-actions/sruja-security.yml .github/workflows/
```

### Recommended for SDLC “auto architecture maintenance”

- Add `templates/github-actions/sruja-architecture-pr.yml` to gate PRs on:
  - `sruja drift-pr` (new violations only)
  - `sruja lint` for blueprint files under `architecture/**/*.sruja` and/or `docs/architecture/**/*.sruja`
- When CI fails, fix locally by updating your `.sruja` blueprint using the `sruja-architecture` skill, then re-run `sruja lint` before pushing.

## Configuration

### Required Secrets

None for basic functionality. For enhanced features:

| Secret | Purpose | Required For |
|--------|---------|--------------|
| `OPENAI_API_KEY` | LLM-enhanced analysis | Optional enrichment |
| `GITHUB_TOKEN` | PR comments | `sruja-pr.yml` |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SRUJA_INTENT_PATH` | `docs/architecture` | Path to ADRs and intent docs |
| `SRUJA_TRACES_PATH` | - | Path to runtime traces JSON |

## Examples

### Basic Drift Detection

```yaml
# .github/workflows/sruja-drift.yml
name: Architecture Drift
on: [push, pull_request]
jobs:
  drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash
      - name: Check PATH
        run: echo "$HOME/.local/bin" >> $GITHUB_PATH
      - name: Run drift check
        run: sruja drift -r .
```

### PR-Scoped Analysis

```yaml
# .github/workflows/sruja-pr.yml
name: PR Architecture Check
on: [pull_request]
jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash
      - name: Check for new drift
        run: sruja drift-pr -r . -b origin/main -f github-actions
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Scheduled Security Scan

```yaml
# .github/workflows/sruja-security.yml
name: Security Analysis
on:
  schedule:
    - cron: '0 6 * * *'  # Daily at 6 AM UTC
  workflow_dispatch:
    inputs:
      repo:
        description: Repository to scan
        required: true
        default: ${{ github.repository }}

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: ${{ inputs.repo }}
      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash
      - name: Run security analysis
        run: |
          sruja security -r . -f json > security-report.json
          echo "::notice title=Security Report::Security analysis completed"
          echo "::warning file=security-report.json::Report uploaded as artifact"
      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: security-report
          path: security-report.json
```

### CTO Report on Schedule

```yaml
# .github/workflows/sruja-stakeholder.yml
name: Weekly Stakeholder Reports
on:
  schedule:
    - cron: '0 8 * * 1'  # Monday 8 AM UTC
  workflow_dispatch:
    inputs:
      report_type:
        description: Type of report (cto, sre, devops, security, product)
        required: true
        default: 'cto'
      repo:
        description: Repository to analyze
        required: true
        default: ${{ github.repository }}

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: ${{ inputs.repo }}
      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash
      - name: Generate Report
        run: |
          sruja ${{ inputs.report_type }} -r . -f json > report.json
      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: ${{ inputs.report_type }}-report
          path: report.json
```

## Customization

### Custom Thresholds

Create `.sruja.yaml` in your repository:

```yaml
# .sruja.yaml
version: 1
settings:
  drift:
    max_cycles: 0          # Fail on any circular dependency
    max_orphans: 5        # Allow up to 5 orphan modules
    max_coupling: 10       # Max dependencies per module
  security:
    require_auth: true    # Require authentication indicators
    require_encryption: true  # Require encryption indicators
  exclude:
    - "**/test/**"
    - "**/tests/**"
    - "**/*.test.*"
```

### Matrix Builds

```yaml
# Multi-repo analysis
jobs:
  analyze:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        repo: [repo-a, repo-b, repo-c]
    steps:
      - uses: actions/checkout@v4
        with:
          repository: ${{ matrix.repo }}
      - name: Install Sruja
        run: curl -fsSL https://sruja.ai/install.sh | bash
      - name: Analyze
        run: sruja quickstart -r . -f json > report-${{ matrix.repo }}.json
```

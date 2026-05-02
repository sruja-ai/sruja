# Sruja GitHub Actions Templates

This directory contains reusable GitHub Actions workflows for Sruja.

## Available Workflows

| Workflow | Purpose | Triggers |
|----------|---------|----------|
| `sruja-architecture-pr.yml` | PR gate: drift + blueprint lint | Pull Request |
| `sruja-onboard.yml` | Onboarding brief (job summary + annotations) | Push, PR |

## Quick Start

Copy the desired workflow to `.github/workflows/`:

```bash
# Copy onboarding brief (recommended for new repos / teams)
cp templates/github-actions/sruja-onboard.yml .github/workflows/
```

### Recommended for SDLC “auto architecture maintenance”

- Add `templates/github-actions/sruja-architecture-pr.yml` to gate PRs on:
  - `sruja drift-pr` (new violations only)
  - `sruja lint` for blueprint files under `architecture/**/*.sruja` and/or `docs/architecture/**/*.sruja`
- When CI fails, fix locally by updating your `.sruja` blueprint using the `sruja-architecture` skill, then re-run `sruja lint` before pushing.

## Configuration

### Required Secrets

None for basic functionality. For optional enrichment:

| Secret | Purpose | Required For |
|--------|---------|--------------|
| `OPENAI_API_KEY` | LLM-enhanced analysis | Optional enrichment |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SRUJA_INTENT_PATH` | `docs/architecture` | Path to ADRs and intent docs |
| `SRUJA_TRACES_PATH` | - | Path to runtime traces JSON |

## Examples

### PR gate: drift + blueprint lint

```yaml
# .github/workflows/sruja-architecture-pr.yml
name: Sruja Architecture PR Check
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

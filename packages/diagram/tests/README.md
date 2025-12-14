# Layout & Diagram Improvement Workflow

This directory contains e2e tests and tooling for iteratively improving the layout and diagram packages.

## Overview

The improvement workflow uses Playwright e2e tests to:

1. Collect layout quality metrics for all examples
2. Compare current metrics with baseline
3. Generate actionable improvement suggestions
4. Track progress over iterations

## Quick Start

### Run Improvement Workflow

```bash
npm run improve
```

This will:

- Run e2e tests to collect metrics
- Compare with baseline (if exists)
- Generate improvement suggestions
- Save reports to `tests/results/`

### Run E2E Tests Manually

```bash
# Run all tests
npm run test:e2e

# Run with UI
npm run test:e2e:ui

# Run specific test
npx playwright test tests/iterative-optimization.spec.ts
```

## Files Structure

```
tests/
├── iterative-optimization.spec.ts    # Main e2e test
├── improve-layout.ts                  # Improvement workflow script
├── utils/
│   ├── metrics-collector.ts          # Collect layout metrics
│   ├── comparison.ts                 # Compare baseline vs current
│   └── improvement-suggestions.ts    # Generate suggestions
└── results/
    ├── baseline-metrics.json         # Baseline metrics (first run)
    ├── latest-metrics.json          # Current metrics
    ├── iteration-report.md           # Comparison report
    └── improvement-suggestions.md    # Actionable suggestions
```

## Iterative Improvement Process

### 1. Initial Baseline

On first run, the workflow will:

- Collect metrics for all examples
- Save as baseline: `baseline-metrics.json`
- Generate initial suggestions

### 2. Make Improvements

Based on suggestions in `improvement-suggestions.md`:

- Fix critical issues first (containment violations, overlaps)
- Address high-priority issues (edge crossings, spacing)
- Refine medium/low priority issues

### 3. Re-run and Compare

```bash
npm run improve
```

The workflow will:

- Collect new metrics
- Compare with baseline
- Show improvements/regressions
- Generate updated suggestions

### 4. Track Progress

Check `iteration-report.md` for:

- Examples that improved
- Examples that regressed
- Average score changes

## Metrics Collected

- **Overlapping Nodes**: Nodes that overlap (critical)
- **Spacing Violations**: Nodes too close together
- **Edge Crossings**: Edges that cross each other
- **Edges Over Nodes**: Edges passing through nodes
- **Parent-Child Violations**: Children outside parent bounds (critical)
- **Label Issues**: Overlaps and clipping
- **Layout Quality Score**: Overall weighted score (0-100)

## Improvement Suggestions

Suggestions are categorized by priority:

- **Critical**: Must fix (containment violations, severe overlaps)
- **High**: Significant impact (many edge crossings, spacing issues)
- **Medium**: Moderate impact (label issues, layout selection)
- **Low**: Nice to have (consistency, viewport utilization)

Each suggestion includes:

- Affected examples
- Code locations to modify
- Expected improvement
- Related files

## Best Practices

1. **Start with Critical Issues**: Fix containment violations and overlaps first
2. **Test Incrementally**: Make small changes and re-run tests
3. **Track Regressions**: If a change causes regressions, investigate immediately
4. **Update Baseline**: After significant improvements, update baseline:
   ```bash
   cp tests/results/latest-metrics.json tests/results/baseline-metrics.json
   ```

## Environment Variables

- `FULL_SUITE=true`: Run all examples (default: 10 for faster iteration)
- `CI=true`: Disable UI, use headless mode

## Troubleshooting

### Tests Fail to Load Examples

- Ensure dev server is running: `npm run dev`
- Check that examples are available in `@sruja/shared`

### Metrics Collection Fails

- Check browser console logs in test output
- Verify React Flow is rendering correctly
- Ensure `__CYBER_GRAPH__` is exposed in test app

### No Baseline Found

- First run creates baseline automatically
- Or manually copy `latest-metrics.json` to `baseline-metrics.json`

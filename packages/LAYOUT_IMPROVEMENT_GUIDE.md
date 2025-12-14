# Layout & Diagram Improvement Guide

This guide explains how to iteratively improve the layout and diagram packages using e2e tests.

## Quick Start

```bash
cd packages/diagram
npm run improve
```

This runs the improvement workflow:

1. Collects layout metrics for all examples
2. Compares with baseline
3. Generates improvement suggestions
4. Saves reports to `tests/results/`

## Improvement Workflow

### 1. Initial Setup

First run creates a baseline:

```bash
cd packages/diagram
npm run improve
```

This will:

- Run e2e tests
- Collect metrics for all examples
- Save baseline: `tests/results/baseline-metrics.json`
- Generate initial suggestions: `tests/results/improvement-suggestions.md`

### 2. Review Suggestions

Open `tests/results/improvement-suggestions.md` to see:

- **Critical** issues (must fix): Containment violations, severe overlaps
- **High** priority: Many edge crossings, spacing issues
- **Medium** priority: Label issues, layout selection
- **Low** priority: Consistency, viewport utilization

Each suggestion includes:

- Affected examples
- Code locations to modify
- Expected improvement
- Related files

### 3. Make Improvements

Start with critical issues:

#### Fix Containment Violations

**Location**: `packages/layout/src/algorithms/hierarchy.ts`

**Issue**: Children outside parent bounds

**Fix**:

- Ensure child positions are relative to parent
- Add padding checks in `detectContainmentViolations`
- Adjust parent sizing in `sizing.ts`

#### Fix Node Overlaps

**Location**: `packages/layout/src/algorithms/overlap.ts`

**Issue**: Nodes overlapping each other

**Fix**:

- Improve overlap detection
- Add overlap resolution in coordinate calculation
- Adjust spacing in layout rules

#### Reduce Edge Crossings

**Location**: `packages/layout/src/algorithms/edge-router.ts`

**Issue**: Too many edge crossings

**Fix**:

- Improve routing algorithm
- Adjust node positioning to reduce crossings
- Use edge bundling for dense graphs

### 4. Re-run and Compare

After making changes:

```bash
npm run improve
```

Check `tests/results/iteration-report.md` for:

- Examples that improved
- Examples that regressed
- Average score changes

### 5. Update Baseline

After significant improvements:

```bash
cp tests/results/latest-metrics.json tests/results/baseline-metrics.json
```

This sets a new baseline for future comparisons.

## Common Improvement Patterns

### Pattern 1: Adjust Spacing

**Problem**: Nodes too close together

**Solution**: Update spacing in layout rules or presets

```typescript
// packages/diagram/src/utils/layoutRules.ts
{
  engine: "sruja",
  direction: "DOWN",
  options: {
    nodeSpacing: 200, // Increase from 150
    layerSpacing: 250, // Increase from 180
  },
}
```

### Pattern 2: Improve Edge Routing

**Problem**: Edges passing through nodes

**Solution**: Enhance routing algorithm

```typescript
// packages/layout/src/algorithms/edge-router.ts
// Add node avoidance logic
// Use spatial index to check intersections
```

### Pattern 3: Fix Hierarchy

**Problem**: Children outside parents

**Solution**: Ensure proper coordinate transformation

```typescript
// packages/layout/src/algorithms/hierarchy.ts
// Child positions must be relative to parent
// Add padding checks
// Ensure parent size accommodates children
```

### Pattern 4: Optimize Layout Selection

**Problem**: Wrong layout for graph type

**Solution**: Add/refine layout rules

```typescript
// packages/diagram/src/utils/layoutRules.ts
// Add new rule with appropriate priority
// Refine condition logic
// Test with affected examples
```

## Metrics to Track

### Critical Metrics (Must Fix)

- **Overlapping Nodes**: Should be 0
- **Parent-Child Violations**: Should be 0

### High Priority Metrics

- **Edge Crossings**: Minimize
- **Edges Over Nodes**: Minimize
- **Spacing Violations**: Minimize

### Quality Score Targets

- **Average Score**: Target 80+
- **Grade**: Target B or better
- **No Critical Violations**: All examples should have 0 overlaps and 0 containment violations

## Testing Strategy

### Incremental Testing

1. Make small changes
2. Run `npm run improve`
3. Check for regressions
4. If regressions, investigate and fix
5. If improvements, continue

### Focused Testing

Test specific examples:

```bash
# Run for specific example
npx playwright test tests/iterative-optimization.spec.ts --grep "example-name"
```

### Full Suite

Run all examples (slower):

```bash
FULL_SUITE=true npm run improve
```

## Troubleshooting

### Tests Fail

- Check dev server is running: `npm run dev`
- Verify examples are available
- Check browser console in test output

### No Improvements

- Review suggestions carefully
- Check code locations
- Verify changes are applied
- Test incrementally

### Regressions

- Revert changes if severe
- Investigate root cause
- Make smaller incremental changes
- Test each change separately

## Best Practices

1. **Start Small**: Fix one issue at a time
2. **Test Often**: Run tests after each change
3. **Track Progress**: Review iteration reports
4. **Update Baseline**: After significant improvements
5. **Document Changes**: Note what was changed and why

## Resources

- **Architecture Docs**: `packages/layout/ARCHITECTURE.md`
- **Diagram Docs**: `packages/diagram/ARCHITECTURE.md`
- **Test Docs**: `packages/diagram/tests/README.md`
- **Suggestions**: `packages/diagram/tests/results/improvement-suggestions.md`

## Next Steps

1. Run initial baseline: `npm run improve`
2. Review suggestions
3. Fix critical issues first
4. Iterate and improve
5. Track progress over time

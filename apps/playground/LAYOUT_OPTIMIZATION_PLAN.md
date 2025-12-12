# Playwright-Based Layout Engine Optimization Plan

## Overview

This plan outlines a systematic approach to optimize the layout engine using Playwright tests. The goal is to iteratively improve layout quality across all examples by collecting metrics, analyzing results, and refining the layout algorithms and rules.

## Phase 1: Baseline Measurement & Data Collection

### 1.1 Comprehensive Test Suite

**Objective**: Establish baseline metrics for all examples

**Existing Tests**:
- `baseline-metrics.spec.ts`: Collect baseline metrics and save to `tests/results`
- `layout-comparison.spec.ts`: Compare selected engines/configurations across examples
- `optimization-loop.spec.ts`: Compare current metrics with baseline and generate reports
- `layout-stability.spec.ts`: Regression checks for layout stability
- `parent-child-sizing.spec.ts`: Validate parent-child containment and sizing
- `all-examples-quality.spec.ts`: Aggregate quality checks across all examples
- `rules-based-layout.spec.ts`: Verify rule selection effectiveness
- `diagram-quality.spec.ts`: Unit-level quality metrics validation

**Metrics to Collect**:
```typescript
interface LayoutMetrics {
    // Example info
    exampleName: string;
    category: string;
    timestamp: string;

    // Quality scores
    weightedScore: number;
    overallScore: number;
    grade: 'A' | 'B' | 'C' | 'D' | 'F';
    
    // Specific metrics
    overlappingNodes: number;
    edgeCrossings: number;
    edgesOverNodes: number;
    edgeBends: number;
    spacingViolations: number;
    parentChildViolations: number;
    
    // Layout characteristics
    nodeCount: number;
    edgeCount: number;
    hasHierarchy: boolean;
    currentLevel: string;
    selectedEngine: 'sruja' | 'c4level';
    selectedDirection: string;
    
    // Performance
    layoutTime: number; // milliseconds
    renderTime: number;
    
    // Visual metrics
    viewportUtilization: number;
    aspectRatio: number;
    diagramBounds: { x: number; y: number; width: number; height: number };

    // Full quality snapshot
    qualityMetrics: DiagramQualityMetrics;
}
```

### 1.2 Automated Metrics Collection

**Implementation**: `tests/utils/metrics-collector.ts`

```typescript
// Key APIs
export async function collectLayoutMetrics(/* ... */): Promise<LayoutMetrics> { /* implemented */ }
export async function collectBaselineForAllExamples(/* ... */): Promise<Map<string, LayoutMetrics>> { /* implemented */ }
```

### 1.3 Results Storage

**Format**: JSON/Markdown files in `tests/results/` (auto-created by tests)
- `baseline-metrics.json`: Initial baseline
- `comparison-{timestamp}.json`: Before/after comparisons
- `comparison-report-{timestamp}.md`: Human-readable report
- `engine-comparison-{timestamp}.json`: Engine selection summary

## Phase 2: Analysis & Pattern Detection

### 2.1 Identify Problem Patterns

**Analysis Script**: `scripts/analyze-metrics.ts`

**Patterns to Detect**:
1. **Low-scoring examples**: Examples consistently scoring < 70
2. **Common violations**: Most frequent quality issues
3. **Engine mismatches**: Cases where wrong engine is selected
4. **Level-specific issues**: Problems at specific C4 levels
5. **Size-related issues**: Problems with small/large diagrams
6. **Hierarchy issues**: Problems with parent-child relationships

**Output**: 
- Report identifying top 10 problem cases
- Categorized by issue type
- Suggestions for rule improvements

### 2.2 Rule Effectiveness Analysis

**Create**: `scripts/analyze-rules.ts`

**Analysis**:
- Which rules are most frequently triggered?
- Which rules produce best results?
- Which rules should be re-prioritized?
- Are there missing rules for common patterns?

**Output**:
- Rule performance report
- Priority adjustment recommendations
- New rule suggestions

## Phase 3: Iterative Optimization

### 3.1 Optimization Loop

```
┌─────────────────┐
│  Run Tests      │
│  Collect Metrics│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Analyze Results│
│  Identify Issues│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Generate       │
│  Hypotheses     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Implement      │
│  Changes        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Test Changes   │
│  Compare        │
└────────┬────────┘
         │
         ▼
    ┌────────┐
    │ Better?│
    └───┬────┘
        │
    Yes │ No
        │
        ▼
    Continue / Revert
```

### 3.2 Optimization Strategies

#### A. Rule Refinement
- **Adjust priorities**: Move effective rules higher
- **Refine conditions**: Make rules more specific
- **Add new rules**: Cover missing patterns
- **Remove ineffective rules**: Clean up unused rules

#### B. Layout Algorithm Tuning
- **Spacing parameters**: Optimize node/edge spacing
- **Direction selection**: Improve direction heuristics
- **Hierarchy handling**: Better parent-child sizing
- **Edge routing**: Reduce crossings and bends

#### C. Quality Metric Weights
- **Analyze correlations**: Which metrics matter most?
- **Adjust weights**: Focus on high-impact metrics
- **Custom weights per level**: Different priorities for L0/L1/L2/L3

### 3.3 Automated Optimization Tests

**Create**: `tests/optimization-loop.spec.ts`

```typescript
test('optimization iteration', async ({ page }) => {
    const baseline = await loadBaselineMetrics();
    const current = await collectCurrentMetrics(page);
    
    // Compare and report
    const improvement = compareMetrics(baseline, current);
    
    // Assert improvements
    expect(improvement.avgScoreDelta).toBeGreaterThan(0);
    expect(improvement.failingExamples).toBeLessThan(baseline.failingExamples);
});
```

## Phase 4: Continuous Improvement

### 4.1 Regression Prevention

**Test**: `tests/regression.spec.ts`

**Checks**:
- No example should drop below baseline score
- Critical examples must maintain quality
- Performance should not degrade

### 4.2 Performance Monitoring

**Track**:
- Layout calculation time
- Render time
- Memory usage
- Test execution time

**Alert**: If performance degrades > 20%

### 4.3 Automated Reporting

**Create**: `scripts/generate-report.ts`

**Report Includes**:
1. **Summary**: Overall quality trends
2. **Top Issues**: Most common problems
3. **Improvements**: What got better
4. **Regressions**: What got worse
5. **Recommendations**: Next steps

**Format**: Markdown report + JSON data

## Phase 5: Advanced Optimization

### 5.1 Machine Learning Integration (Future)

**Potential**:
- Learn optimal rule priorities from data
- Predict best layout engine for new examples
- Auto-generate rules from patterns

### 5.2 A/B Testing Framework

**Create**: `tests/ab-testing.spec.ts`

**Test**:
- Multiple layout configurations side-by-side
- User preference data (if available)
- Statistical significance testing

### 5.3 Visual Regression Testing

**Tool**: Playwright screenshot comparison

**Test**:
- Visual appearance of layouts
- Detect visual regressions
- Ensure consistent rendering

## Implementation Steps

### Week 1: Foundation
- [ ] Create `MetricsCollector` utility
- [ ] Build baseline metrics collection test
- [ ] Set up results storage structure
- [ ] Run baseline for all examples

### Week 2: Analysis Tools
- [ ] Build metrics analysis script
- [ ] Create pattern detection logic
- [ ] Generate first analysis report
- [ ] Identify top 10 problem cases

### Week 3: First Optimization Cycle
- [ ] Implement fixes for top issues
- [ ] Refine rules based on analysis
- [ ] Run comparison tests
- [ ] Measure improvements

### Week 4: Automation & CI
- [ ] Integrate into CI pipeline
- [ ] Set up automated reporting
- [ ] Create regression test suite
- [ ] Document process

## Test Structure

```
tests/
├── baseline-metrics.spec.ts          # Initial baseline collection
├── layout-comparison.spec.ts          # Compare engines/configs
├── optimization-loop.spec.ts          # Iterative optimization
├── regression.spec.ts                 # Prevent regressions
├── rules-effectiveness.spec.ts        # Test rule selection
├── utils/
│   ├── metrics-collector.ts            # Metrics collection
│   ├── layout-helpers.ts               # Test helpers
│   └── comparison.ts                   # Compare results
├── results/
│   ├── baseline-metrics.json
│   └── optimization-runs/
└── scripts/
    ├── analyze-metrics.ts              # Analyze collected data
    ├── analyze-rules.ts                 # Analyze rule effectiveness
    └── generate-report.ts               # Generate reports
```

## Success Criteria

### Quality Targets
- **Overall**: 80%+ examples score ≥ 70
- **Critical**: 100% of critical examples score ≥ 80
- **Average**: Average weighted score ≥ 75
- **No Regressions**: No example drops > 5 points

### Performance Targets
- **Layout Time**: < 500ms for typical examples
- **Render Time**: < 200ms
- **Test Suite**: Complete in < 10 minutes

### Coverage
- **Examples**: 100% of examples tested
- **Levels**: All C4 levels (L0, L1, L2, L3)
- **Scenarios**: Expanded/collapsed, focused/unfocused

## Tools & Commands

### Run Tests
```bash
# Collect baseline metrics
npm run test:baseline

# Run optimization tests
npm run test:optimize

# Run regression tests
npm run test:regression

# Run all layout tests
npm run test:layout
```

### Analysis
```bash
# Analyze collected metrics
npm run analyze:metrics

# Analyze rule effectiveness
npm run analyze:rules

# Generate report
npm run report:generate
```

### Playwright Config
- `playwright.config.ts` uses `webServer` to run `npm run dev` with `baseURL` `http://localhost:5173`.
- Tests are parallelized and run headless by default; HTML reporter enabled.

### Quality Weights
- Adjust via `DEFAULT_QUALITY_WEIGHTS` in `src/utils/diagramQuality.ts`.
- Consider per-level overrides when analyzing correlations.

## Monitoring Dashboard (Future)

**Visual Dashboard**:
- Quality trends over time
- Example-by-example scores
- Rule effectiveness charts
- Performance metrics
- Issue categorization

## Notes

- **Incremental**: Make small, testable changes
- **Data-Driven**: Base decisions on metrics, not intuition
- **Automated**: Minimize manual analysis
- **Documented**: Track all changes and rationale
- **Reversible**: Keep ability to revert changes

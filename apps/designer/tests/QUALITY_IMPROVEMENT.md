# Diagram Quality Iterative Improvement

This directory contains e2e tests and scripts for iteratively improving diagram quality based on quantitative metrics.

## Overview

The quality improvement system:

1. **Measures** layout quality using TypeScript metrics (edge crossings, overlaps, alignment, etc.)
2. **Tracks** quality scores over multiple runs
3. **Analyzes** improvements vs baseline
4. **Suggests** areas for improvement

## Quality Metrics

The system measures the following metrics:

- **Score** (0.0-1.0): Overall quality score (target: ≥0.85)
- **Edge Crossings**: Number of edges that cross (target: ≤5)
- **Node Overlaps**: Number of overlapping nodes (target: 0)
- **Label Overlaps**: Number of label overlaps (target: 0)
- **Rank Alignment** (0.0-1.0): How well nodes align in ranks (target: ≥0.95)
- **Spacing Consistency** (0.0-1.0): Uniformity of node spacing (target: ≥0.80)
- **Edge Length Variance**: Consistency of edge lengths (lower is better)

## Running Quality Tests

### Single Run

```bash
npm run test:e2e -- tests/diagram-quality-iterative.spec.ts
```

### Iterative Improvement Script

Run multiple iterations to track improvements:

```bash
./scripts/improve-diagram-quality.sh [iterations]
```

Example:

```bash
# Run 5 iterations
./scripts/improve-diagram-quality.sh 5
```

The script will:

1. Run quality tests for each iteration
2. Track scores in `tests/results/quality-iterative/`
3. Compare each run against baseline
4. Suggest improvements

## Results

Results are saved in `tests/results/quality-iterative/`:

- `{example}-history.json`: Full history of all runs
- `{example}-metrics-{run}.json`: Metrics snapshot for each run
- `{example}-report-{run}.txt`: Human-readable report
- `summary.json`: Overall summary across all examples
- `run-{iteration}.log`: Test execution logs

## Understanding Results

### Quality Grades

- **A**: Score ≥ 0.85 (target)
- **B**: Score ≥ 0.70
- **C**: Score ≥ 0.50
- **F**: Score < 0.50

### Example Output

```
=== E-Commerce Platform Quality Metrics (Run 3) ===
Score: 0.723 (target: ≥0.85)
Edge Crossings: 7 (target: ≤5)
Node Overlaps: 0 (target: 0)
Label Overlaps: 0 (target: 0)
Rank Alignment: 92.3% (target: ≥95%)
Spacing Consistency: 78.5% (target: ≥80%)
Nodes: 12, Edges: 18

=== Improvements vs Baseline ===
score: 0.650 → 0.723 (+7.3%)
edgeCrossings: 12 → 7 (+5.0)
```

## Improving Quality

When quality scores are below target, the system suggests improvements:

### Low Score (< 0.85)

- Increase spacing (`nodesep`, `ranksep`)
- Adjust rank constraints
- Improve edge weights

### High Edge Crossings (> 5)

- Adjust edge weights to prioritize important relationships
- Use `constraint=false` for less important edges
- Consider different layout direction

### Node Overlaps (> 0)

- Increase `nodesep` and `ranksep`
- Add stronger rank constraints
- Adjust node sizes

### Poor Rank Alignment (< 95%)

- Strengthen rank constraints (`rank=same`, `rank=min`, `rank=max`)
- Add invisible edges for rank ordering
- Ensure proper node grouping

### Poor Spacing Consistency (< 80%)

- Use adaptive spacing based on node count
- Adjust spacing parameters more aggressively
- Consider cluster optimization

## Integration with CI/CD

Add to your CI pipeline:

```yaml
# .github/workflows/quality.yml
- name: Run Quality Tests
  run: |
    npm run test:e2e -- tests/diagram-quality-iterative.spec.ts

- name: Upload Quality Results
  uses: actions/upload-artifact@v3
  with:
    name: quality-results
    path: tests/results/quality-iterative/
```

## Automated Improvements

Future enhancements could include:

- Automatic parameter tuning based on quality scores
- ML-based constraint optimization
- A/B testing different layout strategies
- Integration with layout refinement system

## Files

- `diagram-quality-iterative.spec.ts`: Main e2e test
- `scripts/improve-diagram-quality.sh`: Iterative improvement script
- `../src/components/SrujaCanvas/qualityMetrics.ts`: Quality metrics implementation
- `../src/components/SrujaCanvas/layoutRefinement.ts`: Layout refinement logic

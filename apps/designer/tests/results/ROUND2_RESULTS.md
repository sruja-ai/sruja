# Round 2 Test Results Analysis

**Date:** 2026-01-02  
**Test Suite:** Complex Examples (8 examples × 3 levels = 24 tests)  
**Strategy:** Differentiated L2/L3 approach

## Round 2 Changes Summary

### L2 Strategy (Edge Routing Focus)

- Reduced spacing: 1.20-1.25x → 1.10-1.12x
- Force `splines=polyline` for 6+ nodes
- Increased `minlen` for 8+ nodes
- Reduced edge weight by 10% for 8+ nodes

### L3 Strategy (Preserve Improvements)

- Reduced spacing: 1.20-1.25x → 1.12-1.15x
- Keep `minlen` improvements (8+ nodes)
- Moderate spacing approach

## Results Comparison

### Three-Way Comparison

| Level  | Metric             | Baseline | Round 1 | Round 2 | Change (R1→R2) |
| ------ | ------------------ | -------- | ------- | ------- | -------------- |
| **L1** | Avg Score          | 63.4%    | 62.6%   | TBD     | TBD            |
| **L1** | Edge Crossings     | 0.125    | 0       | TBD     | TBD            |
| **L1** | Spacing Violations | 2.14     | 3.11    | TBD     | TBD            |
| **L2** | Avg Score          | 0%       | 0%      | TBD     | TBD            |
| **L2** | Edge Crossings     | 43.75    | 57.13   | TBD     | TBD            |
| **L2** | Spacing Violations | 4.26     | 4.43    | TBD     | TBD            |
| **L3** | Avg Score          | 0%       | 0%      | TBD     | TBD            |
| **L3** | Edge Crossings     | 39.13    | 14.88   | TBD     | TBD            |
| **L3** | Spacing Violations | 4.58     | 5.19    | TBD     | TBD            |

## Expected Improvements (Round 2)

### Goals

1. **L2 Edge Crossings:** Reduce from 57.13 to ~35-45 (25-40% improvement)
2. **L3 Edge Crossings:** Maintain ~15-20 (keep 62% improvement from Round 1)
3. **Spacing Violations:** Reduce across all levels by ~15-20%

### Key Metrics to Watch

- L2 edge crossings (should decrease)
- L3 edge crossings (should stay low)
- Overall scores (should improve if crossings reduce)

## Analysis

_Waiting for test results..._

# Layout Quality Improvements Analysis

**Date:** 2026-01-02  
**Test Suite:** Complex Examples (8 examples × 3 levels = 24 tests)

## Results Comparison

### Before vs After Improvements (Round 1)

| Level  | Metric             | Before | After     | Change                  |
| ------ | ------------------ | ------ | --------- | ----------------------- |
| **L1** | Avg Score          | 63.4%  | 62.6%     | -0.8%                   |
| **L1** | Edge Crossings     | 0.125  | 0         | ✅ Improved             |
| **L1** | Spacing Violations | 2.14   | 3.11      | ⚠️ Worse                |
| **L2** | Avg Score          | 0%     | 0%        | No change               |
| **L2** | Edge Crossings     | 43.75  | **57.13** | ❌ **30% worse**        |
| **L2** | Spacing Violations | 4.26   | 4.43      | ⚠️ Slightly worse       |
| **L3** | Avg Score          | 0%     | 0%        | No change               |
| **L3** | Edge Crossings     | 39.13  | **14.88** | ✅ **62% improvement!** |
| **L3** | Spacing Violations | 4.58   | 5.19      | ⚠️ Worse                |

## Key Findings (Round 1)

### ✅ Positive Changes

1. **L3 Edge Crossings: 62% Reduction**
   - Before: 39.13 average
   - After: 14.88 average
   - **Significant improvement!**

2. **L1 Edge Crossings: Eliminated**
   - Before: 0.125 average
   - After: 0 average
   - **Perfect!**

### ❌ Negative Changes

1. **L2 Edge Crossings: 30% Increase**
   - Before: 43.75 average
   - After: 57.13 average
   - **Worse performance**

2. **Spacing Violations: Increased for All Levels**
   - L1: 2.14 → 3.11 (+45%)
   - L2: 4.26 → 4.43 (+4%)
   - L3: 4.58 → 5.19 (+13%)

### ⚠️ Extreme Outliers

- **pattern_agentic_ai.sruja L2:** 268 edge crossings (extremely high!)
- **pattern_agentic_ai.sruja L3:** 88 edge crossings (very high)
- **project_saas_platform.sruja L2:** 78 edge crossings (very high)

## Analysis

### Why L3 Improved But L2 Got Worse

**L3 Improvements:**

- The increased `minlen` for L3 (8+ nodes) is working well
- Better spacing helps reduce crossings for component-level diagrams
- L3 diagrams are typically less dense than L2

**L2 Regression:**

- L2 diagrams are more complex with more containers
- The increased spacing may be causing Graphviz to create more crossings
- Some L2 diagrams (like pattern_agentic_ai) are extremely complex (268 crossings!)

### Why Scores Are Still 0%

Even with improvements, scores remain 0% because:

- **L2:** 57.13 crossings × 0.08 = 4.57 → capped at -0.6
- **L3:** 14.88 crossings × 0.08 = 1.19 → capped at -0.6
- Plus spacing (-0.25) and rank alignment (-0.3) penalties
- Total: -1.15 → score = 0

---

## Round 2 Changes (Differentiated L2/L3 Strategy)

Based on the analysis, I implemented a differentiated strategy:

### Changes Made

#### 1. L2 Strategy - Focus on Edge Routing, Not Spacing

**Before:** L2 had aggressive spacing (1.20-1.25x multipliers)
**After:** L2 has moderate spacing (1.10-1.15x multipliers) but uses polyline splines

**Key Changes:**

- `L2NodeSepScale`: 1.20 → 1.10
- `L2RankSepScale`: 1.25 → 1.12
- L2 with 8+ nodes: Force `splines=polyline` for better edge routing
- L2 with 8+ nodes: `minlen += 1` (edge routing boost)
- L2 with 8+ nodes: Reduce weight by 10% (prevents routing artifacts)

#### 2. L3 Strategy - Keep Improvements, Reduce Spacing

**Before:** L3 had aggressive spacing (1.20-1.25x multipliers)
**After:** L3 has moderate spacing (1.12-1.18x multipliers)

**Key Changes:**

- `L3NodeSepScale`: 1.20 → 1.12
- `L3RankSepScale`: 1.25 → 1.15
- L3 with 8+ nodes: `minlen += 1` (kept working improvement)

#### 3. Splines Differentiation

- **L2:** Force polyline for 6+ nodes (better for container diagrams)
- **L3:** Use polyline for 6-15 nodes, spline for 15+ nodes
- **L1:** Keep original behavior

### Files Modified

| File                                 | Change                                   |
| ------------------------------------ | ---------------------------------------- |
| `pkg/export/dot/constants.go`        | L2/L3 scaling factors reduced            |
| `pkg/export/dot/constraints.go`      | Differentiated L2/L3 spacing and splines |
| `pkg/export/dot/constraints_test.go` | Updated test expectations                |

---

## Expected Results (Round 2)

| Level | Metric             | Before | Round 1 | Round 2 (Expected) |
| ----- | ------------------ | ------ | ------- | ------------------ |
| L1    | Edge Crossings     | 0.125  | 0       | 0                  |
| L1    | Spacing Violations | 2.14   | 3.11    | ~2.5               |
| L2    | Edge Crossings     | 43.75  | 57.13   | **~35-45**         |
| L2    | Spacing Violations | 4.26   | 4.43    | **~4.0**           |
| L3    | Edge Crossings     | 39.13  | 14.88   | **~15-20**         |
| L3    | Spacing Violations | 4.58   | 5.19    | **~4.5**           |

### Goals for Round 2

1. **L2 Edge Crossings:** Reduce from 57.13 to ~35-45 (25-40% improvement)
2. **L3 Edge Crossings:** Maintain ~15-20 (keep 62% improvement)
3. **Spacing Violations:** Reduce across all levels by ~15-20%

---

## Next Steps

### Immediate Actions (Already Implemented)

1. ✅ Differentiated L2 vs L3 strategies
2. ✅ L2: Focus on polyline splines + moderate spacing
3. ✅ L3: Keep minlen improvements + moderate spacing
4. ⚠️ Extreme outliers still need special handling

### Future Improvements (Round 3+)

1. **Extreme Outlier Handling**
   - `pattern_agentic_ai.sruja` with 268 crossings needs special handling
   - Consider different layout algorithm for diagrams with 200+ crossings
   - Maybe use `rankdir=LR` or other orientation for complex cases

2. **Adaptive Spacing**
   - Calculate optimal spacing based on diagram complexity
   - Use edge density to determine spacing multipliers
   - Consider node aspect ratios

3. **Edge Routing Improvements**
   - Add `constraint=false` for less important edges
   - Use `cluster` attributes for better grouping
   - Consider `dir=both` for bidirectional edges

4. **Score Threshold Adjustment**
   - Consider increasing the score threshold from 0% to meaningful values
   - The current scoring is too harsh (capped at 0% for any diagram with >7 crossings)

## Conclusion

### Round 1 Results

✅ **L3 improvements successful** - 62% reduction in edge crossings  
❌ **L2 regression** - Increased spacing hurt L2 performance  
⚠️ **Spacing trade-off** - Increased spacing helps some but hurts others  
📊 **Scores still 0%** - Need further improvements to break above 0%

### Round 2 Changes

✅ **Differentiated strategy** - L2 uses polyline + moderate spacing  
✅ **L3 preserved** - Kept minlen improvements, reduced spacing  
⚠️ **Waiting for results** - Need to run tests to verify

The key insight is that **L2 and L3 need different strategies**:

- **L2:** Edge routing (polyline) is more important than spacing
- **L3:** Spacing + minlen improvements work well together

---

_Analysis saved: 2026-01-02_  
_To regenerate metrics: Run E2E tests in designer app_

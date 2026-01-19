# Test Results Summary

**Date:** 2026-01-02  
**Test Suite:** Complex Examples Only (35 examples × 3 levels = 105 tests)

## Overall Results by Level

### L1 (Context View) - ✅ Excellent

- **Tests:** 35
- **Errors:** 0 (0%)
- **Average Score:** 86.88 (B+)
- **Edge Crossings:** 0.03 (very low)
- **Overlaps:** 0
- **Spacing Violations:** 0.73

**Status:** L1 is performing excellently with no errors and high quality scores.

### L2 (Container View) - ⚠️ Poor

- **Tests:** 35
- **Errors:** 1 (2.86%)
- **Average Score:** 20.01 (F)
- **Edge Crossings:** 9.17 (high)
- **Overlaps:** 0
- **Spacing Violations:** 1.99 (high)

**Issues:**

1. **Graphviz Layout Error:** `concept_systems_thinking.sruja` - "contain_nodes clust cluster_ecommerce rank 11 missing node"
2. **Low Quality Scores:** Even successful tests have very low scores (20%)
3. **High Edge Crossings:** Average of 9.17 crossings per diagram

**Status:** L2 needs significant improvement in layout quality and error handling.

### L3 (Component View) - ❌ Critical Issues

- **Tests:** 35
- **Errors:** 18 (51.43%)
- **Average Score:** 15.73 (F)
- **Edge Crossings:** -0.46 (invalid - indicates failed tests)
- **Overlaps:** -0.51 (invalid - indicates failed tests)
- **Spacing Violations:** 0.22

**Critical Issues:**

1. **Validation Errors:** 18 examples failing with "Invalid element structure in DOT result" at various indices (0, 1, 2, 4, 5)
2. **Failed Examples:**
   - compatible/ecommerce_with_sruja_extensions.sruja
   - concept_systems_thinking.sruja
   - course/ecommerce.sruja
   - demo_flat.sruja
   - demo_governance.sruja
   - demo_implied_relationships.sruja
   - demo_overview.sruja
   - demo_scenarios.sruja
   - demo_slo.sruja
   - demo_views_customization.sruja
   - pattern_agentic_ai.sruja
   - pattern_microservices.sruja
   - pattern_rag_pipeline.sruja
   - project_ecommerce.sruja
   - project_iot_platform.sruja
   - project_saas_platform.sruja
   - reference_mvp.sruja
   - sruja_architecture_v2.sruja

**Status:** L3 has critical validation errors preventing most diagrams from rendering.

## Root Causes

### 1. Element Validation Failures (L3)

- Elements created from `lookup.elements` may have unnormalized `Kind` values
- Elements may have empty `Title` fields
- Elements may have invalid `Width`/`Height` values

**Fix Applied:**

- Added normalization for elements from both `allElements` and `lookup.elements`
- Added validation to ensure Title is never empty
- Added validation to ensure Width/Height are always valid

### 2. Graphviz Layout Errors (L2)

- Cluster containment issues: "missing node" errors in clusters
- This suggests nodes are referenced in clusters but don't exist or aren't properly added

**Next Steps:**

- Investigate cluster creation logic
- Check node addition order for L2 views
- Verify all nodes referenced in clusters are properly added

### 3. Low Quality Scores (L2/L3)

- Even when diagrams render successfully, quality scores are very low (15-20%)
- High edge crossings (9.17 average for L2)
- High spacing violations

**Next Steps:**

- Review spacing constraints for L2/L3
- Optimize edge routing
- Improve rank alignment

## Recommendations

### Immediate (Critical)

1. ✅ **Fixed:** Element validation/normalization for L3
2. **Next:** Test the fix to verify L3 validation errors are resolved
3. **Next:** Investigate and fix Graphviz cluster errors for L2

### Short-term (Quality)

1. Improve spacing consistency for L2/L3 diagrams
2. Reduce edge crossings through better routing
3. Optimize rank alignment for container/component views

### Long-term (Architecture)

1. Add comprehensive validation at element creation time
2. Improve error messages to identify specific validation failures
3. Add regression tests for complex examples

## Test Command

```bash
npm run test:quality:all
```

## Results File

`apps/designer/tests/results/all-examples-metrics.json`

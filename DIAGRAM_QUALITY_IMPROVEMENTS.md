# Diagram Quality Improvements

## Summary

This document summarizes the improvements made to enhance diagram quality for complex examples, particularly focusing on reducing edge crossings, node overlaps, label overlaps, and improving overall layout aesthetics.

## Analysis of Current State

Based on the ecommerce example metrics:

- **Score**: 65 (D grade) → Target: 85+ (B+ grade)
- **Edge Crossings**: 2
- **Node Overlaps**: 2 (User-WebApp, WebApp-Database)
- **Spacing Violations**: 3
- **Edge Label Overlaps**: 3

## Improvements Implemented

### 1. Enhanced Spacing for Complex Diagrams (`pkg/export/dot/constraints.go`)

#### For Very Complex Diagrams (20+ nodes)

- **30% extra horizontal spacing** (NodeSep) to prevent node overlaps
- **35% extra vertical spacing** (RankSep) for better rank separation
- **Increased edge separation** (Sep) from 0.4 to 0.5 for better edge routing

#### For L1 Diagrams with Many Nodes (8+ nodes)

- **Additional 20% horizontal spacing** on top of existing L1 scaling
- **Additional 25% vertical spacing** to prevent person-system overlaps
- Addresses the specific issue where "User overlaps WebApp" and "WebApp overlaps Database"

### 2. Improved Edge Routing (`pkg/export/dot/constraints.go`)

#### Edge Minimum Length

- **Very complex diagrams (20+ nodes)**: minlen = 3 (increased from 2)
- **Dense diagrams (8-19 nodes)**: minlen = 2
- **Simple diagrams**: minlen = 1
- Reduces edge crossings by forcing edges to take longer paths

#### Spline Selection

- **Very complex diagrams (20+ nodes)**: Use "spline" (curved) instead of "polyline"
- Better edge routing for complex diagrams with many connections
- Improved visual appearance while maintaining routing quality

#### Edge Separation

- **Diagrams with 10+ relations**: Increased Sep from 0.4 to 0.5
- Provides more space for edge labels, reducing label overlaps

### 3. Enhanced Edge Label Positioning (`pkg/export/dot/constraints.go`)

#### Adaptive Label Distance

- **Dense diagrams (8-19 nodes)**: labelDistance = 2.0 inches (was 1.5)
- **Very complex diagrams (20+ nodes)**: labelDistance = 2.5 inches
- **Long labels (>30 chars)**: Additional 0.5 inches distance
- Reduces edge label overlaps significantly

### 4. Improved Cluster Margins (`pkg/export/dot/dot_generator.go`)

- **Clusters with 5+ children**: Extra 10px margin
- Better parent-child containment
- Prevents child nodes from touching cluster boundaries

### 5. Enhanced Compound Node Padding (`apps/designer/src/components/SrujaCanvas/compoundNodes.ts`)

- **Clusters with 5+ children**: Padding increased from 40px to 60px
- Ensures children stay fully within parent boundaries
- Matches the increased cluster margins in Graphviz

## Expected Impact

### For Ecommerce Example (Current: 65 score)

- **Node Overlaps**: Should reduce from 2 to 0
  - Increased spacing for L1 diagrams with 8+ nodes
  - Additional 20% horizontal and 25% vertical spacing
- **Edge Crossings**: Should reduce from 2 to 0-1
  - Increased minlen for complex diagrams
  - Better spline routing
- **Label Overlaps**: Should reduce from 3 to 0-1
  - Increased label distance (2.0-2.5 inches)
  - Better edge separation (0.5)
- **Spacing Violations**: Should reduce from 3 to 0-1
  - More aggressive spacing scaling
  - Better rank separation

### Expected Score Improvement

- **Before**: 65 (D grade)
- **Expected After**: 80-85+ (B to B+ grade)
- **Target**: 85+ (B+ grade)

## Testing Recommendations

1. **Test Complex Examples**:
   - `project_ecommerce.sruja` (L1 with persons and systems)
   - `project_saas_platform.sruja` (Complex L2 with many containers)
   - `project_iot_platform.sruja` (Complex system)
   - `pattern_agentic_ai.sruja` (Complex pattern)

2. **Check Quality Metrics**:
   - Open browser console and check `window.__DIAGRAM_QUALITY__`
   - Look for improvements in:
     - `edgeCrossings` (should decrease)
     - `nodeOverlaps` (should be 0)
     - `labelOverlaps` (should decrease)
     - `spacingConsistency` (should increase)
     - `score` (should increase to 85+)

3. **Visual Inspection**:
   - Verify no node overlaps
   - Check edge routing is cleaner
   - Ensure labels don't overlap nodes or other labels
   - Verify parent-child containment is correct

## Files Modified

1. `pkg/export/dot/constraints.go`
   - Enhanced spacing calculations
   - Improved edge routing parameters
   - Better label positioning

2. `pkg/export/dot/dot_generator.go`
   - Increased cluster margins for complex clusters

3. `apps/designer/src/components/SrujaCanvas/compoundNodes.ts`
   - Increased compound node padding for complex clusters

## Next Steps

1. **Rebuild WASM**: Run `make wasm` to rebuild with new Go changes
2. **Test in Browser**: Open complex examples and verify improvements
3. **Run Quality Tests**: Execute Playwright tests to collect new metrics
4. **Iterate**: Based on results, make further refinements if needed

## Notes

- All improvements are backward compatible
- Simple diagrams (< 8 nodes) are unaffected
- Changes are progressive based on diagram complexity
- No changes to scoring algorithm (as requested)

# Current Quality Analysis

## Ecommerce Example Analysis

Based on `ecommerce-quality-metrics.json`:

### Current Metrics

- **Score**: 65 (D grade) → Target: 85+ (B+)
- **Gap**: 20 points needed
- **Node Overlaps**: 2 (CRITICAL)
  - User overlaps WebApp
  - WebApp overlaps Database
- **Edge Crossings**: 2
- **Label Overlaps**: 3
- **Spacing Violations**: 3
- **Parent-Child Containment**: 0 ✅

### Root Causes

1. **Node Overlaps (Most Critical)**
   - L1 diagram with persons and systems
   - Different node sizes causing overlaps
   - Insufficient spacing between person and system nodes

2. **Edge Crossings**
   - Complex diagram with many relationships
   - Need better edge routing

3. **Label Overlaps**
   - Many edges with labels
   - Insufficient label distance

## Improvements Already Applied

Based on code changes in `pkg/export/dot/constraints.go`:

1. ✅ **Increased spacing for L1 diagrams with 8+ nodes**
   - Additional 20% horizontal spacing
   - Additional 25% vertical spacing
   - Should address person-system overlaps

2. ✅ **Enhanced edge routing**
   - Increased minlen for complex diagrams
   - Better spline selection

3. ✅ **Improved label positioning**
   - Adaptive label distance (2.0-2.5 inches)
   - Better edge separation

## Expected Impact

After improvements:

- **Node Overlaps**: 2 → 0 (should be fixed)
- **Edge Crossings**: 2 → 0-1 (improved)
- **Label Overlaps**: 3 → 0-1 (improved)
- **Expected Score**: 65 → 80-85+

## Next Steps

### 1. Rebuild WASM (Required)

Since Go code was changed, rebuild WASM:

```bash
cd /Users/dilipkola/Workspace/sruja
make wasm
cd apps/designer
npm run copy:wasm
```

### 2. Test in Browser

With `npm run dev:website` running:

1. Open http://localhost:4321/designer
2. Load `project_ecommerce.sruja`
3. Check browser console: `window.__DIAGRAM_QUALITY__`
4. Verify improvements:
   - No node overlaps
   - Fewer edge crossings
   - Fewer label overlaps

### 3. Run Quality Tests

```bash
cd apps/designer
npm run test:quality:all
```

### 4. Analyze Results

Check `tests/results/all-examples-metrics.json` for updated scores.

### 5. Iterate if Needed

If score is still below 85:

- Review specific issues in metrics
- Make additional adjustments
- Repeat steps 1-4

## Manual Iteration Workflow

```bash
# 1. Make code changes
# Edit: pkg/export/dot/constraints.go

# 2. Rebuild WASM
make wasm
cd apps/designer
npm run copy:wasm

# 3. Test (dev server should be running)
# Open browser and check quality metrics

# 4. Run tests
npm run test:quality:all

# 5. Check results
cat tests/results/all-examples-metrics.json | grep -A 10 "project_ecommerce"

# 6. Repeat if needed
```

## Key Parameters to Adjust

If further improvements needed:

### For Node Overlaps

- `L1NodeSepScale` in `pkg/export/dot/constants.go` (currently 1.15)
- `L1RankSepScale` in `pkg/export/dot/constants.go` (currently 1.20)
- Additional spacing boost in `BuildConstraints()` (currently 1.20/1.25)

### For Edge Crossings

- Edge `minlen` in `buildEdgeConstraints()` (currently 3 for 20+ nodes)
- Spline type selection

### For Label Overlaps

- `labelDistance` in `buildEdgeConstraints()` (currently 2.0-2.5)
- `Sep` value (currently 0.5 for 10+ relations)

## Files Modified

1. `pkg/export/dot/constraints.go` - Layout constraints and spacing
2. `pkg/export/dot/dot_generator.go` - Cluster margins
3. `apps/designer/src/components/SrujaCanvas/compoundNodes.ts` - Compound node padding

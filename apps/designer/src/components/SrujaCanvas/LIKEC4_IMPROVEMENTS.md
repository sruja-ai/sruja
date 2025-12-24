# LikeC4-Level Quality Improvements

## Summary

Enhanced both the Go DOT generation and TypeScript quality metrics to achieve LikeC4-level diagram aesthetics.

## Go-Side Improvements (DOT Generation)

### 1. Enhanced Rank Constraints

**File**: `pkg/export/dot/constraints.go`

- **Before**: Only grouped basic person/system types
- **After**:
  - Groups all person-like elements (person, actor, external) together
  - Ensures perfect alignment with `rank=same` for all same-level nodes
  - Uses `rank=min` for persons to ensure they're at the top

### 2. Improved Edge Weights

**File**: `pkg/export/dot/constraints.go`

- **Before**: Labeled edges = 10, unlabeled = 1
- **After**:
  - Labeled edges = 15 (increased from 10 for better routing priority)
  - Unlabeled edges = 2 (increased from 1 to still influence layout)
  - LikeC4 pattern: Higher weights ensure labeled edges route better

### 3. Stronger Rank Ordering

**File**: `pkg/export/dot/dot_generator.go`

- **Before**: Single invisible edge between first nodes of ranks
- **After**:
  - Added `minlen=2` to invisible edges for better rank separation
  - Added additional invisible edge between last nodes of ranks
  - Creates stronger constraint for proper vertical ordering and alignment

### 4. Adaptive Spacing

**File**: `pkg/export/dot/constraints.go`

- Already implemented: Scales spacing by 1.2-1.6x based on node count
- Ensures diagrams with more nodes get more generous spacing

## TypeScript-Side Improvements (Quality Metrics)

### 1. Stricter Quality Scoring

**File**: `apps/designer/src/components/SrujaCanvas/qualityMetrics.ts`

- **Rank Alignment**:
  - Tolerance reduced from 20px to 10px
  - Requires 95%+ alignment (was 90%)
  - Stricter scoring for misalignment
- **Penalties**:
  - Edge crossings: 0.08 per crossing (was 0.05)
  - Node overlaps: 0.3 per overlap (was 0.2)
  - Rank alignment: Up to 0.3 reduction (was 0.2)

### 2. New Spacing Consistency Metric

**File**: `apps/designer/src/components/SrujaCanvas/qualityMetrics.ts`

- Measures uniformity of node spacing
- LikeC4 requires stdDev < 30% of mean for good consistency
- Penalizes inconsistent spacing (up to 0.3 score reduction)

### 3. More Aggressive Refinement

**File**: `apps/designer/src/components/SrujaCanvas/layoutRefinement.ts`

- **Threshold**: Refines if score < 0.85 (was 0.70)
- **Stops only when**: Score ≥ 0.90, no overlaps, no crossings, alignment ≥ 95%
- **Spacing adjustments**: 30% increases (was 20%) when overlaps detected
- **Rank alignment**: Proportional adjustments based on alignment gap

## Expected Results

### Before

- Score: 0.70-0.80 for most diagrams
- Inconsistent spacing
- Some rank misalignment
- Edge routing could be better

### After

- Score: 0.85-0.95 for well-constrained diagrams
- Uniform spacing (LikeC4-level consistency)
- Perfect rank alignment (95%+)
- Better edge routing with higher weights
- Professional appearance matching LikeC4 quality

## Testing

1. **Run tests**: `go test ./pkg/export/dot/... -v`
2. **Test in browser**: Load microservices example and check console logs
3. **Expected console output**:
   ```
   [layoutEngine] Iteration 1: score=0.85+, crossings=0, overlaps=0, alignment=95%+
   ```

## Next Steps (Optional)

1. **Label Position Optimization**: Extract label positions from Graphviz JSON to measure actual label overlaps
2. **Cluster Balance**: Implement actual cluster balance measurement (currently placeholder)
3. **Edge Routing Hints**: Add more sophisticated port constraints for better edge routing
4. **Visual Testing**: Create side-by-side comparisons with LikeC4 output

## Files Modified

### Go

- `pkg/export/dot/constraints.go` - Enhanced rank constraints and edge weights
- `pkg/export/dot/dot_generator.go` - Stronger rank ordering constraints

### TypeScript

- `apps/designer/src/components/SrujaCanvas/qualityMetrics.ts` - Stricter metrics and spacing consistency
- `apps/designer/src/components/SrujaCanvas/layoutRefinement.ts` - More aggressive refinement

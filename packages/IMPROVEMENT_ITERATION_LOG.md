# Layout Improvement Iteration Log (RL-Style)

## Approach: Reinforcement Learning Loop

**State** → **Action** → **Reward** → **Iterate**

1. **State**: Collect metrics (current scores, issues)
2. **Action**: Make targeted code improvements
3. **Reward**: Test and measure improvement
4. **Iterate**: Continue until all examples reach B score (80+)

---

## Iteration 1: Initial Improvements

### State (Before)

- **Quick Start Example**: Score 80 (B) - at threshold
- **Critical Issues**:
  - Edge congestion: 20 (very low)
  - Edge crossings: 3
  - Edge label overlaps: 1
  - Viewport score: 68

### Actions Taken

#### 1. Enhanced Edge Routing for Better Distribution

**File**: `packages/layout/src/algorithms/edge-router.ts`

- **Improved `detourAround`**: Added multiple detour strategies (close/medium/far) to spread edges
- **Enhanced `routeOrthogonalAvoid`**:
  - Increased visual clearance from 8→12px
  - Added varying padding based on detour count (spreads edges to reduce congestion)
  - Tracks detoured obstacles to vary distances
- **Increased routing padding**: 6→15px (30→40px for L1)

**Expected Impact**: Better edge distribution, reduced congestion

#### 2. Enhanced Edge Bundling

**File**: `packages/layout/src/algorithms/edge-bundler.ts` & `packages/layout/src/c4-layout.ts`

- Increased `angleTolerance`: 20→30°
- Increased `positionTolerance`: 60→100px
- Reduced `fanOutSpacing`: 10→6px
- Applied aggressive bundling in layout

**Expected Impact**: Better edge organization, reduced visual clutter

#### 3. Improved Label Placement

**File**: `packages/layout/src/c4-layout.ts`

- Increased padding: 8→20px
- Increased max iterations: default→15

**Expected Impact**: Fewer label overlaps

#### 4. Multi-Side Port Selection

**File**: `packages/layout/src/algorithms/edge-router.ts`

- Enhanced `calculateBestPort`: Allows connections from any side
- Added `calculateBestPortWithObstacles`: Obstacle-aware port selection
- Updated layout to use obstacle-aware ports

**Expected Impact**: Better routing paths, fewer crossings

#### 5. Enhanced Unified Router

**File**: `packages/layout/src/algorithms/unified-router.ts`

- Added more offset options (40, 60, 80, -40, -60, -80px)
- Tries 6 different offset paths before detour

**Expected Impact**: Fewer edge crossings

#### 6. Improved Spacing in Layout Rules

**File**: `packages/diagram/src/utils/layoutRules.ts`

- Increased node spacing: 150→200px (simple), 180→220px (L1)
- Increased layer spacing: 180→220px (simple), 200→240px (L1)
- Increased dense graph spacing: 200→250px, 220→280px

**Expected Impact**: More room for edge routing, fewer crossings

### Expected Reward (After Testing)

- Edge congestion: 20 → 50+ (target: 60+)
- Edge crossings: 3 → 1-2 (target: 0-1)
- Edge label overlaps: 1 → 0
- Overall score: 80 → 85+ (target: 85+)

---

## Next Steps

### 1. Test Improvements

```bash
cd packages/diagram
npm run improve:iterative
```

This will:

- Run e2e tests to collect new metrics
- Analyze state (identify issues)
- Generate action plan for next iteration

### 2. Analyze Results

Check `tests/results/latest-metrics.json` for:

- New scores
- Which issues improved
- Which issues remain

### 3. Iterate

- If scores improved → Continue with next iteration
- If scores didn't improve → Analyze why and adjust approach
- If all examples are B+ → Goal achieved!

---

## Iteration Workflow

```bash
# Run iterative improvement workflow
npm run improve:iterative

# Or manually:
# 1. Run tests
npx playwright test tests/iterative-optimization.spec.ts

# 2. Analyze results
cat tests/results/latest-metrics.json

# 3. Make improvements based on issues

# 4. Repeat
```

---

## Key Metrics to Track

### Critical (Must Fix)

- **Overlapping Nodes**: Should be 0
- **Parent-Child Violations**: Should be 0

### High Priority

- **Edge Crossings**: Target 0-1
- **Edge Congestion**: Target 60+
- **Edges Over Nodes**: Target 0

### Quality Targets

- **Average Score**: Target 85+
- **All Examples**: Target B (80+) or better
- **No Critical Violations**: All examples should have 0 overlaps and 0 containment violations

---

## Files Modified (Iteration 1)

1. `packages/layout/src/algorithms/edge-router.ts`
   - Enhanced port selection
   - Improved detour routing
   - Better edge distribution

2. `packages/layout/src/algorithms/edge-bundler.ts`
   - More aggressive bundling

3. `packages/layout/src/algorithms/unified-router.ts`
   - More routing strategies

4. `packages/layout/src/algorithms/label-placer.ts`
   - Better label placement

5. `packages/layout/src/c4-layout.ts`
   - Applied improvements
   - Better bundling options

6. `packages/diagram/src/utils/layoutRules.ts`
   - Increased spacing

---

## Notes

- Focus on **edge congestion** first (biggest impact on score)
- **Edge crossings** are second priority
- **Label overlaps** are minor but easy to fix
- **Viewport utilization** improves with better spacing

Continue iterating until all examples reach B score (80+)!

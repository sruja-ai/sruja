# Layout Quality System

## Overview

This document describes the comprehensive quality scoring and optimization system for ensuring well-structured, beautiful diagrams across all C4 levels.

## Quality Metrics

### 1. Node Overlaps (Weight: 20%)
- **Metric**: Number of overlapping nodes
- **Score Calculation**: `100 - (overlap_count / total_pairs * 100)`
- **Target**: 0 overlaps (score: 100)

### 2. Node Spacing (Weight: 12%)
- **Metric**: Minimum distance between nodes
- **Score Calculation**: Penalizes nodes closer than 20px
- **Target**: Average spacing 100-200px

### 3. Edge Crossings (Weight: 15%)
- **Metric**: Number of edge crossings
- **Score Calculation**: `100 - (crossings / edge_count * 200)`
- **Target**: 0 crossings (score: 100)

### 4. Edges Over Nodes (Weight: 12%)
- **Metric**: Edges passing through/over nodes
- **Score Calculation**: `100 - (edges_over_nodes / edge_count * 150)`
- **Target**: 0 edges over nodes

### 5. Edge Bends (Weight: 8%)
- **Metric**: Total number of bends in edges
- **Score Calculation**: `100 - (avg_bends_per_edge * 20)`
- **Target**: Minimal bends (0-1 per edge)

### 6. Edge Length (Weight: 5%)
- **Metric**: Edge length distribution
- **Score Calculation**: Optimal range 100-300px
- **Target**: Average length 100-300px

### 7. Hierarchy/Containment (Weight: 15%)
- **Metric**: Parent-child containment violations
- **Score Calculation**: `(total_children - violations) / total_children * 100`
- **Target**: All children properly contained

### 8. Viewport Utilization (Weight: 8%)
- **Metric**: How well diagram uses viewport
- **Score Calculation**: Optimal 70-90% utilization
- **Target**: 70-90% viewport usage

### 9. Consistency (Weight: 5%)
- **Metric**: Position consistency for similar node types
- **Score Calculation**: Based on variance in positions
- **Target**: Low variance (high consistency)

### 10. Aspect Ratio (Weight: 5%)
- **Metric**: Diagram width/height ratio
- **Score Calculation**: Optimal range 0.5-2.0
- **Target**: Aspect ratio between 0.5 and 2.0

### 11. Parent-Child Sizing (Included in Hierarchy)
- **Metric**: Parents properly sized to contain children
- **Violations**: Parents too small for children
- **Target**: All parents properly sized

## Weighted Score Calculation

```
weightedScore = 
    overlapScore * 0.20 +
    spacingScore * 0.12 +
    edgeCrossingScore * 0.15 +
    edgesOverNodesScore * 0.12 +
    edgeBendScore * 0.08 +
    edgeLengthScore * 0.05 +
    hierarchyScore * 0.15 +
    viewportScore * 0.08 +
    consistencyScore * 0.05 +
    aspectRatioScore * 0.05
```

## Grade System

- **A**: 90-100 (Excellent)
- **B**: 80-89 (Good)
- **C**: 70-79 (Acceptable)
- **D**: 60-69 (Needs Improvement)
- **F**: <60 (Poor)

## Parent-Child Sizing

### Problem
Parent nodes must be properly sized to contain their children to prevent:
- Horizontal distortion (too wide)
- Vertical distortion (too tall)
- Children extending outside parent bounds

### Solution
1. **Post-layout Processing**: After layout, resize parent nodes to fit children
2. **Validation**: Check all parent-child relationships
3. **Aspect Ratio Control**: Ensure diagram maintains reasonable proportions

### Implementation
- `ensureParentChildSizing()` function in `layoutEngine.ts`
- Processes parents from deepest to shallowest
- Calculates required size: `max(child_bounds) + padding`
- Updates parent width/height accordingly

## Aspect Ratio Control

### Optimal Range
- **Minimum**: 0.5 (not too tall)
- **Maximum**: 2.0 (not too wide)
- **Ideal**: 1.0-1.5 (balanced)

### Penalties
- Too tall (< 0.5): Score decreases linearly
- Too wide (> 2.0): Score decreases more aggressively

## Testing

### Playwright Tests

1. **layout-stability.spec.ts**: Position preservation tests
2. **diagram-quality.spec.ts**: Quality metrics across all levels
3. **parent-child-sizing.spec.ts**: Parent-child relationship tests

### Running Tests

```bash
npm run test:e2e
```

## Optimization

### Automatic Optimization

The `optimizeLayout()` function:
1. Tries multiple layout engines (ELK, Sruja, C4Level)
2. Tries different directions (DOWN, RIGHT)
3. Calculates quality score for each
4. Returns best configuration

### Incremental Optimization

The `optimizeLayoutIncremental()` function:
1. Starts with current layout
2. Identifies worst quality aspects
3. Tries to improve specific aspects
4. Accepts improvements that increase overall score

## Recommendations

The system provides actionable recommendations:
- "High node overlap. Consider increasing node spacing."
- "Many edge crossings. Try hierarchical layout."
- "Parent nodes too small. Increase parent sizes."
- "Diagram too tall/wide. Adjust layout direction."

## Usage Example

```typescript
import { calculateDiagramQuality, optimizeLayout } from './utils/diagramQuality';

// Calculate quality
const quality = calculateDiagramQuality(nodes, edges, viewportSize);

// Optimize if needed
if (quality.weightedScore < 80) {
    const optimized = await optimizeLayout({
        nodes,
        edges,
        currentLevel: 'L1',
        viewportSize
    });
    // Use optimized.bestLayout
}
```

## Best Practices

1. **Always validate parent-child sizing** after layout
2. **Check aspect ratio** to prevent distortion
3. **Use weighted scores** for optimization decisions
4. **Run quality tests** before deploying
5. **Monitor quality metrics** across different C4 levels

## Future Enhancements

- [ ] Machine learning-based weight optimization
- [ ] Real-time quality feedback in UI
- [ ] Automatic layout direction selection
- [ ] Edge routing optimization
- [ ] Interactive quality dashboard

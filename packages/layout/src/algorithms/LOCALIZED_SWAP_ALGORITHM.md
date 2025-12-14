# Localized Swap Optimizer Algorithm

## Overview

The localized swap optimizer is a post-processing algorithm that reduces edge crossings in C4 diagrams by swapping adjacent nodes within the same rank. This algorithm implements Section 2.3 of `LayoutBestPractice.md`.

## Algorithm Description

### Purpose

After initial layout and edge routing, this algorithm attempts to reduce edge crossings by strategically swapping adjacent nodes that are in the same horizontal rank (similar Y coordinates).

### Key Concepts

1. **Rank**: Nodes are grouped by their vertical position (Y coordinate). Nodes within `RANK_TOLERANCE` pixels (default: 50px) are considered in the same rank.

2. **Adjacent Nodes**: Within each rank, nodes are sorted by X position (left to right). Adjacent nodes are consecutive nodes in this sorted order.

3. **Containment Constraints**: Swaps are only allowed if they maintain parent-child containment relationships. A node must remain within its parent container's boundaries (with padding).

4. **Parent-Child Protection**: Nodes with direct parent-child relationships are never swapped to preserve hierarchy.

## Algorithm Steps

```
1. Count current edge crossings
   - If crossings = 0, return early (no optimization needed)

2. Group nodes by rank
   - Calculate center Y for each node
   - Round to nearest RANK_TOLERANCE to create rank keys
   - Sort nodes within each rank by X position

3. For each rank:
   - For each pair of adjacent nodes:
     a. Skip if nodes have parent-child relationship
     b. Calculate swapped positions
     c. Validate containment constraints
     d. If valid, apply swap and increment counter
     e. Stop if maxSwaps limit reached

4. Return optimized node positions
```

## Time Complexity

- **Crossing Count**: O(e²) where e is the number of edges
- **Rank Grouping**: O(n log n) where n is the number of nodes
- **Swap Attempts**: O(n × s) where s is maxSwaps (typically small)
- **Overall**: O(e² + n log n + n×s)

For typical diagrams:

- Small (10-50 nodes): < 10ms
- Medium (50-200 nodes): 10-50ms
- Large (200+ nodes): 50-200ms

## Space Complexity

- O(n) for rank groups map
- O(n) for result node map
- **Overall**: O(n)

## Configuration

### Constants

- `RANK_TOLERANCE = 50`: Pixels tolerance for grouping nodes into ranks
- `CONTAINMENT_PADDING = 20`: Minimum padding from parent container boundaries

### Parameters

- `maxSwaps`: Maximum number of swaps to attempt (default: 10)
  - Higher values: More optimization attempts, slower execution
  - Lower values: Faster execution, may miss some optimizations

## Usage Example

```typescript
import { applyLocalizedSwaps } from "./localized-swap-optimizer";

// After initial layout and routing
const result = applyLocalizedSwaps(
  positionedNodes,
  edgeRoutes,
  relationships,
  pathsCross, // Function to detect path crossings
  15 // maxSwaps
);

if (result.improved) {
  // Re-route edges with new positions
  positionedNodes = result.nodes;
  // Recalculate edge routes...
}
```

## Edge Cases Handled

1. **No Crossings**: Early exit if no crossings detected
2. **Empty Node Map**: Returns unchanged
3. **Single Node**: No swaps possible
4. **Parent-Child Relationships**: Protected from swapping
5. **Containment Violations**: Swaps that would break containment are rejected
6. **Rank Boundaries**: Nodes near rank boundaries are correctly grouped
7. **Max Swaps Limit**: Respects limit to prevent excessive computation

## Limitations

1. **Local Optimization**: Only swaps adjacent nodes, may miss global optima
2. **No Crossing Re-evaluation**: Doesn't re-count crossings after each swap
3. **Heuristic-Based**: Uses containment checks rather than actual crossing reduction
4. **Single Pass**: Doesn't iterate to find optimal swap sequence

## Future Improvements

1. **Iterative Refinement**: Re-evaluate crossings after swaps and continue if improved
2. **Crossing-Aware Swaps**: Only swap if it actually reduces crossings
3. **Multi-Node Swaps**: Consider swapping non-adjacent nodes in same rank
4. **Energy-Based Selection**: Choose swaps that maximize crossing reduction
5. **Parallel Processing**: Process multiple ranks in parallel

## Related Algorithms

- **Sugiyama Layout**: Initial hierarchical layout
- **Edge Router**: Routes edges after node positioning
- **Crossing Minimizer**: Alternative approach to reduce crossings
- **Force-Directed Layout**: Physics-based positioning

## References

- Section 2.3 of `LayoutBestPractice.md`
- Graph Drawing Algorithms (Battista et al.)
- ELK Layout Algorithm Documentation

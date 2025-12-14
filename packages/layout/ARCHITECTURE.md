# Layout Package Architecture

## Overview

The `@sruja/layout` package provides C4 model layout algorithms for positioning nodes and routing edges in architecture diagrams.

## Package Structure

```
src/
├── algorithms/          # Core layout algorithms
│   ├── core/           # Fundamental algorithms
│   │   ├── coordinates.ts      # Coordinate calculation
│   │   ├── sizing.ts           # Node sizing
│   │   └── spatial-index.ts    # Spatial indexing for performance
│   ├── hierarchy/      # Hierarchy-specific algorithms
│   │   ├── hierarchy.ts        # Parent-child relationships
│   │   └── overlap.ts          # Overlap detection/resolution
│   ├── routing/        # Edge routing algorithms
│   │   ├── edge-router.ts      # Basic edge routing
│   │   ├── unified-router.ts   # Unified routing strategy
│   │   ├── edge-bundler.ts     # Edge bundling
│   │   └── label-placer.ts     # Label positioning
│   ├── layouts/        # Layout strategies
│   │   ├── flowLayout.ts       # Flow-based layout
│   │   ├── sugiyama.ts         # Sugiyama algorithm
│   │   ├── grid-layout.ts      # Grid-based layout
│   │   ├── l0-layout.ts        # L0 (landscape) layout
│   │   ├── l1-layout.ts        # L1 (context) layout
│   │   └── c4-level-layouts.ts # C4 level-specific layouts
│   └── optimization/   # Optimization algorithms
│       ├── optimizer.ts        # General optimizer
│       ├── self-optimizer.ts   # Self-optimization
│       ├── clutter-detection.ts # Clutter detection
│       └── transitions.ts      # Layout transitions
├── c4-model.ts         # C4 model types and graph creation
├── c4-view.ts          # C4 view state management
├── c4-layout.ts         # Main layout function
├── c4-options.ts       # Layout options and presets
├── presets/            # Layout presets
│   └── default.ts      # Default preset
├── text/               # Text measurement
│   ├── TextMeasurer.ts
│   └── wrap.ts
├── geometry/           # Geometry utilities
│   ├── point.ts
│   └── rect.ts
├── utils/              # Utility functions
│   ├── text-measurer.ts
│   ├── cached-text-measurer.ts
│   ├── canvas-text-measurer.ts
│   ├── immutability.ts
│   └── validation.ts
├── theme.ts            # Theme configuration
├── brand.ts            # Branding
├── constants.ts        # Constants
├── metrics.ts          # Layout metrics
├── plugin.ts           # Plugin system
└── types.ts            # Type definitions
```

## Algorithm Categories

### Core Algorithms

- **Coordinates**: Calculate node positions
- **Sizing**: Determine node dimensions
- **Spatial Index**: Fast spatial queries

### Hierarchy Algorithms

- **Hierarchy**: Manage parent-child relationships
- **Overlap**: Detect and resolve node overlaps

### Routing Algorithms

- **Edge Router**: Route edges between nodes
- **Unified Router**: Multi-strategy routing
- **Edge Bundler**: Bundle related edges
- **Label Placer**: Position edge/node labels

### Layout Strategies

- **Flow Layout**: Flow-based positioning
- **Sugiyama**: Layered graph layout
- **Grid Layout**: Grid-based positioning
- **L0/L1 Layouts**: C4 level-specific layouts

### Optimization

- **Optimizer**: General layout optimization
- **Self-Optimizer**: Iterative self-improvement
- **Clutter Detection**: Identify cluttered areas
- **Transitions**: Smooth layout transitions

## Layout Pipeline

1. **Input**: C4Graph + ViewState + Options
2. **Sizing**: Calculate node sizes
3. **Hierarchy**: Build parent-child structure
4. **Layout**: Apply layout algorithm
5. **Routing**: Route edges
6. **Optimization**: Optimize layout
7. **Output**: PositionedC4Node[] + PositionedC4Relationship[]

## Key Design Principles

1. **Immutability**: Layout functions don't mutate input
2. **Composability**: Algorithms can be combined
3. **Performance**: Use spatial indexing for large graphs
4. **Extensibility**: Plugin system for custom algorithms
5. **Quality**: Metrics-driven optimization

## Usage

```typescript
import { layout, createC4Graph, createDefaultViewState } from "@sruja/layout";

const graph = createC4Graph(nodes, relationships);
const viewState = createDefaultViewState();
const options = {
  /* layout options */
};

const result = layout(graph, viewState, options);
// result.nodes: Map<C4Id, PositionedC4Node>
// result.relationships: PositionedC4Relationship[]
```

## Testing

Tests are organized alongside algorithms:

- `algorithms/__tests__/` - Algorithm unit tests
- `src/__tests__/` - Integration tests

Run tests:

```bash
npm test
```

## Performance Considerations

- Use spatial indexing for graphs with 100+ nodes
- Cache text measurements
- Batch coordinate calculations
- Use appropriate layout strategy for graph size

## Future Improvements

- [ ] Parallel layout computation
- [ ] Incremental layout updates
- [ ] GPU-accelerated routing
- [ ] Machine learning-based optimization

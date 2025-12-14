# Diagram Package Architecture

## Overview

The `@sruja/diagram` package provides React Flow components and utilities for rendering C4 architecture diagrams.

## Package Structure

```
src/
├── components/         # React components
│   ├── nodes/         # Node components
│   │   ├── SystemNode.tsx
│   │   ├── ContainerNode.tsx
│   │   ├── ComponentNode.tsx
│   │   ├── PersonNode.tsx
│   │   ├── DataStoreNode.tsx
│   │   ├── QueueNode.tsx
│   │   ├── TopicNode.tsx
│   │   ├── CacheNode.tsx
│   │   ├── FileSystemNode.tsx
│   │   ├── DeploymentNode.tsx
│   │   ├── ExternalContainerNode.tsx
│   │   ├── ExternalComponentNode.tsx
│   │   ├── SystemBoundaryNode.tsx
│   │   ├── ContainerBoundaryNode.tsx
│   │   ├── EnterpriseBoundaryNode.tsx
│   │   └── index.ts
│   ├── edges/         # Edge components
│   │   ├── RelationEdge.tsx
│   │   ├── RoutedEdge.tsx
│   │   └── index.ts
│   └── Legend.tsx     # Legend component
├── utils/             # Utilities
│   ├── jsonToReactFlow.ts      # Convert JSON to React Flow format
│   ├── srujaLayoutEngine.ts    # Layout engine integration
│   ├── layoutEngine.ts         # Legacy layout (deprecated)
│   ├── layoutRules.ts          # Rules-based layout selection
│   ├── diagramQuality.ts      # Quality metrics calculation
│   ├── colorScheme.ts         # Color scheme utilities
│   ├── exportSVG.ts           # SVG export
│   └── exportPNG.ts           # PNG export
├── types/             # TypeScript types
│   └── index.ts
├── styles/            # CSS styles
│   ├── index.css
│   └── nodes.css
└── index.ts          # Public API
```

## Component Architecture

### Node Components

All node components follow a consistent pattern:

- Accept `Node<C4NodeData>` as props
- Render based on `data.type`
- Support parent-child relationships
- Handle selection and interaction

### Edge Components

- **RelationEdge**: Simple relationship edge
- **RoutedEdge**: Edge with custom routing points

### Layout Integration

1. **JSON to React Flow**: `jsonToReactFlow()` converts ArchitectureJSON to React Flow format
2. **Layout Application**: `applySrujaLayout()` applies layout using `@sruja/layout`
3. **Rule Selection**: `selectLayoutConfig()` chooses layout based on graph characteristics

## Layout Rules System

The layout rules system (`layoutRules.ts`) provides:

- **Context Analysis**: Analyze graph characteristics
- **Rule Matching**: Match rules based on priority
- **Config Selection**: Select optimal layout configuration

### Rule Categories

1. **Complexity-based**: Simple/medium/complex graphs
2. **Hierarchy-based**: Hierarchical vs flat
3. **Level-based**: L0/L1/L2/L3 specific rules
4. **Density-based**: Relationship density
5. **Direction-based**: Bidirectional flows

## Quality Metrics

The `diagramQuality.ts` module calculates:

- Overlap detection
- Spacing violations
- Edge crossings
- Edges over nodes
- Parent-child containment
- Label quality
- Overall weighted score

## Testing

### E2E Tests

Located in `tests/`:

- `iterative-optimization.spec.ts`: Main e2e test
- `utils/metrics-collector.ts`: Metrics collection
- `utils/comparison.ts`: Baseline comparison
- `utils/improvement-suggestions.ts`: Suggestions generator

### Test App

`test-app/` contains a minimal React app for testing:

- Loads examples from `@sruja/shared`
- Applies layout
- Exposes state for e2e tests

## Usage

```typescript
import {
  SystemNode,
  ContainerNode,
  jsonToReactFlow,
  applySrujaLayout,
} from '@sruja/diagram';

// Convert JSON to React Flow format
const { nodes, edges } = jsonToReactFlow(architectureJson, { level: 'L2' });

// Apply layout
const { nodes: laidOutNodes, edges: laidOutEdges } = applySrujaLayout(
  nodes,
  edges,
  architectureJson,
  { level: 'L2', direction: 'TB' }
);

// Use in React Flow
<ReactFlow
  nodes={laidOutNodes}
  edges={laidOutEdges}
  nodeTypes={nodeTypes}
  edgeTypes={edgeTypes}
/>
```

## Iterative Improvement Workflow

See `tests/README.md` for the improvement workflow:

1. Run `npm run improve` to collect metrics
2. Review `tests/results/improvement-suggestions.md`
3. Make improvements based on suggestions
4. Re-run to track progress

## Key Design Principles

1. **Separation of Concerns**: Layout logic in `@sruja/layout`, rendering in `@sruja/diagram`
2. **Rules-based**: Declarative layout selection
3. **Metrics-driven**: Quality metrics guide improvements
4. **Extensible**: Easy to add new node/edge types
5. **Performance**: Efficient rendering for large diagrams

## Future Improvements

- [ ] Virtual scrolling for large diagrams
- [ ] Incremental layout updates
- [ ] Interactive layout adjustment
- [ ] Custom layout strategies
- [ ] Real-time collaboration

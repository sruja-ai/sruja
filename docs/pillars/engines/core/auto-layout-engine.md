# Auto-Layout Engine

**Status**: Core Engine  
**Pillars**: Core (Visualization)

[← Back to Engines](../README.md)

## Overview

The Auto-Layout Engine automatically positions nodes and routes edges in architecture diagrams using constraint-based layout algorithms.

**This engine powers the visual diagram layout for all architecture views.**

## Purpose

The Auto-Layout Engine:

- ✅ Automatically positions components
- ✅ Routes edges optimally
- ✅ Supports multiple layout modes
- ✅ Adapts to different architecture patterns
- ✅ Minimizes edge crossings
- ✅ Groups related components
- ✅ Integrates with ReactFlow

## Layout Modes

### 1. Vertical Hierarchy
Used by: Microservices, E-Commerce, Monolith, SaaS

Pattern:
```
UI
↓
APIs / Gateways
↓
Services
↓
DBs / Externals
```

Rules:
- UI at top center
- Gateways/services below
- Databases at bottom
- Even horizontal spacing
- Minimal edge crossings

### 2. Horizontal Flow
Used by: Event-Driven, Pipelines

Pattern:
```
Producer → Queue/Event Bus → Consumers
```

Rules:
- Producer services on left
- Event bus/queue in center
- Consumers on right
- Sequential flow visualization

### 3. Layered
Used by: Multi-layer architectures

Pattern:
```
Layer 1 (Presentation)
Layer 2 (Application)
Layer 3 (Domain)
Layer 4 (Infrastructure)
```

Rules:
- Clear layer separation
- Components grouped by layer
- Cross-layer dependencies visible

### 4. Radial
Used by: Hub-and-spoke architectures

Pattern:
```
    Service A
        |
    Central Hub
        |
    Service B
```

Rules:
- Central hub in center
- Services arranged in circle
- Radial edge routing

## Architecture

```
AutoLayoutEngine
 ├── LayoutModeSelector
 ├── VerticalHierarchyLayout
 ├── HorizontalFlowLayout
 ├── LayeredLayout
 ├── RadialLayout
 ├── EdgeRouter
 ├── ConstraintSolver
 └── PositionCalculator
```

## Layout Algorithm

### Input
- Architecture model (IR)
- Layout mode preference
- Component metadata
- Grouping information

### Process
1. Analyze component types
2. Group by layers/categories
3. Calculate positions
4. Route edges
5. Optimize spacing
6. Minimize crossings

### Output
- Positioned nodes (x, y coordinates)
- Routed edges (with bend points)
- ReactFlow-compatible format

## Integration with ReactFlow

The engine produces:

```ts
interface LayoutResult {
  nodes: PositionedNode[];
  edges: { id: string; source: string; target: string }[];
}
```

Where `PositionedNode` includes:
- id
- x, y coordinates
- width, height (optional)
- style metadata

## Constraint-Based Layout

The engine respects:

- **Hierarchy constraints** - Parent-child relationships
- **Grouping constraints** - Domain/context boundaries
- **Flow constraints** - Directional flow preferences
- **Spacing constraints** - Minimum distances
- **Alignment constraints** - Layer alignment

## Edge Routing

Edge routing strategies:

- **Straight** - Direct connections
- **Orthogonal** - Right-angle routing
- **Curved** - Bezier curves
- **Smart** - Avoids node overlaps

## MCP API

```
layout.apply(model, mode)
layout.verticalHierarchy(model)
layout.horizontalFlow(model)
layout.layered(model)
layout.radial(model)
layout.optimize(model)
```

## Strategic Value

The Auto-Layout Engine provides:

- ✅ Automatic diagram layout
- ✅ Consistent visual presentation
- ✅ Reduced manual positioning
- ✅ Pattern-appropriate layouts
- ✅ Professional diagram appearance

**This is critical for visual diagram quality and user experience.**

## Implementation Status

✅ Architecture designed  
✅ Layout modes specified  
✅ Algorithm defined  
📋 Implementation in progress

---

*The Auto-Layout Engine automatically positions and routes architecture diagrams.*


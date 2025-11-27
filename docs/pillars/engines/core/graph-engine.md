# Graph Engine

**Status**: Core Engine  
**Pillars**: Core (Graph Operations)

[← Back to Engines](../README.md)

## Overview

The Graph Engine provides a complete graph data structure for architecture models, including node building, edge building, cycle detection, and layer classification.

**This is basically a tiny graph database engine embedded in your architecture compiler.**

## Purpose

The Graph Engine:

- ✅ Builds normalized graph from architecture model
- ✅ Creates DAG structure
- ✅ Detects cycles (Tarjan SCC + DFS)
- ✅ Classifies layers (domain → context → module → container)
- ✅ Builds adjacency lists
- ✅ Provides graph utilities
- ✅ Feeds validation engine

## Graph Types

```ts
export type GraphNodeKind =
  | "domain"
  | "context"
  | "module"
  | "container";

export interface GraphNode {
  id: string;
  kind: GraphNodeKind;
  label: string;
  parent?: string; // module → context → domain
}

export interface GraphEdge {
  from: string;
  to: string;
  relation?: string;
}

export interface ArchitectureGraph {
  nodes: Map<string, GraphNode>;
  edges: GraphEdge[];
  adjacency: Map<string, string[]>;
  reverseAdjacency: Map<string, string[]>;
  cycles: string[][];
}
```

## Node Building

Builds nodes from:

- Domains
- Contexts
- Modules
- Containers

Each node includes:

- id
- kind (domain/context/module/container)
- label
- parent (for hierarchy)

## Edge Building

Builds edges from:

### Hierarchy Edges
- module → context
- context → domain
- container → module

### Relation Edges
- User-defined relations from DSL
- Component dependencies
- Service calls

## Cycle Detection

Uses two algorithms:

### Tarjan SCC (Strongly Connected Components)
Finds all cycles in the graph.

### DFS (Depth-First Search)
Alternative cycle detection method.

## Layer Classification

Classifies nodes by layer:

- **Domain** - Top-level business domains
- **Context** - Bounded contexts
- **Module** - Modules within contexts
- **Container** - Components/containers

## Adjacency Lists

Builds:

- **Forward adjacency** - Outgoing edges
- **Reverse adjacency** - Incoming edges

Used for:

- Dependency analysis
- Impact analysis
- Path finding
- Cycle detection

## Graph Operations

### Path Finding
Find paths between nodes.

### Reachability
Check if node A can reach node B.

### Topological Sort
Order nodes by dependencies.

### Centrality Analysis
Find most connected nodes.

### Subgraph Extraction
Extract subgraphs by criteria.

## Integration Points

### Validation Engine
- Cycle detection for validation rules
- Dependency validation
- Boundary checks

### Hotspot Detection
- Centrality analysis
- Dependency analysis

### Impact Analysis
- Reachability queries
- Dependency chains

## MCP API

```
graph.build(model)
graph.nodes()
graph.edges()
graph.cycles()
graph.adjacency(nodeId)
graph.path(from, to)
graph.reachable(from)
graph.topologicalSort()
```

## Strategic Value

The Graph Engine provides:

- ✅ Foundation for all graph-based analysis
- ✅ Cycle detection
- ✅ Dependency analysis
- ✅ Path finding
- ✅ Graph utilities

**This is critical for validation, analysis, and visualization.**

## Implementation Status

✅ Architecture designed  
✅ Graph types specified  
✅ Algorithms defined  
📋 Implementation in progress

---

*The Graph Engine provides core graph operations for architecture models.*


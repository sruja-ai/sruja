# Causal Graph Generator

**Status**: Cross-Pillar Engine  
**Pillars**: All (Systems Thinking)

[← Back to Engines](../README.md)

## Overview

The Causal Graph Generator builds causal graphs from Systems Thinking models, including loops, causal edges, weights, and delays.

**This provides the graph structure for causal analysis, loop detection, and simulation.**

## Purpose

The Causal Graph Generator:

- ✅ Builds causal graph from Systems Thinking model
- ✅ Generates loop structures
- ✅ Assigns edge weights
- ✅ Handles delays
- ✅ Maps concepts to architecture
- ✅ Enables graph analysis
- ✅ Supports visualization

## Graph Structure

The generator produces a **Causal Graph**:

```ts
interface CausalGraph {
  nodes: CausalNode[];
  edges: CausalEdge[];
  loops: Loop[];
  metadata: GraphMetadata;
}
```

### Causal Node

```ts
interface CausalNode {
  id: string;
  type: "concept" | "component" | "stock" | "flow";
  name: string;
  initialValue?: number;
  mappedComponent?: string;
}
```

### Causal Edge

```ts
interface CausalEdge {
  from: string;
  to: string;
  polarity: "+" | "-" | "0";
  weight: number;
  delay?: number;
  description?: string;
}
```

### Loop

```ts
interface Loop {
  id: string;
  type: "reinforcing" | "balancing";
  nodes: string[];
  edges: CausalEdge[];
  strength?: number;
}
```

## Graph Building Process

### Step 1 — Node Creation
- Create nodes for all concepts
- Create nodes for mapped components
- Create nodes for stocks
- Create nodes for flows

### Step 2 — Edge Creation
- Create edges from causal relationships
- Assign polarity (+/-/0)
- Assign weights
- Assign delays

### Step 3 — Loop Detection
- Detect cycles in graph
- Classify as reinforcing or balancing
- Calculate loop strength

### Step 4 — Architecture Mapping
- Map concepts to components
- Create cross-domain edges
- Resolve component dependencies

## Loop Generation

### Reinforcing Loops
Loops where polarity sum is positive:

```
R1: Traffic +-> Latency +-> Retries +-> Load +-> Traffic
```

Characteristics:
- Exponential growth
- Runaway behavior
- Tipping points

### Balancing Loops
Loops where polarity sum is negative:

```
B1: Demand +-> Price -→ Demand
```

Characteristics:
- Stabilization
- Homeostasis
- Self-correction

## Edge Weight Calculation

Weights can be:

- **Fixed** - Explicitly defined in DSL
- **Inferred** - Based on component relationships
- **Dynamic** - Calculated from simulation

Default weight = 1.0

## Delay Handling

Delays are represented as:

- **Edge property** - Delay before effect
- **Node property** - Processing delay
- **Loop property** - Loop delay

## Architecture Mapping

Maps Systems Thinking concepts to architecture:

```
Traffic -> rps("APIService")
Latency -> latency("APIService")
DBLoad -> load("Database")
```

Creates bidirectional edges:
- Concept → Component
- Component → Concept

## Graph Analysis

The generator enables:

- **Cycle detection** - Find all loops
- **Path analysis** - Find causal paths
- **Centrality** - Find influential nodes
- **Reachability** - Find affected nodes
- **Impact analysis** - Predict effects

## Integration Points

### Systems Thinking Compiler
- Receives compiled Systems Thinking model
- Builds graph structure

### Behavior Simulator
- Uses graph for simulation
- Propagates causal effects

### Reinforcement Loop Analyzer
- Analyzes loops in graph
- Calculates loop strength

### AI Causal Reasoning Engine
- Uses graph for reasoning
- Enables causal explanations

### Visual Simulation Dashboard
- Visualizes causal graph
- Shows loop activation

## Visualization

The generator supports:

- **Causal Loop Diagrams (CLDs)** - Visual representation
- **Loop highlighting** - Color-coded loop types
- **Edge animation** - Show causal flow
- **Delay indicators** - Visualize delays
- **Weight visualization** - Edge thickness

## MCP API

```
graph.build(model)
graph.nodes()
graph.edges()
graph.loops()
graph.path(from, to)
graph.cycles()
graph.centrality()
```

## Strategic Value

The Causal Graph Generator provides:

- ✅ Causal graph structure
- ✅ Loop detection
- ✅ Graph analysis
- ✅ Visualization support
- ✅ Foundation for simulation

**This is critical for Systems Thinking analysis and simulation.**

## Implementation Status

✅ Architecture designed  
✅ Graph structure specified  
✅ Loop detection defined  
📋 Implementation in progress

---

*The Causal Graph Generator builds causal graphs from Systems Thinking models.*


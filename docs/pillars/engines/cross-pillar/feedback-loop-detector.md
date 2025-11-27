# Feedback Loop Detector

**Status**: Cross-Pillar Engine  
**Pillars**: All (Systems Thinking)

[← Back to Engines](../README.md)

## Overview

The Feedback Loop Detector discovers reinforcing and balancing loops in architecture and system models, identifying cycles in causal graphs.

**This provides the foundation for loop analysis and Systems Thinking intelligence.**

## Purpose

The Feedback Loop Detector:

- ✅ Discovers reinforcing loops
- ✅ Discovers balancing loops
- ✅ Detects cycles in causal graphs
- ✅ Identifies hidden loops in architecture
- ✅ Classifies loop types
- ✅ Maps loops to architecture components
- ✅ Enables loop analysis

## Detection Process

```
Causal Graph
   ↓
Cycle Detection
   ↓
Polarity Analysis
   ↓
Loop Classification
   ↓
Loop List
```

## Loop Types

### Reinforcing Loops (R)
Loops that amplify effects:

- Exponential growth
- Runaway behavior
- Vicious/virtuous cycles
- Tipping points

Example:
```
R1: Traffic +-> Latency +-> Retries +-> Load +-> Traffic
```

### Balancing Loops (B)
Loops that stabilize effects:

- Self-correction
- Homeostasis
- Dampening behavior
- Equilibrium

Example:
```
B1: Demand +-> Price -→ Demand
```

## Detection Algorithm

### Step 1 — Build Causal Graph
- Merge architecture dependencies
- Merge Systems Thinking causal links
- Create unified graph

### Step 2 — Cycle Detection
Uses cycle detection algorithms:

- **Johnson's algorithm** - Find all cycles
- **Tarjan SCC** - Strongly connected components
- **DFS** - Depth-first search

### Step 3 — Polarity Analysis
Calculate loop polarity:

```
Loop polarity = product of edge polarities
+ * + * + = + (Reinforcing)
+ * - * + = - (Balancing)
- * - = + (Reinforcing from two negatives)
```

### Step 4 — Classification
Classify based on polarity:

- Positive polarity → Reinforcing
- Negative polarity → Balancing

## Loop Discovery

### Architecture Structural Loops
Loops in component dependencies:

```
ServiceA -> ServiceB -> ServiceC -> ServiceA
```

### System Dynamics Loops
Loops in causal relationships:

```
Traffic -> Latency -> Retries -> Load -> Traffic
```

### Operational Loops
Loops discovered from simulation:

- Retry storms
- Load spirals
- Queue buildup

## Integration Points

### Causal Graph Generator
- Receives causal graph
- Detects cycles

### Reinforcement Loop Analyzer
- Uses detected loops
- Analyzes loop strength

### Behavior Simulator
- Tracks loop activation
- Monitors loop effects

### AI Causal Reasoning Engine
- Explains loops
- Provides loop insights

### Visual Simulation Dashboard
- Visualizes loops
- Shows loop activation

## Output

The detector produces:

```ts
interface DetectedLoop {
  id: string;
  type: "reinforcing" | "balancing";
  nodes: string[];
  edges: CausalEdge[];
  polarity: number;
  description?: string;
}
```

## MCP API

```
detector.detect(graph)
detector.reinforcing(graph)
detector.balancing(graph)
detector.cycles(graph)
```

## Strategic Value

The Feedback Loop Detector provides:

- ✅ Loop discovery
- ✅ Loop classification
- ✅ Foundation for analysis
- ✅ Systems Thinking support

**This is critical for Systems Thinking analysis and loop understanding.**

## Implementation Status

✅ Architecture designed  
✅ Detection algorithms specified  
✅ Classification defined  
📋 Implementation in progress

---

*The Feedback Loop Detector discovers reinforcing and balancing loops in architecture and system models.*


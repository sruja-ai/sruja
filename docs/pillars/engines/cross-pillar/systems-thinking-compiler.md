# Systems Thinking Compiler

**Status**: Cross-Pillar Engine  
**Pillars**: All (Systems Thinking)

[← Back to Engines](../README.md)

## Overview

The Systems Thinking Compiler transpiles DSL v3 (with Systems Thinking extensions) into a simulation model that can be executed by the Behavior Simulator.

**This bridges the gap between declarative Systems Thinking DSL and executable simulation models.**

## Purpose

The Systems Thinking Compiler:

- ✅ Transpiles DSL v3 → simulation model
- ✅ Parses Systems Thinking constructs (causal, loops, stocks, flows)
- ✅ Validates Systems Thinking syntax
- ✅ Builds causal graph structure
- ✅ Resolves concept mappings to architecture
- ✅ Generates simulation-ready IR
- ✅ Integrates with Behavior Simulator

## Compilation Process

```
DSL v3 (with Systems Thinking)
   ↓
Parse Systems Thinking constructs
   ↓
Build Causal Graph
   ↓
Resolve Mappings
   ↓
Generate Simulation Model
   ↓
Behavior Simulator
```

## Systems Thinking Constructs

### Concepts
Defines system variables:

```sruja
concepts {
  Traffic
  Latency
  Retries
  Load
  DBLoad
}
```

### Causal Relationships
Defines how concepts affect each other:

```sruja
causal {
  Traffic +-> Latency delay 200ms
  Latency +-> Retries
  Retries +-> Load
  Load +-> Traffic
}
```

**Polarity:**
- `+->` = A increases B (positive correlation)
- `-->` = neutral relationship
- `-→` = A decreases B (negative correlation)

**Delays:**
- `delay 200ms` - Time delay before effect
- `delay 5s` - Seconds
- `delay 10m` - Minutes

### Feedback Loops
Defines reinforcing and balancing loops:

```sruja
loops {
  R1 reinforcing {
    Traffic -> Latency
    Latency -> Retries
    Retries -> Load
    Load -> Traffic
  }
  
  B1 balancing {
    Demand -> Price
    Price -→ Demand
  }
}
```

**Types:**
- **Reinforcing (R)** - Amplifies effects (vicious/virtuous cycles)
- **Balancing (B)** - Stabilizes effects (self-correcting)

### Stocks & Flows
Defines system dynamics with accumulations:

```sruja
stocks {
  PendingRequests initial 0
  Inventory initial 100
}

flows {
  Incoming -> PendingRequests rate 10
  PendingRequests -> Processed rate rps("APIService")
}
```

### Constraints
Defines NFR constraints:

```sruja
constraints {
  Latency < 200ms depends_on [ APIService, DB ]
  DBLoad < 80%
  PendingRequests < 1000
}
```

### Architecture Mappings
Maps concepts to architecture components:

```sruja
map {
  Traffic -> rps("APIService")
  Latency -> latency("APIService")
  DBLoad -> load("Database")
}
```

## Compilation Output

The compiler produces a **Simulation Model**:

```ts
interface SimulationModel {
  concepts: Concept[];
  causalLinks: CausalLink[];
  loops: Loop[];
  stocks: Stock[];
  flows: Flow[];
  constraints: Constraint[];
  mappings: Mapping[];
}
```

### Concept Model

```ts
interface Concept {
  id: string;
  name: string;
  initialValue?: number;
  type?: string;
}
```

### Causal Link Model

```ts
interface CausalLink {
  from: string;
  to: string;
  polarity: "+" | "-" | "0";
  delay?: number;
  weight?: number;
}
```

### Loop Model

```ts
interface Loop {
  id: string;
  type: "reinforcing" | "balancing";
  steps: string[]; // concept IDs in order
  description?: string;
}
```

### Stock Model

```ts
interface Stock {
  id: string;
  name: string;
  initialValue: number;
}
```

### Flow Model

```ts
interface Flow {
  id: string;
  from?: string;
  to: string;
  rate: RateExpression;
}
```

## Integration Points

### DSL Parser
- Parses Systems Thinking DSL
- Validates syntax
- Builds AST

### AST Transformer
- Transforms Systems Thinking AST
- Builds simulation IR

### Behavior Simulator
- Receives compiled simulation model
- Executes simulation
- Produces time-series results

### Causal Graph Generator
- Builds causal graph from compiled model
- Enables graph analysis

## Validation

The compiler validates:

- Concept references exist
- Causal link targets exist
- Loop steps form valid cycles
- Stock/flow references are valid
- Mappings reference valid components
- Constraints reference valid concepts

## MCP API

```
compiler.compile(dsl)
compiler.validate(model)
compiler.toSimulationModel(ir)
```

## Strategic Value

The Systems Thinking Compiler provides:

- ✅ DSL to simulation model conversion
- ✅ Systems Thinking syntax support
- ✅ Simulation model generation
- ✅ Foundation for behavior simulation

**This is critical for Systems Thinking support and simulation capabilities.**

## Implementation Status

✅ Architecture designed  
✅ Compilation process specified  
✅ Model structures defined  
📋 Implementation in progress

---

*The Systems Thinking Compiler transpiles DSL v3 into executable simulation models.*


# Systems Thinking DSL

This document describes the Systems Thinking DSL extension for modeling system behavior, feedback loops, and causal dynamics.

[← Back to Documentation Index](../README.md)

## Overview

Systems Thinking is **Pillar 3** of the Sruja platform. While most architecture tools focus on **structure**, Systems Thinking allows you to model **system behavior**, **feedback loops**, **trade-offs**, **emergent behavior**, **delays**, and **unintended consequences**.

This is a **major differentiator** - no other architecture tool supports systems thinking modeling.

## Why Systems Thinking?

Architecture is not static. Systems thinking explains:

- ✅ Why architectures fail under load
- ✅ Why small changes cause major failures
- ✅ Why coupling accumulates
- ✅ How delays create instability
- ✅ Why distributed monoliths emerge
- ✅ How user behavior creates indirect load
- ✅ Why operations degrade over time
- ✅ Why cost explodes unexpectedly

This delivers **behavior-based intelligence**, not just structure.

---

## Core Concepts

### 1. Causal Relationships

Model how one element affects another:

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

### 2. Feedback Loops

Model reinforcing and balancing loops:

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

### 3. Stocks & Flows

Model system dynamics with accumulations:

```sruja
stocks {
  PendingRequests initial 0
  Inventory initial 100
}

flows {
  Incoming -> PendingRequests rate rps("API")
  Processed -> PendingRequests rate rps("Worker")
  
  Production -> Inventory rate 10
  Sales -> Inventory rate -8
}
```

**Stocks** - Accumulated quantities (queue depth, inventory, users)
**Flows** - Rates of change (incoming, outgoing, processing)

### 4. Concepts

Define system concepts (variables):

```sruja
concepts {
  Traffic
  Latency
  Retries
  Load
  Cost
  UserSatisfaction
}
```

### 5. Constraints

Model NFRs and goals:

```sruja
constraints {
  latency < 200ms depends_on [ APIService, DB ]
  cost <= $3000/month
  throughput > 5k_rps
}
```

### 6. Architecture Mapping

Link system concepts to architecture components:

```sruja
map {
  APIService -> Traffic
  Database -> Latency
  RetryPolicy -> Retries
}
```

---

## Complete Example

```sruja
system "Payment-Load-Loop" {
  concepts {
    Traffic
    Latency
    Retries
    Load
    UserSatisfaction
  }

  causal {
    Traffic +-> Latency delay 200ms "More traffic increases latency"
    Latency +-> Retries "High latency causes retries"
    Retries +-> Load "Retries increase backend load"
    Load +-> Latency "High load increases latency"
    Latency -→ UserSatisfaction "High latency reduces satisfaction"
  }

  loops {
    R1 reinforcing "Traffic-Latency-Retry Loop" {
      Traffic -> Latency
      Latency -> Retries
      Retries -> Load
      Load -> Latency
      Latency -> Traffic
    }
    
    B1 balancing "Auto-scaling Loop" {
      Load -> Scaling
      Scaling -→ Load
    }
  }

  stocks {
    PendingRequests initial 0
    QueueDepth initial 0
  }

  flows {
    Incoming -> PendingRequests rate rps("API")
    Processed -> PendingRequests rate rps("Worker")
    
    Enqueued -> QueueDepth rate rps("Producer")
    Dequeued -> QueueDepth rate -rps("Consumer")
  }

  constraints {
    latency < 200ms depends_on [ APIService, DB ]
    queue_depth < 10000
    user_satisfaction > 0.8
  }

  map {
    APIService -> Traffic
    Database -> Latency
    RetryPolicy -> Retries
    WorkerPool -> Load
  }
}
```

---

## Integration with Architecture

Systems thinking integrates with the core architecture model:

### Linking to Components

```sruja
workspace {
  model {
    system PaymentSystem {
      container APIService
      container Database
    }
  }
  
  systems_thinking {
    system "Payment-Load-Loop" {
      map {
        APIService -> Traffic
        Database -> Latency
      }
    }
  }
}
```

### Linking to Performance DSL

```sruja
performance {
  latency APIService { p99: 120ms }
}

systems_thinking {
  system "Latency-Feedback" {
    causal {
      APIService.latency +-> UserRetries
      UserRetries +-> APIService.load
      APIService.load +-> APIService.latency
    }
  }
}
```

---

## Systems Thinking Engine

The Systems Thinking Engine provides:

### 1. Causal Loop Diagram Generation
Automatically generates CLDs from causal relationships

### 2. Feedback Loop Detection
AI-assisted detection of:
- Reinforcing loops (amplification)
- Balancing loops (stabilization)
- Hidden loops in architecture

### 3. System Dynamics Simulation
Quantitative simulation of:
- Stock accumulation
- Flow rates
- Delay effects
- Emergent behavior

### 4. Leverage Point Detection
Identifies high-impact areas to change:
- Remove synchronous bottlenecks
- Introduce queues
- Add caches to break reinforcement
- Remove shared DB to break negative loops

### 5. Trade-off Explorer
For any decision, shows:
- Short-term effects
- Long-term effects
- Side effects
- Risk propagation
- Unintended consequences

---

## Use Cases

### 1. Retry Storm Analysis
Model how retries create reinforcing loops:

```sruja
loops {
  R1 reinforcing "Retry Storm" {
    ServiceFailure -> Retries
    Retries -> BackendLoad
    BackendLoad -> ServiceFailure
  }
}
```

### 2. Auto-scaling Dynamics
Model balancing loops in scaling:

```sruja
loops {
  B1 balancing "Auto-scaling" {
    Load -> ScaleOut
    ScaleOut -→ Load
  }
}
```

### 3. Cost Escalation
Model how usage drives cost:

```sruja
causal {
  Users +-> APIRequests
  APIRequests +-> ComputeCost
  ComputeCost -→ Budget
  Budget -→ FeatureDevelopment
}
```

### 4. Team Dependency Loops
Model organizational dynamics:

```sruja
loops {
  R1 reinforcing "Team Dependency" {
    TeamADependency -> TeamBLoad
    TeamBLoad -> TeamBDelays
    TeamBDelays -> TeamADependency
  }
}
```

---

## DSL v3 Integration

Systems Thinking is part of **DSL v3** (future):

- **DSL v0.1** - Basic architecture (MVP)
- **DSL v1** - Full architecture modeling
- **DSL v2** - ADRs, journeys, requirements
- **DSL v3** - Systems thinking, causal models, scenarios

---

## Grammar Reference

### Top-Level Structure

```sruja
system <name>? {
  concepts { <ConceptDef>* }
  causal { <CausalRel>* }
  loops { <LoopDef>* }
  stocks { <StockDef>* }
  flows { <FlowDef>* }
  constraints { <ConstraintDef>* }
  map { <MappingDef>* }?
}
```

### Causal Relationships

```
<Identifier> <PolarityArrow> <Identifier> (delay <Duration>)? (<StringLiteral>)?
```

**PolarityArrow:**
- `+->` - Positive (increases)
- `-->` - Neutral
- `-→` - Negative (decreases)
- `->` - Neutral (fallback)

### Feedback Loops

```
<LoopName> <LoopType> <StringLiteral>? "{" <LoopStep>* "}"
```

**LoopType:**
- `reinforcing` - Amplifying loop
- `balancing` - Stabilizing loop

### Stocks

```
<Identifier> initial <Number>
```

### Flows

```
<Identifier> -> <Identifier> rate <RateExpr>
```

**RateExpr:**
- `<Number>` - Fixed rate
- `rps("<Component>")` - Rate from component
- `per_minute <Number>` - Per-minute rate

### Constraints

```
<Identifier> <ConstraintOp> <ConstraintValue> (depends_on [ <IdentifierList> ])?
```

**ConstraintOp:** `<`, `<=`, `>`, `>=`, `=`

---

## Visualization

The Systems Thinking Engine generates:

- **Causal Loop Diagrams (CLDs)** - Visual representation of causal relationships
- **Reinforcing (R) vs Balancing (B) loops** - Color-coded loop types
- **Stock/Flow charts** - System dynamics diagrams
- **Heatmaps** - Feedback strength visualization
- **Delay indicators** - Time delay visualization
- **Simulation graphs** - Behavior over time
- **Team/system interaction overlays** - Organizational dynamics

---

## AI Integration

AI can:

- **Detect hidden loops** in architecture
- **Explain system behavior** in causal terms
- **Suggest leverage points** for improvement
- **Predict emergent behavior** from changes
- **Generate causal narratives** from architecture

Example AI reasoning:

> "Your retry policy with exponential backoff increases load multiplicatively in failure scenarios. Combined with shared DB, this creates a reinforcing failure loop that can collapse the system."

---

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| Causal Relationships | 📋 Planned | Grammar defined |
| Feedback Loops | 📋 Planned | Grammar defined |
| Stocks & Flows | 📋 Planned | Grammar defined |
| CLD Generation | 📋 Planned | Visualization |
| Loop Detection | 📋 Planned | AI-assisted |
| System Dynamics Simulation | 📋 Planned | Quantitative |
| Architecture Mapping | 📋 Planned | Integration |

---

## Business Value

Systems Thinking is **highly differentiating** for large enterprises:

- **CTOs** love system thinking
- **Architects** need causal reasoning
- **Org transformation teams** use system models
- **SLA/NFR teams** need dynamics
- **DevOps/SRE** see system behavior daily
- **Agile/SAFe teams** require dependency modeling
- **Product teams** want behavior understanding

This transforms Sruja from a **diagramming tool** into a **holistic system design intelligence platform**.

---

## References

- [DSL Overview](./dsl-overview.md)
- [DSL Extensions](./dsl-extensions.md)
- [Engines Guide](../guides/engines.md) - Systems Thinking Engine details
- Conversation file - Complete Ohm grammar for Systems Thinking DSL

---

*Systems Thinking is a visionary feature that positions Sruja as a next-generation architecture intelligence platform.*


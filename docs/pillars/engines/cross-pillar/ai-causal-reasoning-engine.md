# AI Causal Reasoning Engine (ACRE)

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (Systems Thinking, Reliability, Performance)

[← Back to Engines](../README.md)

## Overview

The AI Causal Reasoning Engine (ACRE) is the "brain" of the architecture intelligence platform. It enables AI to understand, reason about, predict, and explain architecture as a **living system**.

## Purpose

ACRE must:
- Understand the **causal structure** from DSL (causal links, loops, delays, stocks, flows)
- Understand **architecture structure** (components, boundaries, dependencies)
- Combine **static + dynamic + behavioral** models
- Answer natural-language questions
- Explain **why** something is happening (root cause)
- Predict **what will happen** (forward simulation)
- Detect **emergent behavior** (feedback loops, bottlenecks, cascading failure)
- Suggest **architectural fixes**
- Suggest **system behavior fixes**
- Write **ADR proposals** automatically

## Architecture

```
AI Causal Reasoning Engine (ACRE)
 ├── System Interpreter (Architecture + System DSL → Knowledge Graph)
 ├── Causal Graph Builder
 ├── Loop Analyzer
 ├── Constraint Interpreter
 ├── Behavior Summary Generator
 ├── Reasoning Layer (LLM-based)
 │      ├── Causal Question Answering
 │      ├── Counterfactual Reasoning
 │      ├── Predictive Reasoning
 │      ├── Mitigation Suggestions
 ├── Explanation Layer (Natural Language)
 │      ├── Root Cause Reports
 │      ├── Impact Narratives
 │      ├── Loop Explanations
 │      ├── ADR Draft Generator
 ├── Integration API (MCP)
```

## Inputs

### From Architecture DSL IR:
- Components
- Boundaries/domains
- Dependencies
- Events
- Relations
- Layers
- Teams
- NFR constraints

### From Systems Thinking DSL IR:
- Concepts
- Causal relationships
- Delays
- Stocks
- Flows
- Reinforcing/Balancing loops
- Mappings to architecture

### From Behavior Simulator:
- Time series
- Queue depth
- Constraint violations
- Loop activation events
- Instability metrics

## Core Data Structures

### Causal Graph (CG)
**Nodes:**
- Concepts
- Mapped Components
- Stock values
- Flow rates

**Edges:**
- causality
- polarity
- magnitude
- delay

### Architecture Dependency Graph (ADG)
**Nodes:**
- components
- services
- databases
- events

**Edges:**
- calls
- messages
- data flows

### Loop Graph (LG)
**Nodes:**
- loop names

**Edges:**
- reinforcing → +
- balancing → –

## Intelligence Layers

### 1. Causal Question Answering

Questions like:
> "Why is latency increasing?"  
> "What happens if APIService slows down?"  
> "What causes DBLoad to spike?"  
> "How do retries affect the system?"  
> "What breaks this constraint?"

**Engine uses:**
1. Causal graph traversal
2. Loop analysis
3. Scenario playback
4. Architecture mapping
5. NFR constraints
6. LLM reasoning

### 2. Counterfactual Reasoning

> "What if we remove retries?"  
> "What if DB latency increases by 50%?"  
> "What if traffic grows 5×?"  
> "What if we split PaymentProcessor?"

**Steps:**
1. Modify IR
2. Re-compose
3. Re-run behavior simulation
4. Compare results
5. Summarize differences

### 3. Predictive Reasoning

Predict future system behavior:
- impending overload
- queue buildup
- retry explosion
- cost escalation
- failure cascades
- loop amplification

**LLM uses:**
- time-series trends
- loop polarity
- delays
- stock trends
- dependency weaknesses

### 4. Root Cause Analysis Engine

Given a violation or observed metric spike:

1. Identify affected variable
2. Find upstream causal contributors
3. Rank by influence
4. Detect reinforcing loops involved
5. Trace path across architecture
6. Determine earliest cause
7. Produce root cause explanation

**Example output:**

> Latency increased because:
> 1. PendingRequests increased due to Incoming > Processed
> 2. APIService traffic surged 2.8×
> 3. Retry storm triggered reinforcing loop R1
> 4. DBLoad exceeded 80% → slower queries → more retries

### 5. Mitigation Recommendation Engine

Automatically suggests:
- break the loop
- add caching
- introduce circuit breaker
- add bulkheads
- reduce retry aggression
- add async messaging
- autoscale
- redefine domain boundaries

### 6. ADR Generator

Produces:
- Context
- Problem
- Decision
- Alternatives
- Consequences
- Causal reasoning
- Simulation data

## AI Layers

### Reasoning LLM
**Task:** Structured graph reasoning + causal inference

**Must support:**
- chain-of-thought (internally)
- graph reasoning
- time-series reasoning
- counterfactual evaluation
- systems thinking reasoning

### Narrative LLM
**Task:** Explain results in human-friendly narratives

**Produces:**
- Plain language reports
- Loop explanations
- Root cause timelines
- System stories
- "Flight recorder" narrative

### Recommendations LLM
**Task:** Produce architecture fixes, system fixes, NFR improvements, design alternatives

**Uses:**
- heuristics
- best practices
- domain patterns
- anti-pattern libraries

## Pipeline Execution Flow

```
Query →
  ACRE Kernel →
    Load IR →
      Build causal graph →
      Identify active loops →
      Run partial simulation →
      Compute influence factors →
      Determine causal path →
      Summarize →
      Generate AI narrative →
        Return structured + natural-language output
```

## MCP API

ACRE exposes endpoints:

### `causal.explain(variable)`
Explain why something changed.

### `causal.rootCause(violationId)`
Find root cause.

### `causal.predict(variable, horizon)`
Predict what will happen.

### `causal.whatIf(changeSet)`
Simulate counterfactual.

### `causal.loopAnalysis()`
Explain loops.

### `causal.suggestMitigations(variable)`
Recommend design changes.

### `causal.generateADR(violationId)`
Produce ADR automatically.

### `causal.archImpact(component)`
Show cross-backpressure / load propagation.

## Frontend Visualization

The engine feeds:
- interactive causal-path visualizer
- loop glow overlays
- influence heatmaps
- NFR violation graph
- scenario playback timeline
- system behavior animation
- prediction charts

## Implementation Phases

### Phase 1: Foundations
- ✅ Build causal graph builder
- ✅ Build loop detector
- ✅ Build influence calculator
- ✅ Integrate with simulator

### Phase 2: AI Reasoning
- ✅ Manual prompt templates
- ✅ LLM chain for:
  - causal tracing
  - loop reasoning
  - root cause analysis

### Phase 3: Narratives + Mitigation
- ✅ Human-friendly explanations
- ✅ Mitigation suggestion engine
- ✅ Pattern library:
  - CQRS
  - Saga
  - Hexagonal
  - Anti-corruption layer
  - Event storming
  - Resilience patterns

### Phase 4: Advanced Features
- ✅ Counterfactual engine
- ✅ Predictive reasoning engine
- ✅ AI ADR writer
- ✅ Architecture behavior stories

### Phase 5: UX
- ✅ Visualization
- ✅ Time-travel narrative
- ✅ "Explain this diagram" feature
- ✅ "Why did this happen?" context menu

## Impact

The AI Causal Reasoning Engine gives your platform:
- ✅ Real System Intelligence
- ✅ Predictive Architecture Analysis
- ✅ Deep, Explainable System Modeling
- ✅ Automatic Root Cause Analysis
- ✅ Design Mitigation Recommendations
- ✅ ADR Auto-Generator
- ✅ Powerful AI collaboration layer

**No architecture platform today has this.**

---

*ACRE transforms Sruja from a diagramming tool into a real system reasoning engine for organizations.*


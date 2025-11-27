# Behavior Sensitivity Analysis Engine (BSAE)

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (All - Sensitivity Analysis)

[← Back to Engines](../README.md)

## Overview

The Behavior Sensitivity Analysis Engine (BSAE) quantifies how sensitive your architecture is to load, latency, failures, retries, delays, and topology changes.

**This is the scientific layer required for designing resilient architectures.**

## Purpose

The Behavior Sensitivity Analysis Engine (BSAE):

- ✅ Computes impact of **each parameter** on system behavior
- ✅ Identifies **most sensitive metrics**
- ✅ Detects **tipping points** (phase transitions)
- ✅ Maps which **components** or **loops** cause instability
- ✅ Supports what-if planning
- ✅ Powers AI insights and optimization
- ✅ Enables architecture hardening before production
- ✅ Drives automated scenario generation
- ✅ Integrates with Ranking + Orchestration + Hotspot engines

**This is exactly what SREs and architects need—but no modeling tool provides.**

## Inputs

### From Multi-Simulation Orchestration
- parameter sweeps
- scenario expansions
- chaos tests
- A/B architecture versions

### From Simulation Engine
Time-series for:

- latency
- throughput
- error rate
- queue depth
- retries
- load
- stock/flow data
- constraint signals
- loop activation
- stability index

### From Systems Thinking Model
- causal graph
- reinforcement loops
- delays
- stocks & flows

### From Hotspot Engine
- hotspot severity
- centrality shifts
- drift signals

## Architecture

```
BehaviorSensitivityAnalysisEngine (BSAE)
 ├── ParameterImpactAnalyzer
 ├── ElasticityAnalyzer
 ├── TippingPointDetector
 ├── InstabilitySensitivityAnalyzer
 ├── LoopSensitivityAnalyzer
 ├── ComponentSensitivityProfiler
 ├── ConstraintSensitivityAnalyzer
 ├── InteractionEffectAnalyzer
 ├── SensitivityIndexCalculator
 ├── AI Explainer
 └── BSAE API (MCP)
```

## Types of Sensitivity Analyses

### 1. Parameter Sensitivity
Measures how much system response changes in respect to:

- load
- latency
- retry counts
- concurrency
- errors
- delays
- network partitions

Formula:

```
Sensitivity(M, P) = ∂M / ∂P
```

Discrete approximation from simulation data:
```
ΔM / ΔP
```

### 2. Elasticity Analysis
How well the system scales:

```
Elasticity = ΔThroughput / ΔLoad
```

Low elasticity → bottlenecks / hotspots.

### 3. Tipping Point Detection
Identify load values where system *suddenly collapses*:

- latency spikes
- queue overflow
- retry storms
- error avalanche
- loop overrun

Algorithm:

```
Detect knee-points in metric curves (Kneedle Algorithm)
```

### 4. Instability Sensitivity
Measure how small changes produce big oscillations:

Related to delays & balancing loops.

### 5. Loop Sensitivity
For each loop:

```
loopSensitivity = ∂AI / ∂P
```

Answers:

- Which parameters accelerate the loop?
- What load is required to trigger it?
- What input makes it blow up?

### 6. Component Sensitivity
How sensitive is the system to a specific component?

```
componentSensitivity = ∂system_risk / ∂component_load
```

Useful for:

- identifying fragile services
- API Gateway choke points
- DB overload triggers

### 7. Constraint Sensitivity
Which parameters cause constraint violations?

- latency < 200ms
- db_load < 80%
- cost < $1k/hour
- retries < 3

Outputs:

```
constraint_trigger(P) → list of P values causing violations
```

### 8. Interaction Effect Analysis
Parameters interacting in non-linear ways:

```
load × latency
load × retry
delay × concurrency
```

Detects emergent patterns via statistical techniques:

- ANOVA
- Sobol indices
- Mutual information
- Gaussian process regression

### 9. Bottleneck Sensitivity
How bottleneck location shifts with parameter changes.

Example:

```
DB is bottleneck at load < 2000
Queue is bottleneck at load > 2000
```

### 10. Error Propagation Sensitivity
How errors propagate differently under small changes.

### 11. Resilience Sensitivity
How system recovers from failures based on:

- load level
- region failure
- network partition

### 12. Architecture Variant Sensitivity
"Which version is most robust to change?"

## Output: Sensitivity Index

The core metric:

```
SensitivityIndex = normalize(impactMagnitude × instability × loopRisk)
```

0 = insensitive  
100 = extremely sensitive

Used to:

- score architecture versions
- detect fragile zones
- guide feature work
- support design decisions
- inform scenario generation

## Visualizations

### Parameter → Metric Curves
(load vs latency)  
(retry vs cost)  
(delay vs error rate)  
(latency vs loop activation)

### Heatmap: Sensitivity Matrix
```
          Load   Latency   Retries   Delay
Latency   0.82    0.45       0.71     0.68
Risk      0.91    0.50       0.22     0.81
Loops     0.77    0.33       0.84     0.90
```

### Tipping Point Indicators
Vertical lines at collapse points.

### Loop Sensitivity Graph
Arrows thicken as sensitivity increases.

### Component Fragility Tree
Tree map with fragility scores.

## MCP API

```
sensitivity.run({model, scenario})
sensitivity.matrix({model, scenarios})
sensitivity.tippingPoints()
sensitivity.components()
sensitivity.loops()
sensitivity.constraints()
sensitivity.explain()
```

## Implementation Status

✅ Architecture designed  
✅ Sensitivity types defined  
✅ Analysis algorithms specified  
📋 Implementation in progress

---

*BSAE transforms simulation data into actionable insights about system sensitivity and fragility.*



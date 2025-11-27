# Scenario Comparison Engine (SCE)

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (All - Comparison)

[← Back to Engines](../README.md)

## Overview

The Scenario Comparison Engine (SCE) compares simulations, detects impacts, and quantifies differences between architecture versions and scenarios.

**It turns your platform into a scientific architecture lab.**

## Purpose

The SCE must compare:

- ✅ Architecture alternatives (v1 vs v2 vs v3)
- ✅ Behavior models (systems thinking loops)
- ✅ Performance outcomes (simulation)
- ✅ Risk & resilience profiles
- ✅ Cost estimates
- ✅ NFR outcomes (latency, throughput, durability)
- ✅ Causal changes (why the difference happened)
- ✅ Visual diagrams (structural diff + dynamics diff)

**It is the scientific analysis layer of the architecture platform.**

## Inputs

### Architecture Inputs
- Architecture Model vA
- Architecture Model vB
- IR: Components, Domains, Contexts, Events
- IR: Systems Thinking Model (concepts, loops, stocks, flows)

### Simulation Inputs
From Behavior Simulator:

**Time-series**:
- metric(time) for each concept
- stock levels
- flow rates
- latency / load
- error rates
- retry patterns
- bottleneck detection events
- constraint violation signals

**Loop activation logs**:
- loop activation intensity
- amplification scores
- stability drift

**Operational logs**:
- failures
- queue overflow
- thrashing
- circuit breaker triggers

## Architecture

```
ScenarioComparisonEngine (SCE)
 ├── ModelComparator
 │     ├── structural diff
 │     ├── semantic diff
 │     ├── systems diff
 ├── SimulationComparator
 │     ├── metric time-series diff
 │     ├── loop activation diff
 │     ├── constraint diff
 │     ├── stability diff
 │     ├── risk profile diff
 ├── Analyzer
 │     ├── root cause (structural → behavioral)
 │     ├── causal chain explanation
 │     ├── emergent behavior discovery
 ├── ImpactScorer
 │     ├── performance score
 │     ├── reliability score
 │     ├── cost score
 │     ├── robustness score
 ├── VisualDiffBuilder
 │     ├── architecture diagram diff
 │     ├── dynamic system diff (heatmap)
 │     ├── time-lapse playback comparison
 └── MCP API (compare.*)
```

## Top-Level Outputs

The SCE produces:

### Structural diff
What changed in architecture?  
(New components, removed edges, moved boundaries, changed events)

### Behavioral diff
How did feedback loops differ?  
What new loops emerged?  
Which loops disappeared?

### Simulation diff
Side-by-side time series with deltas:

```
latency_vA[0..T]
latency_vB[0..T]
delta = B - A
```

### NFR outcome diff
Did latency worsen or improve?  
Did availability increase?  
Did throughput drop?

### Risk & Stability diff
Which version is more stable?  
Which version risks runaway reinforcement loops?

### Cost diff
Which version is cheaper?  
Which version reduces load on expensive resources?

### Causal explanation
WHY did the difference occur?  
(Most valuable output)

## Model Comparator

The ModelComparator performs:

### Structural Differences
```
added.components
removed.components
changed.properties
added.relations
removed.relations
domain.moves
context.moves
```

Output example:

```
+ Added: CacheService
- Removed: LegacyBilling
~ Modified: DB (storage class: standard → premium)
```

### Systems Thinking Differences

#### Concepts diff
(new concepts, removed concepts)

#### Polarity diff
(positive → negative shift)

#### Loop signature diff
- new loops introduced
- loops removed
- loop structure changed
- loop polarity changed
- amplification changed

#### Stock/Flow diff
resource accumulation shifted.

## Simulation Comparator

### Time-Series Diff
For each metric M:

```
ΔM(t) = MB(t) - MA(t)
maxΔ
avgΔ
trendΔ
% change
```

We compare metrics like:

- latency
- throughput
- DB load
- queue depth
- error rate
- traffic
- retries
- CPU & memory predictions

### Loop Activation Diff
For each loop L:

```
Δactivation = B.activation - A.activation
Δamplification = B.AI - A.AI
Δpolarity (if changed)
```

Example output:

```
Loop R1 (Retry Storm):
  Amplification +1.4 (B worse)
  Activation Time earlier by 8s
  Latency impact +20%
```

### Constraint Impact
Compare constraints:

```
latency < 200ms
cost < $1.2k/hr
db load < 80%
eventual consistency < 50ms drift
```

SCE computes:

- constraint violation count
- average violation duration
- violation severity

### Stability Analysis (very important)
Compute:

```
StabilityScore = 1 / (sum(reinforcingLoops * AI))
```

Compare stability:

```
Version A Stability: 0.63
Version B Stability: 0.42 (less stable)
```

## Analyzer (Cause & Effect)

Core of the engine.

### Analyzer maps:

```
Structural Change → Behavioral Change → Simulation Change → NFR Change
```

Example:

```
Added CacheService
↓
Reduced DBLoad
↓
Weakens Loop R3
↓
Latency decreases by 18%
↓
Stability improves by 0.21
```

**This is the intelligence layer.**

## MCP API

```
compare.models(archA, archB)
compare.simulations(simA, simB)
compare.scenarios(scenarioA, scenarioB)
compare.explain(diffId)
compare.impact(diffId)
compare.visualize(archA, archB)
```

## UI Features

### Side-by-Side Comparison
Visual diff of architecture diagrams.

### Time-Series Overlay
Compare metrics over time.

### Loop Comparison
Show loop differences visually.

### Impact Summary
Clear summary of all differences.

## Implementation Status

✅ Architecture designed  
✅ Comparison algorithms specified  
✅ Diff types defined  
📋 Implementation in progress

---

*SCE provides scientific analysis of architecture differences through simulation comparison.*



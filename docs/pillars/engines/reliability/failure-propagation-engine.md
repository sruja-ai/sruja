# Failure Propagation Engine (FPE)

**Status**: Advanced Engine  
**Pillars**: Reliability

[← Back to Engines](../README.md)

## Overview

The Failure Propagation Engine (FPE) simulates cascading failures, retry storms, overload chains, backpressure shockwaves, and domino effects inside your architecture.

**It turns your architecture into a causal failure graph.**

## Purpose

The Failure Propagation Engine (FPE):

- ✅ Simulates distributed system failure patterns
- ✅ Models cascading effects across architecture
- ✅ Predicts failure blast radius
- ✅ Detects critical fragility paths
- ✅ Identifies retry loops & feedback spirals
- ✅ Feeds heatmaps, replay engine, sensitivity engine
- ✅ Supports scenario DSL ("fail node", "inject latency", "drop packets")
- ✅ Helps AI reason about resilience & recovery

**This is one of the BIGGEST missing pieces in architecture tools.**

## Input Sources

### Architecture Graph
- nodes (services, DBs, queues)
- edges (calls, events, streams)
- domains & boundaries
- constraints (SLAs, SLOs, timeouts)
- component metadata (retry policy, concurrency limits, queue sizes)

### Scenario Inputs
- fail node X
- inject latency L on edge
- drop-rate = 10%
- retry policy change
- region outage
- network partition

### Metrics & Behavioral Models
- latency models
- concurrency models
- retry models
- queue behavior
- fallback routes
- throttling / backpressure logic

## Outputs

The engine outputs **per-timestamp**:

- failure states (failed / degraded / recovering)
- retry waves
- backpressure levels
- shock propagation path
- cascading failure chains
- error amplification
- recovery patterns
- blast radius score
- resilience score

Also outputs **failure signature**, used for:

- ranking architectures
- informing sensitivity engine
- pattern recognition

## Failure Models Supported

This engine includes the **12 major classes of distributed system failures**:

### 1. Latency spike → Retry storm
Classic AWS architecture meltdown mode:

```
latency ↑ → retries ↑ → load ↑ → latency ↑
```

### 2. Timeout waves
Timeout x concurrency → thread starvation.

### 3. Queue overflow collapse
Message queues backing up → overpowering producer.

### 4. Backpressure shockwaves
Downstream slow → upstream throttling.

### 5. Memory exhaustion
Load spike → caching → OOM → node restart → retry storm.

### 6. Circuit breaker cascade
If one circuit opens → load shifts → cascade opens.

### 7. Event storm amplification
Pub/sub recursively amplifies events.

### 8. Network partition divergence
Split-brain models.

### 9. Region failure → multi-region spillover

### 10. Dependency fan-out collapse
One node failure → 10 downstream → 100 downstream.

### 11. Thread pool starvation

### 12. Balancing loop loss → runaway behavior
When balancing loops fail, reinforcement loops explode.

## Architecture

```
FailurePropagationEngine
 ├── FailureStateModel
 ├── PropagationGraphBuilder
 ├── ShockwaveSimulator
 ├── RetryStormSimulator
 ├── BackpressureModel
 ├── QueueOverflowModel
 ├── CircuitBreakerModel
 ├── NetworkPartitionSimulator
 ├── FanoutFailureAnalyzer
 ├── BlastRadiusCalculator
 ├── RecoveryModel
 ├── MCP Interface
```

## Failure Propagation Algorithm (Simplified)

For each timestamp:

```
1. Apply scenario injections
2. Update node states
3. Compute latency impacts
4. Compute retry amplification
5. Compute load redistribution
6. Apply queue/backpressure models
7. Detect shock propagation
8. Detect collapse points
9. Update recovery timers
10. Emit frame to replay engine
```

Fully deterministic simulation unless stochastic mode enabled.

## Blast Radius Calculation

For any root failure:

```
BlastRadius = count(nodes_affected) + weight(edges_affected) + loopSensitivityFactor
```

Weighted by:

- criticality
- domain boundaries
- SLO impact
- sensitivity indices

## Recovery Model

The engine simulates:

- exponential backoff
- circuit breaker cooldown
- leader election
- pod respawn
- queue drain-down
- fallback activation

This fills the replay timeline with realistic trajectories.

## UI Visualizations

### Cascading Failure Ripple
Animated ripple effect showing shockwave.

### Retry Pulse
Pulsating wave on edges with retry storms.

### Circuit Breaker Glow
Edges glow amber → red when breaking.

### Loop Runaway Visual
Loop arrow thickens and spins faster.

### Collapse Wave
Node fades → shrinks → greys out.

### Recovery Path
Node fades back in using soft glow.

## MCP API

```
failure.inject({type, target, params})
failure.run({model, scenario})
failure.frame(timestamp)
failure.chain(rootNode)
failure.blastRadius(rootNode)
failure.explain(rootNode)
failure.recoveryPlan()
failure.compareScenarios(a, b)
```

## Implementation Stages

### Stage 1 — Failure State Model
✅ node-level failure modeling

### Stage 2 — Propagation Graph
✅ direction-aware propagation

### Stage 3 — Retry Storm Simulator
✅ exponential backoff
✅ retry-load amplification

### Stage 4 — Backpressure + Queue Modeling
✅ real traffic modeling

### Stage 5 — Circuit Breaker / Timeout Modeling

### Stage 6 — Cascading Shockwave Engine

### Stage 7 — Recovery Engine

### Stage 8 — Replay + Heatmap Integration

### Stage 9 — AI "Failure Explanation"

## Impact

This engine transforms your architecture platform into:

- ✅ A real resilience simulator
- ✅ A cascading failure lab
- ✅ A predictive SRE tool
- ✅ A dynamic chaos engineering environment
- ✅ A digital twin for failure analysis
- ✅ A foundation for AI-driven reliability planning

**No other diagramming tool or modeling platform does this.**

## Implementation Status

✅ Architecture designed  
✅ Failure models specified  
✅ Propagation algorithm defined  
📋 Implementation in progress

---

*FPE simulates how failures spread through distributed systems, enabling predictive resilience analysis.*



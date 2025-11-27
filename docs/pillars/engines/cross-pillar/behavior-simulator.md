# Behavior Simulator

**Status**: Cross-Pillar Engine  
**Pillars**: All (System Dynamics Simulation)

[← Back to Engines](../README.md)

## Overview

The Behavior Simulator is a dynamic simulation engine that plugs Systems Thinking DSL into the Global Architecture Platform, providing stock-flow modeling, causal propagation, feedback loop simulation, and scenario analysis.

**This is the most intelligence-defining module of the entire architecture platform.**

## Purpose

The Behavior Simulator answers:

- ✅ Why does latency rise under load?
- ✅ What happens during failure of component X?
- ✅ How do retries amplify system load?
- ✅ What is the impact of deployment delay?
- ✅ What is the steady-state of the system?
- ✅ What happens if traffic grows 30% / min?
- ✅ How do queues fill or drain?
- ✅ Where are the bottlenecks?
- ✅ How do loops reinforce or balance?

**This turns architecture modeling into a predictive analytics engine.**

## Simulator Architecture

```
BehaviorSimulator
 ├── ModelLoader (AST → IR)
 ├── CausalEngine
 │      ├── causal propagation
 │      ├── polarity (+/-)
 │      ├── delays
 │      ├── strength weighting
 ├── LoopEngine
 │      ├── R/B loop resolver
 │      ├── amplification detection
 │      ├── oscillation detection
 ├── StockFlowEngine
 │      ├── stocks
 │      ├── inflows/outflows
 │      ├── queue models
 │      ├── stability analysis
 ├── ConstraintEvaluator
 ├── ScenarioEngine
 │      ├── what-if analysis
 │      ├── traffic surge
 │      ├── failure simulation
 │      ├── slow DB simulation
 │      ├── retry storms
 ├── TimeStepper
 │      ├── discrete time simulation
 │      ├── continuous mode (RK4 optional)
 │      └── delta-t loop
 ├── Results
 │      ├── time series data
 │      ├── metrics
 │      ├── constraint violations
 │      ├── graphs
 │      ├── heatmaps
 └── Integration API (MCP / REST)
```

## Simulation Modes

### Discrete Time Simulation (MVP)

Most intuitive:

```
t = 0 → 1000 steps
Δt = 100ms
```

For each step:

1. Evaluate causes → effects
2. Resolve delays
3. Apply loop multipliers
4. Update stocks
5. Evaluate constraints
6. Emit metrics

MVP-friendly, accurate enough.

### Continuous Time Simulation (Phase 2)

Using numerical solvers (RK4).  
Only needed for complex SD models.

### Event-Driven Simulation (Phase 3)

Simulate:

- retries
- queue spikes
- failures
- back-pressure
- throttling events

## Simulator Inputs

Based on IR created from DSL:

```
SimulatorConfig {
  simulationTime: number
  stepSize: number
  scenario?: Scenario
}
```

### Inputs:

1. Concepts
2. Causal Links
3. Delays
4. Loops
5. Stocks
6. Flows
7. Constraints
8. Architecture mappings
9. External scenario configuration

## Causal Propagation Engine

For each link:

```
A +-> B
```

We compute:

```
B(t + Δt) = B(t) + influence(A) * polarity * weightingFactor
```

### Influence Function

```
influence(A) = normalization(A_value)
```

### Delays

```
if delay(D):
  B receives effect after D milliseconds
```

### Polarity

- `+->` = positive
- `-→` = negative
- `-->` = neutral

### Weighting Factor (optional)

Default = 1.0  
Future DSL: `A +-> B weight 0.3`

## Loop Engine (Reinforcing / Balancing)

Loops are resolved after causals:

### Reinforcing loop (R)
Growth:

- exponential
- runaway
- tipping points

### Balancing loop (B)
Stabilization:

- dampening behavior
- homeostasis

### Algorithm

1. Build directed cycle graph
2. Classify:
   - polarity sum positive → reinforcing
   - polarity sum negative → balancing
3. Apply loop effect:
   - Reinforcing: amplify changes
   - Balancing: dampen changes

## Stock-Flow Engine

Supports:

- ✔ Accumulation
- ✔ Queueing
- ✔ Backpressure
- ✔ Draining

### Stock update formula

```
Stock(t + Δt) = Stock(t) + Inflow(t) - Outflow(t)
```

### Rate forms:

- `rate 10`
- `rate rps("API")`
- `rate per_minute 500`

### Queue model example

If inflow > outflow:

```
PendingRequests increases
```

If outflow > inflow:

```
PendingRequests decreases
```

### Integration with Architecture

`rps("APIService")` is resolved via:

- simulation traffic
- system loops
- dynamic traffic scenarios

## Constraint Evaluation

Each step checks:

```
latency < 200ms depends_on [ APIService, DB ]
```

Evaluation uses:

- component metrics
- causal upstream influences
- loop magnification
- stock levels (Backlog → latency)
- flows (Load → DB backlog)

Violations produce:

- warnings
- heatmap highlights
- severity
- causal chain explanation

## Scenario Engine

This is how real-world load / failures are simulated.

### Traffic Surge

```
scenario traffic_surge {
  start: 10s
  end: 60s
  multiplier: 3.0x
}
```

### Latency Degradation

```
scenario slow_db {
  component: DB
  latency: +80ms
}
```

### Partial Outage

```
scenario failover_test {
  component: PaymentProcessor
  availability: 50%
}
```

### Retry Storm

```
scenario retry_storm {
  multiplier: 2.0x
  applies_to: Retries
}
```

### Traffic Decay / Recovery

```
scenario recovery {
  curve: exponential_decay
}
```

## Time Stepper

```
for t from 0 → simulationTime step Δt:
  applyCausal()
  applyLoops()
  updateStocks()
  checkConstraints()
  recordMetrics()
```

Simple, predictable, fast.

## Output Data Model

```
SimulationResult {
  timeSeries: {
    [variableName]: Array<number>
  }
  constraintViolations: Array<Violation>
  events: Array<SimEvent>
  loopsActivated: Array<LoopActivation>
  metrics: {
    peakLatency: number
    maxQueueDepth: number
    instabilityScore: number
  }
}
```

## Frontend Integration

### Visualizations:

- Causal Loop Diagrams (animated)
- Stocks/flows charts
- Time-series graphs
- Reinforcing/balancing loop heatmaps
- Queue depth visualization
- Constraint violation markers
- Scenario timeline (when events fire)
- Architecture heatmap overlays (architectural nodes glow with load)

## Integration Points

### Systems Thinking Compiler
- Receives compiled simulation model
- Processes causal graphs
- Handles loops and stocks

### Visual Simulation Dashboard
- Provides time-series data
- Enables real-time visualization
- Supports scenario playback

### AI Causal Reasoning Engine (ACRE)
- Provides simulation data for analysis
- Enables causal explanations
- Supports root cause analysis

### Scenario Comparison Engine
- Provides simulation results
- Enables scenario comparison
- Supports impact analysis

## MCP API

```
simulator.run(config)
simulator.step()
simulator.pause()
simulator.results()
simulator.scenario(name)
simulator.constraints()
```

## Strategic Value

The Behavior Simulator provides:

- ✅ Predictive architecture analytics
- ✅ System dynamics simulation
- ✅ Scenario analysis
- ✅ Failure propagation modeling
- ✅ Performance prediction
- ✅ Bottleneck identification

**This is critical for architecture intelligence and predictive analytics.**

## Implementation Status

✅ Architecture designed  
✅ Simulation algorithms specified  
✅ Integration points defined  
📋 Implementation in progress

---

*The Behavior Simulator provides dynamic simulation of architecture behavior and system dynamics.*


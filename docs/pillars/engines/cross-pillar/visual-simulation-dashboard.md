# Visual Simulation Dashboard

**Status**: Cross-Pillar Engine  
**Pillars**: All (Visualization)

[← Back to Engines](../README.md)

## Overview

The Visual Simulation Dashboard is the primary visualization layer for all dynamic analysis engines, providing an interactive UI for running simulations, observing dynamic behavior, and visualizing architecture heatmaps, causal loops, and system dynamics.

**This is the "architecture cockpit" and flagship UX experience of the entire product.**

## Purpose

The Visual Simulation Dashboard:

- ✅ Runs simulations interactively
- ✅ Observes dynamic behavior in real-time
- ✅ Visualizes architecture heatmaps
- ✅ Shows animated causal loops
- ✅ Displays stock & flow dynamics
- ✅ Monitors NFR constraints
- ✅ Compares scenarios
- ✅ Provides AI causal insights
- ✅ Generates reports & recommendations

## Dashboard Architecture

```
SimulationDashboard
 ├── SimulationControlPanel
 ├── RealtimeVisualizations
 │     ├── AnimatedCausalGraph
 │     ├── ArchitectureHeatmap
 │     ├── LoopVisualizer
 │     ├── StockFlowView
 │     ├── QueueDepthView
 │     ├── MetricsCharts
 │     ├── ConstraintViolationsPanel
 │     ├── ScenarioTimeline
 ├── ComparisonPanel
 ├── AICausalInsightsPanel
 └── Export/ADR panel
```

## Top-Level UI Layout

```
+-----------------------------------------------------------+
|  Sidebar: Controls                                        |
|-----------------------------------------------------------|
|  MainPanel: Graphs + Visualizations                       |
|                                                           |
|  - Architecture Heatmap                                   |
|  - Animated Causal Graph                                  |
|  - Loop Activation Graph                                  |
|  - Queue/Stock Flow Panel                                 |
|  - Time-series Chart Panel                                |
|                                                           |
|-----------------------------------------------------------|
|  BottomPanel: Scenario Timeline + AICausalInsights        |
+-----------------------------------------------------------+
```

Built using:

- **React + Next.js**
- **ReactFlow** (architecture + causal diagrams)
- **Recharts / VisX / ECharts** (time-series)
- **react-spring / framer-motion** (animations)
- **shadcn/ui** (controls, tabs, panels)

## Key Features

### Simulation Control Panel

Includes:

- Run / Pause / Step buttons
- Speed slider (0.1× → 20×)
- Scenario selector
- Input parameter overrides
- Metrics selector
- "Record Video" toggle

### Animated Architecture Heatmap

Uses **ReactFlow** with dynamically updated **node and edge colors**.

#### Node Colors:
- Green → stable
- Yellow → nearing threshold
- Orange → degraded
- Red → violating constraints
- Purple → part of reinforcing loop
- Blue → affected by scenario

#### Node Size:
- Increases with load / pressure / queue depth

#### Edge Width:
- Rate of flow
- Request/s propagation
- Retry load

### Animated Causal Graph (Dynamic Behavioral Diagram)

Visualizes:

- Concepts (Traffic, Latency, Retries, DBLoad)
- Polarity of edges (+ or –)
- Flow intensity
- Delays (animated clock icons)

#### Animation Rules:
- Thicker lines = stronger influence
- Pulsating lines = active causal link
- Rotating arrows = loops
- Motion delay markers show latency

### Loop Activation Visualizer

Shows all reinforcing / balancing loops with their current activation intensity.

UI:

```
Reinforcing (R1)  ████████  High
Reinforcing (R2)  ████      Medium
Balancing (B1)    ██        Weak
```

Animation:

- R loops glow red
- B loops glow blue
- On click → highlight the loop in causal graph

### Stock & Flow Visualizer

Shows:

- Queues
- PendingRequests
- Inventory-like stocks
- Production/consumption rates

Animations:

- Fluid filling
- Drain animation
- "Tank" visualization

Pain points / bottlenecks appear as:

- red tanks (overfilled)
- low-flow blue tanks (starvation)

### Metrics Time-Series Panel

Graphs:

- Latency
- DBLoad
- PendingRequests
- Throughput
- Cost
- Error rates
- Retry rates

Use **ECharts** or **VisX**.

Features:

- zoom
- pan
- snapshots
- overlay scenario markers

### Constraint Violations Panel

Shows in a table:

| Time | Constraint | Value | Threshold | Component | Root Cause |
|------|------------|--------|-----------|-----------|------------|

Click → opens ACRE explanation.

### Scenario Timeline

Shows events like:

- traffic surge started
- DB slowdown injected
- retry storm triggered
- loop R1 amplified
- constraint broken

Allows clicking to jump to time-t.

### AI Causal Insights Panel

This panel integrates the **AI Causal Reasoning Engine**.

It shows:

- Why something happened
- How the system behaved
- What loops were activated
- Where the bottleneck lies
- What actions could stabilize the system
- Predicted next 10s / 1min behavior

Example:

> "Latency increased primarily because PendingRequests exceeded  
> Processing capacity for 16 seconds. This triggered reinforcing loop R1  
> (Traffic → Latency → Retries → Load). Mitigation: introduce jittered  
> retries or a queue-based front end."

## Dashboard Modes

### Live Mode
Runs real-time simulation.

### Compare Mode
Side-by-side view:

```
Baseline vs Traffic Surge
Baseline vs DB Slowdown
Baseline vs Retry Off
```

### Scenario Editor Mode
UI to create custom "what-if" scenarios.

### Causal Explorer Mode
Interactive view of causal pathways.

### Loop Explorer Mode
Animated loop exploration.

## Data Flow Architecture

```
[ Simulator ] → 
  TimeSeriesData → 
    DashboardStore → 
      Visualizations
```

### Components subscribe to:

- ticks
- loop activations
- stock levels
- constraints
- events

Uses **zustand** or **jotai** for reactive local state.

## API Integration (MCP / REST)

Exposed endpoints:

### `simulation.run`
Starts new run.

### `simulation.step`
Step-by-step mode.

### `simulation.results`
Fetch time-series.

### `simulation.explain`
AI explanation.

### `simulation.compare`
Compare two runs.

### `simulation.playback`
Return event timeline.

## Frontend Technology Stack

### Core
- Next.js 14
- React 18
- TypeScript
- pnpm / Bun

### Graph
- ReactFlow
- D3 (for causal loop layouts)

### Charts
- VisX or ECharts

### UI
- shadcn/ui
- Framer Motion

### State
- Zustand or Jotai

### Animation
- react-spring
- @react-three/fiber (optional fancy loops)

## Implementation Phases

### Phase 1 — Foundation (MVP)
✔ Simulation Control Panel  
✔ Time-series panel  
✔ Stock-flow visualizer  
✔ Simple architecture heatmap  
✔ Basic causal graph

### Phase 2 — Behavior Animation
✔ Loop activation visualizer  
✔ Edge influence animation  
✔ Queue visualizer  
✔ Constraint violation markers

### Phase 3 — Advanced Analysis
✔ Causal Explorer  
✔ Root Cause Timeline  
✔ Scenario timeline  
✔ Compare mode

### Phase 4 — AI Insights
✔ AI causal narratives  
✔ Mitigation suggestions  
✔ ADR proposer

## Integration Points

### Behavior Simulator
- Receives simulation data
- Displays time-series
- Shows stock & flow

### AI Causal Reasoning Engine (ACRE)
- Provides explanations
- Generates insights
- Suggests mitigations

### Hotspot Detection Engine
- Overlays hotspots on heatmap
- Highlights stress points

### Scenario Comparison Engine
- Enables compare mode
- Side-by-side visualization

### Multi-Simulation Orchestration Engine
- Batch simulation support
- Parallel execution visualization

## MCP API

```
dashboard.run(scenario)
dashboard.step()
dashboard.pause()
dashboard.results()
dashboard.explain(time)
dashboard.compare(scenarioA, scenarioB)
dashboard.playback()
```

## Strategic Value

The Visual Simulation Dashboard provides:

- ✅ Interactive simulation experience
- ✅ Real-time behavior visualization
- ✅ Causal loop visualization
- ✅ Architecture heatmaps
- ✅ AI-powered insights
- ✅ Scenario comparison
- ✅ Professional UX

**This is critical for user experience and platform differentiation.**

## Implementation Status

✅ Architecture designed  
✅ UI components specified  
✅ Integration points defined  
📋 Implementation in progress

---

*The Visual Simulation Dashboard is the primary visualization layer for all dynamic analysis engines.*


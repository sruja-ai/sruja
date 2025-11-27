# Architecture Timeline Engine

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (Operational Excellence, Governance, Evolution)

[← Back to Engines](../README.md)

## Overview

The Architecture Timeline Engine enables **time-travel through architecture evolution** - replaying the entire architecture history from day 1 to today.

## Purpose

This engine provides:
- ✅ Time-based diagram playback
- ✅ Version-to-version comparison
- ✅ Evolution event extraction
- ✅ Architecture drift detection
- ✅ Timeline clustering
- ✅ Annotated ADR overlays
- ✅ Traceability over time
- ✅ Behavior/sequence evolution
- ✅ Code → architecture evolution mapping

## Core Concept

Every architecture commit becomes a **time checkpoint**.

Timeline Engine loads:

```
ArchitectureModel@t0
ArchitectureModel@t1
ArchitectureModel@t2
...
ArchitectureModel@tN
```

Then computes **delta models** and constructs the **Architecture Evolution Graph**.

## Architecture

```
Architecture Timeline Engine
 ├── Version Loader (Git)
 ├── Model Delta Engine (AST → IR diff)
 ├── Evolution Event Classifier
 ├── Visualization Layer (ReactFlow Timeline)
 ├── ADR Timeline Integrator
 ├── Release Correlation Engine
 ├── Code Drift Analyzer
 ├── Playback Engine (animation)
 ├── Diff Viewer (DSL + Graph)
 └── MCP Tools
```

## Types of Architecture Change Events

### 1. Structural Changes
- Component added/removed/renamed/moved
- Container added/deleted
- Boundary changes
- Relation added/removed/changed
- Layering change

### 2. Domain / DDD Changes
- New bounded context
- Context split/merge
- Aggregate rename
- Domain concept added
- Domain event contract changed
- Anti-corruption layer added

### 3. Behavioral Changes
- Sequence changes
- Event flow changes
- Event → command mapping changed
- New async/sync path

### 4. Technology / Implementation Changes
- Tech stack changes
- Replaced service with new version
- Database migration
- Messaging change
- API version change

### 5. Decision Changes (ADRs)
- new ADR created
- ADR accepted/rejected
- ADR superseded
- architecture change that violates ADR
- code violates ADR

### 6. Code Architecture Drift
- code changes structurally incompatible with architecture
- new package imported
- service merges into another
- shared database emerges
- dependency depth increased

## Evolution Event Format

Each event is standardized:

```typescript
interface EvolutionEvent {
  id: string;
  timestamp: string;
  type: 
    | "ComponentAdded"
    | "ComponentRemoved"
    | "ComponentMoved"
    | "BoundaryChanged"
    | "RelationAdded"
    | "RelationRemoved"
    | "DomainSplit"
    | "DomainMerge"
    | "ADRCreated"
    | "ADRSuperseded"
    | "CodeDriftDetected"
    | "PatternShift"
    | "PatternRegression"
    | "TechChange"
    | "StructureRefactor"
    | "NFRChange"
    | "ReleaseAnnotation";
  from?: any;
  to?: any;
  metadata: Record<string, any>;
}
```

## Visual Timeline UI

The UI is designed to feel like **GitKraken + Figma Inspect + GitHub Timeline**, but for architecture.

### Timeline Bar (Horizontal)
- checkpoints for each commit
- colored markers for impactful changes
- zoom (days → weeks → months → years)
- clusters

### Version Playback Viewer
- click play → watch architecture evolve over time
- nodes animate in/out
- edges animate change
- domain boundaries redraw

### Side-by-Side Compare (Diff View)
Left: Architecture@t1  
Right: Architecture@t2

With:
- highlighted changed nodes
- changed edges
- moved boundaries
- new modules/services

### ADR Overlays
Overlay ADR decisions on timeline.

Example:
- ADR-003 "move to event-driven" appears at timestamp
- timeline shows architecture shifts after that

### Code Drift Overlay
Visual indicators:
- 🔴 red edges = drift
- 🟡 yellow nodes = probable drift
- 🟢 green = aligned

### Release Annotations
Pull release tags from Git.

## Evolution Delta Engine (Core Logic)

### Steps:

1. Load Model@t(n) and Model@t(n+1)
2. Compute AST diff
3. Convert to IR diff
4. Compute structural & naming deltas
5. Detect semantic events (AI)
6. Group changes into EvolutionEvent list
7. Store in EvolutionGraph

### AI layer interprets:

- "Component renamed" vs "Component replaced"
- "Domain split" vs "module move"
- "Service merged into another"
- "Refactoring" vs "architectural regression"

## Pattern Evolution Detection

Detect design pattern shifts over time:

- synchronous → event-driven
- monolith → microservices
- distributed monolith emergence
- layered → hexagonal
- REST → GraphQL
- event storming maturity

### AI generates diagnoses:

> "Between v12 and v13, CheckoutService became a hotspot with 14 new inbound dependencies.  
> This resembles a distributed monolith anti-pattern."

## Change Impact + Timeline Correlation

Integrate with Simulation Engine:

For each change:

```
impact = simulate.change(Model@t(n), Model@t(n+1))
```

Timeline UI shows:

- breaking changes
- potential regressions
- scalability impacts
- reliability changes

## MCP Tools

### `timeline.getEvents`
Get evolution events between two versions.

### `timeline.diff`
Return DSL diff + GlobalModel diff.

### `timeline.play`
Return sequence of snapshots for playback animation.

### `timeline.explain`
AI narration explaining evolution.

### `timeline.predict`
Predict future issues based on drift trend.

## History Engine

Supports:

- branching
- merging
- rebasing
- Git history
- rollback

Also:

- tag version of architecture
- annotate with releases
- attach ADRs per version

## Frontend Features

### Filters:
- domain
- module
- component
- ADR
- change type
- pattern type

### Search:
"When did X start depending on Y?"

### Playback Speed:
- slow (presentation)
- medium
- fast (debug)

### Export:
- GIF / MP4 animation
- PDF timeline
- Markdown report
- Architecture evolution narrative

## Implementation Phases

### Phase 1 — Version Loader
- Git integration
- model hashing
- DSL loader per version

### Phase 2 — Model Delta Engine
- AST diff
- renamed vs moved detection
- boundary change detection

### Phase 3 — Event Classifier
- structural handlers
- domain handlers
- naming/renaming inference
- relation change logic

### Phase 4 — Timeline Renderer
- side-by-side diff view
- animation engine
- version selector

### Phase 5 — ADR + Release Correlation
- ADR timeline integration
- version → ADR linking
- semantic impact overlays

### Phase 6 — AI Evolution Narrator
- summarizing changes
- explaining patterns
- predicting risks

## Implementation Status

✅ Architecture designed  
✅ Event types defined  
✅ Delta engine specified  
📋 Git integration in progress  
📋 UI implementation planned

---

*The Architecture Timeline Engine is the "Git History of Architecture" - a flagship premium feature.*


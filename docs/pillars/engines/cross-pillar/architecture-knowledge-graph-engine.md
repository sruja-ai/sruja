# Architecture Evolution Knowledge Graph (AEKG)

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (All - Central Data Model)

[← Back to Engines](../README.md)

## Overview

The Architecture Evolution Knowledge Graph (AEKG) is a **time-aware, multi-layered, multi-system knowledge graph** that stores the entire architectural history, decisions, dependencies, runtime behavior, drift, fixes, and evolution across the organization.

**It becomes the brain of the entire platform.**

## Purpose

AEKG is designed to:

- ✅ Store every architecture model
- ✅ Track how architecture evolves over time
- ✅ Organize relationships between systems, domains, teams, and data
- ✅ Store ADRs, decisions, policies, refactor plans
- ✅ Track dependencies, violations, drifts, and fixes
- ✅ Connect simulation results with real runtime data
- ✅ Support semantic search
- ✅ Power AI agents with contextual memory
- ✅ Enable historical replay & temporal queries

**This transforms your product into a living, evolving architectural memory of the organization.**

## What the Knowledge Graph Stores

### 1. Architecture Structure
Nodes & edges representing:

- systems
- components
- functions
- APIs
- data stores
- infra resources

### 2. Dependencies
For each version:

- sync calls
- async events
- queries
- data flows
- infra-level dependencies
- cross-domain interactions

### 3. Runtime Behavior (from ATOE)
Spans, latency, error rates, load patterns.

### 4. Architecture Drift
Detected deviations + drift types.

### 5. Remediation Actions
Fix patches, code patches, governance fixes.

### 6. ADRs & Decisions
Linked to:

- systems
- domains
- timelines
- violations
- refactor plans

### 7. Evolution Events
Time-stamped events such as:

- system added
- system removed
- new domain
- refactor
- migration
- policy updates
- violations introduced
- violations resolved
- runtime incidents
- auto-remediation events
- architecture score changes

### 8. Governance Rules & Policy Lineage
For every rule:

- where it applies
- which systems violate
- how violations changed over time

### 9. Team Ownership
Bounded contexts, squads, tribes, roles.

### 10. Business Context
Link systems to:

- capabilities
- user journeys
- value streams
- KPIs
- business processes

### Time Dimension
AEKG is **temporal graph** with:

- time-based nodes
- historical edges
- versioned relationships
- temporal traceability

## Architecture

```
ArchitectureEvolutionKnowledgeGraph
 ├── GraphStore (Neo4j/SurrealDB/Dgraph/TerminusDB)
 ├── TemporalLayer
 ├── SchemaManager
 ├── EventIngestor
 │    ├── from ATOE (runtime)
 │    ├── from SSAGE (violations)
 │    ├── from ACH (communication events)
 │    ├── from ADARE (remediations)
 │    ├── from AEP/MAES (simulations)
 │    ├── from Git (history)
 │    └── from IR (model changes)
 ├── LinkerEngine
 ├── EvolutionIndex
 ├── SemanticSearchIndex
 ├── ReasoningEngine
 ├── AI Context Generator
 ├── MCP API
 └── VisualizationEngine
```

## Graph Schema (High-Level)

### Node Types:

- `System`
- `Component`
- `Domain`
- `BoundedContext`
- `Team`
- `ADR`
- `Policy`
- `Event`
- `DataFlow`
- `Infrastructure`
- `UserJourney`
- `Requirement`
- `Simulation`
- `DriftEvent`
- `Remediation`

### Edge Types:

- `depends_on`
- `belongs_to`
- `owns`
- `violates`
- `fixes`
- `evolves_from`
- `simulates`
- `implements`
- `references`
- `triggers`

## Property Graph Example

```
Service → depends_on → Queue
Service → deployed_on → Cluster
ADR → affects → Component
Policy → violated_by → Relation
Team → owns → Domain
Requirement → implemented_by → Component
DriftEvent → detected_in → System
Remediation → fixes → DriftEvent
```

## Temporal Queries

AEKG supports time-aware queries:

```
FIND components WHERE created_at > "2024-01-01"
FIND dependencies WHERE changed_at BETWEEN "2024-01-01" AND "2024-06-01"
FIND drift_events WHERE system = "PaymentService" ORDER BY timestamp DESC
```

## Integration Points

### Event Ingestors

**From ATOE (Runtime):**
- Runtime dependency graph
- Latency/error metrics
- Traffic patterns

**From SSAGE (Governance):**
- Policy violations
- Rule evaluations
- Compliance status

**From ADARE (Remediation):**
- Drift events
- Fix patches
- Root cause analysis

**From MAES (Simulation):**
- Simulation results
- Scenario outcomes
- Performance predictions

**From Git (History):**
- Architecture versions
- Commit metadata
- Evolution timeline

**From IR (Model Changes):**
- Structural changes
- Domain changes
- Component changes

## Semantic Search

AEKG powers semantic search across:

- Architecture descriptions
- ADR rationales
- Domain concepts
- Team discussions
- Requirements
- User journeys

## AI Context Generation

AEKG provides rich context for AI agents:

- Full architecture history
- Related decisions
- Similar past scenarios
- Team patterns
- Evolution trends

## MCP API

```
aekg.query(cypher)
aekg.find(pattern)
aekg.timeline(system, start, end)
aekg.evolution(system)
aekg.related(component)
aekg.search(query)
aekg.context(system)
```

## Visualization

- Interactive graph explorer
- Temporal visualization
- Evolution timeline
- Relationship maps
- Domain boundaries
- Team ownership views

## Implementation Status

✅ Architecture designed  
✅ Graph schema defined  
✅ Event ingestor architecture specified  
📋 Graph store selection in progress  
📋 Implementation planned

---

*AEKG is the central nervous system of the architecture platform - connecting all engines and providing unified intelligence.*



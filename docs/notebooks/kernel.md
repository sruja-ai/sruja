# Architecture Kernel

[← Back to Notebooks Index](./README.md)

## Overview

The **Architecture Kernel** is the execution engine behind Sruja Notebooks. It's a long-running process that:

- Loads and executes DSL cells
- Maintains complete architecture semantic model
- Validates every element (systems, domains, events, APIs, rules...)
- Runs queries (SrujaQL)
- Generates diagrams (C4, flows, state machines)
- Computes diffs & compatibility
- Simulates event flows & lifecycle transitions
- Manages variants, snapshots, versions
- Integrates with AI & MCP
- Monitors code alignment (API, DB, events)
- Provides LSP completions, hovers, symbols, errors

It's a **stateful architecture runtime**.

## Architecture

```
┌──────────────────────────────┐
│         Sruja Kernel          │
├──────────────────────────────┤
│  1. Frontend / Notebook API   │
│  2. DSL Parser                │
│  3. Semantic Model Builder    │
│  4. Symbol Table / Registry   │
│  5. Architecture Store        │
│  6. Validators                │
│  7. Query Engine (SrujaQL)    │
│  8. Version & Diff Engine     │
│  9. Diagram Generator         │
│ 10. Event Simulation Engine   │
│ 11. Variant & Snapshot Engine │
│ 12. MCP Integration Layer     │
│ 13. AI Reasoning Layer        │
│ 14. LSP Support Engine        │
└──────────────────────────────┘
```

## Core Components

### 1. DSL Parser

Parses architecture DSL into AST:

- Architecture DSL
- Policy DSL
- Rule DSL
- Event DSL
- Contracts DSL
- Entities DSL
- Requirements DSL

**Requirements:**
- Tolerant parsing (errors but keeps partial AST)
- Source location mapping
- Incremental parsing (for notebook cells)
- Schema-driven (extensions are easy)

### 2. Semantic Model Builder

Transforms AST → Semantic Model:

**Tasks:**
- Resolve imports
- Validate references (cross-file)
- Merge model fragments from cells
- Inline entities into relationships
- Resolve lifecycle transitions
- Build event graph
- Build dependency graph
- Infer defaults & implicit constructs

**Outputs:**
- DomainModel
- SystemModel
- ComponentModel
- EventModel
- ContractModel
- DataModel
- PolicyModel
- RequirementModel

### 3. Symbol Table / Registry

Central directory of all defined symbols:

- systems
- containers
- components
- entities
- events
- APIs
- rules
- policies
- requirements
- contracts
- snapshots
- variants

**Supports:**
- Go-to-definition
- Rename-symbol
- Hovers & autocompletion
- Cross-references
- Semantic linking

### 4. Architecture Store

Long-lived state of the kernel:

- Active architecture model
- Snapshots (full model copies)
- Variants
- Version histories
- Notebook cell state
- Errors & diagnostics
- Diagram cache

**Operations:**
- `save_snapshot(name)`
- `load_snapshot(name)`
- `fork_variant(name)`
- `merge_variant(base, variant)`

### 5. Validation Engine

Validates architecture elements:

- Entity validator
- Event lifecycle validator
- API contract validator
- Data contract validator
- Component validator
- System boundary validator
- Policy evaluator
- Rule engine
- Constraint solver
- Compatibility validator (versioning)

**Output:**
```go
type Diagnostic struct {
    Severity string // error | warning | hint
    Message  string
    Source   string // element ID
    Location SourceLocation
}
```

### 6. Query Engine (SrujaQL)

SQL-like queries over architecture:

**Examples:**
```
select systems where tag == "public"
select events where pii == true
select components where depends_on contains "BillingDB"
graph dependencies of Billing
diff contracts BillingAPI 1.0 vs 2.0
```

**Supports:**
- Filters
- Projections
- Joins (entity ↔ event ↔ contract)
- Graph queries
- Diff queries

### 7. Version & Diff Engine

Handles versioning:

- Contract version diffs
- Schema evolution
- Entity diffing
- Event diffing
- Lifecycle transition diffs
- Variant diffs
- Snapshot diffs

### 8. Diagram Generator

Built-in diagram engine:

- C4 Diagrams (Context, Container, Component)
- Sequence Diagrams (user flows, event flows)
- Event Graphs (causal chains)
- Domain Models (entities, aggregates)
- Architecture Heatmaps (PII flow, security boundaries)

**Output formats:**
- SVG
- PNG
- Mermaid syntax
- D2 (optional)

### 9. Event Simulation Engine

Simulates domain behavior:

```python
simulate PaymentLifecycle from PENDING events:
  PaymentAuthorized
  PaymentCompleted
```

**Outputs:**
- New entity state
- Invalid transition warnings
- Dependency events
- Compensations (if rules defined)

### 10. Variant & Snapshot Engine

Critical for iterative architecture design:

```python
snapshot "iteration-5"
variant "async-payments" from "iteration-5"
merge variant "async-payments" into "main"
```

**Features:**
- Conflict detection
- Version comparison
- Human-readable merge explanation
- AI-assisted suggestions

## Kernel Data Structures

The kernel maintains a canonical IR (Intermediate Representation):

### ArchitectureStore

```go
type ArchitectureStore struct {
    Domains        map[string]*DomainModel
    Systems        map[string]*SystemModel
    Containers     map[string]*ContainerModel
    Components     map[string]*ComponentModel
    Entities       map[string]*EntityModel
    Events         map[string]*EventModel
    APIs           map[string]*APIContract
    DataContracts  map[string]*DataContract
    EventContracts map[string]*EventContract
    Policies       map[string]*PolicyModel
    Rules          map[string]*RuleModel
    Requirements   map[string]*RequirementModel
    Relations      map[string]*RelationModel
    Imports        []ImportModel
    Metadata       map[string]string
}
```

See [Kernel Data Structures](./kernel-data-structures.md) for complete schema.

## Execution Model

When a cell executes:

```
1. Parse DSL → AST
2. Update semantic model
3. Update symbol table
4. Merge into Architecture Store
5. Run validators
6. Produce diagnostics
7. Generate diagrams (if requested)
8. Update notebook outputs
9. Notify AI agents (optional)
```

## State Reconciliation

Kernel maintains incremental state:

- Delete + replace on per-cell basis
- Keep dependency DAG of definitions
- Recompute only impacted graphs
- Use memoization for diagrams
- Support partial rollbacks

This makes notebooks performant even with thousands of elements.

## Implementation

### Go Implementation

Core kernel in Go:

```go
type Kernel struct {
    parser      *parser.Parser
    modelBuilder *semantic.Builder
    store       *store.ArchitectureStore
    validators  *validator.Engine
    queryEngine *query.Engine
    diagramGen  *diagram.Generator
    // ... other components
}
```

### WASM Compilation

Kernel compiles to WASM for browser execution:

```
GOOS=js GOARCH=wasm go build -o kernel.wasm
```

See [WASM Execution](./wasm-execution.md) for details.

## Performance

**Key strategies:**

- Maintain IR in memory (not JSON)
- Avoid repeated serialization
- Diff-based validation
- Only run validators on impacted nodes
- Cache symbol table
- Cache diagram intermediate structures
- Use WebWorker to avoid blocking UI thread
- Send only deltas (not full model)

## Extension Points

Kernel supports extensions:

- New DSL keywords
- New validators
- New diagram generators
- New query functions
- New policies/rules
- New contract types

**Registration:**
```go
kernel.RegisterExtension(ext Extension)
```

## Next Steps

- [Kernel Data Structures](./kernel-data-structures.md) - Complete IR schema
- [WASM Execution](./wasm-execution.md) - Browser-based execution
- [Kernel Messaging Protocol](./kernel-messaging.md) - Jupyter protocol integration


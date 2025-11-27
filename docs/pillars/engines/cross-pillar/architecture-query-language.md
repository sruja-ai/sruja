# Architecture Query Language (AQL)

**Status**: Advanced Engine  
**Pillars**: Cross-Pillar (All - Query Interface)

[← Back to Engines](../README.md)

## Overview

AQL (Architecture Query Language) is a **declarative query language** for querying architecture models, enabling AI agents, validation rules, and users to extract architectural insights.

## Purpose

AQL makes your platform:

- ✅ **AI-ready** - LLMs can query architecture
- ✅ **MCP-ready** - Standardized query interface
- ✅ **IDE-ready** - Rich architecture insights
- ✅ **Validation-ready** - Rules as queries
- ✅ **Analysis-ready** - Complex architectural queries

**AQL is what makes your platform massively more powerful than Structurizr, D2, or any modeling DSL alone.**

## What is AQL?

AQL = **Architecture Query Language**, a declarative query language for:

- Finding components, systems, domains
- Traversing dependencies
- Detecting patterns
- Validating constraints
- Analyzing relationships
- Querying across time (with AEKG)
- Semantic search
- Risk analysis
- Impact analysis

## Design Goals

1. **Simple & Readable** - Non-technical users can write queries
2. **Expressive** - Complex architectural questions
3. **Fast** - Efficient graph traversal
4. **Composable** - Queries can be combined
5. **AI-Friendly** - Natural language → AQL translation
6. **Extensible** - Plugin-based query functions

## AQL Examples

### Find all services

```aql
FIND components WHERE kind = "service"
```

### Find components with high fan-out

```aql
FIND components 
WHERE outbound_edges > 10
```

### Find cross-domain dependencies

```aql
FIND relations 
WHERE source.domain != target.domain
```

### Find components violating layer rules

```aql
FIND relations 
WHERE source.layer = "UI" AND target.layer = "Database"
```

### Find services without circuit breakers

```aql
FIND components 
WHERE kind = "service" AND circuit_breaker = null
```

### Find all paths between two components

```aql
FIND PATHS FROM PaymentService TO Database
```

### Find components affected by a failure

```aql
FIND components 
REACHABLE FROM AuthService 
WHERE dependency_type = "sync"
```

### Find domain violations

```aql
FIND relations 
WHERE source.domain = "Payments" 
  AND target.domain = "Inventory"
  AND type != "DomainEvent"
```

## AQL Grammar (High-Level)

```aql
Query = 
  FIND <Target> [WHERE <Condition>] [LIMIT <Number>]
  | FIND PATHS FROM <Component> TO <Component>
  | FIND REACHABLE FROM <Component> [WHERE <Condition>]

Target = 
  components | systems | domains | relations | adrs

Condition = 
  <Property> <Operator> <Value>
  | <Condition> AND <Condition>
  | <Condition> OR <Condition>

Operator = 
  = | != | > | < | >= | <= | IN | CONTAINS
```

## Execution Model

AQL operates on:

- **GlobalModel** - Current architecture state
- **AEKG** - Historical and temporal data
- **Runtime Data** - From ATOE (observability)
- **Simulation Results** - From MAES

### Query Execution Flow

```
AQL Query
  ↓
Parse & Validate
  ↓
Optimize (graph traversal planning)
  ↓
Execute (graph traversal)
  ↓
Filter & Transform
  ↓
Return Results
```

## Integration with Validation

Many validation features become **AQL rules**:

```aql
# Rule: No UI → DB direct calls
FIND relations 
WHERE source.kind = "ui" AND target.kind = "database"
# If result > 0 → violation
```

## Using AQL Inside the MCP Server

MCP exposes AQL:

```typescript
queryAQL(queryString: string) -> AQLResult
```

AI agents can use this to:

- Understand architecture
- Validate designs
- Find dependencies
- Analyze impact
- Generate recommendations

## UI Integration

The UI generates AQL automatically:

- Click "Find all services" → Generates AQL
- Filter by domain → Generates AQL
- Search → Generates AQL

Users can also write AQL directly in a query editor.

## Internal AQL API (Plugins + Runtime)

Plugins can run AQL:

```typescript
const results = aqlEngine.execute(`
  FIND components WHERE domain = "Payments"
`);
```

Validation rules can be AQL-powered:

```typescript
const rule = {
  id: "no-ui-db",
  aql: `FIND relations WHERE source.kind="ui" AND target.kind="database"`,
  severity: "error"
};
```

## Advanced Queries

### Temporal Queries (with AEKG)

```aql
FIND components 
WHERE created_at > "2024-01-01"
  AND domain = "Payments"
```

### Semantic Queries

```aql
FIND components 
WHERE description CONTAINS "payment processing"
```

### Risk Analysis Queries

```aql
FIND components 
WHERE fan_in > 20 OR fan_out > 15
  OR circuit_breaker = null
```

### Impact Analysis Queries

```aql
FIND REACHABLE FROM PaymentService
WHERE dependency_type = "sync"
LIMIT 50
```

## Query Optimization

AQL engine optimizes:

- Graph traversal order
- Index usage
- Parallel execution
- Result caching

## Implementation Status

✅ Grammar defined  
✅ Query examples specified  
✅ Execution model designed  
📋 Parser implementation in progress  
📋 Graph traversal engine planned

---

*AQL is the query interface that makes architecture models accessible to AI, validation, and analysis.*


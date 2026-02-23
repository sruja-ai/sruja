# Architecture Intelligence Analysis: Sruja DSL vs Knowledge Graph

## Executive Summary

**Sruja DSL alone is NOT sufficient** for comprehensive architecture intelligence. The current hybrid approach (DSL + Knowledge Graph) is correct. This document explains why and provides recommendations.

---

## Current Architecture

### Two-Model Approach

| Layer | Technology | Purpose |
|-------|------------|---------|
| **DSL + AST** | Sruja Language | File-based, declarative architecture definitions |
| **Knowledge Graph** | sruja-graph | Multi-source, queryable architecture intelligence |

### Data Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  .sruja Files   │────►│  Language AST   │────►│ Knowledge Graph │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │                                               ▲
        │                       ┌─────────────────┐     │
        │                       │  Code Scanner   │─────┤
        │                       └─────────────────┘     │
        │                       ┌─────────────────┐     │
        └───────────────────────│  Chat Extract   │─────┘
                                └─────────────────┘
```

---

## Why DSL Alone Is Insufficient

### 1. **Single Source vs Multi-Source Knowledge**

| DSL (Insufficient) | Knowledge Graph (Needed) |
|-------------------|-------------------------|
| One .sruja file | .sruja + code + chats + ADRs + commits |
| Static declarations | Evolving knowledge |
| Author's intent only | Discovered patterns + intent |
| Manual updates | Auto-extraction |

**Example**: A microservice's actual dependencies discovered by `sruja-scan` may differ from the declared ones in .sruja files.

### 2. **File-Based vs Query-Based Access**

| DSL | Knowledge Graph |
|-----|-----------------|
| "What's in this file?" | "What depends on X?" |
| Linear reading | Graph traversal |
| No inference | Pattern detection |
| Text search | Semantic queries |

**Example Query** (impossible with DSL alone):
```
"Find all services that depend on deprecated PostgreSQL 11"
```

### 3. **Validation Rules vs Intelligence Rules**

| DSL Validation | Knowledge Graph Intelligence |
|---------------|------------------------------|
| Duplicate IDs | "This pattern often leads to outages" |
| Invalid refs | "Similar architectures failed at scale" |
| Syntax errors | "Your SLO is unachievable with current stack" |

### 4. **Temporal Dimension**

| DSL | Knowledge Graph |
|-----|-----------------|
| Point-in-time snapshot | Version history |
| No evolution tracking | Decision evolution |
| Manual change logs | Auto-tracked provenance |

**Example**: "Why did we change from REST to GraphQL?" - requires decision history, not just current state.

### 5. **Relationship Richness**

| DSL Relationships | Graph Relationships |
|------------------|---------------------|
| `A -> B` (simple) | Multiple edge types |
| Single label | Confidence scores |
| No metadata | Source, timestamp, evidence |

```sruja
// DSL: Simple relationship
API -> Database "writes to"

// Graph: Rich relationship
API --[writes_to: {confidence: 0.95, source: code_scan, 
       pattern: "ORM save()", frequency: "high"}]--> Database
```

---

## What Sruja DSL IS Good For

| Use Case | DSL Suitability |
|----------|----------------|
| **Intent Declaration** | ✅ Excellent - "As-designed" architecture |
| **Documentation** | ✅ Excellent - Human-readable |
| **Validation** | ✅ Good - Syntax, basic rules |
| **IDE Support** | ✅ Excellent - LSP, autocomplete |
| **Export to Diagrams** | ✅ Excellent - Mermaid, PlantUML |
| **Version Control** | ✅ Excellent - Text files, diffs |
| **Cross-Project Queries** | ❌ Poor |
| **Pattern Detection** | ❌ Poor |
| **Multi-Source Fusion** | ❌ Poor |
| **Temporal Analysis** | ❌ Poor |

---

## Recommendations

### 1. **Keep Both Models** ✅ (Already done)

The current architecture is correct:
- **DSL** for human authoring and intent
- **Graph** for intelligence and queries

### 2. **Strengthen the Bridge**

```rust
// Current: Language AST → Graph (one-way)
// Recommended: Bidirectional sync

impl KnowledgeGraph {
    /// Generate DSL from graph (round-trip)
    pub fn to_dsl(&self) -> String { ... }
    
    /// Detect drift between DSL and discovered reality
    pub fn detect_drift(&self, ast: &Program) -> Vec<DriftReport> { ... }
}
```

### 3. **Add Semantic Queries**

```rust
// Beyond simple graph traversal
impl KnowledgeGraph {
    /// "What would break if I remove X?"
    pub fn blast_radius(&self, node_id: &str) -> Vec<Impact>;
    
    /// "Is this pattern similar to known anti-patterns?"
    pub fn detect_antipatterns(&self) -> Vec<AntiPattern>;
    
    /// "What decisions led to this architecture?"
    pub fn decision_chain(&self, node_id: &str) -> Vec<Decision>;
}
```

### 4. **Add Provenance Tracking**

Every graph element should track its source:

```rust
pub enum SourceReference {
    DslFile { path: String, line: u32 },
    CodeScan { file: String, commit: String },
    ChatExtraction { session_id: String, message_id: String },
    AdrFile { path: String },
    Inferred { rule: String, confidence: f32 },
}
```

### 5. **Consider Temporal Graph**

```rust
pub struct TemporalGraph {
    snapshots: Vec<(DateTime<Utc>, KnowledgeGraph)>,
    
    /// "How did this evolve?"
    pub fn evolution(&self, node_id: &str) -> Vec<GraphChange>;
    
    /// "What was the architecture on date X?"
    pub fn as_of(&self, date: DateTime<Utc>) -> KnowledgeGraph;
}
```

---

## Comparison with Industry Approaches

| Tool | DSL | Graph | AI/LLM |
|------|-----|-------|--------|
| **Structurizr** | ✅ | ❌ | ❌ |
| **C4Model + PlantUML** | ✅ | ❌ | ❌ |
| **Backstage Catalog** | ✅ | ✅ | ❌ |
| **GitHub Copilot** | ❌ | ❌ | ✅ |
| **Sruja (current)** | ✅ | ✅ | ✅ |

Sruja's hybrid approach is **ahead of the industry**.

---

## Architecture Intelligence Maturity Model

```
Level 5: Predictive    [ ] "This will fail at 10x scale"
Level 4: Prescriptive  [ ] "Add caching here"
Level 3: Diagnostic    [✓] "You have a layer violation"
Level 2: Descriptive   [✓] "Here's your architecture"
Level 1: Documented    [✓] "Architecture exists in files"
```

**Current Sruja**: Level 2-3
**With enhanced Graph**: Level 3-4
**Future with LLM reasoning**: Level 4-5

---

## Conclusion

| Question | Answer |
|----------|--------|
| Is DSL sufficient? | **No** - Only covers 40% of intelligence needs |
| Is Knowledge Graph needed? | **Yes** - Essential for 60% of intelligence needs |
| Is current hybrid approach correct? | **Yes** - Optimal architecture |
| What's missing? | Temporal analysis, pattern detection, prediction |

### Priority Actions

1. **Keep DSL + Graph hybrid** ✅
2. **Add drift detection** between declared (DSL) and discovered (scan)
3. **Enhance MCP tools** for AI to query graph
4. **Add temporal/versioning** to graph
5. **Build pattern library** in skills that query graph

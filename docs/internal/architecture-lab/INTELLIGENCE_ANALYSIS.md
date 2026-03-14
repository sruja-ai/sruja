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
        │                       │  AI Skills      │─────┤
        │                       │ (discovery,     │     │
        │                       │  patterns,      │     │
        │                       │  collaboration) │     │
        │                       └─────────────────┘     │
        │                       ┌─────────────────┐     │
        └───────────────────────│  ADRs / Docs    │─────┘
                                └─────────────────┘
```

**Data Sources:**
- `.sruja` files → Declared architecture (intent)
- Code scanner → Discovered architecture (reality)
- AI skills → Pattern knowledge, review insights, collaboration context
- ADRs/docs → Decision history, requirements

---

## Why DSL Alone Is Insufficient

### 1. **Single Source vs Multi-Source Knowledge**

| DSL (Insufficient) | Knowledge Graph (Needed) |
|-------------------|-------------------------|
| One .sruja file | .sruja + code + ADRs + skills + docs |
| Static declarations | Evolving knowledge |
| Author's intent only | Discovered patterns + intent + AI insights |
| Manual updates | Auto-extraction + skill-enhanced discovery |

**Example**: A microservice's actual dependencies discovered by `sruja-scan` may differ from the declared ones in .sruja files. AI skills can explain WHY and suggest fixes.

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

### 2. **Strengthen the Bridge** 🚧 (Partial)

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

**Status**: One-way sync exists via `sruja-intent`. Round-trip and drift detection need work.

### 3. **Add Semantic Queries** 🚧 (Partial)

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

**Status**: Basic graph queries exist. `blast_radius` and `decision_chain` are planned.

### 4. **Add Provenance Tracking** 📋 (Planned)

Every graph element should track its source:

```rust
pub enum SourceReference {
    DslFile { path: String, line: u32 },
    CodeScan { file: String, commit: String },
    AdrFile { path: String },
    Inferred { rule: String, confidence: f32 },
}
```

**Status**: Basic source tracking exists in `sruja-scan`. Enhanced provenance planned.

### 5. **Consider Temporal Graph** 📋 (Planned)

```rust
pub struct TemporalGraph {
    snapshots: Vec<(DateTime<Utc>, KnowledgeGraph)>,
    
    /// "How did this evolve?"
    pub fn evolution(&self, node_id: &str) -> Vec<GraphChange>;
    
    /// "What was the architecture on date X?"
    pub fn as_of(&self, date: DateTime<Utc>) -> KnowledgeGraph;
}
```

**Status**: Not yet implemented. Priority: Medium.

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

### Maturity Level Details

| Level | Name | Capabilities | Sruja Commands |
|-------|------|--------------|----------------|
| 1 | Documented | Architecture exists in `.sruja` files | `sruja lint`, `sruja export` |
| 2 | Descriptive | Visualize and explore architecture | `sruja quickstart`, `sruja scan` |
| 3 | Diagnostic | Detect issues (cycles, violations, drift) | `sruja drift`, `sruja why` |
| 4 | Prescriptive | Suggest fixes with evidence | `sruja analyze --suggest` (planned) |
| 5 | Predictive | Forecast issues at scale | Future: ML-based prediction |

---

## CLI Quick Reference for Architecture Intelligence

### Core Commands

| Command | Purpose | Output |
|---------|---------|--------|
| `sruja quickstart -r .` | Zero-setup first run | Inventory + top 3 issues + health score |
| `sruja scan -r .` | Scan codebase structure | Components, dependencies, metrics |
| `sruja drift -r .` | Detect drift without baseline | Cycles, orphans, layer violations |
| `sruja drift -r . -a architecture.sruja` | Detect drift vs declared architecture | Intent vs reality comparison |
| `sruja why "question" -r .` | Explain with evidence | Deterministic evidence-based answer |
| `sruja analyze -r .` | Full multi-layer analysis | Structural + semantic + intent |

### Typical Workflows

```bash
# 1. First-time user (no setup required)
sruja quickstart -r .

# 2. Detect structural issues
sruja drift -r .

# 3. Understand why something exists
sruja why "Why does API depend on Cache?" -r .

# 4. Compare code vs declared architecture
sruja drift -r . -a architecture.sruja

# 5. Export context for AI editor
sruja context export -r . -o context.json
```

---

## Implementation Status

### ✅ Implemented

| Feature | Crate | Notes |
|---------|-------|-------|
| DSL + AST | `sruja-language` | Full parser, validator |
| Knowledge Graph | `sruja-graph` | Node/edge storage, basic queries |
| Code Scanning | `sruja-scan` | Multi-language dependency extraction |
| Drift Detection | `sruja-diff`, `sruja-cli` | Cycles, orphans, violations |
| Why Command | `sruja-cli` | Evidence-based explanations |
| Quickstart | `sruja-cli` | Zero-setup first experience |
| Intent Check | `sruja-intent`, `sruja-cli` | Compare declared vs discovered |

### 🚧 In Progress

| Feature | Status | Notes |
|---------|--------|-------|
| Semantic Queries | Partial | Basic graph traversal exists |
| Pattern Detection | Partial | Cycles, orphans detected |
| Multi-source Fusion | Partial | Code + DSL; chat extraction removed |

### 📋 Planned

| Feature | Priority | Notes |
|---------|----------|-------|
| Drift Detection (DSL vs Graph) | High | `detect_drift(ast, graph)` |
| Blast Radius Analysis | Medium | "What breaks if X changes?" |
| Temporal Graph | Medium | Version history, evolution |
| Provenance Tracking | Medium | Source references on edges |
| Pattern Library | Low | Anti-pattern detection rules |

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
3. **Leverage AI skills** for discovery, review, and knowledge management
4. **Enhance CLI + skills integration** for CI/CD workflows
5. **Build pattern library** via sruja-architecture-collaboration skill
6. **Add temporal/versioning** to graph

### Skill-First Approach

For maximum architecture intelligence, use the skill stack:

```bash
# Install all skills
npx skills add sruja-ai/sruja --skill sruja-architecture
npx skills add sruja-ai/sruja --skill sruja-architecture-agent  
npx skills add sruja-ai/sruja --skill sruja-architecture-collaboration
```

Then leverage:
- **CLI** for deterministic checks (`lint`, `drift`, `why`)
- **Skills** for intelligent guidance, discovery, and review
- **Graph** for queries and pattern detection

---

## Practical Query Examples

### What You Can Ask Today

| Question | Command | Result |
|----------|---------|--------|
| "What's my architecture?" | `sruja quickstart -r .` | Component inventory, health score |
| "What structural issues exist?" | `sruja drift -r .` | Cycles, orphans, violations |
| "Why does X depend on Y?" | `sruja why "Why X -> Y?" -r .` | Evidence-based explanation |
| "Is my code matching intent?" | `sruja drift -r . -a arch.sruja` | Intent vs reality delta |
| "What's the full analysis?" | `sruja analyze -r .` | Multi-layer comprehensive report |

### Query Patterns by Use Case

#### 1. Dependency Analysis
```bash
# Find circular dependencies
sruja drift -r . --check cycles

# Find orphan modules
sruja drift -r . --check orphans

# Find layer violations
sruja drift -r . --check layers
```

#### 2. Intent vs Reality
```bash
# Compare declared architecture with code
sruja drift -r . -a architecture.sruja

# Check specific component
sruja intent check -r . -a architecture.sruja --component API
```

#### 3. Evidence-Based Explanations
```bash
# Why does this dependency exist?
sruja why "Why does PaymentService depend on LegacyDB?" -r .

# What's the reasoning for this architecture?
sruja why "Why microservices?" -r .
```

### Future Query Capabilities (Planned)

| Question | Implementation | Priority |
|----------|----------------|----------|
| "What breaks if I remove X?" | `blast_radius()` | Medium |
| "Is this an anti-pattern?" | `detect_antipatterns()` | Medium |
| "How did this evolve?" | `evolution()` | Low |
| "What was decided when?" | `decision_chain()` | Low |

---

## Four-Layer Intelligence Model

Sruja's architecture intelligence operates across four layers:

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 4: Intent                                            │
│  "What did we intend vs what exists?"                       │
│  Commands: sruja drift -a, sruja intent check               │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Semantic                                          │
│  "What does this mean? (vocabulary, patterns)"              │
│  Commands: sruja analyze --semantic                         │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Structural                                        │
│  "What exists? (components, deps, metrics)"                 │
│  Commands: sruja scan, sruja quickstart                     │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Syntactic                                         │
│  "Is the DSL valid?"                                        │
│  Commands: sruja lint                                       │
└─────────────────────────────────────────────────────────────┘
```

Each layer builds on the previous:
- **Syntactic**: Is the `.sruja` file valid?
- **Structural**: What components and dependencies exist?
- **Semantic**: What patterns and relationships mean?
- **Intent**: Does reality match declared architecture?

---

## Leveraging AI Skills for Architecture Intelligence

Skills provide the AI intelligence layer that multiplies CLI and Knowledge Graph capabilities.

### Skill → Intelligence Layer Mapping

| Intelligence Layer | CLI Commands | AI Skill Enhancement |
|-------------------|--------------|---------------------|
| **Syntactic** | `sruja lint` | `sruja-architecture` rules for correct DSL |
| **Structural** | `sruja scan`, `sruja quickstart` | `sruja-architecture-agent` discovery from code |
| **Semantic** | `sruja analyze` | `sruja-architecture` patterns + `sruja-architecture-collaboration` knowledge |
| **Intent** | `sruja drift -a`, `sruja why` | `sruja-architecture-agent` + `sruja-architecture-collaboration` review |

### Skill-Driven Intelligence Maturity

| Maturity Level | CLI Only | CLI + Skills |
|----------------|----------|--------------|
| 1. Documented | `sruja lint` | + Pattern-aware DSL generation |
| 2. Descriptive | `sruja scan` | + Evidence-based discovery |
| 3. Diagnostic | `sruja drift` | + Multi-agent review |
| 4. Prescriptive | (planned) | + Pattern recommendations |
| 5. Predictive | (future) | + ML-enhanced predictions |

### Skills Multiply Intelligence

| Capability | CLI Alone | CLI + Skills |
|------------|-----------|--------------|
| DSL Generation | Manual | Auto from code |
| Pattern Knowledge | Docs lookup | In-editor guidance |
| Review Depth | Lint errors | Multi-perspective analysis |
| Intent Capture | Manual | Auto from docs |
| Team Collaboration | Files + comments | Structured workflows |
| Knowledge Retention | Ad-hoc | Pattern library + ADRs |

**For skill installation and workflows, see [GETTING_STARTED_SKILL.md](../../GETTING_STARTED_SKILL.md).**

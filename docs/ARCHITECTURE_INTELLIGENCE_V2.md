# Architecture Intelligence 2.0: Four-Layer Model for the AI Era

**Status:** Draft  
**Date:** 2026-02-23  
**Goal:** Transform Sruja from structural analysis tool into comprehensive architecture cognition engine capable of detecting semantic erosion, intent violations, and AI-generated coupling.

---

## Terminology Clarification

| Term | Meaning | Sruja Usage |
|------|---------|-------------|
| **Tree-sitter** | Code parsing library (AST extraction) | `sruja-scan/tree_sitter.rs` — extracts imports/exports from source |
| **Tree decomposition / treewidth** | Graph-theoretic analysis (separators, bags) | Not yet implemented; operates on the dependency graph *after* extraction |
| **Structural analysis** | DFS cycles, SCC, centrality, coupling | `sruja-diff` — cycles, orphans, layers, god modules |

Tree-sitter *builds* the graph; structural analysis *analyzes* it. Neither implements full tree decomposition yet. The "tree-sitting is necessary but not sufficient" insight applies to all structural graph analysis.

---

## Executive Summary

Tree-based structural analysis (treewidth, SCC, cycles) provides **necessary but insufficient** intelligence for modern software architecture. This document outlines a 4-layer architecture intelligence model that addresses the new failure modes of AI-era systems:

| Failure Mode | Traditional Analysis | AI-Era Analysis |
|--------------|---------------------|-----------------|
| Circular dependencies | ✅ Detects | ✅ Detects |
| God components | ✅ Detects | ✅ Detects |
| **Semantic erosion** | ❌ Misses | ✅ Layer 2 |
| **Boundary drift** | ❌ Misses | ✅ Layer 3 |
| **AI-generated coupling** | ❌ Misses | ✅ Layer 2+3 |
| **Intent violations** | ❌ Misses | ✅ Layer 3 |
| **Runtime orchestration collapse** | ❌ Misses | ✅ Layer 4 |

---

## Current State Assessment

### What Exists

| Component | Location | Capability |
|-----------|----------|------------|
| **sruja-scan** | `crates/sruja-scan` | Tree-sitter code scanning, `NodeKind`, `EdgeKind`, evidence tracking |
| **sruja-diff** | `crates/sruja-diff` | Drift detection (cycles, orphans, layer violations, god modules); `compare_graphs` for graph-vs-graph |
| **sruja-extract** | `crates/sruja-extract` | LLM-based extraction from conversations (optional) |
| **sruja-graph** | `crates/sruja-graph` | Knowledge graph with decisions, policies, requirements |
| **sruja-engine** | `crates/sruja-engine` | Validation rules engine (DSL cycle/orphan detection) |

### Implementation Nuances (Re-review 2026-02-23)

| Claim | Actual Behavior |
|-------|-----------------|
| God modules = fan-in/fan-out | Only **fan-out** (outgoing deps) is counted; fan-in (dependents) not measured |
| Layer violations = generic layering | Only **frontend→database** direct access, via label heuristics ("frontend", "ui", "web"); no general presentation/service/data model |
| Drift vs baseline | `compare_graphs(actual, proposed)` exists; **CLI never calls it**. `detect_architectural_drift` is scan-only (no baseline). `sruja drift` ignores `architecture_path` |
| DSL→Graph bridge | **Missing**. `Program` (DSL AST) and `sruja_scan::Graph` are different models. No `Program → Graph` converter to compare declared intent vs scan |
| SCC, treewidth | Not implemented; DFS cycles only |

### Gap Analysis

| Layer | Current State | Gap Level | Action Required |
|-------|---------------|-----------|-----------------|
| **Layer 1 - Structural** | ✅ Strong | Minor | Add treewidth, enhance metrics |
| **Layer 2 - Semantic** | ⚠️ Minimal | **Major** | Build from scratch |
| **Layer 3 - Intent vs Reality** | ⚠️ Partial | **Major** | Extend diff + add ADR parsing |
| **Layer 4 - Runtime** | ❌ Missing | **Critical** | New subsystem |

---

## Layer 1: Structural Graph Intelligence

### Purpose

Detect structural hotspots, measure modularity health, identify high-complexity clusters through graph-theoretic analysis.

### Current Capabilities

```
✅ Cycle detection (DFS-based)
✅ Orphan detection (disconnected modules; excludes containment edges)
✅ Layer violation detection (frontend→database only; label heuristics)
✅ God module detection (fan-out threshold = 10; fan-in not measured)
✅ Health score calculation
✅ Evidence-based reporting
```

**Note:** `compare_graphs(actual, proposed)` exists for graph-vs-graph comparison but is not used by the CLI. Baseline comparison (DSL vs scan) requires a `Program → Graph` converter first.

### Enhancements Required

#### 1.1 Treewidth Calculation

**What:** Measure structural complexity via tree decomposition. Higher treewidth = more cyclic/interdependent structure.

**Why:** Treewidth predicts:
- Refactoring difficulty
- Testing complexity
- Maintenance cost
- Team coordination overhead

**Implementation:**

```rust
// crates/sruja-graph/src/treewidth.rs

/// Treewidth analysis for dependency graphs
pub struct TreewidthAnalyzer {
    /// Algorithm: "min-fill" heuristic (fast approximation)
    algorithm: TreewidthAlgorithm,
}

pub enum TreewidthAlgorithm {
    /// Fast O(n²) heuristic - good for large graphs
    MinFill,
    /// Exact O(2^n) - only for small graphs (<50 nodes)
    Exact,
    /// Bucket elimination - O(n * w) where w is treewidth
    BucketElimination,
}

pub struct TreewidthResult {
    /// Approximate treewidth
    pub treewidth: usize,
    /// Separator sets for each bag
    pub decomposition: Vec<TreeBag>,
    /// High-complexity clusters (treewidth > threshold)
    pub hotspots: Vec<ComplexityHotspot>,
    /// Comparison to baseline/industry
    pub complexity_rating: ComplexityRating,
}

pub struct ComplexityHotspot {
    pub nodes: Vec<NodeId>,
    pub treewidth: usize,
    pub suggested_refactor: RefactorSuggestion,
}

pub enum ComplexityRating {
    Low,      // treewidth < 3 - easy to reason about
    Moderate, // treewidth 3-5 - manageable
    High,     // treewidth 5-10 - needs attention
    Critical, // treewidth > 10 - urgent refactor needed
}
```

**CLI Integration:**

```bash
sruja complexity --repo . --include-treewidth

# Output:
# Structural Complexity Report
# ─────────────────────────────
# Treewidth: 7 (High)
# Hotspots:
#   - [auth, user, session, token, cache, db] treewidth=9
#     Suggestion: Extract auth-core from session management
#   - [order, payment, inventory, shipping] treewidth=6
#     Suggestion: Introduce event-driven coordination
```

#### 1.2 Strongly Connected Components (SCC) Analysis

**What:** Identify maximal cyclic subgraphs.

**Why:** SCCs reveal:
- Tightly coupled clusters
- Potential domain boundaries (inverted)
- Refactoring targets

**Implementation:**

```rust
// crates/sruja-graph/src/scc.rs

pub struct SccAnalyzer;

pub struct SccResult {
    /// All SCCs sorted by size (largest first)
    pub components: Vec<Scc>,
    /// SCC condensation graph (DAG of SCCs)
    pub condensation: DiGraph<SccId, ()>,
}

pub struct Scc {
    pub id: SccId,
    pub nodes: Vec<NodeId>,
    /// Is this SCC a cycle?
    pub is_cyclic: bool,
    /// Coupling density within SCC
    pub internal_density: f32,
    /// Suggested boundary
    pub suggested_boundary: Option<String>,
}
```

#### 1.3 Centrality Metrics

**What:** Identify architecturally significant nodes.

**Implementation:**

```rust
// crates/sruja-graph/src/centrality.rs

pub struct CentralityMetrics {
    /// Nodes most depended upon
    pub betweenness: HashMap<NodeId, f64>,
    /// Nodes that connect disparate parts
    pub closeness: HashMap<NodeId, f64>,
    /// Hub nodes in cycles
    pub eigenvector: HashMap<NodeId, f64>,
}

pub struct ArchitecturalHotspot {
    pub node: NodeId,
    pub centrality_score: f64,
    pub role: HotspotRole,
}

pub enum HotspotRole {
    /// Central hub - many depend on it
    Hub,
    /// Bridge - connects otherwise disconnected parts
    Bridge,
    /// Bottleneck - in many critical paths
    Bottleneck,
}
```

#### 1.4 Coupling Metrics

**What:** Measure inter-module coupling beyond simple fan-in/fan-out.

```rust
// crates/sruja-graph/src/coupling.rs

pub struct CouplingMetrics {
    /// Afferent coupling (incoming dependencies)
    pub ca: HashMap<NodeId, usize>,
    /// Efferent coupling (outgoing dependencies)
    pub ce: HashMap<NodeId, usize>,
    /// Instability = Ce / (Ca + Ce)
    pub instability: HashMap<NodeId, f64>,
    /// Abstractness (ratio of abstract types)
    pub abstractness: HashMap<NodeId, f64>,
    /// Distance from main sequence (0 = ideal)
    pub distance: HashMap<NodeId, f64>,
}

pub struct CouplingViolation {
    pub module: NodeId,
    pub violation_type: CouplingViolationType,
    pub current_value: f64,
    pub recommended_value: f64,
    pub suggestion: String,
}

pub enum CouplingViolationType {
    /// Too stable - hard to change, should be abstract
    OverlyStable,
    /// Too unstable - changes too often, should be concrete
    OverlyUnstable,
    /// Zone of pain - concrete + stable
    ZoneOfPain,
    /// Zone of uselessness - abstract + unstable
    ZoneOfUselessness,
}
```

### Layer 1 API Summary

```rust
// crates/sruja-graph/src/lib.rs additions

pub mod treewidth;
pub mod scc;
pub mod centrality;
pub mod coupling;

pub struct StructuralIntelligence {
    graph: Graph,
}

impl StructuralIntelligence {
    pub fn analyze_treewidth(&self) -> TreewidthResult;
    pub fn find_sccs(&self) -> SccResult;
    pub fn compute_centrality(&self) -> CentralityMetrics;
    pub fn compute_coupling(&self) -> CouplingMetrics;
    pub fn generate_complexity_report(&self) -> ComplexityReport;
}
```

---

## Layer 2: Semantic Architecture Model

### Purpose

Extract and analyze semantic relationships that structural graphs cannot see:
- Shared domain concepts across components
- Semantic coupling via vocabulary
- Bounded context identification
- Cross-context language leakage

### Core Concepts

#### 2.1 Semantic Coupling

Two components may be **semantically coupled** even without structural dependencies:

```
┌─────────────────┐     ┌─────────────────┐
│  OrderService   │     │  PaymentService │
│                 │     │                 │
│  processOrder() │     │  processPayment()│
│  calculateTotal()│    │  calculateFee() │
│  validateCart() │     │  validateCard() │
└─────────────────┘     └─────────────────┘
        │                       │
        └───────┬───────────────┘
                │
        Shared Domain Concepts:
        - "order", "payment", "cart"
        - "customer", "total", "fee"
        - Business rules overlap
```

#### 2.2 Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Semantic Intelligence                       │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐│
│  │ Embedding       │  │ Domain          │  │ Context        ││
│  │ Generator       │→ │ Clusterer       │→ │ Mapper         ││
│  └─────────────────┘  └─────────────────┘  └────────────────┘│
│         │                     │                    │          │
│         ▼                     ▼                    ▼          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐│
│  │ Vector Store    │  │ Vocabulary      │  │ Boundary       ││
│  │ (embedding db)  │  │ Graph           │  │ Detector       ││
│  └─────────────────┘  └─────────────────┘  └────────────────┘│
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

### New Crate: `sruja-semantic`

```
crates/sruja-semantic/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── embedding/
    │   ├── mod.rs
    │   ├── provider.rs        # Trait for embedding providers
    │   ├── openai.rs          # OpenAI embeddings
    │   ├── local.rs           # Local model embeddings
    │   └── cache.rs           # Embedding cache
    ├── cluster/
    │   ├── mod.rs
    │   ├── domain.rs          # Domain concept clustering
    │   ├── context.rs         # Bounded context detection
    │   └── dbscan.rs          # DBSCAN clustering algorithm
    ├── vocabulary/
    │   ├── mod.rs
    │   ├── extractor.rs       # Extract domain vocabulary
    │   ├── graph.rs           # Vocabulary relationship graph
    │   └── leakage.rs         # Detect vocabulary leakage
    ├── similarity/
    │   ├── mod.rs
    │   ├── cosine.rs          # Cosine similarity
    │   └── semantic.rs        # Semantic distance
    └── analysis/
        ├── mod.rs
        ├── coupling.rs        # Semantic coupling detection
        ├── boundary.rs        # Boundary violation detection
        └── report.rs          # Semantic analysis reports
```

### Core Types

```rust
// crates/sruja-semantic/src/lib.rs

pub mod embedding;
pub mod cluster;
pub mod vocabulary;
pub mod similarity;
pub mod analysis;

pub use embedding::{EmbeddingProvider, EmbeddingVector};
pub use cluster::{DomainCluster, BoundedContext};
pub use vocabulary::{VocabularyGraph, VocabularyNode};
pub use analysis::{SemanticAnalysis, SemanticCoupling, BoundaryViolation};

/// Main entry point for semantic analysis
pub struct SemanticIntelligence {
    embedding_provider: Box<dyn EmbeddingProvider>,
    vector_store: VectorStore,
    config: SemanticConfig,
}

pub struct SemanticConfig {
    /// Minimum similarity threshold for coupling detection
    pub coupling_threshold: f32,
    /// Minimum cluster size for bounded context
    pub min_context_size: usize,
    /// Vocabulary frequency threshold
    pub vocab_min_frequency: usize,
}
```

### Embedding Provider Trait (Provider-Agnostic)

```rust
// crates/sruja-semantic/src/embedding/provider.rs

use async_trait::async_trait;

/// Trait for embedding providers - allows swapping implementations
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> Result<EmbeddingVector, EmbeddingError>;
    
    /// Generate embeddings for multiple texts (batched)
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<EmbeddingVector>, EmbeddingError>;
    
    /// Get the dimensionality of embeddings
    fn dimension(&self) -> usize;
    
    /// Get provider name for logging/debugging
    fn provider_name(&self) -> &str;
}

pub type EmbeddingVector = Vec<f32>;

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### Provider Implementations

```rust
// crates/sruja-semantic/src/embedding/openai.rs

pub struct OpenAIEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<EmbeddingVector, EmbeddingError> {
        // OpenAI API call
    }
    
    fn dimension(&self) -> usize { 1536 } // text-embedding-3-small
    fn provider_name(&self) -> &str { "openai" }
}

// crates/sruja-semantic/src/embedding/local.rs

pub struct LocalEmbeddingProvider {
    model_path: PathBuf,
    // Use candle or similar for local inference
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<EmbeddingVector, EmbeddingError> {
        // Local model inference
    }
    
    fn dimension(&self) -> usize { 384 } // all-MiniLM-L6-v2
    fn provider_name(&self) -> &str { "local" }
}
```

### Domain Clustering

```rust
// crates/sruja-semantic/src/cluster/domain.rs

pub struct DomainClusterer {
    min_cluster_size: usize,
    epsilon: f32,
}

pub struct DomainCluster {
    /// Cluster ID
    pub id: String,
    /// Representative terms
    pub centroid_terms: Vec<String>,
    /// Components in this cluster
    pub components: Vec<NodeId>,
    /// Semantic coherence score
    pub coherence: f32,
    /// Suggested bounded context name
    pub suggested_context: String,
}

impl DomainClusterer {
    /// Cluster components by their semantic content
    pub fn cluster(
        &self,
        component_embeddings: HashMap<NodeId, EmbeddingVector>,
        vocabulary: &VocabularyGraph,
    ) -> Vec<DomainCluster> {
        // DBSCAN or hierarchical clustering
    }
}
```

### Bounded Context Detection

```rust
// crates/sruja-semantic/src/cluster/context.rs

pub struct BoundedContextDetector {
    min_size: usize,
    coupling_threshold: f32,
}

pub struct BoundedContext {
    /// Context name (auto-generated or from ADR)
    pub name: String,
    /// Components in this context
    pub components: Vec<NodeId>,
    /// Core vocabulary
    pub vocabulary: Vec<String>,
    /// Shared vocabulary (potential leakage)
    pub shared_vocabulary: Vec<SharedTerm>,
    /// Context boundaries
    pub boundaries: Vec<ContextBoundary>,
}

pub struct SharedTerm {
    pub term: String,
    /// Contexts that share this term
    pub contexts: Vec<String>,
    /// Is this intentional (ubiquitous language) or leakage?
    pub is_intentional: bool,
    /// Confidence that this is accidental leakage
    pub leakage_confidence: f32,
}

pub struct ContextBoundary {
    pub from_context: String,
    pub to_context: String,
    /// Structural relationships crossing this boundary
    pub crossing_edges: Vec<EdgeId>,
    /// Is this boundary healthy?
    pub health: BoundaryHealth,
}

pub enum BoundaryHealth {
    /// Clear API-based communication
    Healthy,
    /// Some direct database sharing
    Questionable,
    /// Direct module access, shared state
    Violated,
}

impl BoundedContextDetector {
    /// Detect bounded contexts from semantic clusters
    pub fn detect(
        &self,
        clusters: &[DomainCluster],
        graph: &Graph,
    ) -> Vec<BoundedContext>;
    
    /// Find vocabulary leakage between contexts
    pub fn find_leakage(&self, contexts: &[BoundedContext]) -> Vec<VocabularyLeakage>;
}
```

### Semantic Coupling Analysis

```rust
// crates/sruja-semantic/src/analysis/coupling.rs

pub struct SemanticCouplingAnalyzer {
    similarity_threshold: f32,
}

pub struct SemanticCoupling {
    /// Source component
    pub source: NodeId,
    /// Target component
    pub target: NodeId,
    /// Semantic similarity score (0-1)
    pub similarity: f32,
    /// Shared domain concepts
    pub shared_concepts: Vec<String>,
    /// Is there also structural coupling?
    pub has_structural_coupling: bool,
    /// Coupling type
    pub coupling_type: SemanticCouplingType,
}

pub enum SemanticCouplingType {
    /// Direct shared vocabulary
    SharedVocabulary,
    /// Similar business logic concepts
    BusinessLogic,
    /// Similar data models
    DataModel,
    /// Similar error handling patterns
    ErrorHandling,
    /// Similar configuration/environment
    Configuration,
}

pub struct SemanticCouplingReport {
    /// All detected semantic couplings
    pub couplings: Vec<SemanticCoupling>,
    /// Couplings without structural relationships (hidden coupling)
    pub hidden_couplings: Vec<SemanticCoupling>,
    /// Components with high semantic fan-out
    pub semantic_hubs: Vec<SemanticHub>,
    /// Recommendations
    pub recommendations: Vec<SemanticRecommendation>,
}

impl SemanticCouplingAnalyzer {
    /// Analyze semantic coupling between components
    pub fn analyze(
        &self,
        embeddings: &HashMap<NodeId, EmbeddingVector>,
        graph: &Graph,
    ) -> SemanticCouplingReport;
}
```

### CLI Integration

```bash
# Analyze semantic coupling
sruja semantic analyze --repo . --format json

# Output:
{
  "contexts": [
    {
      "name": "OrderManagement",
      "components": ["order-service", "cart-service", "inventory-service"],
      "vocabulary": ["order", "cart", "item", "quantity", "fulfillment"]
    },
    {
      "name": "Payment",
      "components": ["payment-service", "billing-service"],
      "vocabulary": ["payment", "invoice", "transaction", "fee"]
    }
  ],
  "hiddenCouplings": [
    {
      "source": "order-service",
      "target": "payment-service",
      "similarity": 0.78,
      "sharedConcepts": ["customer", "transaction", "amount"],
      "hasStructuralCoupling": false
    }
  ],
  "vocabularyLeaks": [
    {
      "term": "customer",
      "contexts": ["OrderManagement", "Payment", "UserManagement"],
      "isIntentional": true,
      "leakageConfidence": 0.2
    }
  ]
}
```

---

## Layer 3: Intent vs Reality Comparison

### Purpose

Compare declared architectural intent (ADRs, .sruja files, design docs) against actual implementation to detect:
- Boundary drift
- Intent violations
- Undocumented architectural changes
- AI-generated shortcuts

### Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Intent Intelligence                         │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐│
│  │ Intent Sources  │  │ Intent          │  │ Drift          ││
│  │ Parser          │→ │ Normalizer      │→ │ Detector       ││
│  └─────────────────┘  └─────────────────┘  └────────────────┘│
│         │                     │                    │          │
│         ▼                     ▼                    ▼          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐│
│  │ ADR Parser      │  │ Intent Model    │  │ Drift Report   ││
│  │ .sruja Parser   │  │ (normalized)    │  │ Generator      ││
│  │ Doc Parser      │  │                 │  │                ││
│  └─────────────────┘  └─────────────────┘  └────────────────┘│
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

### Prerequisite: Program → Graph Converter

**Required before full intent comparison.** The DSL produces `Program` (AST); `sruja-diff` and `sruja-scan` use `sruja_scan::Graph`. To compare declared intent (`.sruja`) vs discovered reality (scan):

1. Build a `Program → sruja_scan::Graph` converter (elements → nodes, relations → edges)
2. Then use `compare_graphs(actual: scan_graph, proposed: dsl_graph)` or add `detect_drift(declared: &Program, discovered: &Graph)`
3. Wire `sruja drift --baseline foo.sruja` to load DSL, convert to Graph, compare against scan

**Baseline must be optional** to preserve zero-key first value; `sruja drift -r .` should remain valid without a `.sruja` file.

### Intent Sources

| Source | Parser | What It Captures |
|--------|--------|------------------|
| `.sruja` files | ✅ Existing | Component definitions, relationships, policies |
| ADR files | ⚠️ Partial | Decisions, context, consequences |
| Design docs | ❌ New | Requirements, constraints, patterns |
| Code comments | ❌ New | Intent annotations, TODO patterns |
| PR descriptions | ❌ New | Change intent, rationale |

### New Crate: `sruja-intent`

```
crates/sruja-intent/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── parser/
    │   ├── mod.rs
    │   ├── adr.rs              # ADR file parsing
    │   ├── sruja.rs            # .sruja file parsing (reuse existing)
    │   ├── doc.rs              # Design doc parsing
    │   └── comment.rs          # Code comment intent extraction
    ├── model/
    │   ├── mod.rs
    │   ├── intent.rs           # Normalized intent model
    │   ├── boundary.rs         # Declared boundaries
    │   └── policy.rs           # Declared policies
    ├── compare/
    │   ├── mod.rs
    │   ├── aligner.rs          # Align intent with reality
    │   ├── drift.rs            # Drift detection
    │   └── violation.rs        # Intent violation detection
    └── report/
        ├── mod.rs
        └── drift.rs            # Drift reporting
```

### Core Types

```rust
// crates/sruja-intent/src/lib.rs

pub mod parser;
pub mod model;
pub mod compare;
pub mod report;

pub use model::{IntentModel, DeclaredBoundary, DeclaredPolicy};
pub use compare::{IntentAligner, DriftDetector};
pub use report::{DriftReport, IntentViolation};

/// Main entry point for intent analysis
pub struct IntentIntelligence {
    intent_sources: Vec<Box<dyn IntentSource>>,
    reality_graph: Graph,
}

pub trait IntentSource {
    fn parse(&self, path: &Path) -> Result<IntentModel, IntentError>;
    fn source_type(&self) -> IntentSourceType;
}

pub enum IntentSourceType {
    SrujaFile,
    AdrFile,
    DesignDoc,
    CodeComments,
    PrDescription,
}
```

### ADR Parser

```rust
// crates/sruja-intent/src/parser/adr.rs

pub struct AdrParser;

pub struct ParsedAdr {
    pub number: u32,
    pub title: String,
    pub status: AdrStatus,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    /// Extracted structural implications
    pub implications: Vec<StructuralImplication>,
}

pub enum AdrStatus {
    Proposed,
    Accepted,
    Deprecated,
    Superseded { by: u32 },
    Rejected,
}

pub struct StructuralImplication {
    /// Component affected
    pub component: String,
    /// Boundary change implied
    pub boundary_change: Option<BoundaryChange>,
    /// New policy implied
    pub new_policy: Option<String>,
    /// Constraint implied
    pub constraint: Option<String>,
}

pub struct BoundaryChange {
    pub component: String,
    pub change_type: BoundaryChangeType,
    pub description: String,
}

pub enum BoundaryChangeType {
    Added,
    Removed,
    Expanded,
    Contracted,
    Split,
    Merged,
}

impl AdrParser {
    /// Parse ADR directory
    pub fn parse_dir(&self, dir: &Path) -> Result<Vec<ParsedAdr>, IntentError>;
    
    /// Parse single ADR file (supports multiple formats)
    pub fn parse_file(&self, file: &Path) -> Result<ParsedAdr, IntentError>;
}
```

### Intent Model

```rust
// crates/sruja-intent/src/model/intent.rs

/// Normalized representation of architectural intent from all sources
pub struct IntentModel {
    /// Source of this intent
    pub source: IntentSource,
    /// Declared components
    pub components: Vec<DeclaredComponent>,
    /// Declared relationships
    pub relationships: Vec<DeclaredRelationship>,
    /// Declared boundaries
    pub boundaries: Vec<DeclaredBoundary>,
    /// Declared policies
    pub policies: Vec<DeclaredPolicy>,
    /// Declared constraints
    pub constraints: Vec<DeclaredConstraint>,
}

pub struct DeclaredComponent {
    pub id: String,
    pub kind: NodeKind,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub source: SourceReference,
}

pub struct DeclaredRelationship {
    pub source: String,
    pub target: String,
    pub kind: EdgeKind,
    pub label: Option<String>,
    pub source_ref: SourceReference,
}

pub struct DeclaredBoundary {
    /// Boundary name (e.g., "Order Context")
    pub name: String,
    /// Components inside boundary
    pub inside: Vec<String>,
    /// Allowed external connections
    pub allowed_connections: Vec<AllowedConnection>,
    /// Boundary rules
    pub rules: Vec<BoundaryRule>,
    pub source_ref: SourceReference,
}

pub struct AllowedConnection {
    pub target_boundary: String,
    pub via: ConnectionType,
}

pub enum ConnectionType {
    ApiCall,
    EventBus,
    Database,
    DirectCall,
}

pub struct BoundaryRule {
    pub rule_type: BoundaryRuleType,
    pub description: String,
}

pub enum BoundaryRuleType {
    NoDirectDatabaseAccess,
    ApiOnly,
    EventBusOnly,
    NoSharedState,
}
```

### Drift Detection

```rust
// crates/sruja-intent/src/compare/drift.rs

pub struct DriftDetector {
    config: DriftConfig,
}

pub struct DriftConfig {
    /// Severity threshold for reporting
    pub severity_threshold: Severity,
    /// Consider semantic similarity in alignment
    pub use_semantic_similarity: bool,
    /// Minimum confidence for intent extraction
    pub min_confidence: f32,
}

pub struct DriftReport {
    /// Intent model used for comparison
    pub intent_source: String,
    /// Reality graph used for comparison
    pub reality_source: String,
    /// Detected drifts
    pub drifts: Vec<Drift>,
    /// Overall drift score (0-100, lower is better)
    pub drift_score: u8,
    /// Health assessment
    pub health: DriftHealth,
}

pub struct Drift {
    /// Type of drift
    pub kind: DriftKind,
    /// Severity
    pub severity: Severity,
    /// Description
    pub description: String,
    /// Evidence from reality
    pub evidence: Vec<Evidence>,
    /// Intent reference
    pub intent_ref: SourceReference,
    /// Suggested fix
    pub suggestion: Option<String>,
}

pub enum DriftKind {
    /// Component exists in reality but not in intent
    UndocumentedComponent,
    /// Component in intent doesn't exist in reality
    MissingComponent,
    /// Relationship in reality violates declared boundary
    BoundaryViolation,
    /// Relationship in reality not declared in intent
    UndocumentedRelationship,
    /// Technology mismatch
    TechnologyMismatch,
    /// Policy violation
    PolicyViolation,
    /// Constraint violation
    ConstraintViolation,
}

pub enum DriftHealth {
    /// Drift score 0-20: Intent and reality well aligned
    Healthy,
    /// Drift score 21-50: Minor drift, review recommended
    MinorDrift,
    /// Drift score 51-75: Significant drift, action needed
    SignificantDrift,
    /// Drift score 76-100: Critical drift, urgent attention
    CriticalDrift,
}

impl DriftDetector {
    /// Detect drift between intent and reality
    pub fn detect(
        &self,
        intent: &IntentModel,
        reality: &Graph,
    ) -> DriftReport;
    
    /// Detect drift specifically for boundaries
    pub fn detect_boundary_drift(
        &self,
        boundaries: &[DeclaredBoundary],
        reality: &Graph,
    ) -> Vec<BoundaryDrift>;
}
```

### CLI Integration

```bash
# Compare intent vs reality
sruja intent check --repo . --intent ./docs/architecture

# Output:
# Intent vs Reality Comparison
# ─────────────────────────────────────────────────────────────
# Intent Source: docs/architecture/ (3 .sruja files, 5 ADRs)
# Reality Source: scanned from src/
#
# Drift Score: 42/100 (Minor Drift)
#
# Detected Drifts:
# ─────────────────────────────────────────────────────────────
# [MEDIUM] Undocumented Component
#   - Component 'NotificationWorker' exists in code but not in architecture docs
#   - Evidence: src/workers/notification.rs
#   - Suggestion: Add to architecture or document as temporary
#
# [HIGH] Boundary Violation (ADR-003)
#   - OrderService directly accesses Database (bypasses Data Access Layer)
#   - Declared boundary: "All database access through repository layer"
#   - Evidence: src/services/order.rs:245
#   - Suggestion: Introduce OrderRepository or update ADR-003
#
# [LOW] Undocumented Relationship
#   - PaymentService -> AnalyticsService (not documented)
#   - Evidence: src/services/payment.rs:89

# Generate ADR from detected reality
sruja intent propose --repo . --from-drift

# Output ADR template:
# # ADR-XXXX: Document Current Reality
# 
# ## Status
# Proposed
#
# ## Context
# The following components exist in the codebase but are not documented:
# - NotificationWorker
# - AnalyticsService connection from PaymentService
#
# ## Decision
# (To be filled)
#
# ## Consequences
# (To be filled)
```

---

## Layer 4: Runtime Path Intelligence

### Purpose

Track and analyze runtime behavior of AI-heavy systems:
- Agent execution trees
- Tool invocation graphs
- Emergent cycles in workflows
- Dynamic service compositions

**Note:** This layer is targeted for Phase 3-4 after Layers 2 and 3 are stable.

### Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Runtime Intelligence                        │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐│
│  │ Trace           │  │ Trace           │  │ Runtime        ││
│  │ Collector       │→ │ Processor       │→ │ Analyzer       ││
│  └─────────────────┘  └─────────────────┘  └────────────────┘│
│         │                     │                    │          │
│         ▼                     ▼                    ▼          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌────────────────┐│
│  │ OTLP/OTel       │  │ Execution       │  │ Workflow       ││
│  │ Integration     │  │ Graph           │  │ Health         ││
│  └─────────────────┘  └─────────────────┘  └────────────────┘│
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

### New Crate: `sruja-runtime`

```
crates/sruja-runtime/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── trace/
    │   ├── mod.rs
    │   ├── collector.rs       # Trace collection
    │   ├── processor.rs       # Trace processing
    │   └── otlp.rs            # OpenTelemetry integration
    ├── agent/
    │   ├── mod.rs
    │   ├── execution.rs       # Agent execution trees
    │   ├── tool_graph.rs      # Tool invocation graphs
    │   └── workflow.rs        # Workflow analysis
    ├── analysis/
    │   ├── mod.rs
    │   ├── cycle.rs           # Emergent cycle detection
    │   ├── hotspot.rs         # Runtime hotspots
    │   └── anomaly.rs         # Anomaly detection
    └── report/
        ├── mod.rs
        └── runtime.rs         # Runtime reports
```

### Core Types

```rust
// crates/sruja-runtime/src/lib.rs

pub mod trace;
pub mod agent;
pub mod analysis;
pub mod report;

pub use trace::{TraceCollector, TraceProcessor, ExecutionTrace};
pub use agent::{AgentExecutionTree, ToolInvocationGraph};
pub use analysis::{RuntimeAnalyzer, EmergentCycle};

/// Main entry point for runtime analysis
pub struct RuntimeIntelligence {
    trace_collector: Box<dyn TraceCollector>,
    analyzer: RuntimeAnalyzer,
}
```

### Agent Execution Trees

```rust
// crates/sruja-runtime/src/agent/execution.rs

pub struct AgentExecutionTree {
    /// Root execution node
    pub root: ExecutionNode,
    /// Agent that executed
    pub agent_id: String,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// End time
    pub ended_at: DateTime<Utc>,
    /// Total tokens used (for LLM agents)
    pub tokens_used: Option<TokenUsage>,
}

pub struct ExecutionNode {
    /// Node ID
    pub id: String,
    /// Node type
    pub kind: ExecutionNodeKind,
    /// Children
    pub children: Vec<ExecutionNode>,
    /// Duration
    pub duration: Duration,
    /// Status
    pub status: ExecutionStatus,
    /// Input/Output
    pub io: Option<NodeIO>,
}

pub enum ExecutionNodeKind {
    /// Agent thinking/reasoning step
    Reasoning,
    /// Tool invocation
    ToolCall { tool_name: String },
    /// LLM generation
    LlmGeneration,
    /// Conditional branch
    Branch,
    /// Loop
    Loop,
    /// Error handling
    ErrorHandler,
    /// External service call
    ExternalCall { service: String },
}

pub enum ExecutionStatus {
    Success,
    Failed { error: String },
    Timeout,
    Cancelled,
}

pub struct TokenUsage {
    pub prompt: u64,
    pub completion: u64,
    pub total: u64,
}
```

### Tool Invocation Graph

```rust
// crates/sruja-runtime/src/agent/tool_graph.rs

pub struct ToolInvocationGraph {
    /// All tool invocations
    pub invocations: Vec<ToolInvocation>,
    /// Aggregated graph
    pub graph: DiGraph<ToolNode, ToolEdge>,
}

pub struct ToolInvocation {
    /// Invocation ID
    pub id: String,
    /// Tool name
    pub tool: String,
    /// Caller (agent or tool)
    pub caller: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Duration
    pub duration: Duration,
    /// Success/failure
    pub success: bool,
    /// Dependencies (tools that must run first)
    pub depends_on: Vec<String>,
}

pub struct ToolNode {
    pub tool_name: String,
    pub invocation_count: usize,
    pub success_rate: f32,
    pub avg_duration: Duration,
}

pub struct ToolEdge {
    pub from_tool: String,
    pub to_tool: String,
    pub transition_count: usize,
}

impl ToolInvocationGraph {
    /// Find tool chains (common sequences)
    pub fn find_chains(&self, min_length: usize) -> Vec<ToolChain>;
    
    /// Find tools that are always called together
    pub fn find_coupled_tools(&self) -> Vec<ToolCoupling>;
    
    /// Find tools that are never called
    pub fn find_unused_tools(&self) -> Vec<String>;
}
```

### Emergent Cycle Detection

```rust
// crates/sruja-runtime/src/analysis/cycle.rs

pub struct EmergentCycleDetector {
    min_occurrences: usize,
}

pub struct EmergentCycle {
    /// Cycle pattern (A -> B -> C -> A)
    pub pattern: Vec<String>,
    /// How often this cycle occurred
    pub occurrences: usize,
    /// Time range of occurrences
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    /// Severity
    pub severity: CycleSeverity,
    /// Root cause hypothesis
    pub root_cause_hypothesis: Option<String>,
}

pub enum CycleSeverity {
    /// Informational - normal behavior
    Info,
    /// Warning - potential issue
    Warning,
    /// Error - definite problem
    Error,
    /// Critical - system degradation
    Critical,
}

impl EmergentCycleDetector {
    /// Detect emergent cycles from execution traces
    pub fn detect(&self, traces: &[ExecutionTrace]) -> Vec<EmergentCycle>;
}
```

### OpenTelemetry Integration

```rust
// crates/sruja-runtime/src/trace/otlp.rs

pub struct OtlpTraceCollector {
    endpoint: String,
    client: reqwest::Client,
}

impl OtlpTraceCollector {
    /// Collect traces from OpenTelemetry endpoint
    pub async fn collect(&self) -> Result<Vec<ExecutionTrace>, TraceError>;
    
    /// Convert OTLP spans to execution traces
    pub fn convert_spans(&self, spans: Vec<OtlpSpan>) -> Vec<ExecutionTrace>;
}
```

### CLI Integration

```bash
# Analyze runtime behavior
sruja runtime analyze --traces ./traces.json --format json

# Output:
{
  "executionTrees": 1523,
  "toolInvocations": 8432,
  "emergentCycles": [
    {
      "pattern": ["planner", "executor", "validator", "planner"],
      "occurrences": 23,
      "severity": "Warning",
      "rootCauseHypothesis": "Validator rejecting executor output, causing replan loop"
    }
  ],
  "toolChains": [
    {
      "chain": ["search", "extract", "summarize"],
      "occurrences": 156,
      "avgDuration": "2.3s"
    }
  ],
  "unusedTools": ["legacy_search", "deprecated_validator"]
}
```

---

## Cross-Cutting Concerns

### Configuration

```rust
// crates/sruja-config/src/architecture_intelligence.rs

pub struct ArchitectureIntelligenceConfig {
    /// Layer 1: Structural analysis config
    pub structural: StructuralConfig,
    /// Layer 2: Semantic analysis config (requires embedding provider)
    pub semantic: Option<SemanticConfig>,
    /// Layer 3: Intent comparison config
    pub intent: IntentConfig,
    /// Layer 4: Runtime analysis config (optional)
    pub runtime: Option<RuntimeConfig>,
}

pub struct StructuralConfig {
    /// Enable treewidth analysis
    pub treewidth: bool,
    /// Treewidth complexity threshold
    pub treewidth_threshold: usize,
    /// God module fan-out threshold
    pub god_module_threshold: usize,
}

pub struct SemanticConfig {
    /// Embedding provider
    pub embedding_provider: EmbeddingProviderConfig,
    /// Coupling detection threshold
    pub coupling_threshold: f32,
}

pub enum EmbeddingProviderConfig {
    OpenAI { api_key_env: String, model: String },
    Local { model_path: String },
    Custom { endpoint: String },
}

pub struct IntentConfig {
    /// Paths to intent sources
    pub intent_paths: Vec<PathBuf>,
    /// Drift severity threshold
    pub drift_threshold: Severity,
}

pub struct RuntimeConfig {
    /// Trace collection endpoint
    pub trace_endpoint: String,
    /// Trace retention period
    pub retention: Duration,
}
```

### Unified Report

```rust
// crates/sruja-report/src/comprehensive.rs

pub struct ComprehensiveReport {
    /// Layer 1: Structural analysis
    pub structural: StructuralReport,
    /// Layer 2: Semantic analysis (if enabled)
    pub semantic: Option<SemanticReport>,
    /// Layer 3: Intent comparison
    pub intent: IntentReport,
    /// Layer 4: Runtime analysis (if enabled)
    pub runtime: Option<RuntimeReport>,
    /// Overall health score (weighted average)
    pub overall_health: HealthScore,
    /// Top recommendations
    pub top_recommendations: Vec<Recommendation>,
}

pub struct Recommendation {
    pub priority: Priority,
    pub category: RecommendationCategory,
    pub description: String,
    pub affected_components: Vec<NodeId>,
    pub source_layer: Layer,
    pub estimated_effort: Effort,
}

pub enum Layer {
    Structural,
    Semantic,
    Intent,
    Runtime,
}

pub enum Effort {
    Low,     // < 1 day
    Medium,  // 1-5 days
    High,    // > 5 days
}
```

### CLI Unified Command

```bash
# Comprehensive analysis across all layers
sruja analyze --repo . --all-layers

# Output:
# ═══════════════════════════════════════════════════════════════
# 🏗️ Sruja Architecture Intelligence - Comprehensive Report
# ═══════════════════════════════════════════════════════════════
#
# 📊 Overall Health Score: 68/100
#
# ───────────────────────────────────────────────────────────────
# Layer 1: Structural (Score: 75/100)
# ───────────────────────────────────────────────────────────────
#   Treewidth: 7 (High)
#   Cycles: 2 detected
#   God Modules: 3 detected
#   Orphans: 5 detected
#
# ───────────────────────────────────────────────────────────────
# Layer 2: Semantic (Score: 62/100)
# ───────────────────────────────────────────────────────────────
#   Bounded Contexts: 4 detected
#   Hidden Couplings: 12 detected
#   Vocabulary Leaks: 8 detected
#
# ───────────────────────────────────────────────────────────────
# Layer 3: Intent vs Reality (Score: 58/100)
# ───────────────────────────────────────────────────────────────
#   Drift Score: 42/100 (Minor Drift)
#   Undocumented Components: 7
#   Boundary Violations: 3
#   Policy Violations: 2
#
# ───────────────────────────────────────────────────────────────
# 🎯 Top Recommendations
# ───────────────────────────────────────────────────────────────
#
# [HIGH] Break circular dependency in auth module
#   Layer: Structural
#   Affected: auth, session, token
#   Effort: Medium
#   Why: Cycle causes maintenance difficulty and testing complexity
#
# [HIGH] Document NotificationWorker in architecture
#   Layer: Intent
#   Affected: NotificationWorker
#   Effort: Low
#   Why: Undocumented components lead to knowledge silos
#
# [MEDIUM] Extract shared domain concepts from OrderService
#   Layer: Semantic
#   Affected: order-service, payment-service
#   Effort: High
#   Why: Hidden coupling will cause coordination issues
#
# [MEDIUM] Fix boundary violation per ADR-003
#   Layer: Intent
#   Affected: OrderService, Database
#   Effort: Medium
#   Why: Direct database access bypasses repository layer
```

---

## Short-Term Implementation Improvements

These improvements should be done before or alongside the 4-layer phases, based on code review (2026-02-23).

| Priority | Issue | Location | Fix |
|----------|-------|----------|-----|
| **High** | Cycle deduplication | `sruja-diff::find_circular_dependencies` | Same cycle reported multiple times; deduplicate by canonical representation |
| **High** | Import resolution for `./dir` → `./dir/index.ts` | `sruja-scan/tree_sitter::resolve_import` | Add index file resolution (common in TS/JS) |
| **High** | Duplicate cycle/orphan logic in tests | `sruja-scan/tests/drift_unit.rs` | Move tests to `sruja-diff`, call actual `find_circular_dependencies` / `find_orphan_modules` |
| **Medium** | Duplicate health score logic | `sruja-diff` | Unify `calculate_health_score` (diff) and `calculate_drift_health_score`; different penalty schemes |
| **Medium** | God module threshold hardcoded | `sruja-diff::find_god_modules(graph, 10)` | Make threshold configurable via `ScanConfig` or drift options |
| **Low** | Parse failures silently ignored | `sruja-scan/tree_sitter::parse_file` | Log or surface parse errors instead of returning `None` |
| **Low** | No unit tests for `detect_architectural_drift` | `sruja-diff` | Add tests for drift detection directly |

---

## Implementation Phases

### Phase 0: Adoption & Baseline (Align with AI_FIRST)

**Goal:** Align with NEXT_SESSION and AI_FIRST_MODULE_ANALYSIS_FINAL. Complete before or in parallel with Phase 1.

**Deliverables:**
- [ ] Unify drift heuristics in `sruja-diff` (single source of truth)
- [ ] Improve `sruja why` evidence templates (deterministic, no LLM)
- [ ] Reorder README/docs around no-key first value
- [ ] Build `Program → Graph` converter (prerequisite for baseline)
- [ ] Wire `sruja drift --baseline foo.sruja` (optional; scan-only remains default)

### Phase 1: Layer 1 Enhancement (Weeks 1-3)

**Goal:** Strengthen structural analysis foundation

**Deliverables:**
- [ ] Treewidth calculation module (`sruja-graph/src/treewidth.rs`)
- [ ] SCC analysis module (`sruja-graph/src/scc.rs`)
- [ ] Centrality metrics module (`sruja-graph/src/centrality.rs`)
- [ ] Coupling metrics module (`sruja-graph/src/coupling.rs`)
- [ ] CLI command: `sruja complexity`
- [ ] Integration with existing `sruja quickstart`

**Dependencies:** None (builds on existing `sruja-graph`, `sruja-diff`)

**Testing:**
- Unit tests for each algorithm
- Integration tests against known codebases
- Performance benchmarks for large graphs

### Phase 2: Layer 2 Semantic Intelligence (Weeks 4-7)

**Goal:** Enable semantic coupling and bounded context detection

**Deliverables:**
- [ ] New crate: `sruja-semantic`
- [ ] Embedding provider trait and implementations
- [ ] Domain clustering module
- [ ] Bounded context detector
- [ ] Semantic coupling analyzer
- [ ] Vocabulary graph and leakage detection
- [ ] CLI command: `sruja semantic`
- [ ] Integration with `sruja analyze`

**Dependencies:**
- Phase 1 complete
- Embedding provider configured (optional, degrade gracefully if not)

**Testing:**
- Mock embedding provider for deterministic tests
- Integration tests with real embeddings
- Performance tests with large codebases

### Phase 3: Layer 3 Intent Comparison (Weeks 8-11)

**Goal:** Enable intent vs reality drift detection

**Deliverables:**
- [ ] New crate: `sruja-intent`
- [ ] ADR parser
- [ ] Design doc parser
- [ ] Intent normalizer
- [ ] Intent aligner
- [ ] Drift detector
- [ ] CLI command: `sruja intent`
- [ ] ADR generation from drift

**Dependencies:**
- Phase 1 complete
- Phase 2 complete (for semantic alignment)

**Testing:**
- Parser tests for various ADR formats
- Drift detection accuracy tests
- End-to-end tests with sample projects

### Phase 4: Layer 4 Runtime Intelligence (Weeks 12-16)

**Goal:** Enable runtime behavior analysis for AI systems

**Deliverables:**
- [ ] New crate: `sruja-runtime`
- [ ] Trace collector interface
- [ ] OTLP integration
- [ ] Agent execution tree processor
- [ ] Tool invocation graph builder
- [ ] Emergent cycle detector
- [ ] CLI command: `sruja runtime`
- [ ] Runtime dashboard data

**Dependencies:**
- Phase 1 complete (structural patterns for comparison)
- Trace data source configured

**Testing:**
- Mock trace generator
- Integration with OpenTelemetry
- Performance tests with high-volume traces

### Phase 5: Integration & Polish (Weeks 17-18)

**Goal:** Unified experience and documentation

**Deliverables:**
- [ ] Unified `sruja analyze --all-layers` command
- [ ] Comprehensive report generation
- [ ] Documentation updates
- [ ] AGENTS.md updates
- [ ] Example projects
- [ ] Performance optimization

---

## Success Metrics

### Adoption Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time-to-first-value | < 60 seconds | Install → first useful output |
| `quickstart` repeat rate | > 40% within 7 days | Unique users running twice |
| CI integration rate | > 25% of users | `sruja drift` in CI |
| LLM enrichment adoption | > 30% after deterministic onboarding | Users configuring embedding provider |

### Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| False positive rate (drift) | < 15% | User feedback on drift reports |
| Semantic coupling accuracy | > 80% | Manual validation on sample projects |
| ADR parsing success rate | > 90% | Parse errors vs total ADRs |
| Runtime cycle detection latency | < 5 seconds | For 10k traces |

### Health Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Median findings resolved | > 50% within 30 days | Follow-up scans |
| Architecture health improvement | +10 points in 90 days | Repeated `quickstart` scores |
| Undocumented component reduction | -30% in 90 days | Intent coverage tracking |

---

## Appendix A: Two-Developer Split

For parallel work on Phase 0–1, recommended division:

| Track | Developer | Focus | Crates |
|-------|-----------|-------|--------|
| **A — Drift & structural** | Dev 1 | Unify drift, Program→Graph, drift --baseline, SCC | sruja-diff, sruja-language, sruja-cli (drift) |
| **B — Adoption & discovery** | Dev 2 | sruja why templates, README/docs, blast_radius | sruja-cli (why), docs, sruja-mcp, sruja-graph |

**Week 1:** Dev 1 unifies drift heuristics; Dev 2 improves why evidence templates  
**Week 2:** Dev 1 builds Program→Graph; Dev 2 reorders docs  
**Week 3:** Dev 1 wires drift --baseline; Dev 2 polish/tests  
**Week 4+:** Dev 1 adds SCC; Dev 2 adds blast_radius or MCP enhancements  

**Coordination:** Agree on Program→Graph contract and DriftReport vs DiffResult semantics.

### Pending Work (from NEXT_SESSION / AI_FIRST)

| Source | Item | Resolution |
|--------|------|------------|
| NEXT_SESSION | "require architecture baseline" | **Baseline optional** — `sruja drift -r .` (scan-only) must remain valid. Add `--baseline foo.sruja` when provided. |
| AI_FIRST | Adoption-first (quickstart, why, docs) | Phase 0 prioritizes adoption; Phase 1 can run in parallel |
| INTELLIGENCE_ANALYSIS | Drift DSL vs discovered | Implemented via Program→Graph + compare_graphs; prerequisite in Phase 0 |

---

## Appendix B: File Structure After Implementation

```
sruja/
├── crates/
│   ├── sruja-cli/
│   ├── sruja-language/
│   ├── sruja-engine/
│   ├── sruja-export/
│   ├── sruja-diagnostics/
│   ├── sruja-lsp/
│   ├── sruja-wasm/
│   ├── sruja-scan/           # ✅ Existing
│   ├── sruja-graph/          # ✅ Existing + enhanced
│   │   └── src/
│   │       ├── treewidth.rs  # 🆕 Phase 1
│   │       ├── scc.rs        # 🆕 Phase 1
│   │       ├── centrality.rs # 🆕 Phase 1
│   │       └── coupling.rs   # 🆕 Phase 1
│   ├── sruja-diff/           # ✅ Existing
│   ├── sruja-extract/        # ✅ Existing
│   ├── sruja-semantic/       # 🆕 Phase 2
│   │   └── src/
│   │       ├── embedding/
│   │       ├── cluster/
│   │       ├── vocabulary/
│   │       └── analysis/
│   ├── sruja-intent/         # 🆕 Phase 3
│   │   └── src/
│   │       ├── parser/
│   │       ├── model/
│   │       └── compare/
│   ├── sruja-runtime/        # 🆕 Phase 4
│   │   └── src/
│   │       ├── trace/
│   │       ├── agent/
│   │       └── analysis/
│   ├── sruja-config/         # 🆕 Config unification
│   └── sruja-report/         # 🆕 Report generation
│       └── src/
│           └── comprehensive.rs
├── docs/
│   ├── ARCHITECTURE_INTELLIGENCE.md      # ✅ Existing (update)
│   ├── ARCHITECTURE_INTELLIGENCE_V2.md   # 🆕 This document
│   └── LAYER_*.md                        # 🆕 Detailed layer docs
└── skills/
    └── sruja-architecture/   # ✅ Existing (update with 4-layer model)
```

---

## Appendix C: Configuration Example

```toml
# sruja.toml

[structural]
treewidth = true
treewidth_threshold = 7
god_module_threshold = 10

[semantic]
enabled = true
coupling_threshold = 0.7

[semantic.embedding]
provider = "openai"
model = "text-embedding-3-small"
api_key_env = "OPENAI_API_KEY"

[intent]
intent_paths = ["./docs/architecture", "./docs/adr"]
drift_threshold = "warning"

[runtime]
enabled = false  # Enable for AI-heavy systems
trace_endpoint = "http://localhost:4318"
retention_days = 30

[report]
format = "text"
include_recommendations = true
max_recommendations = 10
```

---

## Appendix D: Related Work

### Academic References

1. **Tree Decomposition / Treewidth**
   - Bodlaender, H. L. (1994). "A tourist guide through treewidth"
   - Applied to software dependency graphs for maintainability prediction

2. **Semantic Coupling**
   - Poshyvanyk, D. et al. (2007). "Feature Location using Latent Semantic Indexing"
   - Applied IR techniques to concept location

3. **Bounded Context Detection**
   - Evans, E. (2003). "Domain-Driven Design"
   - Newman, S. (2021). "Building Microservices" (2nd ed.)

4. **Architecture Drift**
   - Perry, D. E. & Wolf, A. L. (1992). "Foundations for the Study of Software Architecture"
   - Incremental drift detection approaches

### Tools Comparison

| Feature | Sruja | ArchUnit | Structure101 | Sonargraph |
|---------|-------|----------|--------------|------------|
| Structural analysis | ✅ | ✅ | ✅ | ✅ |
| Treewidth analysis | 🆕 | ❌ | ❌ | ❌ |
| Semantic coupling | 🆕 | ❌ | ❌ | ❌ |
| Bounded context detection | 🆕 | ❌ | ❌ | ❌ |
| ADR alignment | 🆕 | ❌ | ❌ | ❌ |
| Runtime analysis | 🆕 | ❌ | ❌ | ❌ |
| Zero-key first value | ✅ | ❌ | ❌ | ❌ |
| AI-era focus | ✅ | ❌ | ❌ | ❌ |

---

## Document History

| Date | Change |
|------|--------|
| 2026-02-23 | Initial draft |
| 2026-02-23 | Added terminology (tree-sitter vs tree decomposition); implementation nuances; Program→Graph prerequisite; Phase 0 (adoption); short-term improvements; Appendix A (2-developer split) |

**Version:** 1.1.0  
**Last Updated:** 2026-02-23  
**Authors:** Sruja Team  
**Status:** Draft - Pending Review

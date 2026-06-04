# Context Graph Improvement Plan

**Goal:** Evolve Sruja from a tool that produces context *artifacts* into a coherent *context graph engine* where everything — architecture declarations, discovered structure, decisions, agent learnings, and temporal changes — is one traversable, accumulating graph.

**Motivation:** Postman's "context graph" concept reveals that Sruja's current approach fragments context across 6+ separate artifacts (`repo.sruja`, `graph.json`, `context.json`, `agent_memory.json`, `context_events.jsonl`, `.sruja/decisions/`). Agents must know which artifact to query and manually correlate results. The graph should be the unifying abstraction.

---

## Gap Analysis Summary

| # | Gap | Current State | Target State |
|---|-----|--------------|--------------|
| G1 | Unified graph abstraction | 6 separate artifacts, loosely connected | One graph with multiple serialization views |
| G2 | Temporal dimension | Point-in-time snapshots, overwritten each sync | Delta-based snapshots, trend tracking, temporal queries |
| G3 | Progressive density | Full setup required for value | Useful at every density level (sparse → dense) |
| G4 | Graph-native queries | Command-based MCP tools, manual correlation | Graph traversal queries, single-hop and multi-hop |
| G5 | Agent contribution | Learnings in separate `agent_memory.json` | Learnings as graph annotations, traversable |
| G6 | Context debt metric | `context-score` exists but buried | Prominent, citable, temporal metric |
| G7 | Cross-repo federation | Manual `sruja ai-context -r repoA -r repoB` | Declarative repo manifest, automatic propagation |

---

## Phase 1: Graph Unification (Weeks 1-3)

**Goal:** Make the KnowledgeGraph the single source of truth that all artifacts derive from, rather than separate stores that happen to coexist.

### 1.1 Integrate Agent Learnings into KnowledgeGraph

**Problem:** `agent_memory.json` is a separate store. Learnings are matched by `affected_elements` text search, not graph traversal. An agent can't ask "what learnings exist for this node and its neighbors?"

**Changes:**

**`crates/sruja-graph/src/graph.rs`** — Add learnings to KnowledgeGraph:
```rust
pub struct KnowledgeGraph {
    // ... existing fields ...
    pub learnings: HashMap<String, LearningEntry>,  // NEW: keyed by learning ID
}
```

Add methods:
- `add_learning(entry: LearningEntry)` — inserts and auto-links to affected nodes
- `get_learnings_for_node(node_id: &str) -> Vec<&LearningEntry>` — direct matches
- `get_learnings_for_cluster(node_ids: &[String]) -> Vec<&LearningEntry>` — cluster matches with dedup
- `get_learning_neighbors(learning_id: &str) -> Vec<&LearningEntry>` — traverse `related_ids`

**`crates/sruja-agent/src/memory/types.rs`** — Make `LearningEntry` implement `ContextNode` trait:
- `id()` → `self.id`
- `kind()` → `"learning"` (new NodeKind constant)
- `label()` → first 80 chars of `guardrail_advice`

This makes learnings queryable through the same interface as architecture elements.

**`crates/sruja-cli/src/graph_store.rs`** — In `build_and_save_graph()`:
- After loading `KnowledgeGraph`, load `agent_memory.json` and merge learnings into graph
- On save, persist learnings back to both `kg.json` and `agent_memory.json` (backward compatibility)

**`crates/sruja-cli/src/commands/mcp/run_tool/governance.rs`** — Update `sruja_record_learning`:
- After writing to `agent_memory.json`, also update the in-memory KnowledgeGraph cache
- Return the learning ID so agents can reference it

**Validation:** `sruja agent history` shows learnings with graph context (which nodes they affect, which decisions they relate to).

### 1.2 Integrate Decision Records as Graph Nodes

**Problem:** Decision Records live in `.sruja/decisions/*.md` files and are loaded into `KnowledgeGraph.decisions` during `build_and_save_graph()`. But they're not connected to the graph topology — you can't ask "what decisions affect this node's neighborhood?"

**Changes:**

**`crates/sruja-graph/src/graph.rs`** — Add methods:
- `get_decisions_for_node(node_id: &str) -> Vec<&Decision>` — matches `Decision.affects`
- `get_decisions_for_blast_radius(target_id: &str) -> Vec<&Decision>` — all decisions affecting nodes in the blast radius
- `get_decision_chain(decision_id: &str) -> Vec<&Decision>` — traverse `supersedes` links

**`crates/sruja-cli/src/commands/focus.rs`** — In focus briefing generation:
- After computing blast radius, also surface linked decisions and learnings
- Include decision status (accepted/proposed) so agents know which are binding

**Validation:** `sruja focus --file src/api.rs` returns decisions and learnings that affect the blast radius, not just the target node.

### 1.3 Unify Context Events into Graph

**Problem:** `context_events.jsonl` is an append-only log separate from the graph. Events reference element IDs but the graph doesn't know about events.

**Changes:**

**`crates/sruja-graph/src/graph.rs`** — Add temporal event tracking:
```rust
pub struct KnowledgeGraph {
    // ... existing fields ...
    pub recent_events: Vec<ContextEventSummary>,  // last N events, compact
}

pub struct ContextEventSummary {
    pub timestamp: DateTime<Utc>,
    pub kind: String,
    pub elements: Vec<String>,
    pub outcome: String,
    pub summary: Option<String>,
}
```

**`crates/sruja-cli/src/commands/context_events.rs`** — Add `summarize_recent_events()`:
- Read last 50 events from `context_events.jsonl`
- Compact into `ContextEventSummary` (drop full details, keep kind/elements/outcome/summary)
- Return for graph embedding

**`crates/sruja-cli/src/graph_store.rs`** — In `build_and_save_graph()`:
- After loading graph, embed recent event summaries
- Events older than 30 days are dropped from the embedded set (full log remains in JSONL)

**Validation:** `sruja drift` output includes "recent activity" section showing what changed recently for the affected elements.

### 1.4 Single Graph Query API

**Problem:** MCP tools are command-based (`sruja_get_topology`, `sruja_get_elements`, `sruja_get_focus_briefing`). Each returns a different view. Agents must call multiple tools and correlate.

**Changes:**

**New MCP tool: `sruja_query_graph`** (expand existing stub in `definitions.rs`):

Input:
```json
{
  "path": ".",
  "start": "MySystem.Api",           // starting element
  "traverse": ["depends_on", "calls"], // edge kinds to follow
  "depth": 2,                          // max hops
  "include": ["decisions", "learnings", "events"], // what to attach
  "filters": {
    "node_kind": ["component", "database"],
    "decision_status": ["accepted"]
  }
}
```

Output:
```json
{
  "nodes": [...],
  "edges": [...],
  "decisions": [...],
  "learnings": [...],
  "events": [...],
  "token_estimate": 2400
}
```

This replaces the need to call `sruja_get_topology` + `sruja_get_elements` + `sruja_get_context_events` + manual correlation.

**Implementation in `crates/sruja-cli/src/commands/mcp/run_tool/graph.rs`:**
- Use existing `KnowledgeGraph` traversal (`get_edges_from`, `get_edges_to`)
- BFS up to `depth` hops, collecting nodes and edges
- Attach decisions, learnings, and events for collected nodes
- Apply filters
- Estimate tokens (existing `estimate_tokens` function)

**Validation:** A single MCP call returns a coherent subgraph with all context dimensions.

---

## Phase 2: Temporal Graph (Weeks 4-6)

**Goal:** Give the graph memory — track how it changes over time, detect trends, answer "when did this change?"

### 2.1 Graph Snapshots (Delta-Based)

**Problem:** Each `sruja sync` overwrites `kg.json`. No history of what the graph looked like before.

**Changes:**

**New file: `crates/sruja-graph/src/snapshot.rs`**

Define a graph delta format:
```rust
pub struct GraphSnapshot {
    pub timestamp: DateTime<Utc>,
    pub commit_sha: String,
    pub deltas: Vec<GraphDelta>,
}

pub enum GraphDelta {
    NodeAdded { node_id: String, kind: String, label: String },
    NodeRemoved { node_id: String },
    NodeChanged { node_id: String, field: String, old: String, new: String },
    EdgeAdded { source: String, target: String, kind: String },
    EdgeRemoved { source: String, target: String, kind: String },
    DecisionAdded { decision_id: String, title: String },
    DecisionStatusChanged { decision_id: String, old: String, new: String },
    LearningAdded { learning_id: String, affected_elements: Vec<String> },
}
```

**`crates/sruja-cli/src/graph_store.rs`** — In `build_and_save_graph()`:
- Before overwriting `kg.json`, load the previous graph (if exists)
- Compute deltas between old and new graph
- Append delta to `.sruja/graph_snapshots.jsonl` (new file)
- Limit stored snapshots to last 100 (configurable)

**Delta computation:**
- Node added: present in new, absent in old
- Node removed: present in old, absent in new
- Node changed: present in both, any field differs (compute field-level diff)
- Edge added/removed: same logic on (source, target, kind) tuples
- Decision/learning changes: compare by ID

**Cost:** Delta computation is O(n) where n = max(old_nodes, new_nodes). For typical repos (1000-5000 nodes), this is <100ms.

**Validation:** After two `sruja sync` runs, `.sruja/graph_snapshots.jsonl` contains two snapshot records with deltas.

### 2.2 Temporal Queries

**Problem:** Can't answer "what changed in the last 30 days?" or "when did this dependency appear?"

**Changes:**

**New CLI command: `sruja graph history`**

```
sruja graph history -r . --since 30d                    # all changes in last 30 days
sruja graph history -r . --element MySystem.Api         # changes affecting this element
sruja graph history -r . --kind edge_added              # filter by delta type
sruja graph history -r . --since 2025-01-01 --until 2025-03-01  # date range
```

**Implementation in `crates/sruja-cli/src/commands/graph_history.rs`:**
- Read `.sruja/graph_snapshots.jsonl`
- Filter by timestamp range and element/kind
- Group by commit_sha for "change sets"
- Output as table or JSON

**New MCP tool: `sruja_get_graph_history`**

Same filters as CLI, returns JSON array of change sets.

**Validation:** `sruja graph history -r . --element MySystem.Api --since 30d` shows when edges to/from this node were added or removed.

### 2.3 Drift Velocity (Trend Tracking)

**Problem:** `sruja drift` tells you the current state. Can't tell you if things are getting better or worse.

**Changes:**

**New file: `crates/sruja-cli/src/commands/drift_velocity.rs`**

Compute from graph snapshots:
```rust
pub struct DriftVelocity {
    pub period: String,              // "7d", "30d", "90d"
    pub node_count_delta: i64,       // net new/removed nodes
    pub edge_count_delta: i64,       // net new/removed edges
    pub violation_count_delta: i64,  // net new/removed violations
    pub complexity_delta: f64,       // change in graph density (edges/nodes)
    pub trend: TrendDirection,       // Improving / Stable / Degrading
}

pub enum TrendDirection {
    Improving,  // violations decreasing, complexity stable
    Stable,     // no significant change
    Degrading,  // violations increasing or complexity growing faster than nodes
}
```

**Integration with `sruja drift` output:**
- After showing current violations, show velocity summary
- "In the last 7 days: +3 nodes, +5 edges, +2 violations. Trend: Degrading."

**Integration with `sruja context-score`:**
- Add temporal dimension: "Context score: 72 (was 68 seven days ago, +4)"

**Validation:** `sruja drift -r .` includes a velocity section showing trend direction.

### 2.4 Context Score Time Series

**Problem:** `sruja context-score` is a single number. No way to track improvement over time.

**Changes:**

**Extend `.sruja/context.json` schema:**
```json
{
  "schema_version": 2,
  "updated_at": "...",
  "context_score": {
    "current": 72,
    "history": [
      { "date": "2025-01-01", "score": 65, "commit": "abc1234" },
      { "date": "2025-01-08", "score": 68, "commit": "def5678" },
      { "date": "2025-01-15", "score": 72, "commit": "ghi9012" }
    ]
  }
}
```

**`crates/sruja-cli/src/commands/sync_cmd.rs`** — In `sync()`:
- After computing context_score, append to history array
- Keep last 52 entries (1 year of weekly scores)

**`sruja context-score` output:**
```
Context Score: 72/100
  Architecture Coverage:  85/100
  Decision Completeness:  60/100
  Evidence Freshness:     90/100
  Relationship Density:   55/100
  External Context:       70/100

  Trend: +4 over 30 days (was 68)
  History: 65 → 68 → 72 (last 3 weeks)
```

**Validation:** `sruja context-score` shows trend line.

---

## Phase 3: Progressive Density (Weeks 7-9)

**Goal:** Make the context graph useful at every density level — from zero-setup to fully authored.

### 3.1 Density Tiers (Explicit)

**Problem:** Users don't know what they're getting at each level of investment. The progression from "just installed" to "fully authored" isn't clear.

**Changes:**

**Define density tiers in code and docs:**

| Tier | Name | Setup | What You Get |
|------|------|-------|-------------|
| 0 | Sparse | `sruja start -r .` | File/module graph from scan. Structural drift (cycles, orphans). `sruja focus` with scan-only context. |
| 1 | Medium | `sruja sync -r .` | Enriched graph with boundary inference, centrality, communities. `context-score`. Author evidence. |
| 2 | Dense | Author `repo.sruja` | Declared intent, drift vs intent, policy enforcement. Decisions linked to elements. |
| 3 | Rich | Full adoption | Agent learnings, decision records, temporal tracking, PR integration. |

**New CLI command: `sruja density`**
```
sruja density -r .

Current Density: Tier 1 (Medium)
  ✅ Code scan: 847 nodes, 2,103 edges
  ✅ Boundary inference: 12 communities detected
  ❌ No repo.sruja (declared intent)
  ❌ No decision records
  ❌ No agent learnings

Next step: Author repo.sruja to reach Tier 2.
  Run: sruja sync -r .  (generates author_evidence.json)
  Then: Use sruja-architecture skill in your editor
```

**Implementation:**
- Check for existence of `repo.sruja`, `agent_memory.json`, `.sruja/decisions/`, graph snapshots
- Compute tier based on what's present
- Suggest next step

**Validation:** `sruja density` shows current tier and actionable next step.

### 3.2 Zero-Setup Value (Sparse Tier Enhancement)

**Problem:** Even at Tier 0, Sruja should provide more value. Currently `sruja start` gives a scan summary but not an immediately useful graph.

**Changes:**

**Enhance `sruja start` output:**
- Show top 10 most central modules (by edge count)
- Show detected entrypoints (no incoming edges)
- Show data stores (databases, queues)
- Show potential issues (cycles, god modules)
- Show "quick wins" — high-centrality modules with no description

**New MCP tool: `sruja_get_quick_context`**
- Returns a compact (under 500 tokens) summary of the repo's architecture
- Available immediately after first scan, no setup required
- Includes: primary language, framework, top modules, entrypoints, data stores, issues

**Validation:** After `sruja start -r .` + MCP registration, an agent can immediately ask "what's in this repo?" and get a useful answer.

### 3.3 Auto-Activate Existing Context

**Problem:** Sruja discovers code structure but doesn't automatically incorporate existing context sources (OpenAPI specs, Docker Compose, CI configs, README).

**Changes:**

**Extend scan pipeline (`crates/sruja-scan/src/tree_sitter.rs`):**

Currently the scan already discovers:
- `package.json`, `Cargo.toml` (manifests)
- OpenAPI specs
- Docker/Kubernetes manifests

**Add auto-discovery for:**
- `docker-compose*.yml` — service topology (which services talk to which)
- `.github/workflows/*.yml` — CI pipeline structure
- `terraform/*.tf` or `infra/` — infrastructure dependencies
- `README.md` — extract architecture description section
- `docs/architecture*` or `docs/adr*` — architecture docs
- `.env.example` — service connection strings (which URLs are referenced)

**Store as `auto_context` in KnowledgeGraph metadata:**
```rust
pub struct GraphMetadata {
    // ... existing fields ...
    pub auto_context: AutoContext,
}

pub struct AutoContext {
    pub services_from_compose: Vec<String>,
    pub ci_pipelines: Vec<String>,
    pub infra_dependencies: Vec<String>,
    pub readme_summary: Option<String>,
    pub referenced_urls: Vec<String>,
}
```

**Validation:** `sruja sync -r .` on a repo with `docker-compose.yml` shows service topology in the discovered graph.

### 3.4 Density Progression Prompts

**Problem:** Users don't know what to do next to get more value from Sruja.

**Changes:**

**Integrate density hints into existing commands:**

- `sruja drift` — If no `repo.sruja`: "Tip: Author repo.sruja to get intent-based drift detection. Run: sruja sync -r ."
- `sruja focus` — If no agent learnings: "Tip: Record learnings when Sruja catches issues. Run: sruja agent record -r . -c '...' -H '...' -o failed -g '...'"
- `sruja status` — Show density tier and next step
- `sruja daily` — Show density progression since last run

**Validation:** Every Sruja command includes a contextual hint for how to get more value, based on current density tier.

---

## Phase 4: Federation as Graph Linking (Weeks 10-12)

**Problem:** Cross-repo context requires manual `sruja ai-context -r repoA -r repoB`. No persistent model of inter-repo relationships.

### 4.1 Repo Manifest Format

**New file: `docs/REPO_MANIFEST_SPEC.md`**

Define a manifest format for declaring inter-repo relationships:

```toml
# .sruja/repos.toml (or project-root-level)

[repos]
api-gateway = { path = "../api-gateway", repo_id = "api-gateway" }
user-service = { path = "../user-service", repo_id = "user-service" }
shared-lib = { path = "../shared-lib", repo_id = "shared-lib" }

[edges]
# Explicit cross-repo dependencies
api-gateway -> user-service { kind = "calls", label = "REST API" }
user-service -> shared-lib { kind = "depends_on", label = "shared models" }
```

**Changes to `crates/sruja-cli/src/commands/federation.rs`:**
- `publish()` — Also write `repo_id` and path to manifest
- `compose()` — Read manifest instead of requiring explicit bundle paths
- Auto-discover sibling repos by walking parent directory

**Validation:** `sruja compose -r .` reads `.sruja/repos.toml` and produces `system.index.json` without manual bundle paths.

### 4.2 Scoped Cross-Repo Drift

**Problem:** Can't answer "does this PR break any downstream repos?"

**Changes:**

**New CLI command: `sruja drift --cross-repo`**
```
sruja drift -r . --cross-repo
```

- Load `system.index.json` or `.sruja/repos.toml`
- For each cross-repo edge where this repo is the source:
  - Check if the source element still exists
  - Check if the edge kind still applies
  - Report: "user-service:23 depends on api-gateway:UserService. This PR removes UserService — downstream will break."

**Implementation:**
- Reuse existing `drift` logic but extend to cross-repo edges
- Compare current graph against `system.index.json` cross-repo edges
- Report violations with severity (breaking vs warning)

**Validation:** `sruja drift -r . --cross-repo` reports when a PR breaks cross-repo dependencies.

### 4.3 Shared Schema Imports

**Problem:** Each repo defines its own architecture schema independently. No way to share types, boundaries, or policies across repos.

**Changes:**

**Extend `repo.sruja` syntax:**
```sruja
import { boundary, policy } from "../shared-arch/org-standards.sruja"

MySystem = system "My System" {
  // ... uses imported boundary definitions
}
```

**Implementation:**
- Extend parser to resolve `import` statements from relative paths
- Merge imported definitions into local schema
- Validate that local declarations comply with imported policies

**Validation:** Multiple repos can import shared boundary definitions from a central file.

---

## Phase 5: Agent-Native Graph (Weeks 13-15)

**Goal:** Make the context graph a first-class participant in agent workflows, not just a passive data source.

### 5.1 Graph-Aware Focus Briefings

**Current:** `sruja focus` computes blast radius, surfaces learnings and decisions separately.

**Enhanced:** Focus briefings should be graph-native:

```
Focus: src/api/payment.rs

Graph Context:
  Node: MySystem.PaymentService (component, Rust)
  Neighbors: [MySystem.Database, MySystem.Queue, MySystem.ExternalStripe]
  Blast radius: 4 upstream, 3 downstream

Linked Decisions:
  DR-2025-003: Use idempotency keys for payment retries (accepted)
    Affects: MySystem.PaymentService, MySystem.Queue

Agent Learnings:
  learn_2025_001: "Don't use synchronous Stripe calls in the retry path"
    Kind: guardrail | Confidence: high | Retrieved: 5 times | Utility: 80%

Recent Events:
  2025-01-10: drift detected (new dependency on MySystem.Cache)
  2025-01-08: intent check passed
```

**Changes:**
- `crates/sruja-cli/src/commands/focus.rs` — Restructure output to be graph-centric
- Group information by graph relationship, not by artifact type
- Show one coherent narrative, not six separate sections

### 5.2 Graph Contribution Protocol

**Problem:** Agents discover knowledge during tasks but the flow back to the graph is ad-hoc.

**Changes:**

**Define a contribution protocol:**

When an agent completes a task, it can contribute:

1. **Learnings** (existing) — "what worked/failed"
2. **Observations** (new) — "I noticed X about this node" (lower barrier than learnings)
3. **Relationships** (new) — "I discovered that A depends on B" (agent-discovered edges)

**New NodeKind: `observation`**
```rust
pub struct Observation {
    pub id: String,
    pub node_id: String,        // which node this is about
    pub agent_id: String,       // which agent made this
    pub run_id: String,
    pub observation: String,    // free text
    pub confidence: f64,        // 0.0-1.0
    pub timestamp: DateTime<Utc>,
}
```

**New MCP tool: `sruja_contribute_observation`**
```json
{
  "path": ".",
  "node_id": "MySystem.Api",
  "observation": "This module has 3 different error handling patterns. The Result<T, ApiError> pattern is used in 80% of functions.",
  "confidence": 0.8
}
```

Observations are stored in the KnowledgeGraph alongside learnings but with lower trust (they're not validated). They become candidates for promotion to learnings or decisions.

**Validation:** After an agent task, `sruja focus --file src/api.rs` includes observations from previous agents.

### 5.3 Graph Context for Agent Runs

**Problem:** `sruja agent run` consumes focus briefings but doesn't provide graph-aware verification.

**Changes:**

**Enhanced verification in `agent_run.rs`:**

After an agent completes changes:
1. Re-scan affected files
2. Compare new graph against pre-task graph
3. Report graph-level changes:
   - "New edge added: PaymentService → Cache (was not in original graph)"
   - "Node removed: UserService.Helper (was in blast radius)"
   - "New cycle detected: PaymentService → Queue → PaymentService"

**New verification step: `graph_integrity`**
- Checks that post-task graph doesn't violate declared boundaries
- Checks that no new cycles were introduced in the blast radius
- Checks that cross-repo edges are still valid

**Validation:** `sruja agent run --mode apply` includes graph-level verification in its report.

---

## Phase 6: Context Debt Metric (Weeks 16-17)

**Goal:** Make "context debt" a measurable, citable metric that teams can track and act on.

### 6.1 Context Debt Score

**Problem:** `context-score` measures AI-readiness. Context debt is broader — it's about the total cognitive load of understanding a system.

**Changes:**

**New metric: Context Debt Score (0-100, lower is better)**

Components:
1. **Graph Density** (edges/nodes ratio) — Higher density = more relationships to understand
2. **Orphan Ratio** — Nodes with no connections = undocumented modules
3. **Cycle Count** — Circular dependencies increase cognitive load
4. **Undocumented Ratio** — Nodes without descriptions or decisions
5. **Decision Gap** — Edges without linked decisions = unexplained relationships
6. **Learning Gap** — High-centrality nodes without agent learnings = untested assumptions
7. **Freshness** — Age of last scan/decision/learning

Formula:
```
context_debt = (
  graph_density_penalty * 0.2 +
  orphan_ratio * 0.15 +
  cycle_penalty * 0.15 +
  undocumented_ratio * 0.2 +
  decision_gap * 0.15 +
  learning_gap * 0.1 +
  staleness_penalty * 0.05
) * 100
```

**New CLI command: `sruja context-debt`**
```
sruja context-debt -r .

Context Debt: 34/100 (Moderate)

Breakdown:
  Graph Density:    12  (2,103 edges / 847 nodes = 2.48 ratio)
  Orphan Ratio:     8   (67 modules with no connections)
  Cycle Count:      15  (3 cycles detected)
  Undocumented:     25  (212 nodes without descriptions)
  Decision Gap:     18  (378 edges without linked decisions)
  Learning Gap:     10  (12 high-centrality nodes without learnings)
  Staleness:         5  (last sync 3 days ago)

Top Contributors:
  1. src/utils/helpers.rs — high centrality, no description, no decisions
  2. src/api/routes.rs — 47 outgoing edges, 3 cycles
  3. src/db/queries.rs — no description, no learnings, referenced by 23 modules

Run `sruja focus --file src/utils/helpers.rs` to address the top contributor.
```

**Implementation in new crate: `crates/sruja-context-debt/`**
- Pure computation from KnowledgeGraph
- No I/O, no side effects
- Testable with synthetic graphs

**Validation:** `sruja context-debt` produces a score and actionable breakdown.

### 6.2 Temporal Context Debt Tracking

**Changes:**

**Extend `.sruja/context.json` with debt history:**
```json
{
  "context_debt": {
    "current": 34,
    "history": [
      { "date": "2025-01-01", "score": 42, "commit": "abc1234" },
      { "date": "2025-01-08", "score": 38, "commit": "def5678" },
      { "date": "2025-01-15", "score": 34, "commit": "ghi9012" }
    ]
  }
}
```

**Integration with `sruja daily`:**
- Show context debt trend alongside context score trend
- "Context Debt: 34 (was 42 two weeks ago, -8). Trend: Improving."

**Validation:** `sruja daily` shows context debt trend.

### 6.3 Context Debt in CI

**Changes:**

**New CI gate: `sruja context-debt --check`**
```
sruja context-debt -r . --check --max 40
```

- Computes current debt score
- If score > max, exit with error
- If score increased by >5 from baseline, exit with warning

**GitHub Actions integration:**
```yaml
- name: Context Debt Check
  run: |
    sruja context-debt -r . --check --max 40
```

**Validation:** PR fails if context debt exceeds threshold.

---

## Implementation Order and Dependencies

```
Phase 1 (Graph Unification)
  1.1 Learnings into KG ─────────┐
  1.2 Decisions as graph nodes ──┤
  1.3 Events into graph ─────────┼──→ 1.4 Single Graph Query API
  │                              │
  ▼                              │
Phase 2 (Temporal Graph)         │
  2.1 Graph snapshots ───────────┤
  2.2 Temporal queries ──────────┼──→ 2.3 Drift velocity
  2.4 Score time series ─────────┘
  │
  ▼
Phase 3 (Progressive Density)
  3.1 Density tiers ─────────────┐
  3.2 Zero-setup value ──────────┤
  3.3 Auto-activate context ─────┼──→ 3.4 Density prompts
  │                              │
  ▼                              │
Phase 4 (Federation)             │
  4.1 Repo manifest ─────────────┤
  4.2 Cross-repo drift ──────────┼──→ 4.3 Shared schema imports
  │                              │
  ▼                              │
Phase 5 (Agent-Native Graph)     │
  5.1 Graph-aware focus ─────────┤
  5.2 Contribution protocol ─────┼──→ 5.3 Graph verification
  │                              │
  ▼                              │
Phase 6 (Context Debt Metric)
  6.1 Debt score ────────────────┐
  6.2 Temporal tracking ─────────┼──→ 6.3 CI integration
```

**Critical path:** Phase 1 → Phase 2 → Phase 6 (debt metric needs temporal + unified graph)
**Parallel track:** Phase 3 can proceed in parallel with Phase 2
**Phase 4 and 5 depend on Phase 1** (need unified graph for cross-repo and agent integration)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Graph snapshot storage grows unbounded | Disk usage | Cap at 100 snapshots, delta-based (small) |
| Delta computation slows `sruja sync` | User experience | O(n) comparison, benchmark with 5000-node graph |
| Learning integration pollutes graph with noise | Graph quality | Only surface learnings above confidence threshold |
| Context debt formula is arbitrary | Credibility | Make weights configurable, publish methodology |
| Cross-repo manifest requires manual setup | Adoption | Auto-discover sibling repos, provide `sruja init --federation` |
| New MCP tools increase agent confusion | Usability | Keep progressive disclosure ladder, deprecate old tools slowly |

---

## Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| MCP calls needed for full task context | 3-5 | 1-2 |
| Time to first useful output (new repo) | 30s (scan) | 10s (quick context) |
| Cross-repo context availability | Manual, ~5min | Automatic, <10s |
| Context debt tracking | None | Weekly trend in `sruja daily` |
| Agent learning retrieval in focus | Separate section | Integrated in graph narrative |
| Graph temporal queries | Not possible | <1s for 30-day history |

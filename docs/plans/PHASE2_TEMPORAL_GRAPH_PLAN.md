# Phase 2: Temporal Graph - Implementation Plan

**Goal:** Give the graph memory — track how it changes over time, detect trends, answer "when did this change?"

**Prerequisites:** Phase 1 complete (learnings, decisions, events integrated into KnowledgeGraph)

---

## Task 2.1: Graph Snapshots (Delta-Based)

### Files to Create
- `crates/sruja-graph/src/snapshot.rs`

### Files to Modify
- `crates/sruja-graph/src/lib.rs` (add `pub mod snapshot`)
- `crates/sruja-cli/src/graph_store.rs` (compute deltas before save)

### Implementation

**1. Create `snapshot.rs` with delta types:**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub timestamp: DateTime<Utc>,
    pub commit_sha: String,
    pub deltas: Vec<GraphDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

**2. Add `compute_deltas` function:**

```rust
pub fn compute_deltas(old: &KnowledgeGraph, new: &KnowledgeGraph) -> Vec<GraphDelta> {
    let mut deltas = Vec::new();
    
    // Nodes added (in new, not in old)
    for (id, node) in &new.nodes {
        if !old.nodes.contains_key(id) {
            deltas.push(GraphDelta::NodeAdded {
                node_id: id.clone(),
                kind: node.kind.kind_str().to_string(),
                label: node.label.clone(),
            });
        }
    }
    
    // Nodes removed (in old, not in new)
    for id in old.nodes.keys() {
        if !new.nodes.contains_key(id) {
            deltas.push(GraphDelta::NodeRemoved { node_id: id.clone() });
        }
    }
    
    // Nodes changed (field-level diff)
    for (id, old_node) in &old.nodes {
        if let Some(new_node) = new.nodes.get(id) {
            if old_node.label != new_node.label {
                deltas.push(GraphDelta::NodeChanged {
                    node_id: id.clone(),
                    field: "label".to_string(),
                    old: old_node.label.clone(),
                    new: new_node.label.clone(),
                });
            }
            // Compare other fields: description, technology, etc.
        }
    }
    
    // Edges added/removed (compare by source+target+kind tuple)
    let old_edges: std::collections::HashSet<_> = old.edges.iter()
        .map(|e| (&e.source, &e.target, e.kind.kind_str()))
        .collect();
    let new_edges: std::collections::HashSet<_> = new.edges.iter()
        .map(|e| (&e.source, &e.target, e.kind.kind_str()))
        .collect();
    
    // ... similar logic for decisions and learnings
    
    deltas
}
```

**3. Update `graph_store.rs`:**

In `build_and_save_graph()`, before `save_graph()`:
```rust
// Load previous graph for delta computation
if let Ok(prev_kg) = load_graph_from(&repo.join(GRAPH_FILE)) {
    let deltas = sruja_graph::snapshot::compute_deltas(&prev_kg, &kg);
    if !deltas.is_empty() {
        let snapshot = GraphSnapshot {
            timestamp: Utc::now(),
            commit_sha: commit_sha.clone().unwrap_or_default(),
            deltas,
        };
        append_snapshot(repo, &snapshot)?;
    }
}
```

Add `append_snapshot` function:
```rust
fn append_snapshot(repo: &Path, snapshot: &GraphSnapshot) -> Result<(), CliError> {
    let path = repo.join(".sruja/graph_snapshots.jsonl");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", serde_json::to_string(snapshot)?)?;
    
    // Trim to last 100 snapshots
    trim_snapshots(repo, 100)?;
    Ok(())
}
```

### Validation
- Run `sruja sync` twice with code changes between
- Check `.sruja/graph_snapshots.jsonl` exists with delta records

---

## Task 2.2: Temporal Queries

### Files to Create
- `crates/sruja-cli/src/commands/graph_history.rs`

### Files to Modify
- `crates/sruja-cli/src/commands/mod.rs` (add module)
- `crates/sruja-cli/src/main.rs` (add CLI subcommand)

### Implementation

**1. Create `graph_history.rs`:**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct HistoryQuery {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub element: Option<String>,
    pub kind: Option<String>,
}

pub fn query_history(repo: &Path, query: HistoryQuery) -> Result<Vec<ChangeSet>, CliError> {
    let snapshots = load_snapshots(repo)?;
    
    let filtered: Vec<_> = snapshots.into_iter()
        .filter(|s| {
            if let Some(since) = query.since {
                if s.timestamp < since { return false; }
            }
            if let Some(until) = query.until {
                if s.timestamp > until { return false; }
            }
            true
        })
        .filter(|s| {
            if let Some(ref element) = query.element {
                s.deltas.iter().any(|d| delta_references_element(d, element))
            } else {
                true
            }
        })
        .filter(|s| {
            if let Some(ref kind) = query.kind {
                s.deltas.iter().any(|d| delta_matches_kind(d, kind))
            } else {
                true
            }
        })
        .collect();
    
    Ok(group_into_changesets(filtered))
}
```

**2. Add CLI command:**

```
sruja graph history -r . --since 30d
sruja graph history -r . --element MySystem.Api
sruja graph history -r . --kind edge_added --json
```

**3. Add MCP tool `sruja_get_graph_history`:**

Same filters as CLI, returns JSON array of change sets.

### Validation
- `sruja graph history -r . --since 30d` shows recent changes
- `sruja graph history -r . --element <id>` shows element-specific changes

---

## Task 2.3: Drift Velocity (Trend Tracking)

### Files to Create
- `crates/sruja-cli/src/commands/drift_velocity.rs`

### Files to Modify
- `crates/sruja-cli/src/commands/scan.rs` (integrate velocity into drift output)

### Implementation

**1. Create `drift_velocity.rs`:**

```rust
pub struct DriftVelocity {
    pub period: String,              // "7d", "30d", "90d"
    pub node_count_delta: i64,
    pub edge_count_delta: i64,
    pub violation_count_delta: i64,
    pub complexity_delta: f64,
    pub trend: TrendDirection,
}

pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

pub fn compute_velocity(repo: &Path, period_days: i64) -> Result<DriftVelocity, CliError> {
    let snapshots = load_snapshots(repo)?;
    let cutoff = Utc::now() - chrono::Duration::days(period_days);
    
    let recent: Vec<_> = snapshots.iter()
        .filter(|s| s.timestamp >= cutoff)
        .collect();
    
    // Count net additions/removals
    let mut node_delta: i64 = 0;
    let mut edge_delta: i64 = 0;
    
    for snapshot in &recent {
        for delta in &snapshot.deltas {
            match delta {
                GraphDelta::NodeAdded { .. } => node_delta += 1,
                GraphDelta::NodeRemoved { .. } => node_delta -= 1,
                GraphDelta::EdgeAdded { .. } => edge_delta += 1,
                GraphDelta::EdgeRemoved { .. } => edge_delta -= 1,
                _ => {}
            }
        }
    }
    
    let trend = determine_trend(node_delta, edge_delta, violation_delta);
    
    Ok(DriftVelocity {
        period: format!("{}d", period_days),
        node_count_delta: node_delta,
        edge_count_delta: edge_delta,
        violation_count_delta: violation_delta,
        complexity_delta: compute_complexity_delta(&recent),
        trend,
    })
}
```

**2. Integrate into `sruja drift` output:**

After showing current violations, add:
```
Velocity (7d):
  Nodes: +3
  Edges: +5
  Violations: +2
  Trend: Degrading
```

### Validation
- `sruja drift -r .` includes velocity section

---

## Task 2.4: Context Score Time Series

### Files to Modify
- `crates/sruja-cli/src/commands/sync_cmd.rs` (append score to history)
- `crates/sruja-cli/src/commands/context_score_cmd.rs` (show trend)

### Implementation

**1. Extend `.sruja/context.json` schema:**

```json
{
  "schema_version": 2,
  "context_score": {
    "current": 72,
    "history": [
      { "date": "2025-01-01", "score": 65, "commit": "abc1234" },
      { "date": "2025-01-08", "score": 68, "commit": "def5678" }
    ]
  }
}
```

**2. Update `sync_cmd.rs`:**

After computing context_score:
```rust
let history_entry = ScoreHistoryEntry {
    date: Utc::now(),
    score: context_score.score,
    commit: commit_sha.clone(),
};
append_score_history(repo, history_entry)?;
```

**3. Update context-score output:**

```
Context Score: 72/100
  Trend: +4 over 30 days (was 68)
  History: 65 → 68 → 72 (last 3 weeks)
```

### Validation
- `sruja context-score` shows trend line after multiple syncs

---

## Build & Test Commands

```bash
# Check compilation
cargo check -p sruja-graph -p sruja-cli

# Run tests
cargo test -p sruja-graph -p sruja-cli

# Run specific test
cargo test -p sruja-graph test_snapshot_deltas
```

---

## Dependencies Between Tasks

```
2.1 (Snapshots) ──→ 2.2 (Temporal Queries)
                ──→ 2.3 (Drift Velocity)
2.4 (Score Time Series) - independent, can run in parallel
```

---

## Key Design Decisions

1. **JSONL for snapshots** - Append-only, easy to query, no database needed
2. **100 snapshot limit** - Prevents unbounded growth, configurable
3. **Delta-based** - Small storage footprint, only stores changes
4. **O(n) delta computation** - Fast for typical repos (1000-5000 nodes)

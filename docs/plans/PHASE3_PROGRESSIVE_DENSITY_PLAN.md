# Phase 3: Progressive Density - Implementation Plan

**Goal:** Make the context graph useful at every density level — from zero-setup to fully authored.

**Prerequisites:** Phase 1 complete (unified graph)

---

## Density Tiers Reference

| Tier | Name | Setup | What You Get |
|------|------|-------|-------------|
| 0 | Sparse | `sruja start -r .` | File/module graph from scan. Structural drift (cycles, orphans). `sruja focus` with scan-only context. |
| 1 | Medium | `sruja sync -r .` | Enriched graph with boundary inference, centrality, communities. `context-score`. Author evidence. |
| 2 | Dense | Author `repo.sruja` | Declared intent, drift vs intent, policy enforcement. Decisions linked to elements. |
| 3 | Rich | Full adoption | Agent learnings, decision records, temporal tracking, PR integration. |

---

## Task 3.1: Density Tiers (Explicit)

### Files to Create
- `crates/sruja-cli/src/commands/density.rs`

### Files to Modify
- `crates/sruja-cli/src/commands/mod.rs` (add module)
- `crates/sruja-cli/src/main.rs` (add CLI subcommand)

### Implementation

**1. Create `density.rs`:**

```rust
use std::path::Path;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum DensityTier {
    Sparse = 0,
    Medium = 1,
    Dense = 2,
    Rich = 3,
}

impl std::fmt::Display for DensityTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DensityTier::Sparse => write!(f, "Sparse"),
            DensityTier::Medium => write!(f, "Medium"),
            DensityTier::Dense => write!(f, "Dense"),
            DensityTier::Rich => write!(f, "Rich"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DensityReport {
    pub tier: DensityTier,
    pub tier_name: String,
    pub checks: Vec<DensityCheck>,
    pub next_step: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DensityCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

pub fn compute_density(repo: &Path) -> DensityReport {
    let mut checks = Vec::new();
    
    // Tier 0: Sparse - code scan exists
    let scan_exists = repo.join(".sruja/cache/kg.json").exists();
    checks.push(DensityCheck {
        name: "Code scan".to_string(),
        passed: scan_exists,
        detail: if scan_exists { "Graph exists" } else { "Run `sruja start`" }.to_string(),
    });
    
    // Tier 1: Medium - enriched graph with context score
    let context_json = repo.join(".sruja/context.json");
    let has_context_score = context_json.exists();
    checks.push(DensityCheck {
        name: "Context score".to_string(),
        passed: has_context_score,
        detail: if has_context_score { "Computed" } else { "Run `sruja sync`" }.to_string(),
    });
    
    // Tier 2: Dense - repo.sruja exists
    let repo_sruja = find_repo_sruja(repo);
    checks.push(DensityCheck {
        name: "Declared intent (repo.sruja)".to_string(),
        passed: repo_sruja.is_some(),
        detail: if repo_sruja.is_some() { "Found" } else { "Author repo.sruja" }.to_string(),
    });
    
    // Tier 3: Rich - decisions, learnings, temporal
    let has_decisions = repo.join(".sruja/decisions").is_dir();
    checks.push(DensityCheck {
        name: "Decision records".to_string(),
        passed: has_decisions,
        detail: if has_decisions { "Found" } else { "Create decision records" }.to_string(),
    });
    
    let has_learnings = repo.join(".sruja/agent_memory.json").exists();
    checks.push(DensityCheck {
        name: "Agent learnings".to_string(),
        passed: has_learnings,
        detail: if has_learnings { "Found" } else { "Record learnings via MCP" }.to_string(),
    });
    
    let has_snapshots = repo.join(".sruja/graph_snapshots.jsonl").exists();
    checks.push(DensityCheck {
        name: "Temporal tracking".to_string(),
        passed: has_snapshots,
        detail: if has_snapshots { "Found" } else { "Run `sruja sync` multiple times" }.to_string(),
    });
    
    // Compute tier
    let tier = if has_snapshots && has_learnings && has_decisions && repo_sruja.is_some() {
        DensityTier::Rich
    } else if repo_sruja.is_some() {
        DensityTier::Dense
    } else if has_context_score {
        DensityTier::Medium
    } else {
        DensityTier::Sparse
    };
    
    let next_step = suggest_next_step(tier, &checks);
    
    DensityReport {
        tier,
        tier_name: tier.to_string(),
        checks,
        next_step,
    }
}

fn suggest_next_step(tier: DensityTier, checks: &[DensityCheck]) -> Option<String> {
    match tier {
        DensityTier::Sparse => Some("Run `sruja sync -r .` to reach Tier 1 (Medium).".to_string()),
        DensityTier::Medium => Some("Author repo.sruja to reach Tier 2 (Dense). Run `sruja sync -r .` for author evidence.".to_string()),
        DensityTier::Dense => Some("Record decisions and learnings to reach Tier 3 (Rich).".to_string()),
        DensityTier::Rich => None,
    }
}

fn find_repo_sruja(repo: &Path) -> Option<std::path::PathBuf> {
    for name in &["repo.sruja", "architecture.sruja", "arch.sruja"] {
        let path = repo.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    // Check docs/architecture/
    let arch_dir = repo.join("docs/architecture");
    if arch_dir.is_dir() {
        for entry in std::fs::read_dir(&arch_dir).ok()?.flatten() {
            if entry.path().extension().map(|e| e == "sruja").unwrap_or(false) {
                return Some(entry.path());
            }
        }
    }
    None
}
```

**2. Add CLI command:**

```bash
sruja density -r .
sruja density -r . --json
```

Output:
```
Current Density: Tier 1 (Medium)
  ✅ Code scan: 847 nodes, 2,103 edges
  ✅ Boundary inference: 12 communities detected
  ❌ No repo.sruja (declared intent)
  ❌ No decision records
  ❌ No agent learnings

Next step: Author repo.sruja to reach Tier 2.
  Run: sruja sync -r .  (generates author_evidence.json)
```

### Validation
- `sruja density -r .` shows current tier and next step

---

## Task 3.2: Zero-Setup Value (Sparse Tier Enhancement)

### Files to Modify
- `crates/sruja-cli/src/commands/start.rs` (enhance output)
- `crates/sruja-cli/src/commands/mcp/run_tool/graph.rs` (add `sruja_get_quick_context`)

### Implementation

**1. Enhance `sruja start` output:**

After scan, compute and display:
```rust
fn print_quick_summary(graph: &sruja_scan::Graph) {
    // Top 10 most central modules (by edge count)
    let centrality = sruja_scan::graph::compute_all_centrality(graph);
    let mut top_modules: Vec<_> = centrality.iter().collect();
    top_modules.sort_by(|a, b| b.1.pagerank.partial_cmp(&a.1.pagerank).unwrap());
    
    println!("\nTop 10 Most Central Modules:");
    for (i, (id, score)) in top_modules.iter().take(10).enumerate() {
        println!("  {}. {} (pagerank: {:.3})", i + 1, id, score.pagerank);
    }
    
    // Detected entrypoints (no incoming edges)
    let entrypoints: Vec<_> = graph.nodes.iter()
        .filter(|n| !graph.edges.iter().any(|e| e.target == n.id))
        .collect();
    println!("\nEntrypoints (no incoming edges):");
    for node in entrypoints.iter().take(5) {
        println!("  - {}", node.id);
    }
    
    // Data stores (databases, queues)
    let stores: Vec<_> = graph.nodes.iter()
        .filter(|n| n.kind.kind_str() == "database" || n.kind.kind_str() == "queue")
        .collect();
    println!("\nData Stores:");
    for node in stores.iter().take(5) {
        println!("  - {} ({})", node.id, node.kind.kind_str());
    }
    
    // Potential issues (cycles, god modules)
    let scc = sruja_scan::SccAnalyzer::new(graph);
    let result = scc.analyze();
    if !result.cyclic_sccs.is_empty() {
        println!("\n⚠️  Cycles Detected:");
        for scc in result.cyclic_sccs.iter().take(3) {
            println!("  - {} nodes in cycle", scc.node_ids.len());
        }
    }
}
```

**2. Add `sruja_get_quick_context` MCP tool:**

```rust
"sruja_get_quick_context" => {
    let graph = get_or_scan_graph(graph_cache, repo).await?;
    
    let centrality = sruja_scan::graph::compute_all_centrality(&graph);
    let mut top_modules: Vec<_> = centrality.iter().collect();
    top_modules.sort_by(|a, b| b.1.pagerank.partial_cmp(&a.1.pagerank).unwrap());
    
    let entrypoints: Vec<_> = graph.nodes.iter()
        .filter(|n| !graph.edges.iter().any(|e| e.target == n.id))
        .map(|n| &n.id)
        .collect();
    
    let stores: Vec<_> = graph.nodes.iter()
        .filter(|n| n.kind.kind_str() == "database" || n.kind.kind_str() == "queue")
        .map(|n| json!({ "id": n.id, "kind": n.kind.kind_str() }))
        .collect();
    
    let summary = json!({
        "total_nodes": graph.nodes.len(),
        "total_edges": graph.edges.len(),
        "top_modules": top_modules.iter().take(5).map(|(id, s)| json!({ "id": id, "pagerank": s.pagerank })).collect::<Vec<_>>(),
        "entrypoints": entrypoints,
        "data_stores": stores,
        "token_estimate": 300
    });
    
    finish(Ok(serde_json::to_string_pretty(&summary)?))
}
```

### Validation
- `sruja start -r .` shows enhanced summary
- MCP `sruja_get_quick_context` returns compact repo summary

---

## Task 3.3: Auto-Activate Existing Context

### Files to Modify
- `crates/sruja-scan/src/manifests.rs` (add discovery for new file types)
- `crates/sruja-graph/src/lib.rs` (add `AutoContext` struct)

### Implementation

**1. Add `AutoContext` to graph metadata:**

In `crates/sruja-graph/src/graph.rs`:
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphMetadata {
    // ... existing fields ...
    pub auto_context: AutoContext,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoContext {
    pub services_from_compose: Vec<String>,
    pub ci_pipelines: Vec<String>,
    pub infra_dependencies: Vec<String>,
    pub readme_summary: Option<String>,
    pub referenced_urls: Vec<String>,
}
```

**2. Extend `manifests.rs` to discover new file types:**

```rust
pub fn discover_auto_context(repo_root: &Path) -> AutoContext {
    let mut ctx = AutoContext::default();
    
    // docker-compose*.yml
    for entry in glob::glob(repo_root.join("docker-compose*.yml").to_str().unwrap()).flatten() {
        if let Ok(path) = entry {
            if let Ok(content) = std::fs::read_to_string(&path) {
                ctx.services_from_compose.extend(extract_compose_services(&content));
            }
        }
    }
    
    // .github/workflows/*.yml
    let workflows_dir = repo_root.join(".github/workflows");
    if workflows_dir.is_dir() {
        for entry in std::fs::read_dir(&workflows_dir).flatten() {
            if let Ok(entry) = entry {
                if entry.path().extension().map(|e| e == "yml" || e == "yaml").unwrap_or(false) {
                    ctx.ci_pipelines.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
    }
    
    // terraform/*.tf or infra/
    for dir in &["terraform", "infra", "infrastructure"] {
        let tf_dir = repo_root.join(dir);
        if tf_dir.is_dir() {
            for entry in walkdir::WalkDir::new(&tf_dir).into_iter().flatten() {
                if entry.path().extension().map(|e| e == "tf").unwrap_or(false) {
                    ctx.infra_dependencies.push(entry.path().display().to_string());
                }
            }
        }
    }
    
    // README.md - extract architecture section
    let readme = repo_root.join("README.md");
    if readme.exists() {
        if let Ok(content) = std::fs::read_to_string(&readme) {
            ctx.readme_summary = extract_architecture_section(&content);
        }
    }
    
    // .env.example - extract URLs
    let env_example = repo_root.join(".env.example");
    if env_example.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_example) {
            ctx.referenced_urls = extract_urls_from_env(&content);
        }
    }
    
    ctx
}

fn extract_compose_services(content: &str) -> Vec<String> {
    // Parse YAML and extract service names
    // Simplified: look for top-level keys under 'services:'
    Vec::new() // TODO: implement YAML parsing
}

fn extract_architecture_section(content: &str) -> Option<String> {
    // Find "## Architecture" or "## Overview" section
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;
    let mut section_lines = Vec::new();
    
    for line in &lines {
        if line.to_lowercase().starts_with("## architecture") || line.to_lowercase().starts_with("## overview") {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if in_section {
            section_lines.push(*line);
        }
    }
    
    if section_lines.is_empty() {
        None
    } else {
        Some(section_lines.join("\n"))
    }
}

fn extract_urls_from_env(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for line in content.lines() {
        if let Some(value) = line.split('=').nth(1) {
            let value = value.trim().trim_matches('"');
            if value.starts_with("http://") || value.starts_with("https://") {
                urls.push(value.to_string());
            }
        }
    }
    urls
}
```

**3. Integrate into `build_and_save_graph`:**

```rust
let auto_context = sruja_scan::manifests::discover_auto_context(repo);
kg.metadata.auto_context = auto_context;
```

### Validation
- `sruja sync -r .` on repo with `docker-compose.yml` shows services in metadata
- `sruja sync -r .` on repo with `.github/workflows/` lists CI pipelines

---

## Task 3.4: Density Progression Prompts

### Files to Modify
- `crates/sruja-cli/src/commands/scan.rs` (add hint to drift output)
- `crates/sruja-cli/src/commands/focus.rs` (add hint to focus output)
- `crates/sruja-cli/src/commands/daily.rs` (add density progression)
- `crates/sruja-cli/src/commands/status.rs` (add density tier)

### Implementation

**1. Add density hint helper:**

```rust
pub fn density_hint(repo: &Path) -> Option<String> {
    let density = compute_density(repo);
    match density.tier {
        DensityTier::Sparse => Some("💡 Tip: Run `sruja sync -r .` to enrich the graph and reach Tier 1 (Medium).".to_string()),
        DensityTier::Medium => Some("💡 Tip: Author `repo.sruja` to declare intent and reach Tier 2 (Dense).".to_string()),
        DensityTier::Dense => Some("💡 Tip: Record decisions and learnings to reach Tier 3 (Rich).".to_string()),
        DensityTier::Rich => None,
    }
}
```

**2. Integrate into commands:**

In `sruja drift` output:
```
[existing drift output]

💡 Tip: Author repo.sruja to get intent-based drift detection.
   Run: sruja sync -r .
```

In `sruja focus` output:
```
[existing focus briefing]

💡 Tip: Record learnings when Sruja catches issues.
   Run: sruja agent record -r . -c '...' -H '...' -o failed -g '...'
```

In `sruja status`:
```
Density: Tier 1 (Medium)
Next step: Author repo.sruja to reach Tier 2.
```

In `sruja daily`:
```
[existing daily output]

Density Progression:
  Current: Tier 1 (Medium)
  Since last run: No change
```

### Validation
- `sruja drift -r .` shows density hint
- `sruja focus --file <file>` shows density hint
- `sruja status` shows density tier

---

## Build & Test Commands

```bash
# Check compilation
cargo check -p sruja-graph -p sruja-cli -p sruja-scan

# Run tests
cargo test -p sruja-graph -p sruja-cli -p sruja-scan

# Run specific test
cargo test -p sruja-cli test_density_tier
```

---

## Dependencies Between Tasks

```
3.1 (Density Tiers) ──→ 3.4 (Density Prompts)
3.2 (Zero-Setup Value) - independent
3.3 (Auto-Activate Context) - independent
```

---

## Key Design Decisions

1. **Explicit tiers** - Users know exactly where they stand and what to do next
2. **Actionable next steps** - Every tier transition has a clear command
3. **Non-blocking hints** - Density hints are informational, not errors
4. **Auto-discovery** - Existing context is incorporated without manual setup
5. **Token budget** - Quick context is capped at ~500 tokens for agent consumption

# AI Integration Implementation Checklist

Step-by-step changes to stitch AI components into core CLI workflow.

> **Status (post-refactor):** **sruja-extract** and **sruja-chat** have been removed. The CLI already has quickstart, why, drift, analyze, context (see README and book CLI guide). Use this checklist for further AI stitching; skip steps that reference removed crates.

---

## Step 1: Add Missing Dependencies

**File:** `crates/sruja-cli/Cargo.toml`

```toml
[dependencies]
# ... existing ...
# sruja-extract was removed; LLM extraction lives in sruja-cli/ai if needed
```

---

## Step 2: Create Graph Store Module

**File:** `crates/sruja-cli/src/graph_store.rs` (NEW FILE)

```rust
use sruja_graph::KnowledgeGraph;
use sruja_scan::ScanGraph;
use std::path::Path;
use super::CliError;

const GRAPH_FILE: &str = ".sruja/graph.json";

pub fn load_or_build_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let graph_path = repo.join(GRAPH_FILE);
    
    if graph_path.exists() {
        let json = std::fs::read_to_string(&graph_path)
            .map_err(|e| CliError::Io(e))?;
        let graph: KnowledgeGraph = serde_json::from_str(&json)
            .map_err(|e| CliError::Json(e))?;
        
        // TODO: Check if stale (code changed since last build)
        return Ok(graph);
    }
    
    build_and_save_graph(repo)
}

pub fn build_and_save_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let scan_graph = sruja_scan::scan_repo(repo)
        .map_err(|e| CliError::Validation(e.to_string()))?;
    
    let mut kg = KnowledgeGraph::new();
    sruja_graph::merge_scan_into_graph(&mut kg, &scan_graph, &repo.display().to_string());
    
    save_graph(repo, &kg)?;
    Ok(kg)
}

pub fn save_graph(repo: &Path, graph: &KnowledgeGraph) -> Result<(), CliError> {
    let sruja_dir = repo.join(".sruja");
    std::fs::create_dir_all(&sruja_dir)
        .map_err(|e| CliError::Io(e))?;
    
    let json = serde_json::to_string_pretty(graph)
        .map_err(|e| CliError::Json(e))?;
    
    std::fs::write(sruja_dir.join("graph.json"), json)
        .map_err(|e| CliError::Io(e))?;
    
    Ok(())
}
```

**File:** `crates/sruja-cli/src/main.rs`

Add at top:
```rust
mod graph_store;
```

---

## Step 3: Enhance Quickstart with AI

**File:** `crates/sruja-cli/src/commands/scan.rs`

Find the `quickstart` function and replace with:

```rust
pub async fn quickstart(repo: &str, format: &str) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    if !repo_path.exists() {
        return Err(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository not found: {}", repo),
        )));
    }

    // 1. Structural scan (existing)
    let graph = scan_repo(repo_path).map_err(|e| CliError::Validation(e.to_string()))?;

    // 2. Build knowledge graph (NEW)
    let kg = crate::graph_store::load_or_build_graph(repo_path)?;

    // 3. Semantic analysis (NEW)
    let components: Vec<(String, String)> = graph
        .nodes
        .iter()
        .map(|n| {
            let text = format!(
                "{} {} {}",
                n.label,
                n.technology.as_deref().unwrap_or(""),
                n.path.as_deref().unwrap_or("")
            );
            (n.id.clone(), text)
        })
        .collect();
    let edges: Vec<(String, String)> = graph
        .edges
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();

    let provider = sruja_semantic::embedding::StubEmbeddingProvider::new();
    let semantic_report = sruja_semantic::analyze(&components, &edges, &provider, None)
        .await
        .map_err(|e| CliError::Validation(format!("Semantic analysis failed: {}", e)))?;

    // 4. Calculate health (existing + semantic)
    let health = calculate_health_with_semantic(&graph, &semantic_report);

    // 5. Generate output
    if format == "json" {
        let output = serde_json::json!({
            "inventory": build_inventory(&graph),
            "health_score": health.score,
            "health_status": health.status,
            "findings": health.findings,
            "semantic": {
                "contexts": semantic_report.contexts.len(),
                "hidden_couplings": semantic_report.coupling_report.hidden_couplings.len(),
            },
            "next_steps": build_next_steps(&graph, &semantic_report),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_quickstart_text(&graph, &semantic_report, &health);
    }

    Ok(())
}

fn calculate_health_with_semantic(
    graph: &ScanGraph,
    semantic: &sruja_semantic::SemanticReport,
) -> Health {
    let mut health = calculate_health(graph);  // Existing function
    
    // Deduct for semantic issues
    health.score = health.score.saturating_sub(
        (semantic.coupling_report.hidden_couplings.len() * 2) as u8
    );
    health.score = health.score.saturating_sub(
        (semantic.summary.vocabulary_leak_count * 3) as u8
    );
    
    // Add semantic findings
    for coupling in &semantic.coupling_report.hidden_couplings {
        health.findings.push(Finding {
            severity: "Warning".to_string(),
            message: format!(
                "Hidden coupling: {} ↔ {} (similarity: {:.2})",
                coupling.source, coupling.target, coupling.similarity
            ),
            file: None,
        });
    }
    
    health
}
```

---

## Step 4: Enhance Drift with AI

**File:** `crates/sruja-cli/src/commands/scan.rs`

Find the `drift` function and add semantic analysis:

```rust
pub async fn drift(
    repo: &str,
    architecture: Option<&str>,
    format: &str,
    violations_only: bool,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    
    // 1. Structural scan (existing)
    let scan_graph = scan_repo(repo_path).map_err(|e| CliError::Validation(e.to_string()))?;
    
    // 2. Load/build knowledge graph (NEW)
    let kg = crate::graph_store::load_or_build_graph(repo_path)?;
    
    // 3. Semantic analysis (NEW)
    let components: Vec<(String, String)> = scan_graph
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.label.clone()))
        .collect();
    let edges: Vec<(String, String)> = scan_graph
        .edges
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();
    
    let provider = sruja_semantic::embedding::StubEmbeddingProvider::new();
    let semantic = sruja_semantic::analyze(&components, &edges, &provider, None)
        .await
        .map_err(|e| CliError::Validation(format!("Semantic analysis failed: {}", e)))?;
    
    // 4. Detect violations (existing + semantic)
    let mut violations = detect_violations(&scan_graph, architecture.as_deref())?;
    
    // Add semantic violations
    for coupling in &semantic.coupling_report.hidden_couplings {
        if coupling.similarity > 0.85 {
            violations.push(Violation {
                severity: "Warning".to_string(),
                kind: "HiddenCoupling".to_string(),
                message: format!(
                    "High semantic coupling between {} and {} suggests missing dependency",
                    coupling.source, coupling.target
                ),
                source: coupling.source.clone(),
                target: Some(coupling.target.clone()),
            });
        }
    }
    
    // 5. Output
    if format == "json" {
        let output = serde_json::json!({
            "violations": violations,
            "semantic_summary": {
                "hidden_couplings": semantic.coupling_report.hidden_couplings.len(),
                "vocabulary_leaks": semantic.summary.vocabulary_leak_count,
            },
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        print_drift_text(&violations, &semantic, violations_only);
    }
    
    // Exit with error if violations
    if violations.iter().any(|v| v.severity == "Error") {
        std::process::exit(1);
    }
    
    Ok(())
}
```

---

## Step 5: Enhance Analyze with Knowledge Graph

**File:** `crates/sruja-cli/src/commands/analyze.rs`

Modify the `analyze` function:

```rust
pub async fn analyze(
    repo_root: &str,
    traces: Option<&str>,
    intent: Option<&str>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = Path::new(repo_root);
    
    // Existing: structural + semantic + intent
    let graph = scan_repo(repo_path)?;
    let components: Vec<(String, String)> = graph
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.label.clone()))
        .collect();
    let edges: Vec<(String, String)> = graph
        .edges
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();
    
    let provider = sruja_semantic::embedding::StubEmbeddingProvider::new();
    let semantic = run_semantic_analyze(&components, &edges, &provider, None).await?;
    
    let intent_model = if let Some(intent_dir) = intent {
        Some(IntentModel::from_dir(Path::new(intent_dir))?)
    } else {
        None
    };
    
    // NEW: Load/build knowledge graph
    let kg = crate::graph_store::load_or_build_graph(repo_path)?;
    
    // NEW: Add intent decisions to graph
    if let Some(ref model) = intent_model {
        for adr in &model.adrs {
            let decision = sruja_graph::Decision {
                id: format!("adr-{}", adr.id),
                title: adr.title.clone(),
                context: adr.context.clone().unwrap_or_default(),
                decision: adr.decision.clone().unwrap_or_default(),
                alternatives: vec![],
                consequences: vec![],
                status: sruja_graph::DecisionStatus::Accepted,
                created_at: chrono::Utc::now(),
            };
            // Note: Would need mutable kg, so save for later
        }
    }
    
    // Build comprehensive report
    let report = ComprehensiveReport {
        structural: ReportStructuralSection::from_scan(&graph),
        semantic: ReportSemanticSection::from_semantic(&semantic),
        intent: intent_model.as_ref().map(|m| ReportIntentSection::from_model(m)),
        runtime: if let Some(t) = traces {
            Some(RuntimeSection::from_traces(&load_traces(t)?))
        } else {
            None
        },
        recommendations: build_recommendations(&graph, &semantic, &kg),
    };
    
    // Output
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_comprehensive_report(&report);
    }
    
    Ok(())
}
```

---

## Step 6: Add Unified AI Commands

**File:** `crates/sruja-cli/src/main.rs`

Add to `Commands` enum:

```rust
/// Ask natural language question about architecture
Ask {
    question: String,
    #[arg(long, short = 'r', default_value = ".")]
    repo: String,
    #[arg(long, short = 'f', default_value = "text")]
    format: String,
},
/// Export architecture context for AI tools (Cursor, Copilot)
Context {
    #[arg(long)]
    for_ai: bool,
    #[arg(long, short = 'r', default_value = ".")]
    repo: String,
},
```

Add to match statement:

```rust
Commands::Ask { question, repo, format } => {
    commands::ai::ai_ask(&repo, &question, &format, None).await?;
}
Commands::Context { for_ai, repo } => {
    commands::context::export_context(&repo, for_ai).await?;
}
```

**File:** `crates/sruja-cli/src/commands/context.rs` (NEW FILE)

```rust
use std::path::Path;
use super::CliError;

pub async fn export_context(repo: &str, for_ai: bool) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    let kg = crate::graph_store::load_or_build_graph(repo_path)?;
    
    if for_ai {
        // Generate .cursorrules format
        println!("# Sruja Architecture Context\n");
        println!("## Components\n");
        for (id, node) in &kg.nodes {
            println!("- {} ({})", node.label, node.kind);
        }
        
        println!("\n## Dependencies\n");
        for edge in &kg.edges {
            if let (Some(src), Some(tgt)) = (kg.nodes.get(&edge.source), kg.nodes.get(&edge.target)) {
                println!("- {} → {}", src.label, tgt.label);
            }
        }
        
        println!("\n## Architecture Decisions\n");
        for (id, decision) in &kg.decisions {
            println!("- **{}**: {}", decision.title, decision.decision);
        }
        
        println!("\n## Bounded Contexts\n");
        // Would need semantic analysis here
        println!("(Run `sruja analyze` for semantic context detection)\n");
    } else {
        // Human-readable markdown
        println!("{}", serde_json::to_string_pretty(&kg)?);
    }
    
    Ok(())
}
```

Add to `commands/mod.rs`:
```rust
pub mod context;
```

---

## Step 7: Editor integration (skills + CLI — no MCP)

**Do not add an MCP server.** The `sruja-mcp` crate and `sruja mcp` / `sruja ai` commands were **removed**. Editors integrate via:

1. **Skill** – `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture` (and optionally `sruja-architecture-agent`).
2. **CLI** – `sruja quickstart`, `sruja drift`, `sruja intent check`, `sruja why`, `sruja context export`.

See [INSTALL_AS_SKILL.md](INSTALL_AS_SKILL.md) and [skills/README.md](../skills/README.md).

---

## Step 8: Test the Integration

```bash
# Test AI-enhanced quickstart
cd /Users/dilipkola/Workspace/sruja
cargo build --release

./target/release/sruja quickstart -r . --format json | jq '.semantic'

# Test knowledge graph persistence
./target/release/sruja analyze -r .
cat .sruja/graph.json | jq '.nodes | length'

# Test unified ask command
./target/release/sruja ask "What is the architecture?" -r .

# Test context export
./target/release/sruja context --for-ai -r . > .cursorrules
cat .cursorrules
```

---

## Step 9: Update Documentation

**File:** `README.md`

Change:
```markdown
### AI-Assisted Discovery (Optional)
```

To:
```markdown
### AI-Enhanced Analysis (Core Feature)

Sruja uses AI to provide deeper insights:

```bash
# AI-enhanced quickstart (uses local model by default)
sruja quickstart -r .

# Semantic drift detection
sruja drift -r .

# Natural language queries
sruja ask "Why do we use microservices?"

# Export for AI assistants
sruja context --for-ai > .cursorrules
```

For better results, install Ollama:
```bash
curl -fsSL https://ollama.ai/install.sh | sh
ollama pull llama3.2
```
```

---

## Summary of Changes

### New Files (4)
1. `crates/sruja-cli/src/graph_store.rs` (if pursued)
2. `crates/sruja-cli/src/commands/context.rs` (if pursued)
3. `.sruja/graph.json` (generated)
4. `.cursorrules` (generated)

*(No `mcp.rs` — MCP not used; skills + CLI only.)*

### Modified Files (6)
1. `crates/sruja-cli/Cargo.toml` - (sruja-extract removed; add any new deps as needed)
2. `crates/sruja-cli/src/main.rs` - Add mod declarations and commands
3. `crates/sruja-cli/src/commands/mod.rs` - Add modules
4. `crates/sruja-cli/src/commands/scan.rs` - Enhance quickstart/drift
5. `crates/sruja-cli/src/commands/analyze.rs` - Add graph integration
6. `README.md` - Update AI positioning

---

## Next Steps

1. Implement these changes
2. Test with real repos
3. Add Ollama integration (Step 4 in main plan)
4. ~~Add MCP server integration~~ **Not applicable** — use skills + CLI (see Step 7)
5. Update all documentation

This checklist makes AI core to every command, not optional.

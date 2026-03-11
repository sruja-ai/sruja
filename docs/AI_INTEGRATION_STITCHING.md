# AI Integration Stitching Plan

**Goal:** Transform Sruja from "tree-sitter + optional AI" to "AI-first architecture intelligence" by integrating existing AI components into the core workflow.

> **Status (post-refactor):** The crates **sruja-chat** and **sruja-extract** have been removed. Architecture intelligence (quickstart, why, drift, analyze, context) now lives in **sruja-cli** (with sruja-graph, sruja-scan, sruja-diff, sruja-intent). This document remains as a historical/target plan; update component names when implementing.

---

## Current State: Fragmented AI

### What Exists (But Disconnected)

| Component | Location | Current Use | Problem |
|-----------|----------|-------------|---------|
| **sruja-scan** | `crates/sruja-scan` | Tree-sitter structural analysis | No AI enhancement |
| **sruja-semantic** | `crates/sruja-semantic` | Embeddings, bounded contexts | Stub provider by default, optional |
| **sruja-graph** | `crates/sruja-graph` | Knowledge graph, decisions | Used by CLI (quickstart, analyze) |
| **CLI AI commands** | *(removed)* | — | **Skills + CLI only** — use `sruja quickstart`, `drift`, `intent check`, `why`; editor skill interprets output. |
| **MCP server** | *(removed)* | — | **Not in repo** — integrate via skills + CLI; see [INSTALL_AS_SKILL](INSTALL_AS_SKILL.md). |

### Current Workflow (AI is Optional)

```bash
# Structural only (no AI)
sruja quickstart -r .
sruja drift -r .
sruja analyze -r .

# No separate `sruja ai` subcommands — use the skill in Cursor/Copilot and CLI for evidence
# sruja quickstart -r . && sruja intent check -r . -i .

# Desktop app (separate product)
sruja-app  # Has chat, agents, extraction
```

---

## Target State: AI-First Unified Product

### Vision: AI Enhances Every Step

```bash
# AI is CORE, not optional
sruja quickstart -r .
  → Structural scan
  → AI semantic analysis (local model by default)
  → AI-generated architecture hypothesis
  → Recommendations with reasoning

sruja drift -r .
  → Structural drift (cycles, orphans)
  → Semantic drift (behavioral changes)
  → AI explains WHY it matters
  → AI suggests fixes

sruja analyze -r .
  → Merges: structural + semantic + intent + AI reasoning
  → Knowledge graph with AI-inferred decisions
  → AI recommendations

sruja ask "why this design?"  # Unified AI query
  → Uses knowledge graph
  → Provides evidence + reasoning
  → Remembers context (memory loop)
```

---

## Integration Architecture

### Phase 1: Make AI Default in Core Commands

#### 1.1 Enhance `quickstart` with AI

**File:** `crates/sruja-cli/src/commands/scan.rs`

**Current:**
```rust
pub async fn quickstart(repo: &str, format: &str) -> Result<(), CliError> {
    let graph = scan_repo(repo_path)?;  // Tree-sitter only
    let health = calculate_health(&graph);
    print_results(&graph, &health);
}
```

**Enhanced:**
```rust
pub async fn quickstart(repo: &str, format: &str) -> Result<(), CliError> {
    // 1. Structural scan (existing)
    let graph = scan_repo(repo_path)?;
    
    // 2. AI semantic analysis (NEW - always on)
    let provider = EmbeddingProvider::from_env_or_stub();
    let semantic = run_semantic_analyze(&components, &edges, &provider, None).await?;
    
    // 3. AI architecture hypothesis (NEW)
    let hypothesis = if provider.is_real() {
        generate_architecture_hypothesis(&graph, &semantic).await?
    } else {
        None  // Fallback gracefully
    };
    
    // 4. Enhanced health with AI insights
    let health = calculate_health_with_ai(&graph, &semantic, hypothesis.as_ref());
    
    print_results(&graph, &semantic, &health, hypothesis.as_ref());
}
```

**Key changes:**
- Add `sruja-semantic` to `Cargo.toml` (already there ✓)
- Create `generate_architecture_hypothesis()` using existing LLM infrastructure
- Graceful fallback if no LLM available (stub provider)

#### 1.2 Enhance `drift` with AI

**File:** `crates/sruja-cli/src/commands/scan.rs` (drift function)

**Add:**
```rust
pub async fn drift(repo: &str, architecture: Option<&str>, format: &str) -> Result<(), CliError> {
    // 1. Structural drift (existing)
    let scan_graph = scan_repo(repo_path)?;
    let structural_drift = detect_structural_drift(&scan_graph, baseline);
    
    // 2. Semantic drift (NEW)
    let semantic = run_semantic_analyze(&components, &edges, &provider, None).await?;
    let semantic_drift = detect_semantic_drift(&semantic, &previous_semantic);
    
    // 3. AI explanation of drift (NEW)
    let ai_explanation = if provider.is_real() {
        explain_drift_with_ai(&structural_drift, &semantic_drift).await?
    } else {
        None
    };
    
    print_drift_report(&structural_drift, &semantic_drift, ai_explanation.as_ref());
}
```

#### 1.3 Enhance `analyze` with Knowledge Graph

**File:** `crates/sruja-cli/src/commands/analyze.rs`

**Current:** Already has structural + semantic + intent  
**Missing:** Knowledge graph integration, AI reasoning

**Add:**
```rust
pub async fn analyze(repo: &str, ...) -> Result<(), CliError> {
    // Existing: structural + semantic + intent
    let graph = scan_repo(repo_path)?;
    let semantic = run_semantic_analyze(...)?;
    let intent = check_intent(...)?;
    
    // NEW: Build knowledge graph
    let mut kg = KnowledgeGraph::new();
    merge_scan_into_graph(&mut kg, &graph, repo);
    
    // NEW: Extract decisions from code comments, ADRs
    if let Some(intent_dir) = intent_dir {
        let decisions = extract_decisions_from_adrs(intent_dir).await?;
        for d in decisions {
            kg.add_decision(d)?;
        }
    }
    
    // NEW: AI-inferred decisions
    let ai_decisions = infer_decisions_from_patterns(&graph, &semantic).await?;
    for d in ai_decisions {
        kg.add_decision(d)?;
    }
    
    // NEW: AI recommendations using graph
    let recommendations = generate_recommendations(&kg, &report).await?;
    
    print_comprehensive_report(&report, &kg, &recommendations);
}
```

---

### Phase 2: Unify AI Commands

#### 2.1 Consolidate `ai` subcommands into core

**Current:**
```bash
sruja ai explain --topic "X"
sruja ai ask "question"
sruja ai memory
```

**Future:**
```bash
sruja explain "X"          # AI explain built into core
sruja ask "question"       # Natural language query
sruja why "question"       # Alias for ask (more intuitive)
sruja context --for-ai     # Export context for AI tools
```

**Implementation:**
```rust
// In main.rs
Commands {
    // ... existing commands ...
    
    /// Explain architecture concept with AI reasoning
    Explain {
        topic: String,
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    
    /// Ask natural language question about architecture
    Ask {
        question: String,
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        graph: Option<String>,
    },
    
    /// Export architecture context for AI tools
    Context {
        #[arg(long)]
        for_ai: bool,
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
}
```

**Context command implementation:**
```rust
pub async fn export_context(repo: &str, for_ai: bool) -> Result<(), CliError> {
    let kg = load_or_build_knowledge_graph(repo)?;
    
    if for_ai {
        // Export for Cursor/Copilot
        let cursor_rules = kg.to_cursor_rules();
        println!("{}", cursor_rules);
    } else {
        // Human-readable
        println!("{}", kg.to_markdown());
    }
}
```

---

### Phase 3: Integrate Knowledge Graph into CLI

#### 3.1 Add graph persistence to CLI

**New file:** `crates/sruja-cli/src/graph_store.rs`

```rust
use sruja_graph::KnowledgeGraph;
use std::path::Path;

const GRAPH_FILE: &str = ".sruja/graph.json";

pub fn load_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let graph_path = repo.join(GRAPH_FILE);
    if graph_path.exists() {
        let json = std::fs::read_to_string(graph_path)?;
        Ok(serde_json::from_str(&json)?)
    } else {
        Ok(KnowledgeGraph::new())
    }
}

pub fn save_graph(repo: &Path, graph: &KnowledgeGraph) -> Result<(), CliError> {
    let sruja_dir = repo.join(".sruja");
    std::fs::create_dir_all(&sruja_dir)?;
    let json = serde_json::to_string_pretty(graph)?;
    std::fs::write(sruja_dir.join("graph.json"), json)?;
    Ok(())
}

pub fn build_and_save_graph(repo: &Path) -> Result<KnowledgeGraph, CliError> {
    let scan = scan_repo(repo)?;
    let mut kg = KnowledgeGraph::new();
    merge_scan_into_graph(&mut kg, &scan, &repo.display().to_string());
    
    // Add semantic insights
    let semantic = run_semantic_analyze(...)?;
    merge_semantic_into_graph(&mut kg, &semantic);
    
    // Add AI-inferred decisions
    let decisions = infer_decisions(&scan, &semantic).await?;
    for d in decisions {
        kg.add_decision(d)?;
    }
    
    save_graph(repo, &kg)?;
    Ok(kg)
}
```

#### 3.2 Update commands to use persistent graph

```rust
// In analyze.rs, drift.rs, etc.
pub async fn analyze(repo: &str, ...) -> Result<(), CliError> {
    let repo_path = Path::new(repo);
    
    // Load or build graph
    let mut kg = load_graph(&repo_path)?;
    let needs_rebuild = /* check if code changed */;
    
    if needs_rebuild {
        kg = build_and_save_graph(&repo_path)?;
    }
    
    // Use graph for analysis
    let report = generate_report(&kg)?;
    print_report(&report);
}
```

---

### Phase 4: Embed AI Model by Default

#### 4.1 Add Ollama auto-setup

**New file:** `crates/sruja-cli/src/ai/local_model.rs`

```rust
pub fn ensure_local_model() -> Result<Box<dyn EmbeddingProvider>, CliError> {
    // Try Ollama first
    if is_ollama_running() {
        return Ok(Box::new(OllamaEmbeddingProvider::new("llama3.2")?));
    }
    
    // Fallback to stub
    eprintln!("💡 Tip: Install Ollama for AI-enhanced analysis:");
    eprintln!("   curl -fsSL https://ollama.ai/install.sh | sh");
    eprintln!("   ollama pull llama3.2");
    Ok(Box::new(StubEmbeddingProvider::new()))
}

fn is_ollama_running() -> bool {
    std::process::Command::new("ollama")
        .arg("list")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

**Use in commands:**
```rust
let provider = ensure_local_model()?;
let semantic = run_semantic_analyze(&components, &edges, &*provider, None).await?;
```

#### 4.2 Hybrid approach: Local + Cloud

```rust
pub fn get_embedding_provider() -> Box<dyn EmbeddingProvider> {
    // Priority: Cloud API > Ollama > Stub
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        return Box::new(OpenAIEmbeddingProvider::new(&key));
    }
    
    if is_ollama_running() {
        return Box::new(OllamaEmbeddingProvider::new("llama3.2"));
    }
    
    Box::new(StubEmbeddingProvider::new())
}
```

---

### Phase 5: MCP server (not pursued)

**Status:** The **sruja-mcp** crate and **sruja ai** / **sruja mcp** commands were **removed**. Integration is **skills + CLI** only:

- Install: `npx skills add https://github.com/sruja-ai/sruja --skill sruja-architecture-agent`
- CLI: `sruja quickstart`, `sruja drift`, `sruja intent check`, `sruja why`, `sruja context export`

No MCP server or `sruja mcp` in this repo.

---

## Implementation Priority

### P0: Core AI Integration (Week 1-2)
1. ✅ Add AI to `quickstart` (semantic + hypothesis)
2. ✅ Add AI to `drift` (semantic drift + explanation)
3. ✅ Enhance `analyze` with knowledge graph
4. ✅ Add persistent graph storage (`.sruja/graph.json`)

### P1: Unified Commands (Week 3)
5. ✅ Consolidate `ai` subcommands
6. ✅ Add `sruja ask` and `sruja context`
7. ✅ Auto-detect and use local models (Ollama)

### P2: AI Tooling (Week 4)
8. ~~MCP server integration~~ **Removed** — use skills + CLI
9. ✅ Export for Cursor/Copilot rules
10. ✅ Documentation and examples

---

## Success Metrics

### Before (Current)
- `sruja quickstart`: 0% AI, pure tree-sitter
- `sruja drift`: 0% AI, structural only
- `sruja ai explain`: Optional, requires API key
- Knowledge graph: Desktop app only
- MCP: **Removed** (use skills + CLI)

### After (AI-First)
- `sruja quickstart`: 100% AI-enhanced (local model by default)
- `sruja drift`: Semantic + AI explanation
- `sruja ask`: Core command, always available
- Knowledge graph: Core feature, persisted in CLI
- MCP: **Not applicable** — skills + CLI

---

## Code Changes Summary

### New Files
- `crates/sruja-cli/src/graph_store.rs` - Graph persistence
- `crates/sruja-cli/src/ai/local_model.rs` - Ollama integration
- `crates/sruja-cli/src/ai/inference.rs` - Decision inference

### Modified Files
- `crates/sruja-cli/Cargo.toml` - Add sruja-graph, sruja-extract
- `crates/sruja-cli/src/main.rs` - Add unified commands
- `crates/sruja-cli/src/commands/scan.rs` - Enhance quickstart/drift
- `crates/sruja-cli/src/commands/analyze.rs` - Add graph integration
- ~~`crates/sruja-mcp/...`~~ **Removed**

### Configuration
- `.sruja/graph.json` - Persisted knowledge graph
- `.sruja/config.toml` - AI settings (model, provider)
- `.cursorrules` - Auto-generated from graph

---

## Fallback Strategy

**No API key / No Ollama:**
- Use `StubEmbeddingProvider` for semantic analysis
- Skip AI-enhanced features (hypothesis, drift explanation)
- Still get: structural analysis + basic semantic clustering
- Show hint: "Install Ollama for AI insights"

**API key available:**
- Full AI-enhanced analysis
- Cloud model (OpenAI, Anthropic, etc.)

**Ollama available:**
- Full AI-enhanced analysis
- Local model (privacy-first)

---

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_quickstart_with_stub_provider() {
    let result = quickstart("test-repo", "json").await;
    assert!(result.is_ok());
    // Should work without LLM
}

#[test]
fn test_quickstart_with_ollama() {
    if is_ollama_running() {
        let result = quickstart("test-repo", "json").await;
        assert!(result.is_ok());
        // Should have AI hypothesis
    }
}
```

### Integration Tests
```bash
# Test AI-enhanced quickstart
sruja quickstart -r . --format json | jq '.hypothesis'

# Test semantic drift
sruja drift -r . --format json | jq '.semantic_drift'

# Test knowledge graph persistence
sruja analyze -r .
cat .sruja/graph.json | jq '.decisions'
```

---

## Documentation Updates

- [ ] Update README.md: AI is core, not optional
- [ ] Update QUICKSTART.md: Show AI features prominently
- [ ] Add AI_CONFIGURATION.md: How to set up Ollama/API keys
- [ ] Add MCP_INTEGRATION.md: How to connect Cursor/Copilot
- [ ] Update examples/: Show AI-enhanced workflows

---

## Key Principle

**AI should feel native, not bolted on:**

❌ Bad: "Run `sruja ai explain` for AI insights"  
✅ Good: "Sruja analyzes your architecture with AI reasoning"

❌ Bad: "Requires OPENAI_API_KEY for semantic analysis"  
✅ Good: "Uses local model by default, upgrade to cloud for better results"

❌ Bad: "Knowledge graph is in the desktop app"  
✅ Good: "Knowledge graph is core to every command"

---

This plan stitches together your existing AI components into a unified, AI-first product where AI enhances every interaction, not just optional add-on commands.

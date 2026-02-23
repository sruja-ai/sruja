# SemanticIntelligence API (Dev 2 → Dev 1 Handoff)

**Purpose:** Document the semantic analysis API so Dev 1 can consume it for optional semantic alignment in intent drift (Week 11+).

**Crate:** `sruja-semantic`

---

## Entry Point

```rust
pub async fn sruja_semantic::analyze(
    components: &[(String, String)],      // (component_id, text) from scan
    structural_edges: &[(String, String)], // (source_id, target_id) from scan
    provider: &dyn EmbeddingProvider,
    config: Option<SemanticConfig>,
) -> Result<SemanticReport, EmbeddingError>
```

- **Input:** Components and edges derived from `sruja_scan::Graph` (see below).
- **Provider:** Use `StubEmbeddingProvider::new()` for zero-config / CI; no API key. Use rig/OpenAI providers when embedding API is available.
- **Output:** `SemanticReport` (contexts, coupling, vocabulary leaks, summary).

---

## Getting Input from Scan Graph

```rust
// From sruja_scan::Graph (nodes, edges)
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

let structural_edges: Vec<(String, String)> = graph
    .edges
    .iter()
    .map(|e| (e.source.clone(), e.target.clone()))
    .collect();
```

---

## Output: SemanticReport

| Field | Type | Description |
|-------|------|-------------|
| `contexts` | `Vec<BoundedContext>` | Detected bounded contexts (name, components, vocabulary) |
| `clusters` | `Vec<DomainCluster>` | Raw domain clusters |
| `coupling` | `SemanticCouplingReport` | Hidden couplings, semantic hubs, recommendations |
| `vocabulary_leaks` | `Vec<VocabularyLeak>` | Terms leaking across contexts |
| `summary` | `SemanticSummary` | component_count, context_count, hidden_coupling_count, vocabulary_leak_count, health_score (0–100) |

**BoundedContext:** `name`, `components: Vec<String>`, `vocabulary`, `shared_vocabulary`  
**SemanticCouplingReport:** `hidden_couplings` (source, target, similarity, shared_concepts), `recommendations`  
**SemanticSummary:** Used for CLI and for aligning intent: `health_score`, counts.

---

## Use in Intent / Drift (Dev 1)

- **Optional.** If intent drift uses semantic alignment: call `analyze()` with scan graph–derived components/edges; use `SemanticReport::summary` and/or `contexts` / `coupling.hidden_couplings` to enrich drift or recommendations.
- **Stub:** Dev 1 can proceed without this; when ready, wire the above and use `StubEmbeddingProvider` if no embedding API.

---

## Dependencies

- `sruja-semantic` depends on `sruja-scan` only for the *shape* of data (components + edges); CLI builds components/edges from `sruja_scan::scan_repo()`. Dev 1 does not need to depend on `sruja-semantic` unless adding semantic alignment.

---

*Document version: 2026-02-23 (Week 11 handoff).*

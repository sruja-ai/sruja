//! Adaptive Hybrid Retrieval (AHR).
//!
//! Dynamically evaluates query complexity and routes to the optimal retrieval
//! mechanism: fast graph traversal for structural queries, or semantic search
//! for cross-cutting conceptual queries, with optional graph-refinement pass.
//!
//! This eliminates the false dichotomy of "graph OR embeddings" by using both
//! when each is strongest:
//! - Graph traversal: exact IDs, "what is X", neighbors, blast radius, cycles
//! - Semantic search: cross-cutting concerns, vague questions, conceptual similarity
//! - Hybrid: semantic discovery → graph refinement for multi-hop precision

use crate::graph::KnowledgeGraph;
use crate::query::QueryResult;
use serde::{Deserialize, Serialize};

/// How complex the query is, determining which retrieval path to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryComplexity {
    /// Direct lookup: exact ID, "what is X", "describe Y"
    Structural,
    /// Needs graph traversal: "why", "how does X connect to Y", blast radius
    Relational,
    /// Cross-cutting or vague: "authentication concerns", "performance bottlenecks"
    Conceptual,
}

/// Which retrieval strategy was selected.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalStrategy {
    GraphOnly,
    SemanticOnly,
    SemanticThenGraph,
}

/// Result from the hybrid retrieval router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridResult {
    pub query: String,
    pub complexity: QueryComplexity,
    pub strategy: RetrievalStrategy,
    pub graph_result: Option<QueryResult>,
    pub semantic_candidates: Vec<SemanticCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCandidate {
    pub element_id: String,
    pub score: f32,
    pub label: Option<String>,
}

/// Classifies a natural language query into a complexity tier.
///
/// Uses a lightweight deterministic heuristic (no LLM call) based on:
/// - Presence of known element IDs or dot-notation references → Structural
/// - Relational keywords (why, how, connect, depend, blast) → Relational
/// - Everything else → Conceptual (needs semantic search)
pub fn classify_query(query: &str, graph: &KnowledgeGraph) -> QueryComplexity {
    let q = query.to_lowercase();

    if graph.resolve_entity(query).is_some() {
        let relational_keywords = [
            "why",
            "how",
            "connect",
            "depend",
            "impact",
            "blast",
            "upstream",
            "downstream",
            "path",
            "between",
            "cycle",
        ];
        if relational_keywords.iter().any(|kw| q.contains(kw)) {
            return QueryComplexity::Relational;
        }
        return QueryComplexity::Structural;
    }

    if q.contains('.') && q.split('.').all(|p| !p.is_empty()) {
        let candidate: String = q
            .split_whitespace()
            .find(|w| w.contains('.'))
            .unwrap_or("")
            .to_string();
        if graph.nodes.contains_key(&candidate) {
            return QueryComplexity::Structural;
        }
    }

    let relational_keywords = [
        "why",
        "how does",
        "connect",
        "depend",
        "relationship",
        "between",
    ];
    if relational_keywords.iter().any(|kw| q.contains(kw)) {
        return QueryComplexity::Relational;
    }

    QueryComplexity::Conceptual
}

/// Selects the retrieval strategy based on query complexity.
pub fn select_strategy(complexity: QueryComplexity, has_semantic_index: bool) -> RetrievalStrategy {
    match complexity {
        QueryComplexity::Structural => RetrievalStrategy::GraphOnly,
        QueryComplexity::Relational => {
            if has_semantic_index {
                RetrievalStrategy::SemanticThenGraph
            } else {
                RetrievalStrategy::GraphOnly
            }
        }
        QueryComplexity::Conceptual => {
            if has_semantic_index {
                RetrievalStrategy::SemanticOnly
            } else {
                RetrievalStrategy::GraphOnly
            }
        }
    }
}

/// Executes the graph-only retrieval path.
pub fn execute_graph_only(graph: &KnowledgeGraph, query: &str) -> HybridResult {
    let graph_result = graph.query(query).ok();
    HybridResult {
        query: query.to_string(),
        complexity: classify_query(query, graph),
        strategy: RetrievalStrategy::GraphOnly,
        graph_result,
        semantic_candidates: Vec::new(),
    }
}

/// Builds a `HybridResult` that includes semantic candidates alongside
/// an optional graph refinement pass.
///
/// The caller provides pre-computed semantic candidates (from an external
/// embedding index like `sruja index semantic`). This function refines them
/// by verifying which candidates actually exist in the graph and enriching
/// with graph context.
pub fn execute_hybrid(
    graph: &KnowledgeGraph,
    query: &str,
    semantic_candidates: Vec<SemanticCandidate>,
) -> HybridResult {
    let complexity = classify_query(query, graph);
    let strategy = if semantic_candidates.is_empty() {
        RetrievalStrategy::GraphOnly
    } else {
        RetrievalStrategy::SemanticThenGraph
    };

    let graph_result = if let Some(top) = semantic_candidates.first() {
        graph.query(&format!("describe {}", top.element_id)).ok()
    } else {
        graph.query(query).ok()
    };

    let verified: Vec<SemanticCandidate> = semantic_candidates
        .into_iter()
        .filter(|c| graph.nodes.contains_key(&c.element_id))
        .collect();

    HybridResult {
        query: query.to_string(),
        complexity,
        strategy,
        graph_result,
        semantic_candidates: verified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArchitectureNode, NodeKind};

    fn test_graph() -> KnowledgeGraph {
        let mut g = KnowledgeGraph::new();
        g.add_node(ArchitectureNode {
            id: "API.Routes".to_string(),
            kind: NodeKind::Service,
            label: "API Routes".to_string(),
            technology: Some("Express".to_string()),
            ..ArchitectureNode::default()
        })
        .unwrap();
        g.add_node(ArchitectureNode {
            id: "Database.Users".to_string(),
            kind: NodeKind::Database,
            label: "Users DB".to_string(),
            technology: Some("PostgreSQL".to_string()),
            ..ArchitectureNode::default()
        })
        .unwrap();
        g
    }

    #[test]
    fn classify_structural_exact_id() {
        let g = test_graph();
        assert_eq!(
            classify_query("what is API Routes?", &g),
            QueryComplexity::Structural
        );
    }

    #[test]
    fn classify_relational_why() {
        let g = test_graph();
        assert_eq!(
            classify_query("why does API Routes exist?", &g),
            QueryComplexity::Relational
        );
    }

    #[test]
    fn classify_conceptual_vague() {
        let g = test_graph();
        assert_eq!(
            classify_query("authentication security concerns", &g),
            QueryComplexity::Conceptual
        );
    }

    #[test]
    fn strategy_graph_only_for_structural() {
        let strategy = select_strategy(QueryComplexity::Structural, true);
        assert!(matches!(strategy, RetrievalStrategy::GraphOnly));
    }

    #[test]
    fn strategy_semantic_for_conceptual_when_available() {
        let strategy = select_strategy(QueryComplexity::Conceptual, true);
        assert!(matches!(strategy, RetrievalStrategy::SemanticOnly));
    }

    #[test]
    fn strategy_fallback_to_graph_without_semantic_index() {
        let strategy = select_strategy(QueryComplexity::Conceptual, false);
        assert!(matches!(strategy, RetrievalStrategy::GraphOnly));
    }

    #[test]
    fn execute_graph_only_returns_result() {
        let g = test_graph();
        let result = execute_graph_only(&g, "describe API Routes");
        assert!(result.graph_result.is_some());
        assert!(result.semantic_candidates.is_empty());
    }

    #[test]
    fn execute_hybrid_filters_invalid_candidates() {
        let g = test_graph();
        let candidates = vec![
            SemanticCandidate {
                element_id: "API.Routes".to_string(),
                score: 0.95,
                label: Some("API Routes".to_string()),
            },
            SemanticCandidate {
                element_id: "NonExistent.Thing".to_string(),
                score: 0.80,
                label: None,
            },
        ];
        let result = execute_hybrid(&g, "authentication concerns", candidates);
        assert_eq!(result.semantic_candidates.len(), 1);
        assert_eq!(result.semantic_candidates[0].element_id, "API.Routes");
    }
}

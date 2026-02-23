//! Graph RAG: retrieve architecture context from the knowledge graph for agents.
//!
//! Uses the existing query API and graph structure to build context strings
//! that augment LLM prompts with relevant architecture information.

use sruja_graph::KnowledgeGraph;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Retrieve architecture context from the graph for the given question/message.
///
/// Runs graph queries and formats results as text for use in agent preambles.
/// Returns empty string if graph is empty or query yields nothing useful.
pub async fn retrieve_graph_context(
    graph: &Arc<RwLock<KnowledgeGraph>>,
    question: &str,
    max_evidence_items: usize,
) -> String {
    let g = graph.read().await;
    let stats = g.stats();

    if stats.total_nodes == 0 && stats.total_decisions == 0 && stats.total_policies == 0 {
        return String::new();
    }

    let query_result = match g.query(question) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };

    let mut parts = Vec::new();

    if !query_result.answer.is_empty() && query_result.confidence > 0.3 {
        parts.push(format!("Answer from graph: {}", query_result.answer));
    }

    for (i, ev) in query_result.evidence.iter().take(max_evidence_items).enumerate() {
        parts.push(format!(
            "[{}] {}: {}",
            i + 1,
            match &ev.kind {
                sruja_graph::query::EvidenceKind::Decision => "Decision",
                sruja_graph::query::EvidenceKind::Policy => "Policy",
                sruja_graph::query::EvidenceKind::Requirement => "Requirement",
                sruja_graph::query::EvidenceKind::Node => "Component",
                sruja_graph::query::EvidenceKind::Edge => "Relationship",
            },
            ev.excerpt
        ));
    }

    if parts.is_empty() {
        // Fallback: summarize graph stats for awareness
        parts.push(format!(
            "Graph has {} components, {} decisions, {} policies. No specific evidence matched the query.",
            stats.total_nodes, stats.total_decisions, stats.total_policies
        ));
    }

    parts.join("\n")
}

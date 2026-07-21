use std::collections::BTreeMap;
use std::path::Path;

use sruja_scan::{EdgeKind, NodeKind};

/// Machine-readable repo context for agents (JSON).
#[derive(Debug, serde::Serialize)]
pub struct DiscoverContextJson {
    pub repo: String,
    pub scan_scope: sruja_scan::scan_scope::ScanScope,
    pub components: usize,
    pub edges: usize,
    pub primary_language: String,
    pub framework: Option<String>,
    pub architecture_style: String,
    pub domain: Option<String>,
    pub suggested_areas: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiscoverEnrichment {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrative_markdown: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DiscoverEdgeConfidenceBreakdown {
    pub(crate) extracted: usize,
    pub(crate) inferred: usize,
    pub(crate) ambiguous: usize,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DiscoverCommunity {
    pub(crate) id: u32,
    pub(crate) suggested_label: String,
    pub(crate) member_count: usize,
    pub(crate) cohesion: f64,
    pub(crate) top_members: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DiscoverExplanationJson {
    pub(crate) context: DiscoverContextJson,
    pub(crate) kind_counts: BTreeMap<String, usize>,
    pub(crate) reasoning: Vec<String>,
    pub(crate) top_directories: Vec<DiscoverDirectorySummary>,
    pub(crate) key_elements: Vec<DiscoverElementSummary>,
    pub(crate) god_nodes: Vec<DiscoverElementSummary>,
    pub(crate) key_relationships: Vec<DiscoverRelationshipSummary>,
    pub(crate) surprising_connections: Vec<DiscoverRelationshipSummary>,
    pub(crate) suggested_questions: Vec<String>,
    pub(crate) confidence: DiscoverConfidence,
    pub(crate) edge_confidence_breakdown: DiscoverEdgeConfidenceBreakdown,
    pub(crate) architecture_digest: String,
    pub(crate) communities: Vec<DiscoverCommunity>,
    pub(crate) next_steps: Vec<String>,
    pub enrichment: Option<DiscoverEnrichment>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DiscoverDirectorySummary {
    pub(crate) area: String,
    pub(crate) nodes: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct DiscoverElementSummary {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<String>,
    pub(crate) incoming: usize,
    pub(crate) outgoing: usize,
    pub(crate) pagerank: f64,
    pub(crate) why_it_matters: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DiscoverRelationshipSummary {
    pub(crate) source: String,
    pub(crate) target: String,
    pub(crate) kind: String,
    pub(crate) confidence: String,
    pub(crate) why_it_matters: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DiscoverConfidence {
    pub(crate) level: String,
    pub(crate) signals: Vec<String>,
    pub(crate) blind_spots: Vec<String>,
}

pub(crate) fn relative_graph_path(repo: &str, repo_path: &Path, path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let repo_prefix = repo_path
        .canonicalize()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.replace('\\', "/")));
    let repo_arg_norm = repo.replace('\\', "/").trim_end_matches('/').to_string();

    if let Some(prefix) = repo_prefix {
        return normalized
            .strip_prefix(&format!("{}/", prefix))
            .or_else(|| normalized.strip_prefix(&prefix))
            .unwrap_or(normalized.as_str())
            .trim_start_matches('/')
            .trim_start_matches("./")
            .to_string();
    }

    normalized
        .strip_prefix(&format!("{}/", repo_arg_norm))
        .or_else(|| normalized.strip_prefix(&repo_arg_norm))
        .unwrap_or(normalized.as_str())
        .trim_start_matches('/')
        .trim_start_matches("./")
        .to_string()
}

pub(crate) fn kind_priority(kind: NodeKind) -> u8 {
    match kind.as_str() {
        NodeKind::SERVICE => 6,
        NodeKind::FRONTEND => 5,
        NodeKind::DATABASE => 4,
        NodeKind::EXTERNAL_API => 3,
        NodeKind::CONTAINER => 2,
        NodeKind::COMPONENT => 2,
        NodeKind::QUEUE => 2,
        NodeKind::SYSTEM => 1,
        NodeKind::MODULE => 0,
        _ => 0,
    }
}

pub(crate) fn explain_node_relevance(node: &sruja_scan::Node, incoming: usize, outgoing: usize) -> String {
    match node.kind.as_str() {
        NodeKind::DATABASE => format!(
            "Acts as a data boundary with {} upstream dependency(ies).",
            incoming
        ),
        NodeKind::EXTERNAL_API => format!(
            "Represents an external integration touched by {} internal component(s).",
            incoming
        ),
        NodeKind::SERVICE | NodeKind::FRONTEND | NodeKind::CONTAINER | NodeKind::COMPONENT => {
            if incoming > 0 && outgoing > 0 {
                format!(
                    "Sits in the middle of the graph with {} incoming and {} outgoing dependency(ies).",
                    incoming, outgoing
                )
            } else if outgoing > 0 {
                format!(
                    "Looks like a coordinator with {} downstream dependency(ies).",
                    outgoing
                )
            } else if incoming > 0 {
                format!(
                    "Looks like a shared dependency referenced by {} upstream component(s).",
                    incoming
                )
            } else {
                "Was ranked highly despite being lightly connected, which usually means it is a clean entry point.".to_string()
            }
        }
        _ => {
            if incoming > outgoing {
                format!(
                    "Mostly acts as a shared dependency with {} incoming reference(s).",
                    incoming
                )
            } else if outgoing > 0 {
                format!(
                    "Mostly acts as a caller with {} outgoing dependency(ies).",
                    outgoing
                )
            } else {
                "Provides a standalone boundary candidate in the scanned graph.".to_string()
            }
        }
    }
}

pub(crate) fn explain_edge_relevance(
    edge: &sruja_scan::Edge,
    target_kind: Option<NodeKind>,
    evidence_count: usize,
) -> String {
    let evidence_suffix = if evidence_count > 0 {
        format!(" Backed by {} evidence item(s).", evidence_count)
    } else {
        String::new()
    };

    let base = match target_kind.as_ref().map(|k| k.as_str()) {
        Some(NodeKind::DATABASE) => {
            "Highlights a core service-to-data dependency that should usually stay intentional."
        }
        Some(NodeKind::EXTERNAL_API) => {
            "Highlights an external integration boundary that is important for change planning."
        }
        Some(NodeKind::SERVICE) | Some(NodeKind::FRONTEND) => {
            "Connects high-signal runtime or user-facing components."
        }
        _ if matches!(
            edge.kind.as_str(),
            EdgeKind::CALLS | EdgeKind::DEPENDS_ON | EdgeKind::USES
        ) =>
        {
            "Represents a meaningful internal dependency worth validating as a boundary."
        }
        _ => "Captures a structurally important relationship in the scanned graph.",
    };

    format!("{base}{evidence_suffix}")
}

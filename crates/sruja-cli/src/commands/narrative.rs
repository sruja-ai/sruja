use serde::Serialize;

use super::federation::SystemIndex;
use super::CliError;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct NarrativeResult {
    pub title: String,
    pub sections: Vec<NarrativeSection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NarrativeSection {
    pub heading: String,
    pub body: String,
}

#[allow(dead_code)]
pub fn render_trace_narrative(index: &SystemIndex, query: &str) -> NarrativeResult {
    let mut sections = Vec::new();

    let repo_count = index.repos.len();
    let node_count = index.nodes.len();
    let edge_count = index.edges.len();

    sections.push(NarrativeSection {
        heading: "System Overview".to_string(),
        body: format!(
            "This system spans {} repositories with {} architectural elements and {} relationships.",
            repo_count, node_count, edge_count
        ),
    });

    let services: Vec<&str> = index
        .nodes
        .iter()
        .filter(|n| n.kind == "service")
        .map(|n| n.label.as_str())
        .collect();

    if !services.is_empty() {
        sections.push(NarrativeSection {
            heading: "Services".to_string(),
            body: format!(
                "The system contains {} services: {}.",
                services.len(),
                services.join(", ")
            ),
        });
    }

    let cross_repo_edges: Vec<_> = index
        .edges
        .iter()
        .filter(|e| {
            let src_repo = index
                .nodes
                .iter()
                .find(|n| n.canonical_id == e.source)
                .map(|n| &n.repo_id);
            let tgt_repo = index
                .nodes
                .iter()
                .find(|n| n.canonical_id == e.target)
                .map(|n| &n.repo_id);
            src_repo != tgt_repo
        })
        .collect();

    if !cross_repo_edges.is_empty() {
        sections.push(NarrativeSection {
            heading: "Cross-Repo Dependencies".to_string(),
            body: format!(
                "There are {} cross-repository dependencies.",
                cross_repo_edges.len()
            ),
        });
    }

    NarrativeResult {
        title: format!("Trace: {}", query),
        sections,
    }
}

#[allow(dead_code)]
pub fn render_explain_narrative(
    index: &SystemIndex,
    element_id: &str,
) -> Result<NarrativeResult, CliError> {
    let node = index
        .nodes
        .iter()
        .find(|n| n.canonical_id == element_id || n.local_id == element_id)
        .ok_or_else(|| CliError::validation(format!("Element '{}' not found", element_id)))?;

    let mut sections = Vec::new();

    let incoming: Vec<&str> = index
        .edges
        .iter()
        .filter(|e| e.target == node.canonical_id)
        .filter_map(|e| {
            index
                .nodes
                .iter()
                .find(|n| n.canonical_id == e.source)
                .map(|n| n.label.as_str())
        })
        .collect();

    let outgoing: Vec<&str> = index
        .edges
        .iter()
        .filter(|e| e.source == node.canonical_id)
        .filter_map(|e| {
            index
                .nodes
                .iter()
                .find(|n| n.canonical_id == e.target)
                .map(|n| n.label.as_str())
        })
        .collect();

    sections.push(NarrativeSection {
        heading: "What it does".to_string(),
        body: format!(
            "{} is a {} in the {} repository.",
            node.label, node.kind, node.repo_id
        ),
    });

    if !incoming.is_empty() {
        sections.push(NarrativeSection {
            heading: "Incoming".to_string(),
            body: format!("Receives input from: {}.", incoming.join(", ")),
        });
    }

    if !outgoing.is_empty() {
        sections.push(NarrativeSection {
            heading: "Outgoing".to_string(),
            body: format!("Sends output to: {}.", outgoing.join(", ")),
        });
    }

    Ok(NarrativeResult {
        title: node.label.clone(),
        sections,
    })
}

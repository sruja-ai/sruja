//! Build evidence bundle from scan graph + memory for the LLM.
//!
//! Reuses pattern from why command: merge scan into KnowledgeGraph, query, collect file refs.

use std::path::Path;

use sruja_graph::{merge_scan_into_graph, KnowledgeGraph};
use sruja_scan::{scan_repo, Graph};

use super::memory::load_facts;
use super::Fact;
use crate::commands::CliError;

/// Result of building context: text for the prompt and the list of allowed evidence paths.
#[derive(Debug, Clone)]
pub struct BuildContextResult {
    pub text: String,
    pub evidence_paths: Vec<String>,
}

/// Build a single evidence string for the prompt: graph-derived context + relevant facts.
/// Uses scan at repo_root; if graph_file is Some, load graph from file instead of scanning.
/// Returns the text and the list of file paths that were included (for validating LLM citations).
pub fn build_context(
    repo_root: &Path,
    question_or_topic: &str,
    graph_file: Option<&Path>,
    max_evidence_items: usize,
) -> Result<BuildContextResult, CliError> {
    let scan_graph: Graph = if let Some(p) = graph_file {
        let content = std::fs::read_to_string(p).map_err(CliError::Io)?;
        serde_json::from_str(&content).map_err(CliError::Json)?
    } else {
        scan_repo(repo_root).map_err(|e| CliError::Scan(e.to_string()))?
    };

    let repo_str = graph_file
        .and_then(|p| p.parent())
        .and_then(|p| p.to_str())
        .unwrap_or_else(|| repo_root.to_str().unwrap_or("."));

    let mut kg = KnowledgeGraph::new();
    merge_scan_into_graph(&mut kg, &scan_graph, repo_str);

    // Graph query for RAG-style context
    let query_result = match kg.query(question_or_topic) {
        Ok(r) => r,
        Err(_) => {
            // Fallback: still include file list and memory
            let mut parts = Vec::new();
            parts.push("No direct graph match for the question.".to_string());
            parts.push(format!(
                "Graph has {} nodes, {} edges.",
                scan_graph.nodes.len(),
                scan_graph.edges.len()
            ));
            let file_refs = collect_file_evidence(&scan_graph);
            if !file_refs.is_empty() {
                parts.push("Relevant file paths from scan:".to_string());
                for f in file_refs.iter().take(max_evidence_items) {
                    parts.push(format!("  - {}", f));
                }
            }
            let facts = load_facts(repo_root).unwrap_or_default();
            let relevant = facts_relevant(&facts, question_or_topic, max_evidence_items);
            if !relevant.is_empty() {
                parts.push("Stored facts (memory):".to_string());
                for fact in relevant {
                    parts.push(format!("  - [{}] {} (confidence: {})", fact.fact_id, fact.statement, fact.confidence));
                }
            }
            return Ok(BuildContextResult {
                text: parts.join("\n"),
                evidence_paths: file_refs,
            });
        }
    };

    let mut parts = Vec::new();
    if !query_result.answer.is_empty() && query_result.confidence > 0.3 {
        parts.push(format!("Answer from graph: {}", query_result.answer));
    }
    for (i, ev) in query_result
        .evidence
        .iter()
        .take(max_evidence_items)
        .enumerate()
    {
        parts.push(format!("[{}] {}", i + 1, ev.excerpt));
    }
    let file_refs = collect_file_evidence(&scan_graph);
    if !file_refs.is_empty() {
        parts.push("File paths from scan:".to_string());
        for f in file_refs.iter().take(max_evidence_items) {
            parts.push(format!("  - {}", f));
        }
    }
    let facts = load_facts(repo_root).unwrap_or_default();
    let relevant = facts_relevant(&facts, question_or_topic, max_evidence_items);
    if !relevant.is_empty() {
        parts.push("Stored facts (memory):".to_string());
        for fact in relevant {
            parts.push(format!(
                "  - [{}] {} (confidence: {})",
                fact.fact_id, fact.statement, fact.confidence
            ));
        }
    }
    Ok(BuildContextResult {
        text: parts.join("\n"),
        evidence_paths: file_refs,
    })
}

fn collect_file_evidence(scan_graph: &Graph) -> Vec<String> {
    let mut files = std::collections::HashSet::new();
    for edge in &scan_graph.edges {
        for ev in &edge.evidence {
            if let Some(ref f) = ev.file {
                files.insert(f.clone());
            }
        }
    }
    for node in &scan_graph.nodes {
        if let Some(ref p) = node.path {
            files.insert(p.clone());
        }
    }
    let mut v: Vec<_> = files.into_iter().collect();
    v.sort();
    v
}

/// Pick facts that might be relevant (by tag or statement substring). Simple heuristic.
fn facts_relevant<'a>(facts: &'a [Fact], question: &str, max: usize) -> Vec<&'a Fact> {
    let q_lower = question.to_lowercase();
    let mut scored: Vec<(f64, &Fact)> = facts
        .iter()
        .filter(|f| f.status != "deprecated" && f.confidence >= 0.25)
        .map(|f| {
            let mut score = f.confidence;
            if f.statement.to_lowercase().contains(&q_lower) {
                score += 0.3;
            }
            for tag in &f.tags {
                if q_lower.contains(&tag.to_lowercase()) {
                    score += 0.2;
                    break;
                }
            }
            (score, f)
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(max).map(|(_, f)| f).collect()
}

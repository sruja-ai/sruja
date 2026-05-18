//! Graph-aware session context pruning suggestions (Phase 3).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sruja_scan::Graph;
use std::collections::HashSet;

use crate::commands::context::types::TokenBudget;
use crate::commands::CliError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPruneSuggestion {
    pub schema_version: String,
    pub active_element_ids: Vec<String>,
    pub session_element_ids: Vec<String>,
    pub keep_ids: Vec<String>,
    pub compress_ids: Vec<String>,
    pub max_depth: usize,
    pub rationale: String,
    pub warnings: Vec<String>,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone)]
struct ResolvedId {
    id: String,
    ambiguous_matches: Vec<String>,
}

fn resolve_id_best_effort(id: &str, all_ids_sorted: &[String]) -> ResolvedId {
    let needle = id.trim();
    if needle.is_empty() {
        return ResolvedId {
            id: needle.to_string(),
            ambiguous_matches: Vec::new(),
        };
    }
    if all_ids_sorted.iter().any(|x| x == needle) {
        return ResolvedId {
            id: needle.to_string(),
            ambiguous_matches: Vec::new(),
        };
    }
    let suffix = format!(".{needle}");
    let matches: Vec<String> = all_ids_sorted
        .iter()
        .filter(|x| x.ends_with(&suffix))
        .cloned()
        .collect();
    match matches.len() {
        0 => ResolvedId {
            id: needle.to_string(),
            ambiguous_matches: Vec::new(),
        },
        1 => ResolvedId {
            id: matches[0].clone(),
            ambiguous_matches: Vec::new(),
        },
        _ => {
            let chosen = matches
                .iter()
                .min()
                .cloned()
                .unwrap_or_else(|| needle.to_string());
            ResolvedId {
                id: chosen,
                ambiguous_matches: matches,
            }
        }
    }
}

fn graph_id_catalog(graph: &Graph) -> Vec<String> {
    let mut ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    ids.sort();
    ids
}

/// Union of nodes reachable within `max_depth` hops from each active id (upstream + downstream).
fn reachable_from_active(
    graph: &Graph,
    active_resolved: &[String],
    max_depth: usize,
) -> HashSet<String> {
    let mut keep = HashSet::new();
    for id in active_resolved {
        keep.insert(id.clone());
        let radius = graph.blast_radius(id, max_depth);
        for n in radius.upstream.iter().chain(radius.downstream.iter()) {
            keep.insert(n.id.clone());
        }
    }
    keep
}

/// Suggest which session element ids to compress vs keep based on graph connectivity to active focus.
pub fn suggest_context_prune(
    graph: &Graph,
    active_element_ids: &[String],
    session_element_ids: &[String],
    max_depth: usize,
) -> ContextPruneSuggestion {
    let max_depth = max_depth.clamp(1, 4);
    let catalog = graph_id_catalog(graph);
    let known: HashSet<&str> = catalog.iter().map(String::as_str).collect();

    let mut warnings = Vec::new();
    let mut active_resolved = Vec::new();
    for raw in active_element_ids {
        let r = resolve_id_best_effort(raw, &catalog);
        if !r.ambiguous_matches.is_empty() {
            warnings.push(format!("active id {raw:?} ambiguous; using {}", r.id));
        }
        if !known.contains(r.id.as_str()) && !r.id.is_empty() {
            warnings.push(format!(
                "active id {raw:?} not in architecture graph; kept as focus"
            ));
        }
        active_resolved.push(r.id);
    }
    active_resolved.sort();
    active_resolved.dedup();

    let reachable = reachable_from_active(graph, &active_resolved, max_depth);
    let active_set: HashSet<&str> = active_resolved.iter().map(String::as_str).collect();

    let mut keep_ids = Vec::new();
    let mut compress_ids = Vec::new();

    for raw in session_element_ids {
        let r = resolve_id_best_effort(raw, &catalog);
        if !r.ambiguous_matches.is_empty() {
            warnings.push(format!("session id {raw:?} ambiguous; using {}", r.id));
        }
        let id = r.id;
        if active_set.contains(id.as_str()) || reachable.contains(id.as_str()) {
            keep_ids.push(id);
        } else if known.contains(id.as_str()) {
            compress_ids.push(id);
        } else {
            if !id.is_empty() {
                warnings.push(format!(
                    "session id {raw:?} not in graph; suggested compress (no structural path)"
                ));
            }
            compress_ids.push(id);
        }
    }

    keep_ids.sort();
    keep_ids.dedup();
    compress_ids.sort();
    compress_ids.dedup();
    compress_ids.retain(|id| !keep_ids.contains(id));

    let rationale = if active_resolved.is_empty() {
        "No active elements: compress all session ids except exact graph matches.".to_string()
    } else if compress_ids.is_empty() {
        format!(
            "All {} session element(s) are within depth-{max_depth} topology of active focus {:?}.",
            session_element_ids.len(),
            active_resolved
        )
    } else {
        format!(
            "Compress {} session element(s) with no path within depth {max_depth} to active {:?}; keep {} connected.",
            compress_ids.len(),
            active_resolved,
            keep_ids.len()
        )
    };

    let json_preview = json!({
        "keep_ids": keep_ids,
        "compress_ids": compress_ids,
    });
    let estimated_tokens = TokenBudget::estimate_tokens(&json_preview.to_string());

    ContextPruneSuggestion {
        schema_version: "context_prune/v1".to_string(),
        active_element_ids: active_element_ids.to_vec(),
        session_element_ids: session_element_ids.to_vec(),
        keep_ids,
        compress_ids,
        max_depth,
        rationale,
        warnings,
        estimated_tokens,
    }
}

pub fn suggest_context_prune_json(
    graph: &Graph,
    active_element_ids: &[String],
    session_element_ids: &[String],
    max_depth: usize,
) -> Result<String, CliError> {
    let suggestion =
        suggest_context_prune(graph, active_element_ids, session_element_ids, max_depth);
    Ok(serde_json::to_string_pretty(&suggestion)?)
}

/// Collect element ids from agent facts (`facts.drift` violations) for session pruning.
pub fn infer_session_element_ids_from_facts(active: &str, facts: &Value) -> Vec<String> {
    let mut ids = vec![active.to_string()];
    if let Some(drift) = facts
        .get("facts")
        .and_then(|f| f.get("drift"))
        .or_else(|| facts.get("drift"))
    {
        push_violation_locations(drift, &mut ids);
    }
    ids.sort();
    ids.dedup();
    ids
}

fn push_violation_locations(drift: &Value, ids: &mut Vec<String>) {
    if let Some(violations) = drift.get("violations").and_then(|v| v.as_array()) {
        for v in violations {
            if let Some(loc) = v.get("location").and_then(|l| l.as_str()) {
                let t = loc.trim();
                if !t.is_empty() {
                    ids.push(t.to_string());
                }
            }
        }
    }
    if let Some(drifts) = drift.get("drifts").and_then(|v| v.as_array()) {
        for d in drifts {
            if let Some(loc) = d.get("location").and_then(|l| l.as_str()) {
                let t = loc.trim();
                if !t.is_empty() {
                    ids.push(t.to_string());
                }
            }
        }
    }
}

pub fn parse_id_list_arg(arguments: &Value, key: &str) -> Result<Vec<String>, CliError> {
    let Some(arr) = arguments.get(key).and_then(|v| v.as_array()) else {
        return Err(CliError::validation(format!(
            "Missing or invalid array: {key}"
        )));
    };
    let mut out = Vec::new();
    for v in arr {
        let Some(s) = v.as_str() else {
            return Err(CliError::validation(format!(
                "{key} must be an array of strings"
            )));
        };
        let t = s.trim();
        if !t.is_empty() {
            out.push(t.to_string());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::{Edge, EdgeKind, Node};

    fn mini_graph() -> Graph {
        let mut g = Graph::default();
        for id in ["A", "B", "C", "D"] {
            g.nodes.push(Node {
                id: id.to_string(),
                label: id.to_string(),
                ..Default::default()
            });
        }
        g.edges.push(Edge {
            source: "A".into(),
            target: "B".into(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: Vec::new(),
            confidence: Default::default(),
        });
        g.edges.push(Edge {
            source: "B".into(),
            target: "C".into(),
            kind: EdgeKind::new(EdgeKind::CALLS),
            evidence: Vec::new(),
            confidence: Default::default(),
        });
        g
    }

    #[test]
    fn prune_keeps_connected_session_ids() {
        let g = mini_graph();
        let s = suggest_context_prune(
            &g,
            &["A".to_string()],
            &["C".to_string(), "D".to_string()],
            2,
        );
        assert!(s.keep_ids.contains(&"C".to_string()));
        assert!(s.compress_ids.contains(&"D".to_string()));
    }

    #[test]
    fn infer_session_ids_from_facts_includes_violation_locations() {
        let facts = serde_json::json!({
            "facts": {
                "drift": {
                    "violations": [{ "location": "B", "message": "x" }]
                }
            }
        });
        let ids = infer_session_element_ids_from_facts("A", &facts);
        assert!(ids.contains(&"A".to_string()));
        assert!(ids.contains(&"B".to_string()));
    }

    #[test]
    fn prune_keeps_active_ids() {
        let g = mini_graph();
        let s = suggest_context_prune(&g, &["B".to_string()], &["B".to_string()], 1);
        assert!(s.keep_ids.contains(&"B".to_string()));
        assert!(s.compress_ids.is_empty());
    }
}

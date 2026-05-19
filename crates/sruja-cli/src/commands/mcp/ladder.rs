use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

use super::super::CliError;
use super::helpers::{
    estimate_tokens, kind_matches_filter, push_resolution_warnings, resolve_id_best_effort,
    trim_text,
};

pub(crate) fn bfs_radius(
    adjacency: &HashMap<String, Vec<String>>,
    target: &str,
    max_depth: usize,
) -> Vec<Value> {
    use std::collections::{HashSet, VecDeque};
    if max_depth == 0 {
        return Vec::new();
    }
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    let mut out: Vec<(String, usize)> = Vec::new();

    visited.insert(target.to_string());
    queue.push_back((target.to_string(), 0));

    while let Some((cur, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        let Some(nexts) = adjacency.get(&cur) else {
            continue;
        };
        for next in nexts {
            if visited.insert(next.clone()) {
                let nd = depth + 1;
                out.push((next.clone(), nd));
                queue.push_back((next.clone(), nd));
            }
        }
    }

    out.sort_by(|a, b| (a.1, a.0.as_str()).cmp(&(b.1, b.0.as_str())));
    out.into_iter()
        .map(|(id, depth)| json!({ "id": id, "depth": depth }))
        .collect()
}

pub(crate) fn enforce_max_tokens_on_json_array_fields(
    value: &mut Value,
    max_tokens: usize,
    shrink_fields: &[&str],
) -> Result<bool, CliError> {
    let mut truncated = false;
    loop {
        let text = serde_json::to_string_pretty(value)?;
        if estimate_tokens(&text) <= max_tokens {
            break;
        }
        let mut shrunk_any = false;
        for key in shrink_fields {
            if let Some(arr) = value.get_mut(*key).and_then(|v| v.as_array_mut()) {
                if !arr.is_empty() {
                    arr.pop();
                    shrunk_any = true;
                }
            }
        }
        if !shrunk_any {
            break;
        }
        truncated = true;
    }
    Ok(truncated)
}

pub(crate) fn sync_element_ids_from_array(response: &mut Value, elements_key: &str) {
    let mut ids = response
        .get(elements_key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ids.sort();
    ids.dedup();
    response["element_ids"] = json!(ids);
}

pub(crate) fn push_token_budget_warning(response: &mut Value, max_tokens: usize) {
    let Ok(text) = serde_json::to_string_pretty(response) else {
        return;
    };
    if estimate_tokens(&text) <= max_tokens {
        return;
    }
    let msg = format!(
        "Response still exceeds max_tokens ({max_tokens}) after truncation; use a smaller max_tokens budget, fewer ids, or raise max_tokens."
    );
    let arr = match response.get_mut("warnings").and_then(|w| w.as_array_mut()) {
        Some(a) => a,
        None => {
            let mut existing = Vec::new();
            if let Some(v) = response.get("warnings") {
                if let Some(s) = v.as_str() {
                    existing.push(json!(s));
                } else if let Some(a) = v.as_array() {
                    existing.extend(a.iter().cloned());
                }
            }
            existing.push(json!(msg));
            response["warnings"] = Value::Array(existing);
            return;
        }
    };
    arr.push(json!(msg));
}

pub(crate) fn finalize_ladder_response(
    response: &mut Value,
    max_tokens: usize,
    shrink_fields: &[&str],
    sync_ids_from: Option<&str>,
) -> Result<(), CliError> {
    let truncated = enforce_max_tokens_on_json_array_fields(response, max_tokens, shrink_fields)?;
    if truncated {
        response["truncated"] = json!(true);
    }
    if let Some(key) = sync_ids_from {
        sync_element_ids_from_array(response, key);
    }
    push_token_budget_warning(response, max_tokens);
    set_estimated_tokens(response)
}

pub(crate) fn finalize_topology_response(
    response: &mut Value,
    max_tokens: usize,
) -> Result<(), CliError> {
    let truncated =
        enforce_max_tokens_on_json_array_fields(response, max_tokens, &["upstream", "downstream"])?;
    if truncated {
        response["truncated"] = json!(true);
    }
    response["element_ids"] = json!(collect_topology_element_ids(response));
    push_token_budget_warning(response, max_tokens);
    set_estimated_tokens(response)
}

pub(crate) fn attach_index_validation_log(
    response: &mut Value,
    source_file: &str,
    diagnostics: &[sruja_diagnostics::Diagnostic],
) -> Result<(), CliError> {
    if diagnostics.len() <= 8 {
        return Ok(());
    }
    let repo = Path::new(source_file)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let stem = Path::new(source_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("architecture");
    let storage_name = format!("index-{stem}.txt");
    let text = crate::commands::diagnostic_vfs::diagnostics_to_text(diagnostics);
    let truncation = crate::commands::diagnostic_vfs::truncate_and_store_if_needed(
        repo,
        &storage_name,
        &text,
        crate::commands::diagnostic_vfs::INDEX_VALIDATION_LOG_TOKEN_BUDGET,
    )?;
    if let Some(validation) = response.get_mut("validation") {
        validation["diagnostic_log"] = serde_json::to_value(&truncation)?;
    }
    Ok(())
}

pub(crate) fn set_estimated_tokens(value: &mut Value) -> Result<(), CliError> {
    let text = serde_json::to_string_pretty(value)?;
    value["estimated_tokens"] = json!(estimate_tokens(&text));
    Ok(())
}

pub(crate) fn build_architecture_index_from_program(
    source_file: &str,
    program: &sruja_language::ast::Program,
    kind_filter: Option<&[String]>,
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    use sruja_diagnostics::codes;
    let (elements, relations) = sruja_language::collect_elements(program);

    let mut node_ids = elements.keys().cloned().collect::<Vec<_>>();
    node_ids.sort();
    let edges = relations
        .iter()
        .map(|r| (r.from.as_string(), r.to.as_string()))
        .collect::<Vec<_>>();
    let scc = sruja_graph::SccAnalyzer::new().analyze(&node_ids, &edges);
    let cyclic_nodes = scc
        .components
        .iter()
        .filter(|c| c.is_cyclic)
        .flat_map(|c| c.nodes.iter().cloned())
        .collect::<std::collections::HashSet<_>>();
    let cycle_samples = scc
        .components
        .iter()
        .filter(|c| c.is_cyclic)
        .take(3)
        .map(|c| json!({ "id": c.id, "size": c.nodes.len(), "nodes": c.nodes }))
        .collect::<Vec<_>>();

    let validator = sruja_engine::Validator::with_default_rules();
    let diagnostics = validator.validate_sync(program);
    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Error)
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == sruja_diagnostics::Severity::Warning)
        .count();
    let policy_count = diagnostics
        .iter()
        .filter(|d| d.code == codes::CODE_POLICY_VIOLATION)
        .count();
    let policy_samples = diagnostics
        .iter()
        .filter(|d| d.code == codes::CODE_POLICY_VIOLATION)
        .take(3)
        .map(|d| {
            json!({
                "code": d.code,
                "message": trim_text(Some(d.message.as_str()), 180),
                "location": {
                    "file": d.location.file.clone(),
                    "line": d.location.line,
                    "column": d.location.column
                }
            })
        })
        .collect::<Vec<_>>();

    let mut entries = Vec::new();
    for (id, elem) in elements {
        let kind = elem.assignment.kind.to_string();
        if let Some(filter) = kind_filter {
            if !kind_matches_filter(&kind, filter) {
                continue;
            }
        }
        let title = elem
            .assignment
            .title
            .as_deref()
            .and_then(|t| trim_text(Some(t), 120))
            .unwrap_or_else(|| id.clone());
        let (description, technology) = elem
            .assignment
            .body
            .as_ref()
            .map(|b| (b.description.as_deref(), b.technology.as_deref()))
            .unwrap_or((None, None));
        entries.push(json!({
            "id": id.clone(),
            "kind": kind,
            "title": title,
            "technology": trim_text(technology, 80),
            "description": trim_text(description, 160),
            "in_cycle": cyclic_nodes.contains(&id)
        }));
    }
    entries.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .cmp(&b.get("id").and_then(|v| v.as_str()))
    });

    let element_ids = entries
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };

    let mut response = json!({
        "schema_version": "architecture_index/v1",
        "source": { "kind": "dsl", "file": source_file },
        "element_ids": element_ids,
        "elements": entries,
        "validation": {
            "errors": error_count,
            "warnings": warning_count,
            "policy_violations": { "count": policy_count, "samples": policy_samples },
            "cycles": { "cyclic_sccs": scc.cyclic_sccs, "largest_scc_size": scc.largest_scc_size, "samples": cycle_samples }
        },
        "next_suggested_tool": "sruja_get_topology",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    attach_index_validation_log(&mut response, source_file, &diagnostics)?;
    finalize_ladder_response(
        &mut response,
        max_tokens,
        &["elements", "element_ids"],
        Some("elements"),
    )?;
    Ok(serde_json::to_string_pretty(&response)?)
}

pub(crate) fn build_architecture_index_from_scan(
    graph: &sruja_scan::Graph,
    kind_filter: Option<&[String]>,
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let mut node_ids = graph.nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>();
    node_ids.sort();
    let edges = graph
        .edges
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect::<Vec<_>>();
    let scc = sruja_graph::SccAnalyzer::new().analyze(&node_ids, &edges);
    let cyclic_nodes = scc
        .components
        .iter()
        .filter(|c| c.is_cyclic)
        .flat_map(|c| c.nodes.iter().cloned())
        .collect::<std::collections::HashSet<_>>();

    let mut entries = Vec::new();
    let mut nodes = graph.nodes.clone();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    for n in nodes {
        let kind = n.kind.as_str().to_string();
        if let Some(filter) = kind_filter {
            if !kind_matches_filter(&kind, filter) {
                continue;
            }
        }
        entries.push(json!({
            "id": n.id,
            "kind": kind,
            "title": trim_text(Some(n.label.as_str()), 120),
            "technology": trim_text(n.technology.as_deref(), 80),
            "description": trim_text(n.path.as_deref(), 160),
            "in_cycle": cyclic_nodes.contains(&n.id)
        }));
    }

    let element_ids = entries
        .iter()
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };

    let mut response = json!({
        "schema_version": "architecture_index/v1",
        "source": { "kind": "scan" },
        "element_ids": element_ids,
        "elements": entries,
        "validation": {
            "errors": 0,
            "warnings": 0,
            "policy_violations": { "count": 0, "samples": [] },
            "cycles": { "cyclic_sccs": scc.cyclic_sccs, "largest_scc_size": scc.largest_scc_size, "samples": [] }
        },
        "next_suggested_tool": "sruja_get_topology",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_ladder_response(
        &mut response,
        max_tokens,
        &["elements", "element_ids"],
        Some("elements"),
    )?;
    Ok(serde_json::to_string_pretty(&response)?)
}

pub(crate) fn build_topology_from_program(
    source_file: &str,
    program: &sruja_language::ast::Program,
    id: &str,
    depth: usize,
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let (elements, relations) = sruja_language::collect_elements(program);
    let mut all_ids = elements.keys().cloned().collect::<Vec<_>>();
    all_ids.sort();
    let resolved = resolve_id_best_effort(id, &all_ids);
    let target = resolved.id.clone();

    let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
    let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
    for r in relations {
        let src = r.from.as_string();
        let tgt = r.to.as_string();
        outgoing.entry(src.clone()).or_default().push(tgt.clone());
        incoming.entry(tgt).or_default().push(src);
    }
    for v in outgoing.values_mut() {
        v.sort();
        v.dedup();
    }
    for v in incoming.values_mut() {
        v.sort();
        v.dedup();
    }

    let upstream = bfs_radius(&incoming, &target, depth);
    let downstream = bfs_radius(&outgoing, &target, depth);

    let mut warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };
    push_resolution_warnings(&mut warnings, id, &resolved);

    let mut response = json!({
        "schema_version": "topology/v1",
        "source": { "kind": "dsl", "file": source_file },
        "target": target,
        "max_depth": depth,
        "upstream": upstream,
        "downstream": downstream,
        "element_ids": [],
        "next_suggested_tool": "sruja_get_elements",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_topology_response(&mut response, max_tokens)?;
    Ok(serde_json::to_string_pretty(&response)?)
}

pub(crate) fn build_topology_from_scan(
    graph: &sruja_scan::Graph,
    id: &str,
    depth: usize,
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let mut all_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    all_ids.sort();
    let resolved = resolve_id_best_effort(id, &all_ids);
    let radius = graph.blast_radius(&resolved.id, depth);

    let mut warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };
    push_resolution_warnings(&mut warnings, id, &resolved);

    let mut response = json!({
        "schema_version": "topology/v1",
        "source": { "kind": "scan" },
        "target": radius.target,
        "max_depth": radius.max_depth,
        "upstream": radius.upstream.iter().map(|n| json!({"id": n.id, "depth": n.depth})).collect::<Vec<_>>(),
        "downstream": radius.downstream.iter().map(|n| json!({"id": n.id, "depth": n.depth})).collect::<Vec<_>>(),
        "element_ids": [],
        "next_suggested_tool": "sruja_get_elements",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_topology_response(&mut response, max_tokens)?;
    Ok(serde_json::to_string_pretty(&response)?)
}

pub(crate) fn collect_topology_element_ids(topology: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    if let Some(t) = topology.get("target").and_then(|v| v.as_str()) {
        ids.push(t.to_string());
    }
    for key in ["upstream", "downstream"] {
        if let Some(arr) = topology.get(key).and_then(|v| v.as_array()) {
            for n in arr {
                if let Some(id) = n.get("id").and_then(|v| v.as_str()) {
                    ids.push(id.to_string());
                }
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

pub(crate) fn build_elements_from_program(
    source_file: &str,
    program: &sruja_language::ast::Program,
    ids: &[String],
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let (elements, _relations) = sruja_language::collect_elements(program);
    let mut all_ids = elements.keys().cloned().collect::<Vec<_>>();
    all_ids.sort();

    let mut out = Vec::new();
    let mut warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };
    for req in ids {
        let resolved = resolve_id_best_effort(req, &all_ids);
        push_resolution_warnings(&mut warnings, req, &resolved);
        let Some(elem) = elements.get(&resolved.id) else {
            out.push(json!({ "id": resolved.id, "requested_id": req, "missing": true }));
            continue;
        };
        let title = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| resolved.id.clone());
        let (
            description,
            technology,
            owner,
            domain,
            criticality,
            gotchas,
            runbooks,
            sources,
            constraint_count,
            convention_count,
        ) = elem
            .assignment
            .body
            .as_ref()
            .map(|b| {
                let sources = b
                    .sources
                    .iter()
                    .take(5)
                    .map(|s| {
                        json!({
                            "kind": s.kind.as_str(),
                            "path": s.path,
                            "description": s.description
                        })
                    })
                    .collect::<Vec<_>>();
                (
                    b.description.clone(),
                    b.technology.clone(),
                    b.owner.clone(),
                    b.domain.clone(),
                    b.criticality.as_ref().map(|c| c.as_str().to_string()),
                    b.gotchas.iter().take(5).cloned().collect::<Vec<_>>(),
                    b.runbooks.iter().take(5).cloned().collect::<Vec<_>>(),
                    sources,
                    b.constraints.len(),
                    b.conventions.len(),
                )
            })
            .unwrap_or((
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                0,
                0,
            ));

        out.push(json!({
            "id": resolved.id,
            "kind": elem.assignment.kind.to_string(),
            "title": title,
            "description": description,
            "technology": technology,
            "tags": elem.assignment.tag_refs,
            "owner": owner,
            "domain": domain,
            "criticality": criticality,
            "gotchas": gotchas,
            "runbooks": runbooks,
            "constraint_count": constraint_count,
            "convention_count": convention_count,
            "sources": sources
        }));
    }

    out.sort_by(|a, b| {
        a.get("id")
            .and_then(|v| v.as_str())
            .cmp(&b.get("id").and_then(|v| v.as_str()))
    });

    let mut response = json!({
        "schema_version": "elements/v1",
        "source": { "kind": "dsl", "file": source_file },
        "requested_ids": ids,
        "elements": out,
        "element_ids": [],
        "next_suggested_tool": "sruja_get_task_context",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_ladder_response(
        &mut response,
        max_tokens,
        &["elements", "element_ids"],
        Some("elements"),
    )?;
    Ok(serde_json::to_string_pretty(&response)?)
}

pub(crate) fn build_elements_from_scan(
    graph: &sruja_scan::Graph,
    ids: &[String],
    max_tokens: usize,
    warning: Option<&str>,
) -> Result<String, CliError> {
    let mut by_id: HashMap<&str, &sruja_scan::Node> = HashMap::new();
    for n in &graph.nodes {
        by_id.insert(n.id.as_str(), n);
    }

    let mut all_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    all_ids.sort();

    let mut out = Vec::new();
    let mut warnings = match warning {
        Some(w) => vec![w.to_string()],
        None => Vec::new(),
    };
    for id in ids {
        let resolved = resolve_id_best_effort(id, &all_ids);
        push_resolution_warnings(&mut warnings, id, &resolved);
        if let Some(n) = by_id.get(resolved.id.as_str()) {
            out.push(json!({
                "id": n.id,
                "kind": n.kind.as_str(),
                "title": n.label,
                "description": n.path,
                "technology": n.technology,
                "tags": [],
                "owner": null,
                "domain": null,
                "criticality": null,
                "gotchas": [],
                "runbooks": [],
                "constraint_count": 0,
                "convention_count": 0,
                "sources": n.sources
            }));
        } else {
            out.push(json!({ "id": resolved.id, "requested_id": id, "missing": true }));
        }
    }

    let mut response = json!({
        "schema_version": "elements/v1",
        "source": { "kind": "scan" },
        "requested_ids": ids,
        "elements": out,
        "element_ids": [],
        "next_suggested_tool": "sruja_get_task_context",
        "truncated": false,
        "estimated_tokens": 0,
        "warnings": warnings
    });

    finalize_ladder_response(
        &mut response,
        max_tokens,
        &["elements", "element_ids"],
        Some("elements"),
    )?;
    Ok(serde_json::to_string_pretty(&response)?)
}

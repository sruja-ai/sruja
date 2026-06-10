use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::helpers::*;
use super::finish;
use crate::commands::CliError;

fn matches_query(node: &sruja_scan::graph::Node, tokens: &[&str]) -> bool {
    let id = node.id.to_lowercase();
    let label = node.label.to_lowercase();
    let path = node.path.as_deref().unwrap_or("").to_lowercase();
    tokens.iter().all(|t| {
        id.contains(t) || label.contains(t) || (!path.is_empty() && path.contains(t))
    })
}

pub(crate) async fn try_run(
    name: &str,
    arguments: &Value,
    repo: &str,
    graph_cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
) -> Result<Option<String>, CliError> {
    let _run_id = arguments.get("run_id").and_then(|v| v.as_str());
    match name {
        "sruja_propose_change" => {
            let description = arguments
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let mut add_elements = Vec::new();
            if let Some(elements) = arguments.get("add_elements").and_then(|v| v.as_array()) {
                for e in elements {
                    let id = e.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                    let label = e.get("label").and_then(|v| v.as_str()).unwrap_or("");
                    let tech = e.get("technology").and_then(|v| v.as_str()).unwrap_or("");
                    add_elements.push(format!("{}:{}:{}:{}", id, kind, label, tech));
                }
            }

            let mut add_relationships = Vec::new();
            if let Some(rels) = arguments
                .get("add_relationships")
                .and_then(|v| v.as_array())
            {
                for r in rels {
                    let source = r.get("source").and_then(|v| v.as_str()).unwrap_or("");
                    let target = r.get("target").and_then(|v| v.as_str()).unwrap_or("");
                    let label = r.get("label").and_then(|v| v.as_str()).unwrap_or("");
                    add_relationships.push(format!("{}->{}:{}", source, target, label));
                }
            }

            let mut remove_elements = Vec::new();
            if let Some(elements) = arguments.get("remove_elements").and_then(|v| v.as_array()) {
                for e in elements {
                    if let Some(id) = e.as_str() {
                        remove_elements.push(id.to_string());
                    }
                }
            }

            crate::commands::propose_create(
                repo,
                crate::commands::ProposeCreateRequest {
                    description: description.to_string(),
                    workflow_id: None,
                    add_elements,
                    add_relationships,
                    remove_elements,
                    remove_relationships: Vec::new(),
                    format: "text".to_string(),
                },
            )
            .await?;
            finish(Ok(
                "Proposal created successfully. Human review required via CLI.".to_string(),
            ))
        }

        "sruja_add_element" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let kind = arguments
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing kind"))?;
            let title = arguments
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing title"))?;
            let description = arguments.get("description").and_then(|v| v.as_str());
            let technology = arguments.get("technology").and_then(|v| v.as_str());

            add_element(repo, id, kind, title, description, technology).await?;
            finish(Ok(format!("Added {} {} to architecture", kind, id)))
        }

        "sruja_add_relationship" => {
            let source = arguments
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing source"))?;
            let target = arguments
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing target"))?;
            let label = arguments.get("label").and_then(|v| v.as_str());
            let technology = arguments.get("technology").and_then(|v| v.as_str());

            add_relationship(repo, source, target, label, technology).await?;
            finish(Ok(format!("Added relationship {} -> {}", source, target)))
        }

        "sruja_get_quick_context" => {
            let graph = get_or_scan_graph(graph_cache, repo).await?;

            let centrality = sruja_scan::graph::centrality::compute_all_centrality(&graph);
            let mut top_modules: Vec<_> = centrality.iter().collect();
            top_modules.sort_by(|a, b| b.1.pagerank.total_cmp(&a.1.pagerank));

            let entrypoints: Vec<&String> = graph
                .nodes
                .iter()
                .filter(|n| !graph.edges.iter().any(|e| e.target == n.id))
                .map(|n| &n.id)
                .collect();

            let stores: Vec<Value> = graph
                .nodes
                .iter()
                .filter(|n| n.kind.as_str() == "database" || n.kind.as_str() == "queue")
                .map(|n| {
                    json!({
                        "id": n.id,
                        "kind": n.kind.as_str()
                    })
                })
                .collect();

            let summary = json!({
                "total_nodes": graph.nodes.len(),
                "total_edges": graph.edges.len(),
                "top_modules": top_modules.iter().take(5).map(|(id, s)| {
                    json!({
                        "id": id,
                        "pagerank": s.pagerank
                    })
                }).collect::<Vec<_>>(),
                "entrypoints": entrypoints,
                "data_stores": stores,
                "auto_context": {
                    "services": graph.auto_context.services_from_compose,
                    "ci_pipelines": graph.auto_context.ci_pipelines,
                    "infra": graph.auto_context.infra_dependencies
                }
            });

            finish(Ok(serde_json::to_string_pretty(&summary)?))
        }

        "sruja_get_evidence_graph" => {
            let max_nodes = arguments
                .get("max_nodes")
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as usize;
            let max_edges = arguments
                .get("max_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(100) as usize;
            let graph = get_or_scan_graph(graph_cache, repo).await?;

            let nodes = graph
                .nodes
                .iter()
                .take(max_nodes)
                .map(|n| {
                    json!({
                        "id": n.id,
                        "kind": n.kind.as_str(),
                        "label": n.label,
                        "path": n.path,
                    })
                })
                .collect::<Vec<_>>();

            let edges = graph
                .edges
                .iter()
                .take(max_edges)
                .map(|e| {
                    json!({
                        "source": e.source,
                        "target": e.target,
                        "kind": format!("{:?}", e.kind),
                        "confidence": format!("{:?}", e.confidence),
                    })
                })
                .collect::<Vec<_>>();

            let out = json!({
                "schema_version": "evidence_graph/v1",
                "repo": repo,
                "node_count": graph.nodes.len(),
                "edge_count": graph.edges.len(),
                "nodes": nodes,
                "edges": edges,
                "truncated": graph.nodes.len() > max_nodes || graph.edges.len() > max_edges,
            });
            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        "sruja_query_graph" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;
            let q = query.to_lowercase();
            let tokens: Vec<&str> = q.split_whitespace().filter(|t| !t.is_empty()).collect();
            let graph = get_or_scan_graph(graph_cache, repo).await?;

            let mut matched = Vec::new();
            for n in &graph.nodes {
                if tokens.is_empty() {
                    break;
                }
                if matches_query(n, &tokens) {
                    matched.push(json!({
                        "id": n.id,
                        "label": n.label,
                        "kind": n.kind.as_str(),
                        "path": n.path,
                    }));
                    if matched.len() >= 25 {
                        break;
                    }
                }
            }

            let mut ids = std::collections::HashSet::new();
            for m in &matched {
                if let Some(id) = m.get("id").and_then(|v| v.as_str()) {
                    ids.insert(id.to_string());
                }
            }

            let mut relationships = Vec::new();
            for e in &graph.edges {
                if ids.contains(&e.source) && ids.contains(&e.target) {
                    relationships.push(json!({
                        "source": e.source,
                        "target": e.target,
                        "kind": format!("{:?}", e.kind),
                        "confidence": format!("{:?}", e.confidence),
                    }));
                    if relationships.len() >= 50 {
                        break;
                    }
                }
            }

            let out = json!({
                "schema_version": "query_graph/v1",
                "query": query,
                "matched_nodes": matched,
                "relationships": relationships,
            });
            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        "sruja_explain_element" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == id)
                .ok_or_else(|| CliError::validation(format!("Element not found: {}", id)))?;

            let mut upstream = Vec::new();
            let mut downstream = Vec::new();
            for e in &graph.edges {
                if e.target == id {
                    upstream.push(json!({
                        "id": e.source,
                        "kind": format!("{:?}", e.kind),
                        "confidence": format!("{:?}", e.confidence),
                    }));
                } else if e.source == id {
                    downstream.push(json!({
                        "id": e.target,
                        "kind": format!("{:?}", e.kind),
                        "confidence": format!("{:?}", e.confidence),
                    }));
                }
            }

            let notes = json!([
                format!("incoming: {}", upstream.len()),
                format!("outgoing: {}", downstream.len()),
            ]);

            let out = json!({
                "schema_version": "explain_element/v1",
                "element": {
                    "id": node.id,
                    "label": node.label,
                    "kind": node.kind.as_str(),
                    "path": node.path,
                },
                "neighbors": {
                    "upstream": upstream,
                    "downstream": downstream,
                },
                "notes": notes,
            });
            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        "sruja_find_path" => {
            let source = arguments
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing source"))?;
            let target = arguments
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing target"))?;
            let max_depth = arguments
                .get("max_depth")
                .and_then(|v| v.as_u64())
                .unwrap_or(8) as usize;
            let graph = get_or_scan_graph(graph_cache, repo).await?;

            let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
            for e in &graph.edges {
                adjacency
                    .entry(e.source.clone())
                    .or_default()
                    .push(e.target.clone());
            }

            use std::collections::{HashSet, VecDeque};
            let mut visited: HashSet<String> = HashSet::new();
            let mut prev: HashMap<String, String> = HashMap::new();
            let mut depth_map: HashMap<String, usize> = HashMap::new();
            let mut q: VecDeque<String> = VecDeque::new();

            visited.insert(source.to_string());
            depth_map.insert(source.to_string(), 0);
            q.push_back(source.to_string());

            while let Some(cur) = q.pop_front() {
                let depth = *depth_map.get(&cur).unwrap_or(&0);
                if cur == target || depth >= max_depth {
                    continue;
                }
                if let Some(nexts) = adjacency.get(&cur) {
                    for next in nexts {
                        if visited.insert(next.clone()) {
                            prev.insert(next.clone(), cur.clone());
                            depth_map.insert(next.clone(), depth + 1);
                            q.push_back(next.clone());
                        }
                    }
                }
            }

            let mut path = Vec::new();
            if !visited.contains(target) {
                let md = format!(
                    "# Path from {source} to {target}\n\nNo path found (max_depth={max_depth}).\n"
                );
                return finish(Ok(md));
            }

            let mut cur = target.to_string();
            path.push(cur.clone());
            while let Some(p) = prev.get(&cur).cloned() {
                cur = p;
                path.push(cur.clone());
                if cur == source {
                    break;
                }
            }
            path.reverse();

            let md = format!(
                "# Path from {source} to {target}\n\n{}\n",
                path.join(" -> ")
            );
            finish(Ok(md))
        }
        _ => Ok(None),
    }
}

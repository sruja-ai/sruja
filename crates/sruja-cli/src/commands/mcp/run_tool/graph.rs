use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::helpers::*;
use super::finish;
use crate::commands::CliError;

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
        _ => Ok(None),
    }
}

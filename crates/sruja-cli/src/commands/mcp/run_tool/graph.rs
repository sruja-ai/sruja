use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
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
        "sruja_get_operational_context" => {
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let graph = get_or_scan_graph(graph_cache, repo).await?;

            let mut out = "# Operational Context\n\n".to_string();

            if let Some(id) = element_id {
                if let Some(node) = graph.nodes.iter().find(|n| n.id == id) {
                    out.push_str(&format!("## {}\n", id));
                    if !node.gotchas.is_empty() {
                        out.push_str("### Gotchas\n");
                        for g in &node.gotchas {
                            out.push_str(&format!("- {}\n", g));
                        }
                    }
                    if !node.operational_constraints.is_empty() {
                        out.push_str("### Constraints\n");
                        for c in &node.operational_constraints {
                            out.push_str(&format!("- {}\n", c));
                        }
                    }
                    if !node.runbooks.is_empty() {
                        out.push_str("### Runbooks\n");
                        for r in &node.runbooks {
                            out.push_str(&format!("- {}\n", r));
                        }
                    }
                } else {
                    return Err(CliError::validation(format!("Element not found: {}", id)));
                }
            } else {
                out.push_str("## Recent Incidents\n");
                if graph.incidents.is_empty() {
                    out.push_str("No incidents recorded.\n");
                } else {
                    for inc in &graph.incidents {
                        out.push_str(&format!("### {} - {}\n", inc.id, inc.title));
                        if let Some(s) = &inc.severity {
                            out.push_str(&format!("- **Severity**: {}\n", s));
                        }
                        if let Some(d) = &inc.date {
                            out.push_str(&format!("- **Date**: {}\n", d));
                        }
                        if !inc.affected.is_empty() {
                            out.push_str("- **Affected**: ");
                            out.push_str(&inc.affected.join(", "));
                            out.push('\n');
                        }
                        if let Some(c) = &inc.cause {
                            out.push_str(&format!("- **Cause**: {}\n", c));
                        }
                        if let Some(r) = &inc.resolution {
                            out.push_str(&format!("- **Resolution**: {}\n", r));
                        }
                        if let Some(l) = &inc.lesson {
                            out.push_str(&format!("- **Lesson**: {}\n", l));
                        }
                        out.push('\n');
                    }
                }

                out.push_str("\n## Tribal Knowledge (Gotchas & Constraints)\n");
                let mut found = false;
                for node in &graph.nodes {
                    if !node.gotchas.is_empty() || !node.operational_constraints.is_empty() {
                        found = true;
                        out.push_str(&format!("### {}\n", node.id));
                        for g in &node.gotchas {
                            out.push_str(&format!("- [Gotcha] {}\n", g));
                        }
                        for c in &node.operational_constraints {
                            out.push_str(&format!("- [Constraint] {}\n", c));
                        }
                    }
                }
                if !found {
                    out.push_str("No specific tribal knowledge recorded for elements.\n");
                }
            }

            finish(Ok(out))
        }

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
                description,
                None,
                add_elements,
                add_relationships,
                remove_elements,
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

        "sruja_get_system_context" => {
            let start = std::path::Path::new(&repo);
            match crate::commands::federation::find_system_index(start) {
                Some(index_path) => {
                    let index = crate::commands::federation::load_system_index(&index_path)?;
                    let summary = format!(
                        "System index: {} repos, {} nodes, {} edges, {} conflicts\nSource: {}\n\n",
                        index.repos.len(),
                        index.nodes.len(),
                        index.edges.len(),
                        index.conflicts.len(),
                        index_path.display()
                    );
                    let json = serde_json::to_string_pretty(&index)
                        .map_err(|e| CliError::validation(e.to_string()))?;
                    finish(Ok(format!("{}{}", summary, json)))
                }
                None => finish(Ok("No system.index.json found. Run `sruja compose` to create a multi-repo system index.".to_string())),
            }
        }

        "sruja_list_elements" => {
            let start = std::path::Path::new(&repo);
            match crate::commands::federation::find_system_index(start) {
                Some(index_path) => {
                    let index = crate::commands::federation::load_system_index(&index_path)?;
                    let filtered = match arguments.get("kind").and_then(|v| v.as_str()) {
                        Some(kind) => crate::commands::federation::filter_system_index_by_kind(&index, kind),
                        None => index,
                    };
                    let kind_label = arguments
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or("all");
                    let mut out = format!(
                        "Found {} {} element(s) across {} repo(s)\n\n",
                        filtered.nodes.len(),
                        kind_label,
                        filtered.repos.len()
                    );
                    for node in &filtered.nodes {
                        out.push_str(&format!(
                            "- [{}] {} ({}){}\n  repo: {}\n",
                            node.kind,
                            node.label,
                            node.canonical_id,
                            node.technology
                                .as_ref()
                                .map(|t| format!(" [{}]", t))
                                .unwrap_or_default(),
                            node.repo_id
                        ));
                    }
                    if !filtered.edges.is_empty() {
                        out.push_str(&format!("\n{} relationship(s):\n", filtered.edges.len()));
                        for edge in &filtered.edges {
                            out.push_str(&format!(
                                "  {} -> {} {}\n",
                                edge.source,
                                edge.target,
                                edge.label.as_deref().unwrap_or("")
                            ));
                        }
                    }
                    if !filtered.conflicts.is_empty() {
                        out.push_str(&format!("\n⚠ {} conflict(s):\n", filtered.conflicts.len()));
                        for c in &filtered.conflicts {
                            out.push_str(&format!("  {}: {}\n", c.key, c.message));
                        }
                    }
                    finish(Ok(out))
                }
                None => finish(Ok("No system.index.json found. Run `sruja compose` to create a multi-repo system index.".to_string())),
            }
        }

        "sruja_get_hydrated_context" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(20000) as usize;
            let enrich = arguments
                .get("enrich")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let enrich_provider = arguments.get("enrich_provider").and_then(|v| v.as_str());
            let enrich_cmd = arguments.get("enrich_cmd").and_then(|v| v.as_str());
            let enrich_model = arguments.get("enrich_model").and_then(|v| v.as_str());
            let enrich_base_url = arguments.get("enrich_base_url").and_then(|v| v.as_str());
            let enrich_timeout_ms = arguments
                .get("enrich_timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(15000);
            let enrich_max_bytes = arguments
                .get("enrich_max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(20000) as usize;

            let hydrated = get_hydrated_context(repo, id, max_tokens, graph_cache).await?;
            if !enrich && enrich_cmd.is_none() {
                return finish(Ok(hydrated));
            }

            let wrapped = enrich_wrapper_json(
                Path::new(&repo),
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                "hydrated_context",
                json!({ "markdown": hydrated }),
            );
            finish(Ok(serde_json::to_string_pretty(&wrapped)?))
        }

        "sruja_semantic_search" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;
            let top_k = arguments.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

            let vector_path = std::path::Path::new(&repo)
                .join(".sruja")
                .join("vectors.json");
            if !vector_path.exists() {
                return finish(Ok("Semantic index not found. Please run `sruja index` first to generate embeddings.".to_string()));
            }

            let index_json = tokio::fs::read_to_string(&vector_path).await?;
            let index: sruja_export::vector::VectorIndex = serde_json::from_str(&index_json)?;

            let mut searcher = sruja_export::vector::SemanticSearcher::new().map_err(|e| {
                CliError::Io(std::io::Error::other(format!(
                    "Failed to init searcher: {}",
                    e
                )))
            })?;

            let results = searcher.search(&index, query, top_k).map_err(|e| {
                CliError::Io(std::io::Error::other(format!("Search failed: {}", e)))
            })?;

            let mut out = format!("# Semantic Search Results for: \"{}\"\n\n", query);
            if results.is_empty() {
                out.push_str("No matching components found.\n");
            } else {
                for (id, score) in results {
                    let node = index.nodes.iter().find(|n| n.id == id);
                    let label = node.map(|n| n.label.as_str()).unwrap_or(&id);
                    let desc = node.map(|n| n.description.as_str()).unwrap_or("");
                    out.push_str(&format!(
                        "- **{}** (Score: {:.2})\n  ID: {}\n  Description: {}\n",
                        label, score, id, desc
                    ));
                }
            }
            finish(Ok(out))
        }

        "sruja_query_graph" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;
            let enrich = arguments
                .get("enrich")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let enrich_provider = arguments.get("enrich_provider").and_then(|v| v.as_str());
            let enrich_cmd = arguments.get("enrich_cmd").and_then(|v| v.as_str());
            let enrich_model = arguments.get("enrich_model").and_then(|v| v.as_str());
            let enrich_base_url = arguments.get("enrich_base_url").and_then(|v| v.as_str());
            let enrich_timeout_ms = arguments
                .get("enrich_timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(15000);
            let enrich_max_bytes = arguments
                .get("enrich_max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(20000) as usize;

            // Adaptive Hybrid Retrieval: single graph load for both classification and metadata
            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let complexity = sruja_graph::classify_query(query, &kg);
            let vector_path = std::path::Path::new(&repo)
                .join(".sruja")
                .join("vectors.json");
            let has_semantic_index = vector_path.exists();
            let strategy = sruja_graph::select_strategy(complexity, has_semantic_index);

            let semantic_results = match strategy {
                sruja_graph::RetrievalStrategy::GraphOnly => Vec::new(),
                _ => {
                    if has_semantic_index {
                        let index_json = tokio::fs::read_to_string(&vector_path).await?;
                        let index: sruja_export::vector::VectorIndex =
                            serde_json::from_str(&index_json)?;
                        let mut searcher =
                            sruja_export::vector::SemanticSearcher::new().map_err(|e| {
                                CliError::Io(std::io::Error::other(format!(
                                    "Failed to init searcher: {}",
                                    e
                                )))
                            })?;
                        searcher.search(&index, query, 5).unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                }
            };

            let hybrid = sruja_graph::execute_hybrid(
                &kg,
                query,
                semantic_results
                    .iter()
                    .map(|(id, score)| sruja_graph::SemanticCandidate {
                        element_id: id.clone(),
                        score: *score,
                        label: None,
                    })
                    .collect(),
            );

            let mut matched_nodes = Vec::new();
            let mut relations = Vec::new();
            let mut seen_nodes = std::collections::HashSet::new();

            let push_kg_node =
                |node: &sruja_graph::ArchitectureNode, score: f32, matched: &mut Vec<Value>| {
                    matched.push(json!({
                        "id": node.id,
                        "label": node.label,
                        "kind": node.kind.as_str(),
                        "score": score,
                        "description": node.description.as_deref()
                    }));
                };

            for candidate in &hybrid.semantic_candidates {
                if let Some(node) = kg.nodes.get(&candidate.element_id) {
                    push_kg_node(node, candidate.score, &mut matched_nodes);
                    seen_nodes.insert(node.id.clone());
                }
            }

            if let Some(ref gr) = hybrid.graph_result {
                for ev in &gr.evidence {
                    if seen_nodes.insert(ev.reference.clone()) {
                        if let Some(node) = kg.nodes.get(&ev.reference) {
                            push_kg_node(node, 1.0, &mut matched_nodes);
                        }
                    }
                }
            }

            // 1-depth neighbor expansion via KnowledgeGraph edges
            let seed_ids: Vec<String> = seen_nodes.iter().cloned().collect();
            for seed_id in &seed_ids {
                for edge in kg.get_edges_from(seed_id) {
                    if seen_nodes.insert(edge.target.clone()) {
                        if let Some(node) = kg.nodes.get(&edge.target) {
                            push_kg_node(node, 0.0, &mut matched_nodes);
                        }
                    }
                }
                for edge in kg.get_edges_to(seed_id) {
                    if seen_nodes.insert(edge.source.clone()) {
                        if let Some(node) = kg.nodes.get(&edge.source) {
                            push_kg_node(node, 0.0, &mut matched_nodes);
                        }
                    }
                }
            }

            for edge in &kg.edges {
                if seen_nodes.contains(&edge.source) && seen_nodes.contains(&edge.target) {
                    relations.push(json!({
                        "source": edge.source,
                        "target": edge.target,
                        "kind": edge.kind.as_str()
                    }));
                }
            }

            let grounded = json!({
                "query": query,
                "complexity": format!("{:?}", complexity),
                "strategy": format!("{:?}", strategy),
                "matched_nodes": matched_nodes,
                "relationships": relations
            });

            if !enrich && enrich_cmd.is_none() {
                return finish(Ok(serde_json::to_string_pretty(&grounded)?));
            }

            let wrapped = enrich_wrapper_json(
                Path::new(&repo),
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                "query_graph",
                grounded,
            );
            finish(Ok(serde_json::to_string_pretty(&wrapped)?))
        }

        "sruja_explain_element" => {
            let element_id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let enrich = arguments
                .get("enrich")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let enrich_provider = arguments.get("enrich_provider").and_then(|v| v.as_str());
            let enrich_cmd = arguments.get("enrich_cmd").and_then(|v| v.as_str());
            let enrich_model = arguments.get("enrich_model").and_then(|v| v.as_str());
            let enrich_base_url = arguments.get("enrich_base_url").and_then(|v| v.as_str());
            let enrich_timeout_ms = arguments
                .get("enrich_timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(15000);
            let enrich_max_bytes = arguments
                .get("enrich_max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(20000) as usize;

            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == element_id)
                .ok_or_else(|| CliError::validation(format!("Element {} not found", element_id)))?;

            // Compute centrality (PageRank)
            let centrality = sruja_scan::graph::compute_all_centrality(&graph);
            let pr = centrality.get(&node.id).map(|s| s.pagerank).unwrap_or(0.0);

            // Immediate neighbors
            let radius = graph.blast_radius(&node.id, 1);
            let upstream: Vec<String> = radius.upstream.iter().map(|u| u.id.clone()).collect();
            let downstream: Vec<String> = radius.downstream.iter().map(|d| d.id.clone()).collect();

            // Notes / Explanatory comments from Comment discovery (explained by edges)
            let mut notes = Vec::new();
            for edge in &graph.edges {
                if edge.target == node.id && edge.kind.kind_str() == "explains" {
                    if let Some(src) = graph.nodes.iter().find(|n| n.id == edge.source) {
                        notes.push(json!({
                            "id": src.id,
                            "label": src.label,
                            "description": src.metadata.get("description").cloned()
                        }));
                    }
                }
            }

            // Compute community
            let raw_communities = sruja_scan::detect_communities(&graph);
            let community_infos = sruja_scan::summarize_communities(&graph, &raw_communities);
            let element_community = raw_communities.get(element_id).cloned();
            let community_detail = element_community.and_then(|cid| {
                community_infos.iter().find(|c| c.id == cid).map(|c| {
                    json!({
                        "id": c.id,
                        "suggested_label": c.suggested_label,
                        "cohesion": c.cohesion,
                        "member_count": c.member_count
                    })
                })
            });

            let grounded = json!({
                "element": {
                    "id": node.id,
                    "label": node.label,
                    "kind": node.kind.as_str(),
                    "pagerank": pr,
                    "description": node.metadata.get("description").cloned(),
                    "technology": node.technology,
                    "path": node.path,
                    "community": community_detail
                },
                "neighbors": {
                    "upstream": upstream,
                    "downstream": downstream
                },
                "notes": notes
            });

            if !enrich && enrich_cmd.is_none() {
                return finish(Ok(serde_json::to_string_pretty(&grounded)?));
            }

            let wrapped = enrich_wrapper_json(
                Path::new(&repo),
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                "explain_element",
                grounded,
            );
            finish(Ok(serde_json::to_string_pretty(&wrapped)?))
        }
        _ => Ok(None),
    }
}

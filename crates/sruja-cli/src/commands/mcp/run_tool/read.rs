use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::helpers::*;
use super::super::ladder::*;
use super::finish;
use crate::commands::{agent_run_to_string, AgentRunOptions, CliError};

pub(crate) async fn try_run(
    name: &str,
    arguments: &Value,
    repo: &str,
    graph_cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
) -> Result<Option<String>, CliError> {
    let run_id = arguments.get("run_id").and_then(|v| v.as_str());
    match name {
        "sruja_get_repomap" => {
            let repomap = crate::commands::discover::discover_repomap(repo, 100, 5000)?;
            finish(Ok(repomap))
        }

        "sruja_list_architecture_index" => {
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(2000)
                .max(200) as usize;
            let kinds: Option<Vec<String>> = arguments.get("kinds").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
            });

            let repo_path = Path::new(&repo);
            let (arch, warning) = load_architecture_program_best_effort(repo_path);
            let out = if let Some((source_file, program)) = arch {
                build_architecture_index_from_program(
                    &source_file,
                    &program,
                    kinds.as_deref(),
                    max_tokens,
                    warning.as_deref(),
                )?
            } else {
                let graph = get_or_scan_graph(graph_cache, repo).await?;
                build_architecture_index_from_scan(
                    &graph,
                    kinds.as_deref(),
                    max_tokens,
                    warning.as_deref(),
                )?
            };
            finish(Ok(out))
        }

        "sruja_get_topology" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let depth = arguments
                .get("depth")
                .and_then(|v| v.as_u64())
                .unwrap_or(1)
                .clamp(1, 4) as usize;
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(5000)
                .max(500) as usize;

            let repo_path = Path::new(&repo);
            let (arch, warning) = load_architecture_program_best_effort(repo_path);
            let out = if let Some((source_file, program)) = arch {
                build_topology_from_program(
                    &source_file,
                    &program,
                    id,
                    depth,
                    max_tokens,
                    warning.as_deref(),
                )?
            } else {
                let graph = get_or_scan_graph(graph_cache, repo).await?;
                build_topology_from_scan(&graph, id, depth, max_tokens, warning.as_deref())?
            };
            finish(Ok(out))
        }

        "sruja_get_elements" => {
            let ids = arguments
                .get("ids")
                .and_then(|v| v.as_array())
                .ok_or_else(|| CliError::validation("Missing ids"))?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();
            if ids.is_empty() {
                return Err(CliError::validation("ids must be non-empty"));
            }
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(8000)
                .max(500) as usize;

            let repo_path = Path::new(&repo);
            let (arch, warning) = load_architecture_program_best_effort(repo_path);
            let out = if let Some((source_file, program)) = arch {
                build_elements_from_program(
                    &source_file,
                    &program,
                    &ids,
                    max_tokens,
                    warning.as_deref(),
                )?
            } else {
                let graph = get_or_scan_graph(graph_cache, repo).await?;
                build_elements_from_scan(&graph, &ids, max_tokens, warning.as_deref())?
            };
            finish(Ok(out))
        }

        "sruja_get_diagnostic_full" => {
            let uri = arguments
                .get("uri")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing uri"))?;
            let content =
                crate::commands::diagnostic_vfs::read_vfs_diagnostic(Path::new(&repo), uri)?;
            finish(Ok(content))
        }

        "sruja_suggest_context_prune" => {
            let active =
                crate::commands::context_prune::parse_id_list_arg(arguments, "active_element_ids")?;
            let session = crate::commands::context_prune::parse_id_list_arg(
                arguments,
                "session_element_ids",
            )?;
            let depth = arguments.get("depth").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
            let graph = get_or_scan_graph(graph_cache, repo).await?;
            finish(crate::commands::context_prune::suggest_context_prune_json(
                &graph, &active, &session, depth,
            ))
        }

        "sruja_get_drift_state" => {
            let graph = get_or_scan_graph(graph_cache, repo).await?;
            finish(crate::commands::drift_state::build_drift_state_json(
                repo, &graph,
            ))
        }

        "sruja_search_memory" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing query".to_string()))?;
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(20) as usize;
            let store = sruja_memory::MemoryStore::open(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            let hits = store
                .search(sruja_memory::SearchMemoryOptions {
                    query,
                    element_id: arguments.get("element_id").and_then(|v| v.as_str()),
                    decision_id: arguments.get("decision_id").and_then(|v| v.as_str()),
                    hitl_kind: arguments.get("hitl_kind").and_then(|v| v.as_str()),
                    source: None,
                    trust: None,
                    limit,
                })
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            finish(Ok(serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "memory_search/v1",
                "query": query,
                "count": hits.len(),
                "hits": hits,
                "note": "hypothesis vs reviewed_truth; never auto-merge into repo.sruja"
            }))?))
        }

        "sruja_get_memory_timeline" => {
            let store = sruja_memory::MemoryStore::open(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            let tl = store
                .timeline(sruja_memory::TimelineOptions {
                    anchor_id: arguments.get("anchor_id").and_then(|v| v.as_str()),
                    anchor_timestamp: arguments.get("anchor_timestamp").and_then(|v| v.as_str()),
                    before: arguments
                        .get("before")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10) as usize,
                    after: arguments
                        .get("after")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10) as usize,
                    decision_id: arguments.get("decision_id").and_then(|v| v.as_str()),
                    element_id: arguments.get("element_id").and_then(|v| v.as_str()),
                })
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            finish(Ok(serde_json::to_string_pretty(&tl)?))
        }

        "sruja_reindex_memory" => {
            let mut store = sruja_memory::MemoryStore::open(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            store
                .reindex()
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            finish(Ok(
                r#"{"ok":true,"schema_version":"memory_index/v1"}"#.to_string()
            ))
        }

        "sruja_get_architecture_context" => {
            let file = arguments
                .get("file")
                .and_then(|v| v.as_str())
                .map(String::from);
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            let intent = arguments
                .get("intent")
                .and_then(|v| v.as_str())
                .map(String::from);
            let content = crate::commands::context::context_string(
                repo,
                "markdown",
                crate::commands::context::ContextRequest {
                    run_id,
                    file: file.as_deref(),
                    element_id: element_id.as_deref(),
                    query: None,
                    base_ref: None,
                    head_ref: None,
                    intent: intent.as_deref(),
                    depth: 2,
                    max_tokens: 10000,
                    cache_friendly: false,
                },
            )
            .await?;
            finish(Ok(content))
        }

        "sruja_get_architecture_summary" => {
            let content = crate::commands::context::context_string(
                repo,
                "markdown",
                crate::commands::context::ContextRequest {
                    run_id,
                    file: None,
                    element_id: None,
                    query: None,
                    base_ref: None,
                    head_ref: None,
                    intent: None,
                    depth: 1,
                    max_tokens: 3000,
                    cache_friendly: false,
                },
            )
            .await?;
            finish(Ok(content))
        }

        "sruja_get_neighbors" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing id"))?;
            let depth = arguments.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let radius = graph.blast_radius(id, depth);

            let mut out = format!("# Neighbors of {}\n\n", id);
            out.push_str("## Upstream (depend on this)\n");
            if radius.upstream.is_empty() {
                out.push_str("- None\n");
            } else {
                for n in radius.upstream {
                    out.push_str(&format!("- {} (depth: {})\n", n.id, n.depth));
                }
            }

            out.push_str("\n## Downstream (this depends on)\n");
            if radius.downstream.is_empty() {
                out.push_str("- None\n");
            } else {
                for n in radius.downstream {
                    out.push_str(&format!("- {} (depth: {})\n", n.id, n.depth));
                }
            }
            finish(Ok(out))
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

            let graph = get_or_scan_graph(graph_cache, repo).await?;
            match graph.find_path(source, target) {
                Some(path) => finish(Ok(format!(
                    "# Path from {} to {}\n\n{}",
                    source,
                    target,
                    path.join(" -> ")
                ))),
                None => finish(Ok(format!("No path found from {} to {}", source, target))),
            }
        }

        "sruja_agent_run" => {
            let goal = arguments
                .get("goal")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing goal"))?;
            let file = arguments.get("file").and_then(|v| v.as_str());
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let query = arguments.get("query").and_then(|v| v.as_str());
            let mode = arguments
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("plan");
            let ai_mode = arguments
                .get("ai_mode")
                .and_then(|v| v.as_str())
                .unwrap_or("standard");
            let max_steps = arguments
                .get("max_steps")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);
            let max_runtime_ms_per_step = arguments
                .get("max_runtime_ms_per_step")
                .and_then(|v| v.as_u64());
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
                .unwrap_or(15_000);
            let enrich_max_bytes = arguments
                .get("enrich_max_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(20_000) as usize;
            let continue_on_error = arguments
                .get("continue_on_error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let text = agent_run_to_string(AgentRunOptions {
                repo,
                goal,
                file,
                element_id,
                query,
                run_id,
                mode,
                ai_mode,
                format: "for-ai",
                max_steps,
                max_runtime_ms_per_step,
                enrich,
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                continue_on_error,
                trajectories: None,
            })
            .await?;
            finish(Ok(text))
        }

        "sruja_get_entrypoints" => {
            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let mut entrypoints = Vec::new();

            let mut has_incoming = HashMap::new();
            for edge in &graph.edges {
                *has_incoming.entry(edge.target.as_str()).or_insert(0) += 1;
            }

            for node in &graph.nodes {
                let is_high_level = matches!(
                    node.kind.as_str(),
                    sruja_scan::NodeKind::SERVICE
                        | sruja_scan::NodeKind::EXTERNAL_API
                        | sruja_scan::NodeKind::SYSTEM
                );
                let no_incoming = has_incoming.get(node.id.as_str()).cloned().unwrap_or(0) == 0;

                if is_high_level || no_incoming {
                    entrypoints.push(format!("- {} ({})", node.id, node.kind));
                }
            }

            if entrypoints.is_empty() {
                finish(Ok("No clear entrypoints discovered.".to_string()))
            } else {
                entrypoints.sort();
                finish(Ok(format!(
                    "# Architecture Entrypoints\n\n{}",
                    entrypoints.join("\n")
                )))
            }
        }

        "sruja_get_data_stores" => {
            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let mut stores = Vec::new();

            for node in &graph.nodes {
                if matches!(
                    node.kind.as_str(),
                    sruja_scan::NodeKind::DATABASE | sruja_scan::NodeKind::QUEUE
                ) {
                    let tech = node
                        .technology
                        .as_deref()
                        .map(|t| format!(" ({})", t))
                        .unwrap_or_default();
                    stores.push(format!("- {}: {}{}", node.id, node.kind, tech));
                }
            }

            if stores.is_empty() {
                finish(Ok(
                    "No data stores (databases/queues) discovered.".to_string()
                ))
            } else {
                stores.sort();
                finish(Ok(format!(
                    "# Discovered Data Stores\n\n{}",
                    stores.join("\n")
                )))
            }
        }

        "sruja_explain_discovery" => {
            let format = arguments
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("text");
            finish(match format {
                "json" => crate::commands::discover::discover_explanation_json(repo),
                "text" => crate::commands::discover::discover_explanation_string(repo),
                _ => Err(CliError::validation(format!(
                    "Unknown format: {}. Use: text or json",
                    format
                ))),
            })
        }
        _ => Ok(None),
    }
}

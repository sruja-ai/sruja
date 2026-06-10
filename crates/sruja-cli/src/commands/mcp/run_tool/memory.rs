use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::helpers::*;
use super::super::ladder::*;
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
        "sruja_get_context_events" => {
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as usize;
            let kind = arguments.get("kind").and_then(|v| v.as_str());
            let sub = arguments.get("details_substring").and_then(|v| v.as_str());
            let decision_id = arguments.get("decision_id").and_then(|v| v.as_str());
            let trace_id = arguments.get("trace_id").and_then(|v| v.as_str());
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let decision_lineage_only = arguments
                .get("decision_lineage_only")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let events = crate::commands::context_events::read_context_events_query(
                Path::new(&repo),
                crate::commands::context_events::ContextEventQuery {
                    limit,
                    kind_filter: kind,
                    details_substring: sub,
                    decision_id,
                    trace_id,
                    run_id: None,
                    element_id,
                    decision_lineage_only,
                },
            )?;
            finish(Ok(serde_json::to_string_pretty(&events)?))
        }

        "sruja_get_decisions" => {
            let items = crate::commands::list_decisions(Path::new(&repo))?;
            finish(Ok(serde_json::to_string_pretty(&items)?))
        }

        "sruja_get_learned_facts" => {
            let path = Path::new(&repo)
                .join(".sruja")
                .join("agent_memory.json");
            if !path.exists() {
                let out = json!({
                    "schema_version": "learned_facts/v1",
                    "repo": repo,
                    "path": path.display().to_string(),
                    "agent_memory": Value::Null,
                });
                return finish(Ok(serde_json::to_string_pretty(&out)?));
            }
            let content = std::fs::read_to_string(&path)?;
            let val: Value =
                serde_json::from_str(&content).map_err(|e| CliError::validation(e.to_string()))?;
            let out = json!({
                "schema_version": "learned_facts/v1",
                "repo": repo,
                "path": path.display().to_string(),
                "agent_memory": val,
            });
            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        "sruja_record_context_event" => {
            let ev = arguments
                .get("event")
                .ok_or_else(|| CliError::validation("missing event object".to_string()))?;
            let line = serde_json::to_string(ev)?;
            crate::commands::context_events::append_context_event_from_json_line(
                Path::new(&repo),
                &line,
            )
            .map_err(CliError::validation)?;
            finish(Ok(r#"{"ok":true}"#.to_string()))
        }

        "sruja_record_decision_event" => {
            let ev = arguments
                .get("event")
                .ok_or_else(|| CliError::validation("missing event object".to_string()))?;
            let line = serde_json::to_string(ev)?;
            crate::commands::context_events::append_context_event_from_json_line(
                Path::new(&repo),
                &line,
            )
            .map_err(CliError::validation)?;
            finish(Ok(r#"{"ok":true}"#.to_string()))
        }

        "sruja_create_decision_record" => {
            let title = arguments
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing title".to_string()))?;
            let record_type = arguments
                .get("record_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing record_type".to_string()))?;
            let scope = arguments.get("scope").and_then(|v| v.as_str());
            let id = crate::commands::create_decision_record(
                Path::new(&repo),
                title,
                record_type,
                scope,
                "sruja_create_decision_record",
                "agent",
                "mcp",
            )?;
            finish(Ok(serde_json::json!({ "id": id }).to_string()))
        }

        "sruja_link_decision_to_element" => {
            let decision_id = arguments
                .get("decision_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing decision_id".to_string()))?;
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing element_id".to_string()))?;
            crate::commands::decision::decision_link(repo, decision_id, element_id).await?;
            finish(Ok(r#"{"ok":true}"#.to_string()))
        }

        "sruja_critique" => {
            let files: Vec<String> = arguments
                .get("files")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let description = arguments
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);
            let proposal_id = arguments
                .get("proposal_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            let base_ref = arguments
                .get("base_ref")
                .and_then(|v| v.as_str())
                .map(String::from);
            let head_ref = arguments
                .get("head_ref")
                .and_then(|v| v.as_str())
                .map(String::from);

            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let baseline_path =
                crate::utils::architecture_path::resolve_architecture_path(Path::new(&repo));
            let program = if let Some(path) = baseline_path {
                let content = std::fs::read_to_string(path).map_err(CliError::Io)?;
                let parser = sruja_language::Parser::new(repo);
                parser.parse(&content).ok()
            } else {
                None
            };

            let engine = sruja_intent::CritiqueEngine::new(graph, program);
            let report = engine.critique(&sruja_intent::CritiqueRequest {
                changed_files: files,
                description,
                proposal_id,
                base_ref,
                head_ref,
            });

            finish(Ok(sruja_intent::format_critique_json(&report)))
        }

        "sruja_search_memory" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;
            let cfg_default = crate::integrations::load_repo_config(Path::new(&repo))
                .and_then(|c| c.context_engineering.bm25_max_results_mcp)
                .unwrap_or(5);
            let max_results = arguments
                .get("max_results")
                .and_then(|v| v.as_u64())
                .unwrap_or(cfg_default as u64) as usize;

            let index = sruja_graph::SparseIndex::build(Path::new(&repo));
            let hits = index.search(query, max_results);

            let out = json!({
                "query": query,
                "doc_count": index.doc_count(),
                "results": hits.iter().map(|h| json!({
                    "path": h.path,
                    "title": h.title,
                    "category": h.category,
                    "score": h.score,
                    "matched_terms": h.matched_terms,
                    "excerpt": h.excerpt,
                    "linked_elements": h.linked_elements,
                })).collect::<Vec<_>>(),
            });
            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        "sruja_bm25_search" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;
            let cfg_default = crate::integrations::load_repo_config(Path::new(&repo))
                .and_then(|c| c.context_engineering.bm25_max_results_mcp)
                .unwrap_or(5);
            let max_results = arguments
                .get("max_results")
                .and_then(|v| v.as_u64())
                .unwrap_or(cfg_default as u64) as usize;

            let index = sruja_graph::SparseIndex::build(Path::new(&repo));
            let hits = index.search(query, max_results);

            let out = json!({
                "query": query,
                "doc_count": index.doc_count(),
                "results": hits.iter().map(|h| json!({
                    "path": h.path,
                    "title": h.title,
                    "category": h.category,
                    "score": h.score,
                    "matched_terms": h.matched_terms,
                    "excerpt": h.excerpt,
                    "linked_elements": h.linked_elements,
                })).collect::<Vec<_>>(),
            });
            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        "sruja_hybrid_query" => {
            let query = arguments
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing query"))?;

            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let complexity = sruja_graph::classify_query(query, &kg);
            let vector_path = std::path::Path::new(&repo)
                .join(".sruja")
                .join("vectors.json");
            let has_semantic = vector_path.exists();
            let strategy = sruja_graph::select_strategy(complexity, has_semantic);

            let semantic_candidates = match strategy {
                sruja_graph::RetrievalStrategy::GraphOnly => Vec::new(),
                _ => {
                    if has_semantic {
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
                        searcher
                            .search(&index, query, 5)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|(id, score)| sruja_graph::SemanticCandidate {
                                element_id: id,
                                score,
                                label: None,
                            })
                            .collect()
                    } else {
                        Vec::new()
                    }
                }
            };

            let result = sruja_graph::execute_hybrid(&kg, query, semantic_candidates);
            finish(Ok(serde_json::to_string_pretty(&result)?))
        }

        "sruja_get_author_evidence" => {
            let evidence = crate::commands::author::load_or_build_author_evidence(repo)?;
            let mut value = serde_json::to_value(&evidence)?;
            value["path"] = json!(crate::commands::author::author_evidence_default_path(repo)
                .display()
                .to_string());
            value["next_suggested_tool"] = json!("sruja_get_task_context");
            set_estimated_tokens(&mut value)?;
            finish(Ok(serde_json::to_string_pretty(&value)?))
        }

        _ => Ok(None),
    }
}

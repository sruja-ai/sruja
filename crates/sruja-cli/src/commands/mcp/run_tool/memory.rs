use serde_json::{json, Value};
use sruja_agent::AgenticMemory;
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
    let run_id = arguments.get("run_id").and_then(|v| v.as_str());
    match name {
        "sruja_get_context_score" => {
            let format = arguments
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("text");
            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let age_hours = crate::utils::context::context_age_hours(Path::new(&repo));
            let score = sruja_graph::compute_context_score(
                &kg,
                graph.nodes.len(),
                Path::new(&repo),
                age_hours,
            );

            if format == "json" {
                finish(Ok(serde_json::to_string_pretty(&score)?))
            } else {
                finish(Ok(format!(
                    "Context Score: {}/100\n\nBreakdown:\n- Coverage: {}%\n- Decisions: {}%\n- Freshness: {}%\n- Density: {}%\n- External: {}%",
                    score.score,
                    score.architecture_coverage.pct_u8(),
                    score.decision_completeness.pct_u8(),
                    score.evidence_freshness.pct_u8(),
                    score.relationship_density.pct_u8(),
                    score.external_context.pct_u8()
                )))
            }
        }

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

        "sruja_get_decision_trace" => {
            let decision_id = arguments
                .get("decision_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing decision_id".to_string()))?;
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as usize;
            let events = crate::commands::context_events::read_context_events_query(
                Path::new(&repo),
                crate::commands::context_events::ContextEventQuery {
                    limit,
                    kind_filter: None,
                    details_substring: None,
                    decision_id: Some(decision_id),
                    trace_id: None,
                    run_id: None,
                    element_id: None,
                    decision_lineage_only: false,
                },
            )?;
            finish(Ok(serde_json::to_string_pretty(&events)?))
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
            let kind = arguments
                .get("kind")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing kind".to_string()))?;
            let summary = arguments
                .get("summary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("missing summary".to_string()))?;
            let decision_id = arguments
                .get("decision_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            let outcome = arguments
                .get("outcome")
                .and_then(|v| v.as_str())
                .unwrap_or("ok")
                .to_string();
            let elements: Option<Vec<String>> = arguments
                .get("elements")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                });
            let evidence_refs: Option<Vec<String>> = arguments
                .get("evidence_refs")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(String::from))
                        .collect()
                });
            let record = crate::commands::context_events::ContextEventRecord {
                schema_version: crate::commands::context_events::CONTEXT_EVENTS_SCHEMA_V2
                    .to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                kind: kind.to_string(),
                outcome,
                policy_fingerprint: crate::commands::context_events::policy_fingerprint(Path::new(
                    &repo,
                )),
                strict: None,
                details: serde_json::json!({}),
                trace_id: arguments
                    .get("trace_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                decision_id,
                run_id: arguments
                    .get("run_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                workflow_id: arguments
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                actor: arguments
                    .get("actor")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                source: arguments
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                tool: arguments
                    .get("tool")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                elements,
                subject_ids: None,
                evidence_refs,
                summary: Some(summary.to_string()),
            };
            crate::commands::context_events::validate_context_event_record(&record)
                .map_err(CliError::validation)?;
            crate::commands::context_events::append_context_event(Path::new(&repo), record);
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

        "sruja_get_agent_learnings" => {
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing element_id"))?;
            let surfaced =
                crate::commands::focus::surface_agent_learnings(Path::new(&repo), element_id, true)
                    .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            finish(Ok(serde_json::to_string_pretty(&surfaced.hits)?))
        }

        "sruja_get_focus_briefing" => {
            let file = arguments.get("file").and_then(|v| v.as_str());
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());

            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let graph = get_or_scan_graph(graph_cache, repo).await?;

            let target_id =
                crate::commands::focus::resolve_target(&kg, Path::new(&repo), file, element_id)?;
            let base_ref = arguments.get("base_ref").and_then(|v| v.as_str());
            let head_ref = arguments.get("head_ref").and_then(|v| v.as_str());
            let temporal = match (base_ref, head_ref) {
                (Some(b), Some(h)) => Some(crate::commands::focus::load_temporal_context(
                    Path::new(&repo),
                    b,
                    h,
                    &target_id,
                )?),
                (Some(b), None) => Some(crate::commands::focus::load_temporal_context(
                    Path::new(&repo),
                    b,
                    "HEAD",
                    &target_id,
                )?),
                (None, Some(_)) => {
                    return Err(CliError::validation(
                        "head_ref requires base_ref for focus temporal context".to_string(),
                    ));
                }
                (None, None) => None,
            };
            let mut briefing = crate::commands::focus::build_focus_briefing(
                &kg,
                &target_id,
                Path::new(&repo),
                graph.nodes.len(),
                temporal,
                true,
            );
            briefing.run_id = Some(
                run_id
                    .map(|s| s.to_string())
                    .unwrap_or_else(crate::utils::run_id::generate_run_id),
            );

            finish(Ok(serde_json::to_string_pretty(&briefing)?))
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

        "sruja_memory_clusters" => {
            let entry_id = arguments.get("entry_id").and_then(|v| v.as_str());
            let tag = arguments.get("tag").and_then(|v| v.as_str());

            let memory = AgenticMemory::load(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

            if let Some(eid) = entry_id {
                let cluster = memory.find_cluster(eid);
                return finish(Ok(serde_json::to_string_pretty(&cluster)?));
            }

            if let Some(t) = tag {
                let entries = memory.find_by_tag(t);
                return finish(Ok(serde_json::to_string_pretty(&entries)?));
            }

            let all_tags = memory.all_tags();
            let mut clusters = Vec::new();
            let mut visited = std::collections::HashSet::new();
            for entry in &memory.learnings {
                if visited.contains(&entry.id) {
                    continue;
                }
                let cluster = memory.find_cluster(&entry.id);
                let ids: Vec<String> = cluster.iter().map(|e| e.id.clone()).collect();
                for id in &ids {
                    visited.insert(id.clone());
                }
                clusters.push(json!({
                    "root_id": entry.id,
                    "size": cluster.len(),
                    "entry_ids": ids,
                }));
            }

            let out = json!({
                "total_entries": memory.learnings.len(),
                "total_tags": all_tags.len(),
                "tags": all_tags,
                "clusters": clusters,
            });
            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        "sruja_get_learned_facts" => {
            let limit = arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(200) as usize;
            let status = arguments.get("status").and_then(|v| v.as_str());
            let facts =
                crate::commands::learn::read_learned_facts(Path::new(&repo), limit, status)?;
            finish(Ok(serde_json::to_string_pretty(&facts)?))
        }

        "sruja_get_evidence_graph" => {
            let p = Path::new(&repo).join(".sruja").join("evidence_graph.json");
            if !p.exists() {
                return Err(CliError::validation(format!(
                    "No evidence graph at {}. Run `sruja learn -r {}` first.",
                    p.display(),
                    repo
                )));
            }
            let text = std::fs::read_to_string(&p).map_err(CliError::Io)?;
            finish(Ok(text))
        }

        "sruja_get_author_evidence" => {
            let evidence = crate::commands::author::load_or_build_author_evidence(repo)?;
            let mut value = serde_json::to_value(&evidence)?;
            value["path"] = json!(crate::commands::author::author_evidence_default_path(repo)
                .display()
                .to_string());
            value["next_suggested_tool"] = json!("sruja_get_focus_briefing");
            set_estimated_tokens(&mut value)?;
            finish(Ok(serde_json::to_string_pretty(&value)?))
        }

        "sruja_get_evidence_for_claim" => {
            let claim_id = arguments
                .get("claim_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing claim_id".to_string()))?;
            let fact = crate::commands::learn::get_learned_fact_by_id(Path::new(&repo), claim_id)?
                .ok_or_else(|| CliError::validation(format!("Unknown claim_id {claim_id}")))?;
            let eg_path = Path::new(&repo).join(".sruja").join("evidence_graph.json");
            let related = if eg_path.exists() {
                let raw: serde_json::Value =
                    serde_json::from_str(&std::fs::read_to_string(&eg_path).map_err(CliError::Io)?)
                        .map_err(CliError::Json)?;
                let empty: Vec<serde_json::Value> = Vec::new();
                let nodes = raw
                    .pointer("/graph/nodes")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or(empty);
                let sid = fact.subject.as_str();
                let oid = fact.object.as_str();
                nodes
                    .into_iter()
                    .filter(|n| {
                        n.get("id")
                            .and_then(|v| v.as_str())
                            .is_some_and(|id| id == sid || id == oid)
                    })
                    .take(8)
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let out = json!({ "fact": fact, "related_scan_nodes": related });
            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        "sruja_record_learn_feedback" => {
            let fact_id = arguments
                .get("fact_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing fact_id".to_string()))?;
            let decision = arguments
                .get("decision")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing decision".to_string()))?;
            let reason = arguments.get("reason").and_then(|v| v.as_str());
            crate::commands::learn::append_learn_feedback(
                Path::new(&repo),
                fact_id,
                decision,
                reason,
            )?;
            finish(Ok(
                json!({ "ok": true, "fact_id": fact_id, "decision": decision }).to_string(),
            ))
        }
        _ => Ok(None),
    }
}

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

        "sruja_get_workflow" => {
            let workflow_id = arguments
                .get("workflow_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing workflow_id"))?;
            let wf = crate::commands::workflow_get(repo, workflow_id)?;
            let base = Path::new(repo)
                .join(".sruja")
                .join("workflows")
                .join(workflow_id);
            let out = serde_json::json!({
                "schema_version": "workflow_get/v1",
                "workflow": wf,
                "paths": {
                    "manifest": base.join("manifest.json"),
                    "inception": {
                        "scope": base.join("inception").join("scope.md"),
                        "impact": base.join("inception").join("impact.json"),
                        "design_review": base.join("inception").join("design-review.md")
                    },
                    "construction": {
                        "task_plan": base.join("construction").join("task-plan.md"),
                        "linked_proposal_ids": base.join("construction").join("linked_proposal_ids.json")
                    },
                    "operations": {
                        "deploy_scope": base.join("operations").join("deploy-scope.json")
                    }
                }
            });
            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        "sruja_workflow_gate_check" => {
            let workflow_id = arguments
                .get("workflow_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing workflow_id"))?;
            let gate = crate::commands::workflow_gate_check(repo, workflow_id)?;
            finish(Ok(serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "workflow_gate_check/v2",
                "workflow_id": workflow_id,
                "allowed": gate.allowed,
                "phase": gate.phase,
                "missing": gate.missing,
                "aidlc_missing": gate.aidlc_missing,
                "aidlc_stage": gate.aidlc_stage,
            }))?))
        }

        "sruja_workflow_summary" => {
            let workflow_id = arguments
                .get("workflow_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing workflow_id"))?;
            let val = crate::commands::workflow::workflow_summary_json_value(repo, workflow_id)?;
            finish(Ok(serde_json::to_string_pretty(&val)?))
        }

        "sruja_workflow_next_steps" => {
            let workflow_id = arguments
                .get("workflow_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing workflow_id"))?;
            let val = crate::commands::workflow::workflow_next_steps_json_value(repo, workflow_id)?;
            finish(Ok(serde_json::to_string_pretty(&val)?))
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

            let enrich_ref = crate::enrichment::EnrichmentRef {
                enrich,
                provider: enrich_provider,
                cmd: enrich_cmd,
                model: enrich_model,
                base_url: enrich_base_url,
                timeout_ms: enrich_timeout_ms,
                max_bytes: enrich_max_bytes,
            };
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
                enrich: &enrich_ref,
                continue_on_error,
                trajectories: None,
            })
            .await?;
            finish(Ok(text))
        }

        _ => Ok(None),
    }
}

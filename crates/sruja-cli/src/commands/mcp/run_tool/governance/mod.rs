use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::helpers::*;
use super::finish;
use crate::commands::CliError;

mod sandbox;
mod suggestions;
mod task_context;
mod verify;

pub(crate) async fn try_run(
    name: &str,
    arguments: &Value,
    repo: &str,
    graph_cache: &Arc<Mutex<HashMap<String, sruja_scan::Graph>>>,
) -> Result<Option<String>, CliError> {
    let _run_id = arguments.get("run_id").and_then(|v| v.as_str());
    match name {
        "sruja_get_task_context" => task_context::handle(arguments, repo, graph_cache).await,

        "sruja_sandbox" => sandbox::handle(arguments, repo).await,

        "sruja_verify_architecture" => verify::handle(arguments, repo, graph_cache).await,

        "sruja_suggest_fix" => suggestions::handle(arguments, repo, graph_cache).await,

        "sruja_get_state_machine" => {
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing element_id"))?;
            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == element_id)
                .ok_or_else(|| {
                    CliError::validation(format!("Element {} not found", element_id))
                })?;

            if node.state_machines.is_empty() {
                return finish(Ok(format!(
                    "No state machines found for element {}.",
                    element_id
                )));
            }

            finish(Ok(serde_json::to_string_pretty(&node.state_machines)?))
        }

        "sruja_get_contract" => {
            let element_id = arguments
                .get("element_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing element_id"))?;
            let contract_name = arguments.get("contract_name").and_then(|v| v.as_str());
            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == element_id)
                .ok_or_else(|| {
                    CliError::validation(format!("Element {} not found", element_id))
                })?;

            if node.contracts.is_empty() {
                return finish(Ok(format!(
                    "No contracts found for element {}.",
                    element_id
                )));
            }

            if let Some(name) = contract_name {
                let contract = node.contracts.iter().find(|c| c.name == name).ok_or_else(
                    || {
                        CliError::validation(format!(
                            "Contract {} not found on element {}",
                            name, element_id
                        ))
                    },
                )?;
                finish(Ok(serde_json::to_string_pretty(contract)?))
            } else {
                finish(Ok(serde_json::to_string_pretty(&node.contracts)?))
            }
        }

        "sruja_preflight_check" => {
            let files = arguments
                .get("target_files")
                .and_then(|v| v.as_array())
                .ok_or_else(|| CliError::validation("Missing target_files array"))?;
            let file_list: Vec<String> = files
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            let intent_hint = arguments
                .get("intent")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let res = crate::commands::preflight::preflight(
                Path::new(&repo),
                file_list,
                intent_hint,
            )
            .await?;
            finish(Ok(serde_json::to_string_pretty(&res)?))
        }

        "sruja_evaluate_proposal" => {
            let gate_cmd = arguments.get("gate_command").and_then(|v| v.as_str());

            let mut out = String::new();
            if let Some(cmd) = gate_cmd {
                out.push_str(&format!("Running gate: {}\n", cmd));

                let output = if cfg!(target_os = "windows") {
                    std::process::Command::new("cmd")
                        .args(["/C", cmd])
                        .current_dir(repo)
                        .output()
                } else {
                    std::process::Command::new("sh")
                        .args(["-c", cmd])
                        .current_dir(repo)
                        .output()
                };

                match output {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        if o.status.success() {
                            out.push_str("✅ Gate Passed\n");
                        } else {
                            out.push_str("❌ Gate Failed\n\n");
                            out.push_str(&stdout);
                            out.push_str(&stderr);
                            out.push_str(
                                "\nRevert your changes or update your hypothesis in the Agentic Memory before trying again.",
                            );
                            return finish(Ok(out));
                        }
                    }
                    Err(e) => {
                        out.push_str(&format!("❌ Gate Execution Failed: {}\n", e));
                        return finish(Ok(out));
                    }
                }
            }

            let kg = crate::graph_store::load_or_build_graph(Path::new(&repo))?;
            let scan_node_count = match sruja_scan::scan_repo(Path::new(&repo)) {
                Ok(g) => g.nodes.len(),
                Err(_) => kg.nodes.len(),
            };
            let score =
                sruja_graph::compute_context_score(&kg, scan_node_count, Path::new(&repo), 0);

            out.push_str(&format!("\n📈 Context Score: {}/100\n", score.score));
            out.push_str(&format!(
                "  - Architecture Coverage: {}/100\n",
                score.architecture_coverage.value
            ));
            out.push_str(&format!(
                "  - Decision Completeness: {}/100\n",
                score.decision_completeness.value
            ));
            out.push_str(&format!(
                "  - Evidence Freshness: {}/100\n",
                score.evidence_freshness.value
            ));
            out.push_str(&format!(
                "  - Relationship Density: {}/100\n",
                score.relationship_density.value
            ));

            if score.score == 100 {
                out.push_str("\n🎉 Perfect Score Achieved! Your hypothesis succeeded.");
            } else {
                out.push_str(
                    "\nReview the Agentic Memory or Context Map to find new optimization opportunities.",
                );
            }

            finish(Ok(out))
        }

        "sruja_record_learning" => {
            use sruja_agent::{AgenticMemory, ExperimentOutcome, LearningEntry};

            let context = arguments
                .get("context")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let hypothesis = arguments
                .get("hypothesis")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let outcome_str = arguments
                .get("outcome")
                .and_then(|v| v.as_str())
                .unwrap_or("success");
            let reason = arguments
                .get("reason")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let guardrail_advice = arguments
                .get("guardrail_advice")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let affected_elements = arguments
                .get("affected_elements")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let hitl_raw = arguments.get("hitl_kind").and_then(|v| v.as_str());
            let hitl_kind = if let Some(h) = hitl_raw {
                let v = h.trim().to_lowercase();
                match v.as_str() {
                    "precedent" | "exception" | "correction" | "guardrail" => Some(v),
                    "" => None,
                    _ => {
                        return Err(CliError::validation(format!(
                            "invalid hitl_kind: expected precedent|exception|correction|guardrail, got {h}"
                        )));
                    }
                }
            } else {
                None
            };

            let outcome = if outcome_str == "failed" {
                ExperimentOutcome::Failed
            } else {
                ExperimentOutcome::Success
            };

            let mut memory = AgenticMemory::load(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            memory.add_learning(LearningEntry {
                id: String::new(),
                kind: Some(match outcome {
                    ExperimentOutcome::Success => sruja_agent::LearningKind::Playbook,
                    ExperimentOutcome::Failed => sruja_agent::LearningKind::Guardrail,
                }),
                timestamp: chrono::Utc::now(),
                run_id: arguments
                    .get("run_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                repo: Some(repo.to_string()),
                selector: None,
                context,
                hypothesis,
                outcome,
                reason,
                guardrail_advice,
                affected_elements,
                evidence_refs: Vec::new(),
                confidence: None,
                tags: Vec::new(),
                hitl_kind,
                related_ids: Vec::new(),
                retrieval_count: 0,
                task_success_after: 0,
                task_total_after: 0,
                category: None,
                signals_match: Vec::new(),
                constraints: None,
                validation: Vec::new(),
                blast_radius: None,
            });
            memory
                .save(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

            finish(Ok(
                "Learning recorded in Agentic Memory successfully.".to_string(),
            ))
        }

        "sruja_verify_task" => {
            use crate::commands::{format_verify_task, verify_task, VerifyTaskOptions};
            let profile = arguments
                .get("profile")
                .and_then(|v| v.as_str())
                .unwrap_or("coding");
            let file = arguments.get("file").and_then(|v| v.as_str());
            let max_runtime_ms = arguments.get("max_runtime_ms").and_then(|v| v.as_u64());

            let output = verify_task(VerifyTaskOptions {
                repo,
                profile,
                file,
                max_runtime_ms,
                evidence_pack: false,
                evidence_pack_dir: None,
            })
            .await?;
            finish(Ok(format_verify_task(&output, "json")))
        }

        "sruja_get_boundaries" => {
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let file = arguments.get("file").and_then(|v| v.as_str());

            let kg = crate::graph_store::load_or_build_graph(Path::new(repo))?;
            let resolved_id = if let Some(f) = file {
                crate::commands::focus::resolve_target(&kg, Path::new(repo), Some(f), None)
                    .ok()
            } else if let Some(eid) = element_id {
                crate::commands::focus::resolve_target(&kg, Path::new(repo), None, Some(eid))
                    .ok()
            } else {
                None
            };

            let arch_path =
                crate::utils::architecture_path::resolve_architecture_path(Path::new(repo));
            let intent = if let Some(ref path) = arch_path {
                sruja_intent::model::IntentModel::from_sruja_file(path).unwrap_or_default()
            } else {
                sruja_intent::model::IntentModel::default()
            };

            let mut boundaries_json = Vec::new();

            if !intent.boundaries.is_empty() {
                for b in &intent.boundaries {
                    if let Some(ref target) = resolved_id {
                        let is_inside = b.inside.iter().any(|inside_id| {
                            target == inside_id
                                || target.starts_with(&format!("{}.", inside_id))
                                || inside_id.starts_with(&format!("{}.", target))
                        });
                        if !is_inside {
                            continue;
                        }
                    }
                    boundaries_json.push(serde_json::json!({
                        "name": b.name,
                        "inside": b.inside,
                        "allowed_connections": b.allowed_connections.iter().map(|ac| {
                            serde_json::json!({
                                "target_boundary": ac.target_boundary,
                                "via": format!("{:?}", ac.via)
                            })
                        }).collect::<Vec<_>>(),
                        "rules": b.rules.iter().map(|r| {
                            serde_json::json!({
                                "type": format!("{:?}", r.rule_type),
                                "description": r.description
                            })
                        }).collect::<Vec<_>>(),
                        "max_depth": b.max_depth,
                        "source_file": b.source_ref.file,
                        "source_line": b.source_ref.line
                    }));
                }
            } else {
                let graph = get_or_scan_graph(graph_cache, repo).await?;
                let inferred = crate::commands::context::logic::infer_boundaries(&graph);
                for inf in inferred {
                    if let Some(ref target) = resolved_id {
                        if inf.from != *target && inf.to != *target {
                            continue;
                        }
                    }
                    boundaries_json.push(serde_json::json!({
                        "name": format!("Inferred coupling boundary ({} to {})", inf.from, inf.to),
                        "inside": vec![inf.from.clone()],
                        "allowed_connections": if inf.allowed {
                            vec![serde_json::json!({ "target_boundary": inf.to.clone(), "via": "DirectCall" })]
                        } else {
                            vec![]
                        },
                        "rules": vec![serde_json::json!({
                            "type": "Custom",
                            "description": inf.reason.clone()
                        })],
                        "max_depth": 2,
                        "source_file": "inferred_from_code",
                        "source_line": null
                    }));
                }
            }

            finish(Ok(serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "boundaries/v1",
                "focus_element_id": resolved_id,
                "boundaries": boundaries_json
            }))?))
        }

        "sruja_check_violations" => {
            let files_arg = arguments.get("files").and_then(|v| v.as_array());
            let file_list: Option<Vec<String>> = files_arg.map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });

            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let report = sruja_diff::detect_architectural_drift(&graph);
            let mut violations = report.violations;

            let baseline_path =
                crate::utils::architecture_path::resolve_architecture_path(
                    std::path::Path::new(&repo),
                );
            if let Some(p) = baseline_path {
                if let Ok(status) =
                    crate::commands::scan::drift::truth_status_from_baseline_compare(&graph, &p)
                {
                    if matches!(status, sruja_diff::TruthStatus::Drifted) {
                        let content = std::fs::read_to_string(&p)?;
                        let parser =
                            sruja_language::Parser::new(p.to_string_lossy().to_string());
                        if let Ok(program) = parser.parse(&content) {
                            let proposed = sruja_diff::program_to_graph(&program);
                            let diff = sruja_diff::compare_graphs(&graph, &proposed);
                            for v in diff.violations {
                                if !violations.iter().any(|rv| {
                                    rv.message == v.message && rv.location == v.location
                                }) {
                                    violations.push(v);
                                }
                            }
                        }
                    }
                }
            }

            if let Some(ref fl) = file_list {
                let mut impacted_ids = std::collections::HashSet::new();
                for f in fl {
                    for node in &graph.nodes {
                        if node
                            .path
                            .as_ref()
                            .is_some_and(|p| p.contains(f) || f.contains(p))
                        {
                            impacted_ids.insert(node.id.clone());
                        }
                    }
                }
                violations.retain(|v| {
                    v.location.as_ref().is_some_and(|l| {
                        fl.iter().any(|f| l.contains(f)) || impacted_ids.contains(l)
                    })
                });
            }

            let json_violations = violations
                .iter()
                .map(|v| {
                    serde_json::json!({
                        "severity": format!("{:?}", v.severity),
                        "kind": format!("{:?}", v.kind),
                        "message": v.message.clone(),
                        "location": v.location.clone(),
                        "suggestion": v.suggestion.clone()
                    })
                })
                .collect::<Vec<_>>();

            finish(Ok(serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "violations/v1",
                "violations": json_violations
            }))?))
        }

        _ => Ok(None),
    }
}

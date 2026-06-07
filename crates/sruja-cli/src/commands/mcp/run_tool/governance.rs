use serde_json::Value;
use sruja_agent::{AgenticMemory, ExperimentOutcome, LearningEntry};
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
        "sruja_get_task_context" => {
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let file = arguments.get("file").and_then(|v| v.as_str());
            let query = arguments.get("query").and_then(|v| v.as_str());
            let base_ref = arguments.get("base_ref").and_then(|v| v.as_str());
            let head_ref = arguments.get("head_ref").and_then(|v| v.as_str());
            let workflow_id = arguments.get("workflow_id").and_then(|v| v.as_str());
            let phase = arguments.get("phase").and_then(|v| v.as_str());
            let depth = arguments.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            let mut max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(10000) as usize;
            if let (Some(wid), Some(ph)) = (workflow_id, phase) {
                if ph == "construction" {
                    max_tokens = max_tokens.min(4000);
                }
                if let Ok(manifest) = crate::commands::workflow_get(repo, wid) {
                    if !manifest.target_elements.is_empty()
                        && element_id.is_none()
                        && file.is_none()
                    {
                        // Prefer workflow target when host passes workflow scope only.
                        let _ = manifest;
                    }
                }
            }
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
            let cache_friendly = arguments
                .get("cache_friendly")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let selectors = crate::commands::context::logic::TaskSelectors {
                element_id,
                file,
                query,
                base_ref,
                head_ref,
                depth: Some(depth),
            };

            let ctx = crate::commands::context::logic::build_task_context(
                &graph, repo, selectors, max_tokens,
            )?;
            if !enrich && enrich_cmd.is_none() {
                if cache_friendly {
                    let arch = crate::commands::context::logic::build_architecture_context(
                        &graph, repo, None, None, depth, max_tokens,
                    )?;
                    let export = crate::commands::context::logic::build_cache_friendly_task_export(
                        repo, &arch, ctx,
                    );
                    return finish(Ok(serde_json::to_string_pretty(&export)?));
                }
                let mut val = serde_json::to_value(&ctx)?;
                if workflow_id.is_some() || phase.is_some() {
                    if let Some(obj) = val.as_object_mut() {
                        if let Some(wid) = workflow_id {
                            obj.insert(
                                "workflow_id".to_string(),
                                serde_json::Value::String(wid.to_string()),
                            );
                        }
                        if let Some(ph) = phase {
                            obj.insert(
                                "workflow_phase".to_string(),
                                serde_json::Value::String(ph.to_string()),
                            );
                        }
                        obj.insert(
                            "max_tokens_applied".to_string(),
                            serde_json::json!(max_tokens),
                        );
                    }
                }
                return finish(Ok(serde_json::to_string_pretty(&val)?));
            }

            let wrapped = enrich_wrapper_json(
                Path::new(&repo),
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                "task_context",
                serde_json::to_value(&ctx)?,
            );
            finish(Ok(serde_json::to_string_pretty(&wrapped)?))
        }

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
                .ok_or_else(|| CliError::validation(format!("Element {} not found", element_id)))?;

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
                .ok_or_else(|| CliError::validation(format!("Element {} not found", element_id)))?;

            if node.contracts.is_empty() {
                return finish(Ok(format!(
                    "No contracts found for element {}.",
                    element_id
                )));
            }

            if let Some(name) = contract_name {
                let contract = node
                    .contracts
                    .iter()
                    .find(|c| c.name == name)
                    .ok_or_else(|| {
                        CliError::validation(format!(
                            "Contract {} not found on element {}",
                            name, element_id
                        ))
                    })?;
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

            let res =
                crate::commands::preflight::preflight(Path::new(&repo), file_list, intent_hint)
                    .await?;
            finish(Ok(serde_json::to_string_pretty(&res)?))
        }

        "sruja_sandbox" => {
            let action = arguments
                .get("action")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing action"))?;

            let name = arguments.get("name").and_then(|v| v.as_str());
            let sruja_dir = Path::new(&repo).join(".sruja");
            let sandbox_dir = sruja_dir.join("sandboxes");
            std::fs::create_dir_all(&sandbox_dir)?;

            match action {
                "create" => {
                    let name =
                        name.ok_or_else(|| CliError::validation("Missing name for create"))?;
                    let target = sandbox_dir.join(name);
                    if target.exists() {
                        return Err(CliError::validation(format!(
                            "Sandbox '{}' already exists",
                            name
                        )));
                    }

                    let output = std::process::Command::new("git")
                        .args([
                            "worktree",
                            "add",
                            "-b",
                            &format!("sruja-sandbox/{}", name),
                            target.to_str().ok_or_else(|| {
                                CliError::validation("Target path is not valid UTF-8")
                            })?,
                        ])
                        .current_dir(repo)
                        .output()?;

                    if !output.status.success() {
                        return Err(CliError::validation(format!(
                            "Git worktree failed: {}",
                            String::from_utf8_lossy(&output.stderr)
                        )));
                    }
                    finish(Ok(format!("✅ Created isolated sandbox at {}. Run your tools and evaluations against this path.", target.display())))
                }
                "discard" => {
                    let name =
                        name.ok_or_else(|| CliError::validation("Missing name for discard"))?;
                    let target = sandbox_dir.join(name);

                    if !target.exists() {
                        return Err(CliError::validation(format!(
                            "Sandbox '{}' not found",
                            name
                        )));
                    }

                    std::process::Command::new("git")
                        .args([
                            "worktree",
                            "remove",
                            "--force",
                            target.to_str().ok_or_else(|| {
                                CliError::validation("Target path is not valid UTF-8")
                            })?,
                        ])
                        .current_dir(repo)
                        .output()?;

                    std::process::Command::new("git")
                        .args(["branch", "-D", &format!("sruja-sandbox/{}", name)])
                        .current_dir(repo)
                        .output()?;

                    finish(Ok(format!("🗑️ Discarded sandbox '{}'.", name)))
                }
                "commit" => {
                    let name =
                        name.ok_or_else(|| CliError::validation("Missing name for commit"))?;
                    let target = sandbox_dir.join(name);

                    if !target.exists() {
                        return Err(CliError::validation(format!(
                            "Sandbox '{}' not found",
                            name
                        )));
                    }

                    // Commit any pending changes in the worktree
                    std::process::Command::new("git")
                        .args(["add", "-A"])
                        .current_dir(&target)
                        .output()?;

                    std::process::Command::new("git")
                        .args(["commit", "-m", &format!("Sruja Sandbox: {}", name)])
                        .current_dir(&target)
                        .output()?;

                    finish(Ok(format!("✅ Sandbox '{}' successfully committed to branch 'sruja-sandbox/{}'. A human can now merge this into the main branch.", name, name)))
                }
                "list" => {
                    if let Ok(entries) = std::fs::read_dir(&sandbox_dir) {
                        let mut sandboxes = Vec::new();
                        for entry in entries.flatten() {
                            if entry.path().is_dir() {
                                sandboxes
                                    .push(format!("- {}", entry.file_name().to_string_lossy()));
                            }
                        }
                        if sandboxes.is_empty() {
                            finish(Ok("No active sandboxes.".to_string()))
                        } else {
                            finish(Ok(format!("Active Sandboxes:\n{}", sandboxes.join("\n"))))
                        }
                    } else {
                        finish(Ok("No active sandboxes.".to_string()))
                    }
                }
                _ => Err(CliError::validation(format!(
                    "Invalid sandbox action: {}",
                    action
                ))),
            }
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
                            out.push_str("\nRevert your changes or update your hypothesis in the Agentic Memory before trying again.");
                            return finish(Ok(out));
                        }
                    }
                    Err(e) => {
                        out.push_str(&format!("❌ Gate Execution Failed: {}\n", e));
                        return finish(Ok(out));
                    }
                }
            }

            // Calculate Context Score as the quality metric
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
                out.push_str("\nReview the Agentic Memory or Context Map to find new optimization opportunities.");
            }

            finish(Ok(out))
        }

        "sruja_record_learning" => {
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
            });
            memory
                .save(Path::new(&repo))
                .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;

            finish(Ok(
                "Learning recorded in Agentic Memory successfully.".to_string()
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
                crate::commands::focus::resolve_target(&kg, Path::new(repo), Some(f), None).ok()
            } else if let Some(eid) = element_id {
                crate::commands::focus::resolve_target(&kg, Path::new(repo), None, Some(eid)).ok()
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
                // Inferred fallback
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

            let baseline_path = crate::utils::architecture_path::resolve_architecture_path(
                std::path::Path::new(&repo),
            );
            if let Some(p) = baseline_path {
                if let Ok(status) =
                    crate::commands::scan::drift::truth_status_from_baseline_compare(&graph, &p)
                {
                    if matches!(status, sruja_diff::TruthStatus::Drifted) {
                        let content = std::fs::read_to_string(&p)?;
                        let parser = sruja_language::Parser::new(p.to_string_lossy().to_string());
                        if let Ok(program) = parser.parse(&content) {
                            let proposed = sruja_diff::program_to_graph(&program);
                            let diff = sruja_diff::compare_graphs(&graph, &proposed);
                            for v in diff.violations {
                                if !violations
                                    .iter()
                                    .any(|rv| rv.message == v.message && rv.location == v.location)
                                {
                                    violations.push(v);
                                }
                            }
                        }
                    }
                }
            }

            // Filter if files are provided
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

        "sruja_suggest_fix" => {
            let violation_arg = arguments.get("violation");

            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let mut suggestions = Vec::new();

            if let Some(val) = violation_arg {
                if let (Some(kind), Some(message)) = (
                    val.get("kind").and_then(|v| v.as_str()),
                    val.get("message").and_then(|v| v.as_str()),
                ) {
                    let loc = val.get("location").and_then(|v| v.as_str()).unwrap_or("");
                    generate_fix_suggestions(kind, message, loc, &mut suggestions);
                }
            } else {
                // Generate suggestions for all active violations in the repo
                let report = sruja_diff::detect_architectural_drift(&graph);
                let mut violations = report.violations;

                let baseline_path = crate::utils::architecture_path::resolve_architecture_path(
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

                for v in violations {
                    let kind_str = format!("{:?}", v.kind);
                    let loc = v.location.as_deref().unwrap_or("");
                    generate_fix_suggestions(&kind_str, &v.message, loc, &mut suggestions);
                }
            }

            finish(Ok(serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "suggest_fix/v1",
                "suggestions": suggestions
            }))?))
        }

        "sruja_verify_architecture" => {
            let files_arg = arguments.get("files").and_then(|v| v.as_array());
            let file_list: Option<Vec<String>> = files_arg.map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });

            // 1. Lint Check
            let baseline_path = crate::utils::architecture_path::resolve_architecture_path(
                std::path::Path::new(&repo),
            );
            let mut lint_ok = true;
            let mut lint_errors = Vec::new();
            let mut program_opt = None;

            if let Some(ref path) = baseline_path {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let parser = sruja_language::Parser::new(path.to_string_lossy().to_string());
                    match parser.parse(&content) {
                        Ok(program) => {
                            let validator = sruja_engine::Validator::with_default_rules();
                            let diagnostics = validator.validate_sync(&program);
                            for d in diagnostics {
                                if d.severity == sruja_diagnostics::Severity::Error {
                                    lint_ok = false;
                                }
                                lint_errors.push(serde_json::json!({
                                    "severity": format!("{:?}", d.severity),
                                    "code": d.code.clone(),
                                    "message": d.message.clone(),
                                    "line": d.location.line,
                                    "column": d.location.column,
                                }));
                            }
                            program_opt = Some(program);
                        }
                        Err(diagnostics) => {
                            lint_ok = false;
                            for d in diagnostics {
                                lint_errors.push(serde_json::json!({
                                    "severity": "Error",
                                    "code": d.code.clone(),
                                    "message": d.message.clone(),
                                    "line": d.location.line,
                                    "column": d.location.column,
                                }));
                            }
                        }
                    }
                }
            }

            // 2. Drift Check
            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let mut drift_violations = Vec::new();
            let report = sruja_diff::detect_architectural_drift(&graph);
            let mut all_violations = report.violations;

            if let Some(ref p) = baseline_path {
                if let Ok(status) =
                    crate::commands::scan::drift::truth_status_from_baseline_compare(&graph, p)
                {
                    if matches!(status, sruja_diff::TruthStatus::Drifted) {
                        if let Some(ref program) = program_opt {
                            let proposed = sruja_diff::program_to_graph(program);
                            let diff = sruja_diff::compare_graphs(&graph, &proposed);
                            for v in diff.violations {
                                if !all_violations
                                    .iter()
                                    .any(|rv| rv.message == v.message && rv.location == v.location)
                                {
                                    all_violations.push(v);
                                }
                            }
                        }
                    }
                }
            }

            // Filter if files are provided
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
                all_violations.retain(|v| {
                    v.location.as_ref().is_some_and(|l| {
                        fl.iter().any(|f| l.contains(f)) || impacted_ids.contains(l)
                    })
                });
            }

            for v in all_violations {
                drift_violations.push(serde_json::json!({
                    "severity": format!("{:?}", v.severity),
                    "kind": format!("{:?}", v.kind),
                    "message": v.message.clone(),
                    "location": v.location.clone(),
                    "suggestion": v.suggestion.clone()
                }));
            }

            // 3. Critique / Intent Check
            let mut critique_violations = Vec::new();
            if let Some(ref fl) = file_list {
                let engine = sruja_intent::CritiqueEngine::new(graph.clone(), program_opt);
                let report = engine.critique(&sruja_intent::CritiqueRequest {
                    changed_files: fl.clone(),
                    description: None,
                    proposal_id: None,
                    base_ref: None,
                    head_ref: None,
                });
                for finding in report.findings {
                    let location = finding
                        .evidence
                        .first()
                        .and_then(|e| e.location.clone())
                        .unwrap_or_default();
                    critique_violations.push(serde_json::json!({
                        "severity": format!("{:?}", finding.severity),
                        "message": finding.detail.clone(),
                        "location": location,
                        "rule_id": format!("{:?}", finding.category)
                    }));
                }
            }

            let ok = lint_ok && drift_violations.is_empty() && critique_violations.is_empty();

            let out = serde_json::json!({
                "schema_version": "verify_architecture/v1",
                "ok": ok,
                "lint": {
                    "ok": lint_ok,
                    "errors": lint_errors
                },
                "drift": {
                    "ok": drift_violations.is_empty(),
                    "violations": drift_violations
                },
                "critique": {
                    "ok": critique_violations.is_empty(),
                    "violations": critique_violations
                }
            });

            finish(Ok(serde_json::to_string_pretty(&out)?))
        }

        _ => Ok(None),
    }
}

fn extract_quoted_strings(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in s.chars() {
        if c == '\'' || c == '"' {
            if in_quotes {
                result.push(current.clone());
                current.clear();
                in_quotes = false;
            } else {
                in_quotes = true;
            }
        } else if in_quotes {
            current.push(c);
        }
    }
    result
}

fn generate_fix_suggestions(
    kind: &str,
    message: &str,
    location: &str,
    suggestions: &mut Vec<serde_json::Value>,
) {
    let quotes = extract_quoted_strings(message);
    match kind {
        "BoundaryViolation" => {
            let boundary_name = quotes
                .first()
                .cloned()
                .unwrap_or_else(|| "Services".to_string());
            let target_component = quotes
                .get(1)
                .cloned()
                .unwrap_or_else(|| "MySystem.DB".to_string());

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "dsl",
                "action": "allowed_connection",
                "file": "architecture.sruja",
                "description": format!("Add '{}' to allowed_connections for boundary '{}' in architecture.sruja if this dependency is intended.", target_component, boundary_name),
                "code_edit": format!("allowed_connections = [\n  {{\n    target_boundary = \"{}\",\n    via = \"ApiCall\"\n  }}\n]", target_component)
            }));

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "code",
                "action": "refactor_code",
                "file": location,
                "description": format!("Refactor the code to avoid direct coupling to '{}'. Services should communicate via APIs/events, not direct database access or direct imports.", target_component),
                "code_edit": null
            }));
        }
        "UndocumentedComponent" => {
            let component_name = quotes
                .first()
                .cloned()
                .unwrap_or_else(|| "UnknownComponent".to_string());

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "dsl",
                "action": "add_element",
                "file": "architecture.sruja",
                "description": format!("Add the undocumented component '{}' to your architecture DSL file to register it.", component_name),
                "code_edit": format!("{} = component \"{}\" {{\n  description \"Automatically discovered component\"\n}}", component_name, component_name)
            }));
        }
        "UndocumentedRelationship" => {
            let src = quotes
                .first()
                .cloned()
                .unwrap_or_else(|| "Source".to_string());
            let tgt = quotes
                .get(1)
                .cloned()
                .unwrap_or_else(|| "Target".to_string());

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "dsl",
                "action": "add_relationship",
                "file": "architecture.sruja",
                "description": format!("Document the relationship '{} -> {}' in your architecture DSL file.", src, tgt),
                "code_edit": format!("{} -> {} \"calls\"", src, tgt)
            }));
        }
        "PolicyViolation" => {
            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "code",
                "action": "refactor_code",
                "file": location,
                "description": "Refactor code to comply with architecture policies. Remove the forbidden dependency or connection.",
                "code_edit": null
            }));
        }
        "MissingComponent" => {
            let component_name = quotes
                .first()
                .cloned()
                .unwrap_or_else(|| "UnknownComponent".to_string());

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "code",
                "action": "refactor_code",
                "file": location,
                "description": format!("Implement the missing component '{}' in your codebase.", component_name),
                "code_edit": null
            }));

            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "dsl",
                "action": "remove_element",
                "file": "architecture.sruja",
                "description": format!("Remove the declared component '{}' from architecture docs if it is no longer planned.", component_name),
                "code_edit": null
            }));
        }
        _ => {
            suggestions.push(serde_json::json!({
                "violation_kind": kind,
                "violation_message": message,
                "fix_target": "code",
                "action": "refactor_code",
                "file": location,
                "description": "Refactor code to comply with the architecture rules.",
                "code_edit": null
            }));
        }
    }
}

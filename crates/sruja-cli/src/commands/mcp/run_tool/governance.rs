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
        "sruja_evaluate_mutation" => {
            let architecture = arguments
                .get("architecture")
                .and_then(|v| v.as_str())
                .unwrap_or("repo.sruja");

            // Execute the evaluation logic
            crate::commands::evaluate(architecture).await?;
            finish(Ok("Fitness functions evaluated successfully. Check output logs/terminal or evolution log.".to_string()))
        }

        "sruja_propose_topology_change" => {
            let desc = arguments
                .get("description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing required argument: description"))?;
            let add_elements: Vec<String> = arguments
                .get("add_elements")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let add_relationships: Vec<String> = arguments
                .get("add_relationships")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            // Run proposal create simulation and return feedback
            crate::commands::propose_create(
                repo,
                desc,
                add_elements,
                add_relationships,
                Vec::new(),
            )
            .await?;
            finish(Ok("Architecture topology change proposed successfully. Proposal ID and details generated.".to_string()))
        }

        "sruja_commit_evolution" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing required argument: id"))?;
            let target = arguments
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing required argument: target"))?;
            let result = arguments
                .get("result")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing required argument: result"))?;
            let detail = arguments
                .get("detail")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Write mutation record directly using our internal helper
            let sruja_dir = std::path::Path::new(&repo).join(".sruja");
            if !sruja_dir.exists() {
                std::fs::create_dir_all(&sruja_dir)?;
            }
            let log_path = sruja_dir.join("evolution.log");
            use std::fs::OpenOptions;
            use std::io::Write;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)?;

            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(
                file,
                "[{}] ID: {} | Target: {} | Result: {} | Output: {}",
                timestamp,
                id,
                target,
                result.to_uppercase(),
                detail
            )?;
            finish(Ok(
                "Evolution mutation successfully committed to history log.".to_string(),
            ))
        }

        "sruja_check_drift" => {
            let architecture = arguments
                .get("architecture")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let content =
                crate::commands::scan::drift_json_string(repo, architecture.as_deref(), false)
                    .await?;
            finish(Ok(content))
        }

        "sruja_get_task_context" => {
            let element_id = arguments.get("element_id").and_then(|v| v.as_str());
            let file = arguments.get("file").and_then(|v| v.as_str());
            let query = arguments.get("query").and_then(|v| v.as_str());
            let base_ref = arguments.get("base_ref").and_then(|v| v.as_str());
            let head_ref = arguments.get("head_ref").and_then(|v| v.as_str());
            let depth = arguments.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            let max_tokens = arguments
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(10000) as usize;
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
                return finish(Ok(serde_json::to_string_pretty(&ctx)?));
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

        "sruja_ai_scratchpad" => {
            let action = arguments
                .get("action")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CliError::validation("Missing action"))?;

            let scratchpad_path = Path::new(&repo).join(".sruja").join("ai-scratchpad.md");

            match action {
                "read" => {
                    if scratchpad_path.exists() {
                        finish(Ok(std::fs::read_to_string(scratchpad_path)?))
                    } else {
                        finish(Ok(
                            "Scratchpad is empty. No learnings recorded yet.".to_string()
                        ))
                    }
                }
                "append" => {
                    let content = arguments
                        .get("content")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| CliError::validation("Missing content for append"))?;

                    std::fs::create_dir_all(Path::new(&repo).join(".sruja"))?;
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(scratchpad_path)?;

                    use std::io::Write;
                    writeln!(file, "\n{}", content)?;
                    finish(Ok("Successfully appended to AI scratchpad.".to_string()))
                }
                _ => Err(CliError::validation(format!("Invalid action: {}", action))),
            }
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

        "sruja_validate_change" => {
            let files = arguments
                .get("files")
                .and_then(|v| v.as_array())
                .ok_or_else(|| CliError::validation("Missing files array".to_string()))?;
            let file_list: Vec<String> = files
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            let graph = get_or_scan_graph(graph_cache, repo).await?;
            let mut impacted_ids = std::collections::HashSet::new();
            for f in &file_list {
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

            let report = sruja_diff::detect_architectural_drift(&graph);
            let mut relevant_violations: Vec<_> = report
                .violations
                .into_iter()
                .filter(|v| {
                    v.location.as_ref().is_some_and(|l| {
                        file_list.iter().any(|f| l.contains(f)) || impacted_ids.contains(l)
                    })
                })
                .collect();

            let baseline_path = crate::utils::architecture_path::resolve_architecture_path(
                std::path::Path::new(&repo),
            );
            if let Some(p) = baseline_path {
                if let Ok(status) =
                    crate::commands::scan::drift::truth_status_from_baseline_compare(&graph, &p)
                {
                    if matches!(status, sruja_diff::TruthStatus::Drifted) {
                        // If we are drifted, run a full compare to get detailed delta violations
                        let content = std::fs::read_to_string(&p)?;
                        let parser = sruja_language::Parser::new(p.to_string_lossy().to_string());
                        if let Ok(program) = parser.parse(&content) {
                            let proposed = sruja_diff::program_to_graph(&program);
                            let diff = sruja_diff::compare_graphs(&graph, &proposed);
                            for v in diff.violations {
                                if !relevant_violations
                                    .iter()
                                    .any(|rv| rv.message == v.message && rv.location == v.location)
                                    && v.location.as_ref().is_some_and(|l| {
                                        file_list.iter().any(|f| l.contains(f))
                                            || impacted_ids.contains(l)
                                    })
                                {
                                    relevant_violations.push(v);
                                }
                            }
                        }
                    }
                }
            }

            if relevant_violations.is_empty() {
                finish(Ok(
                    "✅ No architectural violations detected for the changed files.".to_string(),
                ))
            } else {
                let mut out = "⚠️ Architectural violations detected:\n\n".to_string();
                for v in relevant_violations {
                    out.push_str(&format!(
                        "- [{:?}] {}{}: {}\n",
                        v.severity,
                        v.location.as_deref().unwrap_or("Unknown"),
                        v.rule_id
                            .as_ref()
                            .map(|r| format!(" ({})", r))
                            .unwrap_or_default(),
                        v.message
                    ));
                }
                out.push_str("\nPlease review these findings before committing.");
                finish(Ok(out))
            }
        }
        _ => Ok(None),
    }
}

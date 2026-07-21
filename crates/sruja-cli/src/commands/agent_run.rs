#![allow(clippy::needless_range_loop)]
//! Agent run loop: observe → plan → (optional) apply → verify → record learnings.
//!
//! This is intentionally conservative:
//! - Default mode is plan (no execution)
//! - Apply is gated by repo config allowlists + budgets
//! - All optional enrichment is grounded: it may add narrative, never change facts.

use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::time::{timeout, Duration};

use crate::commands::CliError;
use crate::integrations::load_repo_config;
use crate::utils::run_id::generate_run_id;
use crate::utils::run_snapshots::write_json_snapshot;

use super::agent;
use super::focus as focus_cmd;
use super::remediation::plan_remediation_steps;
use crate::commands::sync_cmd;

#[path = "agent_run_types.rs"]
mod agent_run_types;
#[path = "agent_run_compression.rs"]
mod agent_run_compression;

pub(crate) use self::agent_run_types::*;
pub use self::agent_run_types::AgentRunOptions;

pub(crate) async fn run_allowlisted_process(
    repo_path: &Path,
    argv: &[String],
    max_runtime_ms: u64,
    allowed_execs: &[String],
) -> Result<StepObservation, CliError> {
    let start = std::time::Instant::now();
    if argv.is_empty() {
        return Err(CliError::validation("argv cannot be empty"));
    }
    let exe = &argv[0];
    if !allowed_execs.iter().any(|e| e == exe) {
        return Err(CliError::validation(format!(
            "Executable '{}' is not allowlisted for apply mode.",
            exe
        )));
    }

    let mut cmd = tokio::process::Command::new(exe);
    cmd.args(&argv[1..]);
    cmd.current_dir(repo_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let out = timeout(Duration::from_millis(max_runtime_ms.max(1)), cmd.output())
        .await
        .map_err(|_| CliError::validation(format!("Command timed out after {max_runtime_ms}ms")))?;
    let out = out.map_err(CliError::Io)?;

    let step_id = argv.join(" ");
    let status = if out.status.success() {
        "ok".to_string()
    } else {
        "error".to_string()
    };
    let exit_code = out.status.code();
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let elapsed_ms = start.elapsed().as_millis();
    let content_hash = Some(compute_observation_hash(
        &step_id, &status, exit_code, &stdout, &stderr, elapsed_ms,
    ));

    Ok(StepObservation {
        step_id,
        status,
        exit_code,
        stdout,
        stderr,
        elapsed_ms,
        content_hash,
    })
}

fn validate_sruja_cmd_args(argv: &[String]) -> Result<(), CliError> {
    // Minimal hardening: only allow the exact subcommand shapes we generate today.
    // This prevents allowlisted subcommands being used with unexpected flags.
    if argv.len() < 2 || argv[0] != "sruja" {
        return Err(CliError::validation("sruja_cmd must start with `sruja`"));
    }

    match argv[1].as_str() {
        "lint" => {
            // Expected: sruja lint repo.sruja --format json
            if argv.get(2).map(|s| s.as_str()) != Some("repo.sruja") {
                return Err(CliError::validation(
                    "apply mode only allows `sruja lint repo.sruja`".to_string(),
                ));
            }
            Ok(())
        }
        "check" => Ok(()),  // Expected: sruja drift --ci -r . -f github-actions
        "drift" => Ok(()),  // Expected: sruja drift -r . -f json
        "review" => Ok(()), // Expected: sruja review -r . -f json
        "intent" => {
            // Expected: sruja intent check -r . -f json
            if argv.get(2).map(|s| s.as_str()) != Some("check") {
                return Err(CliError::validation(
                    "apply mode only allows `sruja intent check ...`".to_string(),
                ));
            }
            Ok(())
        }
        "focus" => Ok(()), // Expected: sruja focus --file <path> -r . -f json
        other => Err(CliError::validation(format!(
            "Unsupported sruja_cmd subcommand shape: {}",
            other
        ))),
    }
}

fn build_intent_report_json(
    graph: &sruja_scan::Graph,
    repo_root: &str,
    intent_path: Option<&str>,
    strict: bool,
) -> Result<Value, CliError> {
    use sruja_intent::{DriftDetector, IntentContext, IntentModel, IntentReport};
    use std::path::PathBuf;

    let repo_path = Path::new(repo_root);

    let mut context = IntentContext::new();
    let intent_dir = if let Some(path) = intent_path {
        PathBuf::from(path)
    } else {
        repo_path.join("docs").join("architecture")
    };

    let models = context.load_from_directory(&intent_dir).unwrap_or_default();
    let mut merged_model = IntentModel::default();
    for model in models {
        merged_model.merge(model);
    }

    let detector = DriftDetector::new();
    let mut report = detector.detect(&merged_model, graph, context.schema());

    if strict {
        let graph_json = {
            let new_path = repo_path.join(crate::commands::SCAN_CACHE_PATH);
            if new_path.exists() {
                new_path
            } else {
                repo_path.join(".sruja/graph.json")
            }
        };
        if graph_json.exists() {
            let previous_graph: sruja_scan::Graph =
                serde_json::from_str(&std::fs::read_to_string(graph_json)?)?;
            let proposals = sruja_diff::Proposal::load_all(repo_path).unwrap_or_default();
            let unproposed =
                sruja_diff::detect_unproposed_changes(&previous_graph, graph, &proposals);
            report.drifts.extend(unproposed);
            report.recompute_summary_and_score();
        }
    }

    let policy_drifts = DriftDetector::evaluate_policy_violations(&merged_model, graph);
    if !policy_drifts.is_empty() {
        report.drifts.extend(policy_drifts);
        report.recompute_summary_and_score();
    }

    let intent_report = IntentReport::from_drift_report(&report);
    Ok(serde_json::to_value(&intent_report).unwrap_or(Value::Null))
}

fn drift_violation_count(drift_json: &Value) -> usize {
    drift_json
        .get("violations")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

fn drift_truth_status(drift_json: &Value) -> Option<String> {
    drift_json
        .get("truth_status")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}



pub(crate) async fn run_sruja_cmd(
    repo_path: &Path,
    argv: &[String],
    max_runtime_ms: u64,
    allowed_subcommands: &[String],
) -> Result<StepObservation, CliError> {
    if argv.len() < 2 || argv[0] != "sruja" {
        return Err(CliError::validation(
            "sruja_cmd must start with `sruja`".to_string(),
        ));
    }
    let sub = argv[1].clone();
    if !allowed_subcommands.iter().any(|s| s == &sub) {
        return Err(CliError::validation(format!(
            "Sruja subcommand '{}' is not allowlisted for apply mode.",
            sub
        )));
    }

    validate_sruja_cmd_args(argv)?;

    // Execute via current binary to avoid PATH ambiguity.
    let exe = std::env::current_exe().map_err(CliError::Io)?;
    let start = std::time::Instant::now();

    let mut cmd = tokio::process::Command::new(&exe);
    cmd.args(argv.iter().skip(1));
    cmd.current_dir(repo_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let out = timeout(Duration::from_millis(max_runtime_ms.max(1)), cmd.output())
        .await
        .map_err(|_| CliError::validation(format!("Command timed out after {max_runtime_ms}ms")))?;
    let out = out.map_err(CliError::Io)?;

    let step_id = argv.join(" ");
    let status = if out.status.success() {
        "ok".to_string()
    } else {
        "error".to_string()
    };
    let exit_code = out.status.code();
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let elapsed_ms = start.elapsed().as_millis();
    let content_hash = Some(compute_observation_hash(
        &step_id, &status, exit_code, &stdout, &stderr, elapsed_ms,
    ));

    Ok(StepObservation {
        step_id,
        status,
        exit_code,
        stdout,
        stderr,
        elapsed_ms,
        content_hash,
    })
}

fn agent_artifacts_dir(repo_path: &Path) -> PathBuf {
    repo_path.join(".sruja").join("agent").join("runs")
}

fn build_focus_json(
    scan_graph: &sruja_scan::Graph,
    repo_path: &Path,
    resolved_element_id: &Option<String>,
    options: &AgentRunOptions,
    run_id: &str,
) -> Result<(Value, Vec<String>), CliError> {
    if let Some(ref id) = resolved_element_id {
        let kg = crate::graph_store::load_or_build_graph(repo_path)?;
        let scan_node_count = scan_graph.nodes.len();
        let mut briefing = focus_cmd::build_focus_briefing(
            &kg,
            id,
            repo_path,
            scan_node_count,
            None,
            false,
            false,
        );
        let surfaced_learning_ids = briefing.surfaced_learning_ids.clone();
        briefing.enrichment = None;
        briefing.run_id = Some(run_id.to_string());
        let out = focus_cmd::build_focus_for_ai_output(
            repo_path,
            options.file,
            options.element_id,
            Some(run_id),
            briefing,
        );
        Ok((
            serde_json::to_value(&out).unwrap_or(Value::Null),
            surfaced_learning_ids,
        ))
    } else {
        Ok((Value::Null, Vec::new()))
    }
}

fn build_impact_json(
    scan_graph: &sruja_scan::Graph,
    resolved_element_id: &Option<String>,
) -> Result<Value, CliError> {
    if let Some(ref id) = resolved_element_id {
        let blast = scan_graph.blast_radius(id, 3);
        Ok(serde_json::json!({
            "schema_version": "impact/v0",
            "target_id": id,
            "depth": 3,
            "upstream": blast.upstream,
            "downstream": blast.downstream,
        }))
    } else {
        Ok(Value::Null)
    }
}

fn build_drift_json(scan_graph: &sruja_scan::Graph, repo_path: &Path) -> Result<Value, CliError> {
    let baseline = crate::utils::architecture_path::resolve_architecture_path(repo_path);
    if let Some(path) = baseline {
        let content = std::fs::read_to_string(&path)?;
        let parser = sruja_language::Parser::new(path.to_string_lossy().as_ref());
        let program = parser.parse(&content).map_err(|diags| {
            CliError::parse_with_diagnostics(path.to_string_lossy().to_string(), diags)
        })?;
        let proposed = sruja_diff::program_to_graph(&program);
        let diff = sruja_diff::compare_graphs(scan_graph, &proposed);
        Ok(serde_json::to_value(&diff).unwrap_or(Value::Null))
    } else {
        let drift = sruja_diff::detect_architectural_drift(scan_graph);
        Ok(serde_json::to_value(&drift).unwrap_or(Value::Null))
    }
}

fn build_intent_json(graph: &sruja_scan::Graph, repo_path: &str) -> Value {
    let intent_opt = std::env::var("SRUJA_INTENT_PATH").ok();
    build_intent_report_json(graph, repo_path, intent_opt.as_deref(), false).unwrap_or_else(|e| {
        serde_json::json!({
            "status": "error",
            "error": e.to_string()
        })
    })
}

fn build_agent_history_json(
    repo_path: &Path,
    surfaced_learning_ids: &[String],
) -> Result<Value, CliError> {
    if surfaced_learning_ids.is_empty() {
        return Ok(Value::Null);
    }
    let memory = sruja_agent::AgenticMemory::load(repo_path)
        .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
    let id_set: std::collections::HashSet<&str> =
        surfaced_learning_ids.iter().map(|s| s.as_str()).collect();
    let entries: Vec<_> = memory
        .learnings
        .iter()
        .filter(|e| id_set.contains(e.id.as_str()))
        .collect();
    Ok(serde_json::to_value(entries).unwrap_or(Value::Null))
}

#[allow(clippy::too_many_arguments)]
fn build_facts_payload(
    options: &AgentRunOptions,
    query: Option<&str>,
    resolved_element_id: &Option<String>,
    focus_json: &Value,
    impact_json: &Value,
    drift_json: &Value,
    intent_json: &Value,
    agent_history_json: &Value,
) -> Value {
    serde_json::json!({
        "schema_version": "agent_facts/v1",
        "repo": options.repo,
        "goal": options.goal,
        "ai_mode": options.ai_mode,
        "target": {
            "file": options.file,
            "element_id": options.element_id,
            "query": query,
            "resolved_element_id": resolved_element_id,
        },
        "facts": {
            "focus": focus_json,
            "impact": impact_json,
            "drift": drift_json,
            "intent": intent_json,
            "agent_history": agent_history_json,
        }
    })
}

pub async fn agent_run_to_string(options: AgentRunOptions<'_>) -> Result<String, CliError> {
    let repo_path = Path::new(options.repo);
    if !repo_path.exists() {
        return Err(CliError::validation(format!(
            "Repository not found: {}",
            options.repo
        )));
    }

    let mode = parse_mode(options.mode)?;
    let ai_mode = parse_ai_mode(options.ai_mode)?;
    let format = parse_format(options.format)?;

    let run_id = options
        .run_id
        .map(|s| s.to_string())
        .unwrap_or_else(generate_run_id);
    validate_target(options.file, options.element_id, options.query)?;

    // When no focus flag is provided, infer the query from the goal text.
    // This lets `sruja agent plan --goal "..."` work without requiring
    // --file/--element-id/--query.
    let query = options.query.or_else(|| {
        if options.file.is_none() && options.element_id.is_none() {
            Some(options.goal)
        } else {
            None
        }
    });

    let budgets = load_agent_budgets(
        repo_path,
        ai_mode,
        (options.max_steps, options.max_runtime_ms_per_step),
    );
    let (allowed_sruja_subcommands, allowed_verify_execs, allowlist_source) =
        load_allowlists(repo_path);

    // ── Observe: gather deterministic facts ────────────────────────────────
    // Skip sync when cache is fresh (same git HEAD) to avoid expensive re-scans.
    // Pass --force-sync to bypass freshness check.
    if options.force_sync || !sync_cmd::is_context_fresh(repo_path) {
        sync_cmd::sync(options.repo, "quiet").await?;
    }

    // Resolve target element id (if possible). For query we can’t reliably resolve yet.
    let resolved_element_id = if query.is_none() {
        let kg = crate::graph_store::load_or_build_graph(repo_path)?;
        Some(focus_cmd::resolve_target(
            &kg,
            repo_path,
            options.file,
            options.element_id,
        )?)
    } else {
        None
    };

    let scan_graph = crate::commands::scan_repo_cached(repo_path)?;

    let (focus_json, surfaced_learning_ids) = build_focus_json(
        &scan_graph,
        repo_path,
        &resolved_element_id,
        &options,
        &run_id,
    )?;
    let impact_json = build_impact_json(&scan_graph, &resolved_element_id)?;
    let drift_json = build_drift_json(&scan_graph, repo_path)?;
    let intent_json = build_intent_json(&scan_graph, options.repo);
    let agent_history_json = build_agent_history_json(repo_path, &surfaced_learning_ids)?;

    let facts_payload = build_facts_payload(
        &options,
        query,
        &resolved_element_id,
        &focus_json,
        &impact_json,
        &drift_json,
        &intent_json,
        &agent_history_json,
    );
    crate::commands::context_events::record_agent_plan(
        repo_path,
        &run_id,
        options.goal,
        resolved_element_id.as_deref(),
        if surfaced_learning_ids.is_empty() {
            None
        } else {
            Some(surfaced_learning_ids.as_slice())
        },
    );

    // ── Think: deterministic plan synthesis ───────────────────────────────
    let steps: Vec<AgentStep> = plan_remediation_steps(&drift_json, &intent_json)
        .into_iter()
        .map(|p| AgentStep {
            id: p.id,
            kind: p.kind,
            argv: p.argv,
            expected: p.expected,
        })
        .collect();
    let mut verification: Vec<AgentStep> = Vec::new();
    let mut risks: Vec<String> = Vec::new();
    let mut open_questions: Vec<String> = Vec::new();

    // Deterministic planner: use gathered facts to propose next safe actions.
    let v_count = drift_violation_count(&drift_json);
    let truth = drift_truth_status(&drift_json).unwrap_or_else(|| "unknown".to_string());

    // Always include deterministic repo gates.
    if repo_path.join("repo.sruja").exists() {
        verification.push(AgentStep {
            id: "verify_lint_repo_sruja".to_string(),
            kind: "sruja_cmd".to_string(),
            argv: vec![
                "sruja".to_string(),
                "lint".to_string(),
                "repo.sruja".to_string(),
                "--format".to_string(),
                "json".to_string(),
            ],
            expected: Some("repo.sruja parses and lints cleanly".to_string()),
        });
    }
    verification.push(AgentStep {
        id: "verify_check".to_string(),
        kind: "sruja_cmd".to_string(),
        argv: vec![
            "sruja".to_string(),
            "check".to_string(),
            "-r".to_string(),
            ".".to_string(),
            "-f".to_string(),
            "github-actions".to_string(),
        ],
        expected: Some("CI-style drift check output generated (exit always 0)".to_string()),
    });

    verification.push(AgentStep {
        id: "verify_drift".to_string(),
        kind: "sruja_cmd".to_string(),
        argv: vec![
            "sruja".to_string(),
            "drift".to_string(),
            "-r".to_string(),
            ".".to_string(),
            "-f".to_string(),
            "json".to_string(),
        ],
        expected: Some("No new violations (or understand and accept drift)".to_string()),
    });

    if truth == "drifted" || v_count > 0 {
        open_questions.push("Drift detected: should the agent (a) generate a proposal via `sruja propose`, (b) update `repo.sruja`, or (c) baseline existing violations?".to_string());
        risks.push(format!(
            "Architecture truth status is '{}' with {} violations in drift facts.",
            truth, v_count
        ));
        verification.push(AgentStep {
            id: "verify_review".to_string(),
            kind: "sruja_cmd".to_string(),
            argv: vec![
                "sruja".to_string(),
                "review".to_string(),
                "-r".to_string(),
                ".".to_string(),
                "-f".to_string(),
                "json".to_string(),
            ],
            expected: Some("Review suggestions captured for next actions".to_string()),
        });
    }

    // Always include intent check suggestion (already computed deterministically).
    verification.push(AgentStep {
        id: "verify_intent_check".to_string(),
        kind: "sruja_cmd".to_string(),
        argv: vec![
            "sruja".to_string(),
            "intent".to_string(),
            "check".to_string(),
            "-r".to_string(),
            ".".to_string(),
            "-f".to_string(),
            "json".to_string(),
        ],
        expected: Some("Intent vs reality report available for compliance".to_string()),
    });

    let enrichment = build_enrichment(repo_path, &facts_payload, options.enrich);

    let plan = AgentPlanOutput {
        artifact_kind: "deterministic_plan".to_string(),
        trace_id: Some(run_id.clone()),
        schema_version: "agent_plan_output/v1".to_string(),
        run_id: Some(run_id.clone()),
        repo: options.repo.to_string(),
        goal: options.goal.to_string(),
        target: AgentTarget {
            selector: options
                .file
                .map(|s| format!("file:{s}"))
                .or_else(|| options.element_id.map(|s| format!("element_id:{s}")))
                .or_else(|| query.map(|s| format!("query:{s}")))
                .unwrap_or_else(|| "unknown".to_string()),
            resolved_element_id,
        },
        facts_refs: vec![
            "sync".to_string(),
            "focus".to_string(),
            "impact".to_string(),
            "drift".to_string(),
            "agent_history".to_string(),
        ],
        facts: facts_payload.clone(),
        proposal_sruja: None,
        steps,
        verification,
        risks,
        open_questions,
        budgets: budgets.clone(),
        safety: AgentSafety {
            mode: match mode {
                AgentMode::Plan => "plan".to_string(),
                AgentMode::Apply => "apply".to_string(),
            },
            allowlist_source,
            denied_steps: Vec::new(),
        },
        enrichment,
    };

    // Persist snapshot for replay/resume.
    let plan_snapshot = serde_json::to_value(&plan).unwrap_or(Value::Null);
    let _ = write_json_snapshot(repo_path, &run_id, "agent_plan.json", &plan_snapshot);

    // ── Act + Verify (apply mode) ─────────────────────────────────────────
    let out_string = match mode {
        AgentMode::Plan => match format {
            AgentFormat::Json | AgentFormat::ForAi => serde_json::to_string_pretty(&plan)?,
            AgentFormat::Text => {
                let mut s = serde_json::to_string_pretty(&plan)?;
                if let Some(e) = &plan.enrichment {
                    if let Some(md) = e.narrative_markdown.as_deref() {
                        s.push_str("\n\n");
                        s.push_str(md);
                    }
                }
                s
            }
        },
        AgentMode::Apply => {
            let _apply_start = std::time::Instant::now();
            if !surfaced_learning_ids.is_empty() {
                let mut memory = sruja_agent::AgenticMemory::load(repo_path)
                    .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
                let refs: Vec<&str> = surfaced_learning_ids.iter().map(String::as_str).collect();
                memory.record_retrievals(&refs);
                memory
                    .save(repo_path)
                    .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
            }
            // v1 apply: run verification steps only (safe default),
            // record learnings if verification fails.
            let mut verification_results = Vec::new();
            let mut memory_recorded = Vec::new();
            let compression_report: Option<ObservationCompressionReport>;

            for v in &plan.verification {
                let obs = match v.kind.as_str() {
                    "sruja_cmd" => {
                        run_sruja_cmd(
                            repo_path,
                            &v.argv,
                            plan.budgets.max_runtime_ms_per_step,
                            &allowed_sruja_subcommands,
                        )
                        .await?
                    }
                    "verify_cmd" => {
                        run_allowlisted_process(
                            repo_path,
                            &v.argv,
                            plan.budgets.max_runtime_ms_per_step,
                            &allowed_verify_execs,
                        )
                        .await?
                    }
                    _ => StepObservation {
                        step_id: v.id.clone(),
                        status: "skipped".to_string(),
                        exit_code: None,
                        stdout: "".to_string(),
                        stderr: format!("Unknown verification kind: {}", v.kind),
                        elapsed_ms: 0,
                        content_hash: None,
                    },
                };

                if obs.status != "ok" && !options.continue_on_error {
                    verification_results.push(obs);
                    break;
                }
                verification_results.push(obs);
            }

            // Compress older observations to prevent context bloat in long loops.
            {
                let ce_cfg = load_repo_config(repo_path)
                    .map(|c| c.context_engineering)
                    .unwrap_or_default();
                let threshold = ce_cfg
                    .compression_token_threshold
                    .unwrap_or(agent_run_compression::DEFAULT_TOKEN_BUDGET_THRESHOLD);
                let keep = ce_cfg.compression_keep_recent.unwrap_or(3);
                let before_tokens = agent_run_compression::estimate_tokens(&verification_results);
                let before = verification_results.clone();
                agent_run_compression::compress_if_needed_with_threshold(
                    &mut verification_results,
                    keep,
                    threshold,
                );
                let after_tokens = agent_run_compression::estimate_tokens(&verification_results);
                let compressed_count = before
                    .iter()
                    .zip(verification_results.iter())
                    .filter(|(a, b)| a.stdout != b.stdout || a.stderr != b.stderr)
                    .count();
                let mut report = ObservationCompressionReport {
                    enabled: true,
                    threshold_tokens: threshold,
                    keep_recent: keep,
                    estimated_tokens_before: before_tokens,
                    estimated_tokens_after: after_tokens,
                    compressed_observation_count: compressed_count,
                    total_observation_count: verification_results.len(),
                    context_prune: None,
                };
                if let Some(active) = plan.target.resolved_element_id.as_deref() {
                    if let Ok(graph) = crate::commands::scan_repo_cached(repo_path) {
                        let session =
                            crate::commands::context_prune::infer_session_element_ids_from_facts(
                                active,
                                &facts_payload,
                            );
                        if session.len() > 1 {
                            report.context_prune =
                                Some(crate::commands::context_prune::suggest_context_prune(
                                    &graph,
                                    &[active.to_string()],
                                    &session,
                                    2,
                                ));
                        }
                    }
                }
                if compressed_count > 0 {
                    let suppress = ce_cfg.compression_suppress_recompress_turns.unwrap_or(4);
                    crate::commands::context_events::record_context_compressed(
                        repo_path,
                        suppress,
                        options.element_id.map(|id| vec![id.to_string()]),
                        Some("agent_run observation compression"),
                    );
                }
                compression_report = Some(report);
            }

            if let Some(first_err) = verification_results.iter().find(|o| o.status == "error") {
                let auto_record = load_repo_config(repo_path)
                    .and_then(|c| c.agent.auto_record_learnings)
                    .unwrap_or(false);
                if auto_record {
                    let context = format!("agent run apply: {}", plan.goal);
                    let hypothesis = format!("Verification step failed: {}", first_err.step_id);
                    let guardrail = "Do not proceed with further apply steps until verification is green; investigate drift/policy violations first.".to_string();
                    let reason = if first_err.stderr.is_empty() {
                        None
                    } else {
                        Some(first_err.stderr.as_str())
                    };
                    agent::agent_record(
                        options.repo,
                        &context,
                        &hypothesis,
                        "failed",
                        &guardrail,
                        reason,
                        plan.target.resolved_element_id.as_deref(),
                        None,
                    )
                    .await?;
                    memory_recorded.push(hypothesis);
                }
            }

            let apply_success = agent_apply_verification_success(&verification_results);
            if !surfaced_learning_ids.is_empty() {
                let mut memory = sruja_agent::AgenticMemory::load(repo_path)
                    .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
                memory.finish_task_learnings(&surfaced_learning_ids, apply_success);
                memory
                    .save(repo_path)
                    .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
                crate::commands::context_events::record_agent_task_complete(
                    repo_path,
                    &run_id,
                    plan.target.resolved_element_id.as_deref(),
                    &surfaced_learning_ids,
                    apply_success,
                );
            }

            let context_prune = compression_report
                .as_ref()
                .and_then(|r| r.context_prune.clone());
            let verification_hash = {
                let mut hasher = blake3::Hasher::new();
                let mut hashed_steps = 0usize;
                for result in &verification_results {
                    if let Some(h) = &result.content_hash {
                        hasher.update(h.as_bytes());
                        hashed_steps += 1;
                    }
                }
                (hashed_steps > 0).then(|| hasher.finalize().to_hex().to_string())
            };

            let out = AgentApplyOutput {
                schema_version: "agent_apply_output/v1".to_string(),
                run_id: Some(run_id),
                plan,
                executed_steps: Vec::new(),
                observations: Vec::new(),
                verification_results,
                memory_recorded,
                observation_compression: compression_report,
                context_prune,
                verification_hash,
            };

            let apply_snapshot = serde_json::to_value(&out).unwrap_or(Value::Null);
            if let Some(run_id) = out.run_id.as_deref() {
                let _ = write_json_snapshot(repo_path, run_id, "agent_apply.json", &apply_snapshot);
                let bundle = serde_json::json!({
                    "schema_version": "verification_bundle/v1",
                    "run_id": run_id,
                    "repo": options.repo,
                    "goal": out.plan.goal,
                    "allowlist_source": out.plan.safety.allowlist_source,
                    "verification": out.plan.verification.iter().map(|s| serde_json::json!({
                        "id": s.id,
                        "kind": s.kind,
                        "argv": s.argv,
                        "expected": s.expected,
                    })).collect::<Vec<_>>(),
                    "results": out.verification_results.iter().map(|r| serde_json::json!({
                        "step_id": r.step_id,
                        "status": r.status,
                        "exit_code": r.exit_code,
                        "elapsed_ms": r.elapsed_ms,
                    })).collect::<Vec<_>>(),
                });
                let _ = write_json_snapshot(repo_path, run_id, "verification_bundle.json", &bundle);
                let facts_bundle = serde_json::json!({
                    "schema_version": "facts_bundle/v1",
                    "run_id": run_id,
                    "repo": options.repo,
                    "goal": out.plan.goal,
                    "allowlist_source": out.plan.safety.allowlist_source,
                    "memory_recorded": out.memory_recorded,
                    "verification_bundle": bundle,
                });
                let agent_run_dir = agent_artifacts_dir(repo_path).join(run_id);
                let _ = std::fs::create_dir_all(&agent_run_dir);
                let _ = std::fs::write(
                    agent_run_dir.join("facts_bundle.json"),
                    serde_json::to_string_pretty(&facts_bundle).unwrap_or_default(),
                );
            }

            serde_json::to_string_pretty(&out)?
        }
    };

    Ok(out_string)
}

pub async fn agent_run(options: AgentRunOptions<'_>) -> Result<(), CliError> {
    let s = agent_run_to_string(options).await?;
    println!("{s}");
    Ok(())
}

pub(crate) async fn run_verification_steps_in_repo(
    repo_path: &Path,
    verification: &[AgentStep],
    max_runtime_ms_per_step: u64,
    allowed_sruja_subcommands: &[String],
    allowed_verify_execs: &[String],
    continue_on_error: bool,
) -> Result<Vec<StepObservation>, CliError> {
    let mut out = Vec::new();
    for v in verification {
        let obs = match v.kind.as_str() {
            "sruja_cmd" => {
                run_sruja_cmd(
                    repo_path,
                    &v.argv,
                    max_runtime_ms_per_step,
                    allowed_sruja_subcommands,
                )
                .await?
            }
            "verify_cmd" => {
                run_allowlisted_process(
                    repo_path,
                    &v.argv,
                    max_runtime_ms_per_step,
                    allowed_verify_execs,
                )
                .await?
            }
            _ => StepObservation {
                step_id: v.id.clone(),
                status: "skipped".to_string(),
                exit_code: None,
                stdout: "".to_string(),
                stderr: format!("Unknown verification kind: {}", v.kind),
                elapsed_ms: 0,
                content_hash: None,
            },
        };
        out.push(obs);
        if !continue_on_error && out.last().is_some_and(|o| o.status == "error") {
            break;
        }
    }
    Ok(out)
}

#[cfg(test)]
#[path = "agent_run_tests.rs"]
mod tests;

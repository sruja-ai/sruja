use crate::commands::CliError;
use crate::integrations::load_repo_config;
use crate::utils::run_id::generate_run_id;
use crate::utils::run_snapshots::write_json_snapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::agent_run::{
    run_allowlisted_process, run_sruja_cmd, AgentApplyOutput, AgentPlanOutput, AgentRunOptions,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentPlanFile {
    pub schema_version: String,
    pub plan_id: String,
    pub created_at: String,
    pub plan: AgentPlanOutput,
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if (ch.is_whitespace() || ch == '-' || ch == '_' || ch == '.') && !out.ends_with('-')
        {
            out.push('-');
        }
    }
    out.trim_matches('-').chars().take(40).collect()
}

fn default_plan_path(repo_root: &Path, run_id: &str, goal: &str) -> PathBuf {
    repo_root
        .join("docs")
        .join("plans")
        .join(format!("{}-{}.json", run_id, slugify(goal)))
}

pub fn write_plan_file(
    repo_root: &Path,
    plan: &AgentPlanOutput,
    out_path: Option<&Path>,
) -> Result<PathBuf, CliError> {
    let run_id = plan.run_id.as_deref().unwrap_or("run_unknown");
    let out = out_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| default_plan_path(repo_root, run_id, &plan.goal));

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = AgentPlanFile {
        schema_version: "agent_plan/v1".to_string(),
        plan_id: format!("plan_{}", run_id),
        created_at: chrono::Utc::now().to_rfc3339(),
        plan: plan.clone(),
    };

    std::fs::write(&out, serde_json::to_string_pretty(&file)?)?;
    Ok(out)
}

pub fn read_plan_file(path: &Path) -> Result<AgentPlanFile, CliError> {
    let content = std::fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<AgentPlanFile>(&content).map_err(|e| {
        CliError::validation(format!(
            "Failed to parse plan file {}: {}",
            path.to_string_lossy(),
            e
        ))
    })?;
    Ok(parsed)
}

pub async fn agent_plan(
    mut options: AgentRunOptions<'_>,
    out: Option<&Path>,
    print: bool,
) -> Result<(), CliError> {
    // Force plan mode & json; plan output is the canonical artifact.
    options.mode = "plan";
    options.format = "json";
    let repo_root = Path::new(options.repo);
    let run_id = options
        .run_id
        .map(|s| s.to_string())
        .unwrap_or_else(generate_run_id);

    // Build plan by calling the same observe/plan pipeline as agent_run_to_string.
    // We do it by reusing the existing function that returns JSON, then parsing.
    // This keeps all plan logic centralized in agent_run.
    let s = crate::commands::agent_run_to_string(AgentRunOptions {
        run_id: Some(&run_id),
        ..options
    })
    .await?;

    let plan: AgentPlanOutput = serde_json::from_str(&s).map_err(|e| {
        CliError::validation(format!("Failed to parse agent plan output JSON: {e}"))
    })?;

    // Persist as a first-class plan file.
    let out_path = write_plan_file(repo_root, &plan, out)?;

    if print {
        println!("{}", serde_json::to_string_pretty(&plan)?);
    } else {
        println!("{}", out_path.to_string_lossy());
    }
    Ok(())
}

pub async fn agent_apply(plan_path: &Path, repo: &str, _format: &str) -> Result<(), CliError> {
    let repo_root = Path::new(repo);
    if !repo_root.exists() {
        return Err(CliError::validation(format!(
            "Repository not found: {repo}"
        )));
    }

    let plan_file = read_plan_file(plan_path)?;
    let mut plan = plan_file.plan;

    // Enforce: apply must be executed from an on-disk plan.
    if plan.schema_version != "agent_plan_output/v1" {
        return Err(CliError::validation(format!(
            "Unsupported plan payload schema_version: {}",
            plan.schema_version
        )));
    }

    let run_id = plan.run_id.clone().unwrap_or_else(generate_run_id);
    plan.run_id = Some(run_id.clone());

    let (allowed_sruja_subcommands, allowed_verify_execs, allowlist_source) =
        super::agent_run::load_allowlists(repo_root);
    let budgets = plan.budgets.clone();

    // Apply: run verification steps only (current v1 behavior), but now from explicit plan.
    let apply_start = std::time::Instant::now();
    let mut verification_results = Vec::new();
    let mut memory_recorded = Vec::new();

    for v in &plan.verification {
        let obs = match v.kind.as_str() {
            "sruja_cmd" => {
                run_sruja_cmd(
                    repo_root,
                    &v.argv,
                    budgets.max_runtime_ms_per_step,
                    &allowed_sruja_subcommands,
                )
                .await?
            }
            "verify_cmd" => {
                run_allowlisted_process(
                    repo_root,
                    &v.argv,
                    budgets.max_runtime_ms_per_step,
                    &allowed_verify_execs,
                )
                .await?
            }
            _ => super::agent_run::StepObservation {
                step_id: v.id.clone(),
                status: "skipped".to_string(),
                exit_code: None,
                stdout: "".to_string(),
                stderr: format!("Unknown verification kind: {}", v.kind),
                elapsed_ms: 0,
            },
        };

        verification_results.push(obs);
    }

    // Optional: record a single guardrail if verification produced an error and config enables it.
    let auto_record = load_repo_config(repo_root)
        .and_then(|c| c.agent.auto_record_learnings)
        .unwrap_or(false);
    if auto_record {
        if let Some(first_err) = verification_results.iter().find(|o| o.status == "error") {
            let context = format!("agent apply: {}", plan.goal);
            let hypothesis = format!("Verification step failed: {}", first_err.step_id);
            let guardrail = "Do not proceed with further apply steps until verification is green; investigate drift/policy violations first.".to_string();
            let reason = if first_err.stderr.is_empty() {
                None
            } else {
                Some(first_err.stderr.as_str())
            };
            crate::commands::agent_record(
                repo,
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

    let mut context_prune = None;
    if let Some(active) = plan.target.resolved_element_id.clone() {
        if let Ok(graph) = crate::commands::scan_repo_cached(repo_root) {
            let mut session = vec![active.clone()];
            for v in &plan.verification {
                session.push(v.id.clone());
            }
            session.sort();
            session.dedup();
            if session.len() > 1 {
                context_prune = Some(crate::commands::context_prune::suggest_context_prune(
                    &graph,
                    &[active],
                    &session,
                    2,
                ));
            }
        }
    }

    let out = AgentApplyOutput {
        schema_version: "agent_apply_output/v1".to_string(),
        run_id: Some(run_id.clone()),
        plan: plan.clone(),
        executed_steps: Vec::new(),
        observations: Vec::new(),
        verification_results,
        memory_recorded,
        observation_compression: None,
        context_prune,
    };

    let apply_snapshot = serde_json::to_value(&out).unwrap_or(Value::Null);
    let _ = write_json_snapshot(repo_root, &run_id, "agent_apply.json", &apply_snapshot);
    let _ = std::fs::create_dir_all(repo_root.join(".sruja").join("agent").join("runs"));

    let elapsed = apply_start.elapsed().as_millis();
    let meta = serde_json::json!({ "elapsed_ms": elapsed, "allowlist_source": allowlist_source });
    let _ = write_json_snapshot(repo_root, &run_id, "agent_apply_meta.json", &meta);
    let bundle = serde_json::json!({
        "schema_version": "verification_bundle/v1",
        "run_id": run_id,
        "repo": repo,
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
    let _ = write_json_snapshot(repo_root, &run_id, "verification_bundle.json", &bundle);

    println!("{}", serde_json::to_string_pretty(&out)?);

    Ok(())
}

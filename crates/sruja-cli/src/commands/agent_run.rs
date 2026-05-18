//! Agent run loop: observe → plan → (optional) apply → verify → record learnings.
//!
//! This is intentionally conservative:
//! - Default mode is plan (no execution)
//! - Apply is gated by repo config allowlists + budgets
//! - All optional enrichment is grounded: it may add narrative, never change facts.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::time::{timeout, Duration};

use crate::commands::CliError;
use crate::integrations::{load_repo_config, resolve_enrichment_plan, resolve_openai_auth};
use crate::integrations::{run_cmd_enrichment, run_openai_markdown};
use crate::utils::run_id::generate_run_id;
use crate::utils::run_snapshots::write_json_snapshot;

use super::agent;
use super::focus as focus_cmd;
use super::remediation::plan_remediation_steps;
use crate::commands::sync_cmd;
use sruja_agent::TrajectoryExecutor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentMode {
    Plan,
    Apply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentFormat {
    Text,
    Json,
    ForAi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AiMode {
    Standard,
    Conservative,
    Aggressive,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentRunOptions<'a> {
    pub repo: &'a str,
    pub goal: &'a str,
    pub file: Option<&'a str>,
    pub element_id: Option<&'a str>,
    pub query: Option<&'a str>,
    pub run_id: Option<&'a str>,
    pub mode: &'a str,
    pub ai_mode: &'a str,
    pub format: &'a str,
    pub max_steps: Option<usize>,
    pub max_runtime_ms_per_step: Option<u64>,
    pub enrich: bool,
    pub enrich_provider: Option<&'a str>,
    pub enrich_cmd: Option<&'a str>,
    pub enrich_model: Option<&'a str>,
    pub enrich_base_url: Option<&'a str>,
    pub enrich_timeout_ms: u64,
    pub enrich_max_bytes: usize,
    pub continue_on_error: bool,
    pub trajectories: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct AgentBudgets {
    pub(crate) max_steps: usize,
    pub(crate) max_runtime_ms_per_step: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct AgentTarget {
    pub(crate) selector: String,
    pub(crate) resolved_element_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct AgentStep {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) argv: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) expected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct AgentSafety {
    pub(crate) mode: String,
    pub(crate) allowlist_source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) denied_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct AgentPlanOutput {
    /// Always `deterministic_plan` — steps and verification are reproducible from repo facts.
    pub(crate) artifact_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) trace_id: Option<String>,
    pub(crate) schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) run_id: Option<String>,
    pub(crate) repo: String,
    pub(crate) goal: String,
    pub(crate) target: AgentTarget,
    pub(crate) facts_refs: Vec<String>,
    pub(crate) facts: Value,
    pub(crate) steps: Vec<AgentStep>,
    pub(crate) verification: Vec<AgentStep>,
    pub(crate) risks: Vec<String>,
    pub(crate) open_questions: Vec<String>,
    pub(crate) budgets: AgentBudgets,
    pub(crate) safety: AgentSafety,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) enrichment: Option<AgentEnrichment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct StepObservation {
    pub(crate) step_id: String,
    pub(crate) status: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) elapsed_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct AgentApplyOutput {
    pub(crate) schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) run_id: Option<String>,
    pub(crate) plan: AgentPlanOutput,
    pub(crate) executed_steps: Vec<String>,
    pub(crate) observations: Vec<StepObservation>,
    pub(crate) verification_results: Vec<StepObservation>,
    pub(crate) memory_recorded: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) observation_compression: Option<ObservationCompressionReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct ObservationCompressionReport {
    pub(crate) enabled: bool,
    pub(crate) threshold_tokens: usize,
    pub(crate) keep_recent: usize,
    pub(crate) estimated_tokens_before: usize,
    pub(crate) estimated_tokens_after: usize,
    pub(crate) compressed_observation_count: usize,
    pub(crate) total_observation_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) context_prune: Option<crate::commands::context_prune::ContextPruneSuggestion>,
}

/// Rolling summary compression for agent observation streams.
///
/// When accumulated step output exceeds `TOKEN_BUDGET_THRESHOLD`, older
/// observations are compressed to decision-only summaries: status, exit code,
/// and the first line of stderr (if error). This prevents context bloat in
/// long-running agent loops while preserving the signal needed for reasoning.
mod observation_compression {
    use super::StepObservation;

    pub const DEFAULT_TOKEN_BUDGET_THRESHOLD: usize = 8000;
    const CHARS_PER_TOKEN: usize = 4;
    pub const DEFAULT_MAX_COMPRESSED_OUTPUT_LEN: usize = 120;
    const MAX_COMPRESSED_OUTPUT_LEN: usize = DEFAULT_MAX_COMPRESSED_OUTPUT_LEN;

    pub(crate) fn estimate_tokens(observations: &[StepObservation]) -> usize {
        observations
            .iter()
            .map(|o| {
                (o.step_id.len() + o.status.len() + o.stdout.len() + o.stderr.len())
                    / CHARS_PER_TOKEN
            })
            .sum()
    }

    fn compress_single(obs: &StepObservation) -> StepObservation {
        let compressed_stdout = if obs.stdout.len() > MAX_COMPRESSED_OUTPUT_LEN {
            let first_line = obs.stdout.lines().next().unwrap_or("");
            if first_line.len() > MAX_COMPRESSED_OUTPUT_LEN {
                format!("{}...", &first_line[..MAX_COMPRESSED_OUTPUT_LEN])
            } else {
                format!("{} [+{} chars compressed]", first_line, obs.stdout.len())
            }
        } else {
            obs.stdout.clone()
        };

        let compressed_stderr = if obs.stderr.len() > MAX_COMPRESSED_OUTPUT_LEN {
            let first_line = obs.stderr.lines().next().unwrap_or("");
            if first_line.len() > MAX_COMPRESSED_OUTPUT_LEN {
                format!("{}...", &first_line[..MAX_COMPRESSED_OUTPUT_LEN])
            } else {
                format!("{} [+{} chars compressed]", first_line, obs.stderr.len())
            }
        } else {
            obs.stderr.clone()
        };

        StepObservation {
            step_id: obs.step_id.clone(),
            status: obs.status.clone(),
            exit_code: obs.exit_code,
            stdout: compressed_stdout,
            stderr: compressed_stderr,
            elapsed_ms: obs.elapsed_ms,
        }
    }

    /// Compresses older observations when total token count exceeds the budget.
    ///
    /// The most recent `keep_recent` observations are preserved verbatim.
    /// Older observations are reduced to decision-only summaries.
    #[cfg(test)]
    pub fn compress_if_needed(observations: &mut [StepObservation], keep_recent: usize) {
        compress_if_needed_with_threshold(
            observations,
            keep_recent,
            DEFAULT_TOKEN_BUDGET_THRESHOLD,
        );
    }

    pub fn compress_if_needed_with_threshold(
        observations: &mut [StepObservation],
        keep_recent: usize,
        threshold: usize,
    ) {
        if estimate_tokens(observations) <= threshold {
            return;
        }

        let total = observations.len();
        if total <= keep_recent {
            return;
        }

        let compress_count = total - keep_recent;
        for obs in observations.iter_mut().take(compress_count) {
            *obs = compress_single(obs);
        }
    }
}

fn agent_enrichment(
    status: &str,
    provider: &str,
    model: Option<String>,
    error: Option<String>,
    narrative_markdown: Option<String>,
) -> AgentEnrichment {
    AgentEnrichment {
        artifact_kind: "llm_interpretation".to_string(),
        status: status.to_string(),
        provider: provider.to_string(),
        model,
        error,
        narrative_markdown,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentEnrichment {
    /// Always `llm_interpretation` when present — narrative only; never changes facts.
    artifact_kind: String,
    status: String,
    provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    narrative_markdown: Option<String>,
}

fn parse_mode(mode: &str) -> Result<AgentMode, CliError> {
    match mode.to_lowercase().as_str() {
        "plan" => Ok(AgentMode::Plan),
        "apply" => Ok(AgentMode::Apply),
        other => Err(CliError::validation(format!(
            "Invalid --mode '{}'. Use 'plan' or 'apply'.",
            other
        ))),
    }
}

fn parse_format(format: &str) -> Result<AgentFormat, CliError> {
    match format.to_lowercase().as_str() {
        "text" => Ok(AgentFormat::Text),
        "json" => Ok(AgentFormat::Json),
        "for-ai" | "for_ai" => Ok(AgentFormat::ForAi),
        other => Err(CliError::validation(format!(
            "Invalid --format '{}'. Use 'text', 'json', or 'for-ai'.",
            other
        ))),
    }
}

fn parse_ai_mode(ai_mode: &str) -> Result<AiMode, CliError> {
    match ai_mode.to_lowercase().as_str() {
        "standard" => Ok(AiMode::Standard),
        "conservative" => Ok(AiMode::Conservative),
        "aggressive" => Ok(AiMode::Aggressive),
        other => Err(CliError::validation(format!(
            "Invalid --ai-mode '{}'. Use 'standard', 'conservative', or 'aggressive'.",
            other
        ))),
    }
}

fn validate_target(
    file: Option<&str>,
    element_id: Option<&str>,
    query: Option<&str>,
) -> Result<(), CliError> {
    let count = file.is_some() as u8 + element_id.is_some() as u8 + query.is_some() as u8;
    if count != 1 {
        return Err(CliError::validation(
            "Exactly one of --file, --element-id, or --query must be provided.".to_string(),
        ));
    }
    Ok(())
}

fn default_budgets() -> AgentBudgets {
    AgentBudgets {
        max_steps: 8,
        max_runtime_ms_per_step: 30_000,
    }
}

fn budgets_for_mode(mode: AiMode) -> AgentBudgets {
    match mode {
        AiMode::Conservative => AgentBudgets {
            max_steps: 5,
            max_runtime_ms_per_step: 20_000,
        },
        AiMode::Standard => default_budgets(),
        AiMode::Aggressive => AgentBudgets {
            max_steps: 12,
            max_runtime_ms_per_step: 45_000,
        },
    }
}

fn load_agent_budgets(
    repo_path: &Path,
    ai_mode: AiMode,
    overrides: (Option<usize>, Option<u64>),
) -> AgentBudgets {
    // Conservative defaults; config may tighten or loosen later.
    // We treat overrides as hints, bounded by config once agent config exists.
    let mut b = budgets_for_mode(ai_mode);
    if let Some(ms) = overrides.1 {
        b.max_runtime_ms_per_step = ms;
    }
    if let Some(steps) = overrides.0 {
        b.max_steps = steps;
    }

    // Clamp using repo config if present.
    if let Some(cfg) = load_repo_config(repo_path) {
        if let Some(max_steps) = cfg.agent.max_steps {
            b.max_steps = b.max_steps.min(max_steps.max(1));
        }
        if let Some(max_ms) = cfg.agent.max_runtime_ms_per_step {
            b.max_runtime_ms_per_step = b.max_runtime_ms_per_step.min(max_ms.max(1));
        }
    }
    b
}

fn default_allowed_sruja_subcommands() -> Vec<String> {
    vec![
        "sync".to_string(),
        "check".to_string(),
        "drift".to_string(),
        "review".to_string(),
        "lint".to_string(),
        "intent".to_string(), // for `intent check`
    ]
}

fn default_allowed_verify_executables() -> Vec<String> {
    vec![
        "cargo".to_string(),
        "npm".to_string(),
        "just".to_string(),
        "make".to_string(),
    ]
}

pub(crate) fn load_allowlists(repo_path: &Path) -> (Vec<String>, Vec<String>, String) {
    if let Some(cfg) = load_repo_config(repo_path) {
        let sruja = cfg
            .agent
            .allowed_sruja_subcommands
            .unwrap_or_else(default_allowed_sruja_subcommands);
        let verify = cfg
            .agent
            .allowed_verify_executables
            .unwrap_or_else(default_allowed_verify_executables);
        return (sruja, verify, ".sruja/config.toml [agent]".to_string());
    }
    (
        default_allowed_sruja_subcommands(),
        default_allowed_verify_executables(),
        "defaults".to_string(),
    )
}

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

    Ok(StepObservation {
        step_id: argv.join(" "),
        status: if out.status.success() {
            "ok".to_string()
        } else {
            "error".to_string()
        },
        exit_code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        elapsed_ms: start.elapsed().as_millis(),
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
        "check" => Ok(()),  // Expected: sruja check -r . -f github-actions
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
        other => Err(CliError::validation(format!(
            "Unsupported sruja_cmd subcommand shape: {}",
            other
        ))),
    }
}

fn build_intent_report_json(
    repo_root: &str,
    intent_path: Option<&str>,
    strict: bool,
) -> Result<Value, CliError> {
    use sruja_intent::{DriftDetector, IntentContext, IntentModel, IntentReport};
    use std::path::PathBuf;

    let repo_path = Path::new(repo_root);
    let graph = sruja_scan::scan_repo(repo_path)?;

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
    let mut report = detector.detect(&merged_model, &graph, context.schema());

    if strict {
        let graph_json = repo_path.join(".sruja").join("graph.json");
        if graph_json.exists() {
            let previous_graph: sruja_scan::Graph =
                serde_json::from_str(&std::fs::read_to_string(graph_json)?)?;
            let proposals = sruja_diff::Proposal::load_all(repo_path).unwrap_or_default();
            let unproposed =
                sruja_diff::detect_unproposed_changes(&previous_graph, &graph, &proposals);
            report.drifts.extend(unproposed);
            report.recompute_summary_and_score();
        }
    }

    let policy_drifts = crate::compliance::evaluate_policy_violations(&merged_model, &graph);
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

#[allow(clippy::too_many_arguments)]
fn build_enrichment(
    repo_path: &Path,
    facts_payload: &Value,
    enrich: bool,
    enrich_provider: Option<&str>,
    enrich_cmd: Option<&str>,
    enrich_model: Option<&str>,
    enrich_base_url: Option<&str>,
    enrich_timeout_ms: u64,
    enrich_max_bytes: usize,
) -> Option<AgentEnrichment> {
    if !enrich && enrich_cmd.is_none() {
        return None;
    }

    let plan = resolve_enrichment_plan(
        repo_path,
        enrich_cmd,
        enrich_model,
        enrich_base_url,
        Some(enrich_timeout_ms),
        Some(enrich_max_bytes),
    );
    let provider = enrich_provider.unwrap_or(plan.provider.as_str());
    let limits = plan.limits;
    let stdin_payload = serde_json::to_vec(facts_payload).unwrap_or_default();

    if provider == "cmd" {
        let Some(cmd) = plan.cmd.as_deref() else {
            return Some(agent_enrichment(
                "skipped",
                "cmd",
                None,
                Some("No command configured. Pass --enrich-cmd or set SRUJA_ENRICH_CMD (or .sruja/config.toml [integrations].cmd).".to_string()),
                None,
            ));
        };
        return Some(match run_cmd_enrichment(cmd, &stdin_payload, limits) {
            Ok(md) => agent_enrichment("ok", "external_cmd", None, None, Some(md)),
            Err(e) => agent_enrichment("error", "external_cmd", None, Some(e), None),
        });
    }

    if provider != "openai" {
        return Some(agent_enrichment(
            "skipped",
            provider,
            None,
            Some(
                "Unsupported provider. Use provider=cmd (recommended) or provider=openai."
                    .to_string(),
            ),
            None,
        ));
    }

    let model = plan.model.as_deref().unwrap_or("gpt-4o-mini");
    let base_url = plan
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let Some(api_key) = resolve_openai_auth() else {
        return Some(agent_enrichment(
            "skipped",
            "openai",
            Some(model.to_string()),
            Some("Missing API key (set OPENAI_API_KEY or SRUJA_ENRICH_API_KEY; SRUJA_LLM_API_KEY is deprecated).".to_string()),
            None,
        ));
    };

    let user_prompt = format!(
        r#"You are assisting an AI coding agent.\n\nYou MUST only use the JSON facts provided below. Do not invent modules, APIs, or file paths. If something is unknown, say \"unknown\".\n\nProduce markdown with these sections:\n- \"One-paragraph plan\"\n- \"Risks / unknowns to verify\" (bullets)\n- \"Suggested test/verification steps\" (bullets)\n- \"Clarifying questions\" (bullets)\n\nJSON facts:\n{}"#,
        facts_payload
    );

    match run_openai_markdown(
        "You are a careful repo assistant. Never fabricate.",
        &user_prompt,
        model,
        base_url,
        &api_key,
    ) {
        Ok(md) => Some(agent_enrichment(
            "ok",
            "openai",
            Some(model.to_string()),
            None,
            Some(md),
        )),
        Err(e) => Some(agent_enrichment(
            "error",
            "openai",
            Some(model.to_string()),
            Some(e),
            None,
        )),
    }
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

    Ok(StepObservation {
        step_id: argv.join(" "),
        status: if out.status.success() {
            "ok".to_string()
        } else {
            "error".to_string()
        },
        exit_code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        elapsed_ms: start.elapsed().as_millis(),
    })
}

fn agent_artifacts_dir(repo_path: &Path) -> PathBuf {
    repo_path.join(".sruja").join("agent").join("runs")
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

    let budgets = load_agent_budgets(
        repo_path,
        ai_mode,
        (options.max_steps, options.max_runtime_ms_per_step),
    );
    let (allowed_sruja_subcommands, allowed_verify_execs, allowlist_source) =
        load_allowlists(repo_path);

    // ── Observe: gather deterministic facts ────────────────────────────────
    // Always run sync in both plan/apply; it’s the deterministic grounding step.
    sync_cmd::sync(options.repo, "quiet").await?;

    // Resolve target element id (if possible). For query we can’t reliably resolve yet.
    let resolved_element_id = if options.query.is_none() {
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

    let focus_json = if let Some(ref id) = resolved_element_id {
        // Build the same JSON as focus --format for-ai would emit.
        let kg = crate::graph_store::load_or_build_graph(repo_path)?;
        let scan_node_count = match sruja_scan::scan_repo(repo_path) {
            Ok(g) => g.nodes.len(),
            Err(_) => kg.nodes.len(),
        };
        let mut briefing =
            focus_cmd::build_focus_briefing(&kg, id, repo_path, scan_node_count, None);
        // No focus-specific enrichment here; agent enrichment is handled at the end.
        briefing.enrichment = None;
        briefing.run_id = Some(run_id.clone());
        let out = focus_cmd::build_focus_for_ai_output(
            repo_path,
            options.file,
            options.element_id,
            Some(&run_id),
            briefing,
        );
        serde_json::to_value(&out).unwrap_or(Value::Null)
    } else {
        Value::Null
    };

    let impact_json = if let Some(ref id) = resolved_element_id {
        let g = sruja_scan::scan_repo(repo_path)?;
        let blast = g.blast_radius(id, 3);
        serde_json::json!({
            "schema_version": "impact/v0",
            "target_id": id,
            "depth": 3,
            "upstream": blast.upstream,
            "downstream": blast.downstream,
        })
    } else {
        Value::Null
    };

    let drift_json = {
        // Reuse drift logic via compare_graphs when baseline exists, else drift detector.
        let graph = crate::commands::scan_repo_cached(repo_path)?;
        let baseline = crate::utils::architecture_path::resolve_architecture_path(repo_path);
        if let Some(path) = baseline {
            let content = std::fs::read_to_string(&path)?;
            let parser = sruja_language::Parser::new(path.to_string_lossy().as_ref());
            let program = parser.parse(&content).map_err(|diags| {
                CliError::parse_with_diagnostics(path.to_string_lossy().to_string(), diags)
            })?;
            let proposed = sruja_diff::program_to_graph(&program);
            let diff = sruja_diff::compare_graphs(&graph, &proposed);
            serde_json::to_value(&diff).unwrap_or(Value::Null)
        } else {
            let drift = sruja_diff::detect_architectural_drift(&graph);
            serde_json::to_value(&drift).unwrap_or(Value::Null)
        }
    };

    let intent_json = {
        let intent_opt = std::env::var("SRUJA_INTENT_PATH").ok();
        build_intent_report_json(options.repo, intent_opt.as_deref(), false).unwrap_or_else(|e| {
            serde_json::json!({
                "status": "error",
                "error": e.to_string()
            })
        })
    };

    let agent_history_json = if let Some(ref id) = resolved_element_id {
        // Directly load AgenticMemory (same as agent_history json mode), and filter.
        let memory = sruja_agent::AgenticMemory::load(repo_path)
            .map_err(|e| CliError::Io(std::io::Error::other(e.to_string())))?;
        let entries = memory.find_relevant(id);
        serde_json::to_value(entries).unwrap_or(Value::Null)
    } else {
        Value::Null
    };

    let facts_payload = serde_json::json!({
        "schema_version": "agent_facts/v1",
        "repo": options.repo,
        "goal": options.goal,
        "ai_mode": options.ai_mode,
        "target": {
            "file": options.file,
            "element_id": options.element_id,
            "query": options.query,
            "resolved_element_id": resolved_element_id,
        },
        "facts": {
            "focus": focus_json,
            "impact": impact_json,
            "drift": drift_json,
            "intent": intent_json,
            "agent_history": agent_history_json,
        }
    });

    crate::commands::context_events::record_agent_plan(
        repo_path,
        &run_id,
        options.goal,
        resolved_element_id.as_deref(),
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

    let enrichment = build_enrichment(
        repo_path,
        &facts_payload,
        options.enrich,
        options.enrich_provider,
        options.enrich_cmd,
        options.enrich_model,
        options.enrich_base_url,
        options.enrich_timeout_ms,
        options.enrich_max_bytes,
    );

    let plan = build_agent_plan_output(
        &run_id,
        &run_id,
        options.repo,
        options.goal,
        options.file,
        options.element_id,
        options.query,
        resolved_element_id,
        facts_payload.clone(),
        steps,
        verification,
        risks,
        open_questions,
        budgets.clone(),
        allowlist_source,
        mode,
        enrichment,
    );

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
            let apply_start = std::time::Instant::now();
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
                    .unwrap_or(observation_compression::DEFAULT_TOKEN_BUDGET_THRESHOLD);
                let keep = ce_cfg.compression_keep_recent.unwrap_or(3);
                let before_tokens = observation_compression::estimate_tokens(&verification_results);
                let before = verification_results.clone();
                observation_compression::compress_if_needed_with_threshold(
                    &mut verification_results,
                    keep,
                    threshold,
                );
                let after_tokens = observation_compression::estimate_tokens(&verification_results);
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

            // MaTTS self-contrast: execute N sandboxed trajectories and distill
            // learnings from success vs failure outcomes.
            let traj_count = options
                .trajectories
                .or_else(|| load_repo_config(repo_path).and_then(|c| c.agent.default_trajectories));
            if let Some(n) = traj_count {
                if n >= 2 {
                    let auto_record = load_repo_config(repo_path)
                        .and_then(|c| c.agent.auto_record_learnings)
                        .unwrap_or(false);
                    let sandbox_cfg = load_repo_config(repo_path)
                        .map(|c| c.sandbox)
                        .unwrap_or_default();
                    let policy = sandbox_cfg
                        .policy
                        .unwrap_or_else(|| "warn_and_degrade".to_string());
                    let cleanup_on_success = sandbox_cfg.cleanup_on_success.unwrap_or(true);
                    let keep_on_failure = sandbox_cfg.keep_on_failure.unwrap_or(false);

                    let mut outcomes = Vec::new();
                    outcomes.push(build_trajectory_outcome(
                        "primary",
                        &plan.goal,
                        apply_start.elapsed().as_millis(),
                        &verification_results,
                        plan.target.resolved_element_id.as_deref(),
                    ));

                    let sandbox_ok = sruja_agent::matts::is_sandbox_available(repo_path);
                    if sandbox_ok {
                        let names = sruja_agent::matts::sandbox_names(&plan.goal, n);
                        let exec = WorktreeVerificationExecutor {
                            repo_root: repo_path,
                            goal: &plan.goal,
                            verification: &plan.verification,
                            max_runtime_ms_per_step: plan.budgets.max_runtime_ms_per_step,
                            allowed_sruja_subcommands: &allowed_sruja_subcommands,
                            allowed_verify_execs: &allowed_verify_execs,
                            resolved_element_id: plan.target.resolved_element_id.as_deref(),
                            cleanup_on_success,
                            keep_on_failure,
                            sandbox_names: &names,
                        };
                        let extra = exec.run_n(n.saturating_sub(1)).await?;
                        outcomes.extend(extra);
                    } else if policy == "fail_fast" {
                        return Err(CliError::validation(
                            "MaTTS trajectories requested, but git worktrees are unavailable. Enable worktrees or set [sandbox].policy = \"warn_and_degrade\".".to_string(),
                        ));
                    } else {
                        memory_recorded.push(format!(
                            "MaTTS requested {} trajectories, but sandboxing is unavailable; proceeding with primary only.",
                            n
                        ));
                    }

                    let (_contrast, notes) = sruja_agent::matts::maybe_distill_and_record(
                        repo_path,
                        n,
                        &outcomes,
                        auto_record,
                    );
                    memory_recorded.extend(notes);
                }
            }

            let out = AgentApplyOutput {
                schema_version: "agent_apply_output/v1".to_string(),
                run_id: Some(run_id),
                plan,
                executed_steps: Vec::new(),
                observations: Vec::new(),
                verification_results,
                memory_recorded,
                observation_compression: compression_report,
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

#[allow(clippy::too_many_arguments)]
fn build_agent_plan_output(
    run_id: &str,
    trace_id: &str,
    repo: &str,
    goal: &str,
    file: Option<&str>,
    element_id: Option<&str>,
    query: Option<&str>,
    resolved_element_id: Option<String>,
    facts: Value,
    steps: Vec<AgentStep>,
    verification: Vec<AgentStep>,
    risks: Vec<String>,
    open_questions: Vec<String>,
    budgets: AgentBudgets,
    allowlist_source: String,
    mode: AgentMode,
    enrichment: Option<AgentEnrichment>,
) -> AgentPlanOutput {
    AgentPlanOutput {
        artifact_kind: "deterministic_plan".to_string(),
        trace_id: Some(trace_id.to_string()),
        schema_version: "agent_plan_output/v1".to_string(),
        run_id: Some(run_id.to_string()),
        repo: repo.to_string(),
        goal: goal.to_string(),
        target: AgentTarget {
            selector: file
                .map(|s| format!("file:{s}"))
                .or_else(|| element_id.map(|s| format!("element_id:{s}")))
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
        facts,
        steps,
        verification,
        risks,
        open_questions,
        budgets,
        safety: AgentSafety {
            mode: match mode {
                AgentMode::Plan => "plan".to_string(),
                AgentMode::Apply => "apply".to_string(),
            },
            allowlist_source,
            denied_steps: Vec::new(),
        },
        enrichment,
    }
}

fn build_trajectory_outcome(
    trajectory_id: &str,
    goal: &str,
    elapsed_ms: u128,
    verification_results: &[StepObservation],
    affected_element_id: Option<&str>,
) -> sruja_agent::TrajectoryOutcome {
    let all_ok = verification_results.iter().all(|o| o.status == "ok");
    let last_exit = verification_results.last().and_then(|o| o.exit_code);
    let summary = verification_results
        .last()
        .map(|o| {
            format!(
                "[{}] {} exit={}",
                o.step_id,
                o.status,
                o.exit_code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "?".to_string())
            )
        })
        .unwrap_or_default();

    sruja_agent::TrajectoryOutcome {
        trajectory_id: trajectory_id.to_string(),
        goal: goal.to_string(),
        status: if all_ok {
            sruja_agent::TrajectoryStatus::Success
        } else {
            sruja_agent::TrajectoryStatus::Failed
        },
        exit_code: last_exit,
        summary,
        elapsed_ms,
        affected_elements: affected_element_id
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

fn create_sandbox_worktree(repo_root: &Path, name: &str) -> Result<PathBuf, CliError> {
    let sandbox_dir = repo_root.join(".sruja").join("sandboxes");
    std::fs::create_dir_all(&sandbox_dir)?;
    let target = sandbox_dir.join(name);
    if target.exists() {
        // Best-effort validation: if it exists but isn't a worktree, remove and recreate.
        if is_git_worktree(repo_root, &target) {
            return Ok(target);
        }
        let _ = std::fs::remove_dir_all(&target);
    }
    // Prune stale worktree references to reduce spurious failures.
    let _ = std::process::Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(repo_root)
        .output();
    let out = std::process::Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            &format!("sruja-sandbox/{}", name),
            target
                .to_str()
                .ok_or_else(|| CliError::validation("Sandbox path is not valid UTF-8"))?,
        ])
        .current_dir(repo_root)
        .output()?;
    if !out.status.success() {
        // If the branch already exists, retry by detaching instead of failing.
        let err = String::from_utf8_lossy(&out.stderr);
        if err.contains("already exists") {
            let out2 = std::process::Command::new("git")
                .args([
                    "worktree",
                    "add",
                    "--detach",
                    target
                        .to_str()
                        .ok_or_else(|| CliError::validation("Sandbox path is not valid UTF-8"))?,
                ])
                .current_dir(repo_root)
                .output()?;
            if out2.status.success() {
                return Ok(target);
            }
            return Err(CliError::validation(format!(
                "git worktree add failed after retry: {}",
                String::from_utf8_lossy(&out2.stderr)
            )));
        }
        return Err(CliError::validation(format!(
            "git worktree add failed: {}",
            err
        )));
    }
    Ok(target)
}

fn discard_sandbox_worktree(repo_root: &Path, name: &str) -> Result<(), CliError> {
    let target = repo_root.join(".sruja").join("sandboxes").join(name);
    let _ = std::process::Command::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            target
                .to_str()
                .ok_or_else(|| CliError::validation("Sandbox path is not valid UTF-8"))?,
        ])
        .current_dir(repo_root)
        .output();
    let _ = std::process::Command::new("git")
        .args(["branch", "-D", &format!("sruja-sandbox/{}", name)])
        .current_dir(repo_root)
        .output();
    Ok(())
}

fn is_git_worktree(repo_root: &Path, path: &Path) -> bool {
    // A git worktree has a `.git` file pointing to the actual git dir.
    let git_marker = path.join(".git");
    if !git_marker.exists() {
        return false;
    }
    if let Ok(content) = std::fs::read_to_string(&git_marker) {
        if content.starts_with("gitdir:") {
            return true;
        }
    }
    // Fallback: ask git whether the worktree is recognized.
    std::process::Command::new("git")
        .args(["worktree", "list"])
        .current_dir(repo_root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .is_some_and(|s| s.contains(path.to_string_lossy().as_ref()))
}

async fn run_verification_steps_in_repo(
    repo_path: &Path,
    verification: &[AgentStep],
    max_runtime_ms_per_step: u64,
    allowed_sruja_subcommands: &[String],
    allowed_verify_execs: &[String],
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
            },
        };
        out.push(obs);
    }
    Ok(out)
}

struct WorktreeVerificationExecutor<'a> {
    repo_root: &'a Path,
    goal: &'a str,
    verification: &'a [AgentStep],
    max_runtime_ms_per_step: u64,
    allowed_sruja_subcommands: &'a [String],
    allowed_verify_execs: &'a [String],
    resolved_element_id: Option<&'a str>,
    cleanup_on_success: bool,
    keep_on_failure: bool,
    sandbox_names: &'a [String],
}

impl<'a> sruja_agent::TrajectoryExecutor for WorktreeVerificationExecutor<'a> {
    type Error = CliError;

    fn run_trajectory<'b>(
        &'b self,
        trajectory_id: &'b str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<sruja_agent::TrajectoryOutcome, Self::Error>>
                + Send
                + 'b,
        >,
    > {
        Box::pin(async move {
            // Map t1..tN to sandbox_names[1..] (t1 => sandbox_names[1]).
            let idx = trajectory_id
                .trim_start_matches('t')
                .parse::<usize>()
                .unwrap_or(1);
            let name = self
                .sandbox_names
                .get(idx)
                .cloned()
                .unwrap_or_else(|| format!("matts-{}", trajectory_id));

            let sandbox_path = create_sandbox_worktree(self.repo_root, &name)?;
            let start = std::time::Instant::now();
            let results = run_verification_steps_in_repo(
                &sandbox_path,
                self.verification,
                self.max_runtime_ms_per_step,
                self.allowed_sruja_subcommands,
                self.allowed_verify_execs,
            )
            .await?;
            let outcome = build_trajectory_outcome(
                &name,
                self.goal,
                start.elapsed().as_millis(),
                &results,
                self.resolved_element_id,
            );
            let ok = outcome.status == sruja_agent::TrajectoryStatus::Success;
            if ok {
                if self.cleanup_on_success {
                    let _ = discard_sandbox_worktree(self.repo_root, &name);
                }
            } else if !self.keep_on_failure {
                let _ = discard_sandbox_worktree(self.repo_root, &name);
            }
            Ok(outcome)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::observation_compression;
    use super::StepObservation;

    fn make_obs(id: &str, stdout_len: usize) -> StepObservation {
        StepObservation {
            step_id: id.to_string(),
            status: "ok".to_string(),
            exit_code: Some(0),
            stdout: "x".repeat(stdout_len),
            stderr: String::new(),
            elapsed_ms: 100,
        }
    }

    #[test]
    fn compress_noop_under_threshold() {
        let mut obs = vec![make_obs("step_1", 100), make_obs("step_2", 100)];
        let original_len: usize = obs.iter().map(|o| o.stdout.len()).sum();
        observation_compression::compress_if_needed(&mut obs, 1);
        let after_len: usize = obs.iter().map(|o| o.stdout.len()).sum();
        assert_eq!(
            original_len, after_len,
            "Should not compress under threshold"
        );
    }

    #[test]
    fn compress_reduces_older_observations() {
        let mut obs: Vec<StepObservation> = (0..10)
            .map(|i| make_obs(&format!("step_{}", i), 4000))
            .collect();
        observation_compression::compress_if_needed(&mut obs, 2);

        for i in 0..8 {
            assert!(
                obs[i].stdout.len() < 4000,
                "Older observation {} should be compressed, got len {}",
                i,
                obs[i].stdout.len()
            );
        }
        assert_eq!(obs[8].stdout.len(), 4000, "Recent observations preserved");
        assert_eq!(obs[9].stdout.len(), 4000, "Recent observations preserved");
    }

    #[test]
    fn compress_preserves_status_and_exit_code() {
        let mut obs = vec![
            StepObservation {
                step_id: "failing".to_string(),
                status: "error".to_string(),
                exit_code: Some(1),
                stdout: "x".repeat(5000),
                stderr: "error: something failed\ndetails...".to_string(),
                elapsed_ms: 200,
            },
            make_obs("recent", 100),
        ];
        observation_compression::compress_if_needed(&mut obs, 1);
        assert_eq!(obs[0].status, "error");
        assert_eq!(obs[0].exit_code, Some(1));
        assert_eq!(obs[0].step_id, "failing");
    }
}

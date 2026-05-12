//! Agent run loop: observe → plan → (optional) apply → verify → record learnings.
//!
//! This is intentionally conservative:
//! - Default mode is plan (no execution)
//! - Apply is gated by repo config allowlists + budgets
//! - All optional enrichment is grounded: it may add narrative, never change facts.

use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::time::{timeout, Duration};

use crate::commands::CliError;
use crate::integrations::{load_repo_config, resolve_enrichment_plan, resolve_openai_auth};
use crate::integrations::{run_cmd_enrichment, run_openai_markdown};

use super::agent;
use super::focus as focus_cmd;
use crate::commands::sync_cmd;

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
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AgentBudgets {
    max_steps: usize,
    max_runtime_ms_per_step: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AgentTarget {
    selector: String,
    resolved_element_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AgentStep {
    id: String,
    kind: String,
    argv: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AgentSafety {
    mode: String,
    allowlist_source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    denied_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AgentPlanOutput {
    schema_version: String,
    repo: String,
    goal: String,
    target: AgentTarget,
    facts_refs: Vec<String>,
    facts: Value,
    steps: Vec<AgentStep>,
    verification: Vec<AgentStep>,
    risks: Vec<String>,
    open_questions: Vec<String>,
    budgets: AgentBudgets,
    safety: AgentSafety,
    #[serde(skip_serializing_if = "Option::is_none")]
    enrichment: Option<AgentEnrichment>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct StepObservation {
    step_id: String,
    status: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    elapsed_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AgentApplyOutput {
    schema_version: String,
    plan: AgentPlanOutput,
    executed_steps: Vec<String>,
    observations: Vec<StepObservation>,
    verification_results: Vec<StepObservation>,
    memory_recorded: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AgentEnrichment {
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

fn load_allowlists(repo_path: &Path) -> (Vec<String>, Vec<String>, String) {
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

async fn run_allowlisted_process(
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
            return Some(AgentEnrichment {
                status: "skipped".to_string(),
                provider: "cmd".to_string(),
                model: None,
                error: Some("No command configured. Pass --enrich-cmd or set SRUJA_ENRICH_CMD (or .sruja/config.toml [integrations].cmd).".to_string()),
                narrative_markdown: None,
            });
        };
        return Some(match run_cmd_enrichment(cmd, &stdin_payload, limits) {
            Ok(md) => AgentEnrichment {
                status: "ok".to_string(),
                provider: "external_cmd".to_string(),
                model: None,
                error: None,
                narrative_markdown: Some(md),
            },
            Err(e) => AgentEnrichment {
                status: "error".to_string(),
                provider: "external_cmd".to_string(),
                model: None,
                error: Some(e),
                narrative_markdown: None,
            },
        });
    }

    if provider != "openai" {
        return Some(AgentEnrichment {
            status: "skipped".to_string(),
            provider: provider.to_string(),
            model: None,
            error: Some(
                "Unsupported provider. Use provider=cmd (recommended) or provider=openai."
                    .to_string(),
            ),
            narrative_markdown: None,
        });
    }

    let model = plan.model.as_deref().unwrap_or("gpt-4o-mini");
    let base_url = plan
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let Some(api_key) = resolve_openai_auth() else {
        return Some(AgentEnrichment {
            status: "skipped".to_string(),
            provider: "openai".to_string(),
            model: Some(model.to_string()),
            error: Some("Missing API key (set OPENAI_API_KEY or SRUJA_ENRICH_API_KEY; SRUJA_LLM_API_KEY is deprecated).".to_string()),
            narrative_markdown: None,
        });
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
        Ok(md) => Some(AgentEnrichment {
            status: "ok".to_string(),
            provider: "openai".to_string(),
            model: Some(model.to_string()),
            error: None,
            narrative_markdown: Some(md),
        }),
        Err(e) => Some(AgentEnrichment {
            status: "error".to_string(),
            provider: "openai".to_string(),
            model: Some(model.to_string()),
            error: Some(e),
            narrative_markdown: None,
        }),
    }
}

async fn run_sruja_cmd(
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
        let mut briefing = focus_cmd::build_focus_briefing(&kg, id, repo_path, scan_node_count);
        // No focus-specific enrichment here; agent enrichment is handled at the end.
        briefing.enrichment = None;
        let out = focus_cmd::build_focus_for_ai_output(
            repo_path,
            options.file,
            options.element_id,
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

    // ── Think: deterministic plan synthesis (v1: conservative) ────────────
    let steps: Vec<AgentStep> = Vec::new();
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

    let plan = AgentPlanOutput {
        schema_version: "agent_plan_output/v1".to_string(),
        repo: options.repo.to_string(),
        goal: options.goal.to_string(),
        target: AgentTarget {
            selector: options
                .file
                .map(|s| format!("file:{s}"))
                .or_else(|| options.element_id.map(|s| format!("element_id:{s}")))
                .or_else(|| options.query.map(|s| format!("query:{s}")))
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
            // v1 apply: run verification steps only (safe default),
            // record learnings if verification fails.
            let mut verification_results = Vec::new();
            let mut memory_recorded = Vec::new();

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

            // Record learning if any verification result errored.
            if let Some(first_err) = verification_results.iter().find(|o| o.status == "error") {
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
                )
                .await?;
                memory_recorded.push(hypothesis);
            }

            let out = AgentApplyOutput {
                schema_version: "agent_apply_output/v1".to_string(),
                plan,
                executed_steps: Vec::new(),
                observations: Vec::new(),
                verification_results,
                memory_recorded,
            };

            serde_json::to_string_pretty(&out)?
        }
    };

    // Future: persist facts bundle to .sruja/agent/runs/
    let _ = std::fs::create_dir_all(agent_artifacts_dir(repo_path));
    Ok(out_string)
}

pub async fn agent_run(options: AgentRunOptions<'_>) -> Result<(), CliError> {
    let s = agent_run_to_string(options).await?;
    println!("{s}");
    Ok(())
}

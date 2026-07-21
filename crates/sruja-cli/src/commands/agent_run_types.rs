use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

use crate::commands::CliError;
use crate::integrations::{load_repo_config, EnrichmentResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMode {
    Plan,
    Apply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentFormat {
    Text,
    Json,
    ForAi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiMode {
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
    pub enrich: &'a crate::enrichment::EnrichmentRef<'a>,
    pub continue_on_error: bool,
    pub force_sync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentBudgets {
    pub max_steps: usize,
    pub max_runtime_ms_per_step: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentTarget {
    pub selector: String,
    pub resolved_element_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentStep {
    pub id: String,
    pub kind: String,
    pub argv: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentSafety {
    pub mode: String,
    pub allowlist_source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub denied_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentPlanOutput {
    pub artifact_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    pub schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    pub repo: String,
    pub goal: String,
    pub target: AgentTarget,
    pub facts_refs: Vec<String>,
    pub facts: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposal_sruja: Option<String>,
    pub steps: Vec<AgentStep>,
    pub verification: Vec<AgentStep>,
    pub risks: Vec<String>,
    pub open_questions: Vec<String>,
    pub budgets: AgentBudgets,
    pub safety: AgentSafety,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrichment: Option<AgentEnrichment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StepObservation {
    pub step_id: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub elapsed_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

pub fn agent_apply_verification_success(results: &[StepObservation]) -> bool {
    !results.is_empty()
        && results
            .iter()
            .all(|r| matches!(r.status.as_str(), "ok" | "skipped"))
}

pub fn compute_observation_hash(
    step_id: &str,
    status: &str,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
    elapsed_ms: u128,
) -> String {
    let hash_input = format!(
        "step_id:{}\nstatus:{}\nexit_code:{:?}\nstdout:{}\nstderr:{}\nelapsed_ms:{}",
        step_id, status, exit_code, stdout, stderr, elapsed_ms
    );
    blake3::hash(hash_input.as_bytes()).to_hex().to_string()
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) context_prune: Option<crate::commands::context_prune::ContextPruneSuggestion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) verification_hash: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEnrichment {
    pub artifact_kind: String,
    pub status: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrative_markdown: Option<String>,
}

pub fn agent_enrichment(result: EnrichmentResult) -> AgentEnrichment {
    AgentEnrichment {
        artifact_kind: "llm_interpretation".to_string(),
        status: result.status,
        provider: result.provider,
        model: result.model,
        error: result.error,
        narrative_markdown: result.narrative_markdown,
    }
}

pub fn parse_mode(mode: &str) -> Result<AgentMode, CliError> {
    match mode.to_lowercase().as_str() {
        "plan" => Ok(AgentMode::Plan),
        "apply" => Ok(AgentMode::Apply),
        other => Err(CliError::validation(format!(
            "Invalid --mode '{}'. Use 'plan' or 'apply'.",
            other
        ))),
    }
}

pub fn parse_format(format: &str) -> Result<AgentFormat, CliError> {
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

pub fn parse_ai_mode(ai_mode: &str) -> Result<AiMode, CliError> {
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

pub fn validate_target(
    file: Option<&str>,
    element_id: Option<&str>,
    query: Option<&str>,
) -> Result<(), CliError> {
    let count = file.is_some() as u8 + element_id.is_some() as u8 + query.is_some() as u8;
    if count > 1 {
        return Err(CliError::validation(
            "Only one of --file, --element-id, or --query may be provided at a time.".to_string(),
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

pub fn load_agent_budgets(
    repo_path: &Path,
    ai_mode: AiMode,
    overrides: (Option<usize>, Option<u64>),
) -> AgentBudgets {
    let mut b = budgets_for_mode(ai_mode);
    if let Some(ms) = overrides.1 {
        b.max_runtime_ms_per_step = ms;
    }
    if let Some(steps) = overrides.0 {
        b.max_steps = steps;
    }

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
        "intent".to_string(),
        "focus".to_string(),
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

pub fn load_allowlists(repo_path: &Path) -> (Vec<String>, Vec<String>, String) {
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

pub fn build_enrichment(
    repo_path: &Path,
    facts_payload: &Value,
    enrich: &crate::enrichment::EnrichmentRef<'_>,
) -> Option<AgentEnrichment> {
    crate::integrations::build_enrichment(
        repo_path,
        facts_payload,
        enrich,
        "You are a careful repo assistant. Never fabricate.",
        crate::integrations::DEFAULT_ENRICHMENT_PROMPT_TEMPLATE,
    )
    .map(agent_enrichment)
}

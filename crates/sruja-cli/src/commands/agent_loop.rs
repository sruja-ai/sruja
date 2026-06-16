//! `sruja agent loop` — the closed-loop autonomous coding agent.
//!
//! Drives the full cognition loop (comprehend -> plan -> execute via tools ->
//! critique -> replan until approved) against a real workspace using the
//! `sruja-agent` crate's `Agent::run_loop`.
//!
//! This is the CLI-first path that turns Sruja from a passive harness into an
//! autonomous actor graded by its own deterministic tools.

use std::path::Path;
use std::sync::Arc;

use sruja_agent::llm::OpenAiClient;
use sruja_agent::tool::ToolRegistry;
use sruja_agent::{
    AgentConfig, AgentError, LoopConfig, ModelMapping,
};
use serde::Deserialize;

use super::CliError;

/// Options received from the CLI.
#[derive(Debug)]
pub struct AgentLoopOptions<'a> {
    pub repo: &'a str,
    pub goal: &'a str,
    pub max_iterations: Option<usize>,
    pub no_tdd: bool,
    pub dry_run: bool,
    pub model: Option<&'a str>,
    pub base_url: Option<&'a str>,
    pub spend_cap_usd: Option<f64>,
    pub no_oscillation_detection: bool,
    pub format: &'a str,
}

/// `.sruja/loop.toml` manifest — persists loop defaults so teams share config.
#[derive(Debug, Default, Deserialize)]
struct LoopManifest {
    #[serde(default = "default_max_iterations")]
    max_iterations: usize,
    #[serde(default = "default_tdd")]
    tdd: bool,
    #[serde(default = "default_true")]
    review_every_change: bool,
    #[serde(default)]
    dry_run: bool,
    #[serde(default)]
    shell_allowlist: Vec<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    models: Option<ModelManifest>,
    #[serde(default)]
    spend_cap_usd: Option<f64>,
    #[serde(default = "default_true")]
    detect_oscillation: bool,
}

#[derive(Debug, Default, Deserialize)]
struct ModelManifest {
    cheap: Option<String>,
    mid: Option<String>,
    premium: Option<String>,
    review: Option<String>,
}

fn default_max_iterations() -> usize {
    3
}
fn default_tdd() -> bool {
    true
}
fn default_true() -> bool {
    true
}

/// Entry point for `sruja agent loop`.
pub async fn agent_loop(options: &AgentLoopOptions<'_>) -> Result<(), CliError> {
    let repo_path = Path::new(&options.repo);

    // ── Load .sruja/loop.toml for defaults ────────────────────────────────
    let manifest = load_loop_manifest(repo_path);

    // ── Resolve configuration (CLI > manifest > built-in defaults) ────────
    let max_iterations = options
        .max_iterations
        .or(if manifest.max_iterations != 3 {
            Some(manifest.max_iterations)
        } else {
            None
        })
        .unwrap_or(3);

    let tdd = if options.no_tdd { false } else { manifest.tdd };

    let dry_run = options.dry_run || manifest.dry_run;

    let model = options
        .model
        .or(manifest.model.as_deref())
        .unwrap_or("gpt-4o-mini");

    let base_url = options
        .base_url
        .or(manifest.base_url.as_deref())
        .unwrap_or("https://api.openai.com/v1");

    // Build model mapping from manifest (if present) or defaults.
    let models = if let Some(mm) = &manifest.models {
        let defaults = ModelMapping::default();
        ModelMapping {
            cheap: mm.cheap.clone().unwrap_or_else(|| defaults.cheap.clone()),
            mid: mm.mid.clone().unwrap_or_else(|| defaults.mid.clone()),
            premium: mm.premium.clone().unwrap_or_else(|| defaults.premium.clone()),
            review: mm.review.clone().unwrap_or_else(|| defaults.review.clone()),
        }
    } else {
        ModelMapping {
            cheap: model.to_string(),
            mid: model.to_string(),
            premium: model.to_string(),
            review: model.to_string(),
        }
    };

    // ── Create LLM client ─────────────────────────────────────────────────
    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("SRUJA_ENRICH_API_KEY"))
        .map_err(|_| {
            CliError::validation(
                "No API key found. Set OPENAI_API_KEY or SRUJA_ENRICH_API_KEY environment variable.",
            )
        })?;

    let llm = OpenAiClient::new(&api_key, base_url, model)
        .map_err(|e| CliError::validation(format!("Failed to create LLM client: {e}")))?;

    // ── Build tools + agent ───────────────────────────────────────────────
    let tools = ToolRegistry::with_builtin(
        repo_path.to_path_buf(),
        manifest.shell_allowlist.clone(),
    );

    let config = AgentConfig {
        models,
        tdd,
        review_every_change: manifest.review_every_change,
        dry_run,
        ..Default::default()
    };

    let agent = sruja_agent::Agent::builder()
        .llm(Arc::new(llm))
        .tools(tools)
        .config(config)
        .memory(repo_path)
        .build()
        .map_err(agent_err_to_cli)?;

    // ── Run the loop ──────────────────────────────────────────────────────
    let spend_cap_usd = options.spend_cap_usd.or(manifest.spend_cap_usd);
    let detect_oscillation =
        if options.no_oscillation_detection { false } else { manifest.detect_oscillation };

    let loop_config = LoopConfig {
        max_iterations,
        spend_cap_usd,
        detect_oscillation,
        ..Default::default()
    };

    let result = agent
        .run_loop(options.goal, &loop_config)
        .await
        .map_err(agent_err_to_cli)?;

    // ── Report ────────────────────────────────────────────────────────────
    match options.format {
        "json" => {
            let json = serde_json::to_string_pretty(&result)?;
            println!("{json}");
        }
        _ => {
            print_loop_result_human(&result);
        }
    }

    Ok(())
}

fn load_loop_manifest(repo: &Path) -> LoopManifest {
    let path = repo.join(".sruja/loop.toml");
    match std::fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => LoopManifest::default(),
    }
}

fn print_loop_result_human(result: &sruja_agent::LoopResult) {
    let status = if result.converged { "CONVERGED" } else { "NOT CONVERGED" };
    println!("═══════════════════════════════════════════");
    println!("  Agent Loop: {status}");
    println!("═══════════════════════════════════════════");
    println!();
    println!("Goal: {}", result.goal);
    println!(
        "Iterations: {} | Termination: {:?}",
        result.iteration_count(),
        result.termination
    );
    println!(
        "Tokens: {} prompt + {} completion = {} total (~${:.4})",
        result.total_usage.prompt_tokens,
        result.total_usage.completion_tokens,
        result.total_usage.total_tokens,
        result.total_usage.estimated_cost_usd()
    );
    println!();

    for iter in &result.iterations {
        let mark = if iter.critique_approved { "PASS" } else { "FAIL" };
        println!(
            "  [{}/{}] {} | plan:{} succeed:{} failed:{} score:{:.1} {}",
            iter.iteration,
            result.iteration_count(),
            mark,
            iter.subtask_count,
            iter.succeeded,
            iter.failed,
            iter.critique_score,
            if iter.replanned { "(replanned)" } else { "" }
        );
        for issue in &iter.critique_issues {
            println!("         issue: {issue}");
        }
    }

    println!();
    if let Some(critique) = result.final_result.critique.as_ref() {
        println!("Final critique: score={:.1} approved={}", critique.score, critique.approved);
        if !critique.issues.is_empty() {
            println!("Issues:");
            for issue in &critique.issues {
                println!("  - {issue}");
            }
        }
    }
}

fn agent_err_to_cli(e: AgentError) -> CliError {
    CliError::validation(format!("Agent error: {e}"))
}

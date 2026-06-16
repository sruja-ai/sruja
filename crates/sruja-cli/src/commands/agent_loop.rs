//! `sruja agent loop` — the closed-loop autonomous coding agent.
//!
//! Drives the full cognition loop (comprehend -> plan -> execute via tools ->
//! critique -> replan until approved) against a real workspace using the
//! `sruja-agent` crate's `Agent::run_loop`.
//!
//! This is the CLI-first path that turns Sruja from a passive harness into an
//! autonomous actor graded by its own deterministic tools.
//!
//! # Configuration
//!
//! Uses the industry-standard resolution chain:
//! 1. CLI flags (highest priority)
//! 2. Environment variables (provider-specific)
//! 3. `.sruja/config.toml` (non-secrets only)
//! 4. Built-in defaults (lowest priority)
//!
//! ## Multi-Provider Support
//!
//! Configure different providers for different tasks in `.sruja/config.toml`:
//!
//! ```toml
//! [integrations]
//! default_provider = "zai"
//!
//! [integrations.providers.zai]
//! base_url = "https://api.z.ai/api/coding/paas/v4"
//! key_env = "ZAI_API_KEY"
//!
//! [agent.models]
//! cheap = { provider = "zai", model = "GLM-4-Flash" }
//! mid = { provider = "zai", model = "GLM-4.7" }
//! premium = { provider = "openrouter", model = "anthropic/claude-sonnet-4" }
//! review = { provider = "openrouter", model = "google/gemini-2.5-flash" }
//! ```
//!
//! See `config::resolve_multi_provider_config` for details.

use std::path::Path;
use std::sync::Arc;

use sruja_agent::llm::{OpenAiClient, TieredClient};
use sruja_agent::tool::ToolRegistry;
use sruja_agent::verify::{all_passed, run_verification_steps, VerifyOptions};
use sruja_agent::{AgentConfig, AgentError, GoalSpec, LoopConfig, LoopManifest, ModelMapping};

use super::CliError;
use crate::config;

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

/// Entry point for `sruja agent loop`.
pub async fn agent_loop(options: &AgentLoopOptions<'_>) -> Result<(), CliError> {
    let repo_path = Path::new(&options.repo);

    // ── Load .sruja/loop.toml for defaults ────────────────────────────────
    let manifest = LoopManifest::load_from_path(repo_path);

    // ── Resolve multi-provider configuration ──────────────────────────────
    // Supports different providers for different tasks (cheap/mid/premium/review)
    let multi_config = config::resolve_multi_provider_config(repo_path)?;

    // ── Resolve loop-specific configuration (CLI > manifest > defaults) ───
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

    // Build model mapping from multi-provider config.
    // Each tier can use a different provider/model.
    let models = ModelMapping {
        cheap: multi_config.cheap.model.clone(),
        mid: multi_config.mid.model.clone(),
        premium: multi_config.premium.model.clone(),
        review: multi_config.review.model.clone(),
    };

    // ── Create LLM client ─────────────────────────────────────────────────
    // Build per-tier clients so each tier can hit its own provider with the
    // correct API key and base URL. The TieredClient routes by model name.
    let mid_config = &multi_config.mid;
    let model = options.model.unwrap_or(&mid_config.model);
    let base_url = options.base_url.unwrap_or(&mid_config.base_url);

    let default_client = Arc::new(
        OpenAiClient::new(&mid_config.api_key, base_url, model)
            .map_err(|e| CliError::validation(format!("Failed to create LLM client: {e}")))?,
    );

    let mut tiered = TieredClient::new(default_client.clone());
    for tier_cfg in [
        &multi_config.cheap,
        &multi_config.mid,
        &multi_config.premium,
        &multi_config.review,
    ] {
        // Only register a separate client when this tier's credentials differ
        // from the default (mid-tier). Same key + URL = same client.
        let needs_own_client =
            tier_cfg.api_key != mid_config.api_key || tier_cfg.base_url != mid_config.base_url;

        if needs_own_client {
            let client = OpenAiClient::new(&tier_cfg.api_key, &tier_cfg.base_url, &tier_cfg.model)
                .map_err(|e| CliError::validation(format!("Failed to create LLM client: {e}")))?;
            tiered = tiered.with_route(&tier_cfg.model, Arc::new(client));
        }
    }

    // ── Build goal spec (CLI statement + manifest criteria/constraints) ───
    // CLI --goal always wins for the statement; the manifest's [goal] section
    // provides structured acceptance criteria and constraints.
    let goal_spec = if manifest.goal.has_criteria() || !manifest.goal.constraints.is_empty() {
        GoalSpec {
            statement: options.goal.to_string(),
            acceptance_criteria: manifest.goal.acceptance_criteria.clone(),
            target_files: manifest.goal.target_files.clone(),
            target_elements: manifest.goal.target_elements.clone(),
            constraints: manifest.goal.constraints.clone(),
        }
    } else {
        GoalSpec::new(options.goal)
    };

    let goal_prompt = goal_spec.to_prompt();

    // ── Build tools + agent ───────────────────────────────────────────────
    let tools =
        ToolRegistry::with_builtin(repo_path.to_path_buf(), manifest.shell_allowlist.clone());

    let config = AgentConfig {
        models,
        tdd,
        review_every_change: manifest.review_every_change,
        dry_run,
        ..Default::default()
    };

    let agent = sruja_agent::Agent::builder()
        .llm(Arc::new(tiered))
        .tools(tools)
        .config(config)
        .memory(repo_path)
        .build()
        .map_err(agent_err_to_cli)?;

    // ── Run the loop ──────────────────────────────────────────────────────
    let spend_cap_usd = options.spend_cap_usd.or(manifest.spend_cap_usd);
    let detect_oscillation = if options.no_oscillation_detection {
        false
    } else {
        manifest.detect_oscillation
    };

    let loop_config = LoopConfig {
        max_iterations,
        spend_cap_usd,
        detect_oscillation,
        ..Default::default()
    };

    let result = agent
        .run_loop(&goal_prompt, &loop_config)
        .await
        .map_err(agent_err_to_cli)?;

    // ── Deterministic verification (independent grader) ───────────────────
    // If the manifest defines [[verify]] steps, run them after the loop.
    // These are the agent-independent check — the loop cannot fake a passing
    // `cargo test`. Verification failure is reported but does not discard
    // the loop result.
    let verify_passed = if !manifest.verify_steps.is_empty() {
        let verify_opts = VerifyOptions {
            allowed_executables: manifest.shell_allowlist.clone(),
            ..Default::default()
        };
        let verify_results =
            run_verification_steps(&manifest.verify_steps, &verify_opts, repo_path).await;
        let passed = all_passed(&verify_results);
        if !passed {
            eprintln!("⚠  Verification FAILED:");
            for r in &verify_results {
                if !r.status.is_pass() {
                    eprintln!(
                        "  ✗ {}: exit={:?} stderr={}",
                        r.step_id,
                        r.exit_code,
                        r.stderr.trim()
                    );
                }
            }
        } else {
            println!("✓  All {} verification steps passed", verify_results.len());
        }
        Some(passed)
    } else {
        None
    };

    // ── Report ────────────────────────────────────────────────────────────
    match options.format {
        "json" => {
            let mut value = serde_json::to_value(&result)?;
            if let Some(vp) = verify_passed {
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("verification_passed".into(), vp.into());
                }
            }
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        _ => {
            print_loop_result_human(&result);
            if let Some(passed) = verify_passed {
                println!();
                if passed {
                    println!("Verification: PASSED");
                } else {
                    println!("Verification: FAILED (loop result may be unreliable)");
                }
            }
        }
    }

    Ok(())
}

fn print_loop_result_human(result: &sruja_agent::LoopResult) {
    let status = if result.converged {
        "CONVERGED"
    } else {
        "NOT CONVERGED"
    };
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
        let mark = if iter.critique_approved {
            "PASS"
        } else {
            "FAIL"
        };
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
        println!(
            "Final critique: score={:.1} approved={}",
            critique.score, critique.approved
        );
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

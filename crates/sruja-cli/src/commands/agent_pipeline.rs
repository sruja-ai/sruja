use std::path::PathBuf;
use std::sync::Arc;

use sruja_agent::llm::{OpenAiClient, TieredClient};
use sruja_agent::pipeline::PipelineOrchestrator;

use super::error::CliError;
use crate::config;

/// Handle `sruja agent pipeline` command.
///
/// Resolves multi-provider config from `.sruja/config.toml`, builds a
/// `TieredClient` that routes each stage's model to its provider, then
/// runs the pipeline.
pub async fn handle_pipeline(
    repo: &str,
    goal: &str,
    dry_run: bool,
    judge_only: bool,
    focus: Option<String>,
    max_cycles: Option<usize>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = PathBuf::from(repo);

    // ── Resolve multi-provider configuration ──────────────────────────────
    let multi_config = config::resolve_multi_provider_config(&repo_path)?;

    // ── Build TieredClient (same pattern as agent_loop) ──────────────────
    let mid_config = &multi_config.mid;
    let default_client = Arc::new(
        OpenAiClient::new(&mid_config.api_key, &mid_config.base_url, &mid_config.model)
            .map_err(|e| CliError::validation(format!("LLM client error: {e}")))?,
    );

    let mut tiered = TieredClient::new(default_client.clone());
    for tier_cfg in [
        &multi_config.cheap,
        &multi_config.mid,
        &multi_config.premium,
        &multi_config.review,
    ] {
        let needs_own_client =
            tier_cfg.api_key != mid_config.api_key || tier_cfg.base_url != mid_config.base_url;

        if needs_own_client {
            let client = Arc::new(
                OpenAiClient::new(&tier_cfg.api_key, &tier_cfg.base_url, &tier_cfg.model)
                    .map_err(|e| CliError::validation(format!("LLM client error: {e}")))?,
            );
            tiered = tiered.with_route(&tier_cfg.model, client.clone());
            let family = model_family(&tier_cfg.model);
            if !family.is_empty() {
                tiered = tiered.with_provider_name_contains(family, client);
            }
        }
    }

    // ── Build orchestrator with tiered routing ───────────────────────────
    let orchestrator = PipelineOrchestrator::new(
        &repo_path,
        goal.to_string(),
        dry_run,
        focus,
        max_cycles,
    )
    .with_tiered(tiered);

    // ── Judge-only short circuit ─────────────────────────────────────────
    if judge_only {
        return handle_judge_only(orchestrator).await;
    }

    println!("Pipeline goal: {goal}");
    println!(
        "{}",
        if dry_run {
            "Dry-run mode: analyzer + judge only, no code changes"
        } else {
            "Full pipeline: analyzer → prober → fixer → judge"
        }
    );

    let mut orchestrator = orchestrator;
    let result = orchestrator.run().await;

    // ── Print summary ────────────────────────────────────────────────────
    println!("\n{}", "=".repeat(60));
    println!("Pipeline {}", if result.converged { "CONVERGED ✅" } else { "STOPPED" });
    println!("Reason: {}", result.reason);
    println!("Cycles: {} | Stages: {}", result.cycles, result.stages.len());
    println!("Lessons recorded: {}", result.lessons_recorded);

    if let Some(ref score) = result.scorecard {
        println!("\n{}", format_scorecard(score));
    }

    if format == "json" {
        let summary = serde_json::json!({
            "converged": result.converged,
            "reason": result.reason,
            "cycles": result.cycles,
            "lesson_count": result.lessons_recorded,
            "scorecard": result.scorecard.as_ref().map(|s| {
                serde_json::json!({
                    "total": s.total,
                    "functional_correctness": s.functional_correctness,
                    "code_quality": s.code_quality,
                    "test_coverage": s.test_coverage,
                    "ux_quality": s.ux_quality,
                    "cost_efficiency": s.cost_efficiency,
                    "summary": s.summary,
                })
            }),
        });
        println!("\n{}", serde_json::to_string_pretty(&summary).unwrap_or_default());
    }

    println!("\nReport: .sruja/pipeline/LIVE.md");

    if !result.converged {
        eprintln!("\nPipeline stopped: {}", result.reason);
    }

    Ok(())
}

async fn handle_judge_only(orchestrator: PipelineOrchestrator) -> Result<(), CliError> {
    println!("Judge-only mode: re-scoring current state...");
    let scorecard = orchestrator.judge_only().await
        .map_err(|e| CliError::validation(format!("Judge failed: {e}")))?;
    println!("{}", format_scorecard(&scorecard));
    Ok(())
}

/// Extract the alphabetic prefix of a model name for name-substring routing.
fn model_family(model: &str) -> String {
    let base = model.rsplit('/').next().unwrap_or(model);
    base.chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>()
        .to_lowercase()
}

fn format_scorecard(score: &sruja_agent::pipeline::Scorecard) -> String {
    format!(
        "\
Scorecard
---------
functional_correctness: {fc}/5
code_quality:           {cq}/5
test_coverage:          {tc}/5
ux_quality:             {ux}/5
cost_efficiency:        {ce}/5
total:                  {t:.1}/5
summary:                {summary}
improved_from_previous: {imp}",
        fc = score.functional_correctness,
        cq = score.code_quality,
        tc = score.test_coverage,
        ux = score.ux_quality,
        ce = score.cost_efficiency,
        t = score.total,
        summary = score.summary,
        imp = score.improved_from_previous,
    )
}

use std::path::PathBuf;

use sruja_agent::pipeline::PipelineOrchestrator;

use super::error::CliError;

/// Handle `sruja agent pipeline` command.
///
/// The pipeline is auto-generated from the goal if a `.sruja/pipelines/{name}.toml`
/// doesn't exist yet. Generated files stay for editing and re-running.
pub async fn handle_pipeline(
    repo: &str,
    goal: &str,
    dry_run: bool,
    judge_only: bool,
    _focus: Option<String>,
    max_cycles: Option<usize>,
    format: &str,
) -> Result<(), CliError> {
    let repo_path = PathBuf::from(repo);

    let orchestrator = PipelineOrchestrator::new(
        &repo_path,
        goal.to_string(),
        dry_run,
        _focus,
        max_cycles,
    );

    if judge_only {
        println!("Judge-only mode: re-scoring current state...");
        let scorecard = orchestrator.judge_only().await
            .map_err(|e| CliError::validation(format!("Judge failed: {e}")))?;
        let output = format_scorecard(&scorecard);
        println!("{output}");
        return Ok(());
    }

    println!("Pipeline goal: {goal}");
    println!(
        "{}",
        if dry_run {
            "Dry-run mode: analyzer + judge only, no code changes"
        } else {
            "Full pipeline: analyzer → prober → confirmer → fixer → auditor → retester → judge"
        }
    );

    let mut orchestrator = orchestrator;
    let result = orchestrator.run().await;

    println!("\n{}", "=".repeat(60));
    println!("Pipeline {}", if result.converged { "CONVERGED ✅" } else { "STOPPED" });
    println!("Reason: {}", result.reason);
    println!("Cycles: {} | Stages: {}", result.cycles, result.stages.len());
    println!("Lessons recorded: {}", result.lessons_recorded);

    if let Some(ref score) = result.scorecard {
        let output = format_scorecard(score);
        println!("\n{output}");
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
        return Err(CliError::validation(format!("pipeline did not converge: {}", result.reason)));
    }

    Ok(())
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

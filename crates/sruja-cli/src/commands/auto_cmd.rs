use std::path::PathBuf;

use sruja_agent::goal::GoalSpec;
use sruja_agent::PipelineConfig;

use crate::commands::{agent_loop, AgentLoopOptions};

/// Run the autonomous execution loop.
pub async fn auto_run(
    repo: &str,
    goal: &str,
    max_iterations: Option<usize>,
    dry_run: bool,
    yes: bool,
    pipeline: Option<&str>,
    resume: bool,
    format: &str,
    show_details: bool,
) -> Result<(), crate::commands::CliError> {
    let pipeline_override = if let Some(p) = pipeline {
        Some(PathBuf::from(p))
    } else {
        // No explicit pipeline — generate one from the goal and write it.
        let goal_spec = GoalSpec::new(goal);
        let cfg = PipelineConfig::from_goal(&goal_spec);
        let auto_path = PathBuf::from(repo).join(".sruja/auto-pipeline.toml");
        let toml_str = toml::to_string_pretty(&cfg)
            .map_err(|e| crate::commands::CliError::validation(format!("pipeline serialization: {e}")))?;
        std::fs::create_dir_all(auto_path.parent().unwrap())
            .map_err(|e| crate::commands::CliError::validation(format!("cannot create .sruja dir: {e}")))?;
        std::fs::write(&auto_path, &toml_str)
            .map_err(|e| crate::commands::CliError::validation(format!("cannot write pipeline: {e}")))?;
        if show_details {
            eprintln!(
                "Generated pipeline for goal: {}",
                auto_path.display()
            );
            eprintln!("  Edit this file to customize the agent stages, then re-run with:");
            eprintln!("  sruja auto --pipeline {} \"{}\"", auto_path.display(), goal);
            eprintln!();
        }
        // Still run with the generated pipeline (in memory).
        Some(auto_path)
    };

    let options = AgentLoopOptions {
        repo,
        goal,
        max_iterations,
        no_tdd: false,
        dry_run,
        model: None,
        base_url: None,
        spend_cap_usd: None,
        no_oscillation_detection: false,
        format,
        force_proceed: yes,
        no_default_grader: false,
        steer: false,
        resume,
        show_plan: false,
        plan_only: dry_run,
        checkpoint: true,
        no_checkpoint: false,
        changelog: false,
        show_pipeline: false,
        pipeline_override,
        verbose: show_details,
    };

    agent_loop(&options).await
}

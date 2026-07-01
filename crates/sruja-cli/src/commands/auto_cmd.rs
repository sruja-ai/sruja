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
) -> Result<(), crate::commands::CliError> {
    if let Some(p) = pipeline {
        eprintln!("⚠️  Pipeline-based execution is not yet implemented. Running with defaults.");
        eprintln!("   (pipeline path: {})", p);
    }

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
    };

    agent_loop(&options).await
}

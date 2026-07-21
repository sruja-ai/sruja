use super::commands::*;
use super::Cli;

mod agent;
mod dispatch;
mod dsl_inspect;
mod workflow;

pub async fn run_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref rules_path) = cli.classification_rules {
        sruja_scan::set_classification_rules_path(Some(std::path::PathBuf::from(rules_path)));
    }

    let result = match cli.command {
        Commands::Agent(AgentArgs { cmd }) => agent::handle(cmd).await,
        Commands::Workflow(WorkflowArgs { cmd }) => workflow::handle_workflow(cmd).await,
        Commands::Aidlc(AidlcArgs { cmd }) => workflow::handle_aidlc(cmd).await,
        cmd @ (Commands::Dsl(..) | Commands::Inspect(..)) => dsl_inspect::handle(cmd).await,
        other => dispatch::handle(other).await,
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            e.report();
            std::process::exit(e.exit_code());
        }
    }
}

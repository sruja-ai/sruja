//! Sruja CLI
//!
//! Command-line interface for the Sruja DSL tool.

mod cli;
mod commands;
mod compliance;
mod context_detection;
mod graph_store;
mod modules;
mod report;
mod utils;

use clap::Parser;
pub use cli::{run_command, Cli, Commands, ContextIntent, IntentCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().ok();
    let cli = Cli::parse();

    run_command(cli.command).await
}

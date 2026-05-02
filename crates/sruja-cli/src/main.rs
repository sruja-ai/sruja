//! Sruja CLI
//!
//! Command-line interface for the Sruja DSL tool.

mod cli;
mod commands;
mod compliance;
mod context_detection;
mod graph_store;
mod integrations;
mod modules;
mod report;
mod scoring;
mod utils;

use clap::Parser;
pub use cli::{run_command, Cli, Commands, ContextIntent, IntentCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .with_target(false)
        .compact()
        .init();

    run_command(cli).await
}

//! Sruja CLI
//!

#![warn(missing_docs)]
//! Command-line interface for the Sruja DSL tool.

#[allow(missing_docs)]
mod cli;
#[allow(missing_docs)]
mod commands;
pub mod config;
#[allow(missing_docs)]
mod context_detection;
pub mod enrichment;
#[allow(missing_docs)]
mod graph_store;
#[allow(missing_docs)]
mod integrations;
#[allow(missing_docs)]
mod modules;
#[allow(missing_docs)]
mod report;
#[allow(missing_docs)]
mod scoring;
#[allow(missing_docs)]
mod utils;

use clap::Parser;
pub use cli::{
    run_command, Cli, Commands, ContextIntent, DecisionCommand, DiscoverCommand, EventCommand,
    IntentCommand,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "error",
        1 => "warn",
        2 => "info",
        _ => "debug",
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

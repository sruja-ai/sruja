//! Sruja CLI
//!
//! Command-line interface for the Sruja DSL tool.

mod commands;

use clap::{Parser, Subcommand};
use commands::*;

#[derive(Parser)]
#[command(name = "sruja")]
#[command(about = "Sruja DSL CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version information
    Version,
    /// Lint a Sruja file
    Lint {
        /// Path to .sruja file
        file: String,
    },
    /// Export a Sruja file to various formats
    Export {
        /// Export format (json, mermaid, markdown, dot)
        format: String,
        /// Path to .sruja file
        file: String,
        /// Include pre-computed views in JSON output
        #[arg(long)]
        extended: bool,
    },
    /// Format a Sruja file
    Fmt {
        /// Path to .sruja file
        file: String,
    },
    /// Start LSP server (stdio)
    Lsp {
        /// Use stdio transport
        #[arg(long)]
        stdio: bool,
    },
    /// Compile a Sruja file
    Compile {
        /// Path to .sruja file
        file: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Version => commands::version(),
        Commands::Lint { file } => commands::lint(&file).await,
        Commands::Export { format, file, extended } => commands::export(&format, &file, extended).await,
        Commands::Fmt { file } => commands::fmt(&file).await,
        Commands::Lsp { .. } => commands::lsp().await,
        Commands::Compile { file } => commands::compile(&file).await,
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

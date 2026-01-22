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
        /// Mermaid view level (1=context, 2=container, 3=component)
        #[arg(long, default_value_t = 1)]
        view_level: u8,
        /// Mermaid focus node ID for view levels 2/3
        #[arg(long)]
        target: Option<String>,
    },
    /// Format a Sruja file
    Fmt {
        /// Path to .sruja file
        file: String,
    },
    /// List elements from a file
    List {
        /// Path to .sruja file
        file: String,
    },
    /// Print architecture tree
    Tree {
        /// Path to .sruja file
        file: String,
    },
    /// Initialize a new Sruja project
    Init {
        /// Project name (optional)
        name: Option<String>,
    },
    /// Show differences between two architecture files
    Diff {
        /// First file
        file1: String,
        /// Second file
        file2: String,
        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Explain an element
    Explain {
        /// Element ID to explain
        element_id: String,
        /// Path to .sruja file
        #[arg(long)]
        file: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Import from external format
    Import {
        /// Format (json)
        format: String,
        /// File to import
        file: String,
    },
    /// Calculate architecture health score
    Score {
        /// Path to .sruja file
        file: Option<String>,
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
    /// Validate architecture against rules
    Validate {
        /// Path to .sruja file or directory
        file: String,
        /// External constraint files
        #[arg(long, short = 'c')]
        constraints: Vec<String>,
        /// Fail on violations
        #[arg(long)]
        fail_on_violations: bool,
        /// Output as JSON
        #[arg(long)]
        format_json: bool,
    },
    /// Change management
    Change {
        #[command(subcommand)]
        action: ChangeAction,
    },
}

#[derive(Subcommand)]
enum ChangeAction {
    /// Create a new change record
    Create {
        /// Title of the change
        title: String,
        /// Description of the change
        #[arg(long, short = 'd')]
        description: Option<String>,
        /// Context/background
        #[arg(long, short = 'c')]
        context: Option<String>,
        /// Status (proposed, approved, rejected, implemented)
        #[arg(long, short = 's', default_value = "proposed")]
        status: String,
    },
    /// Validate a change record
    Validate {
        /// Path to change file
        file: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Version => commands::version(),
        Commands::Lint { file } => commands::lint(&file).await,
        Commands::Export {
            format,
            file,
            extended,
            view_level,
            target,
        } => commands::export(&format, &file, extended, view_level, target.as_deref()).await,
        Commands::Fmt { file } => commands::fmt(&file).await,
        Commands::Lsp { .. } => commands::lsp().await,
        Commands::Compile { file } => commands::compile(&file).await,
        Commands::Validate {
            file,
            constraints,
            fail_on_violations,
            format_json,
        } => commands::validate(&file, constraints, fail_on_violations, format_json).await,
        Commands::Change { action } => match action {
            ChangeAction::Create {
                title,
                description,
                context,
                status,
            } => {
                commands::change_create(
                    &title,
                    description.as_deref(),
                    context.as_deref(),
                    status.as_deref(),
                )
                .await
            }
            ChangeAction::Validate { file } => commands::change_validate(&file).await,
        },
        Commands::List { file } => commands::list(&file).await,
        Commands::Tree { file } => commands::tree(&file).await,
        Commands::Init { name } => commands::init(name.as_deref()).await,
        Commands::Diff {
            file1,
            file2,
            format,
        } => commands::diff(&file1, &file2, &format).await,
        Commands::Explain {
            element_id,
            file,
            json,
        } => commands::explain(&element_id, file.as_deref(), json).await,
        Commands::Import { format, file } => commands::import(&format, &file).await,
        Commands::Score { file } => commands::score(file.as_deref()).await,
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

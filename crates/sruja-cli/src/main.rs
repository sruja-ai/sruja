//! Sruja CLI
//!
//! Command-line interface for the Sruja DSL tool.

mod commands;
mod modules;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sruja")]
#[command(
    about = "Architecture-as-code and drift intelligence",
    long_about = None,
    after_help = "Common: sruja quickstart -r .  |  sruja why \"...\" -r .  |  sruja drift -r ."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version information
    Version,
    /// Scan a repository and infer an architecture graph
    Scan {
        /// Path to repository root (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Output path for inferred graph JSON (use "-" for stdout)
        #[arg(long, default_value = "sruja.graph.json")]
        output: String,
    },
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
        /// Check if file would be reformatted (CI mode, exits with error if changes needed)
        #[arg(long)]
        check: bool,
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
    /// Ask "why" questions about architecture (requires repo scan context)
    Why {
        /// Question to answer (e.g., "Why did we choose Kafka?")
        question: String,
        /// Path to repository root for context
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to graph JSON from previous scan (optional, otherwise scans repo)
        #[arg(long)]
        graph: Option<String>,
    },
    /// Detect architectural drift in codebase
    Drift {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to .sruja architecture file (optional)
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output format (text, json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Only show violations, not suggestions
        #[arg(long)]
        violations_only: bool,
    },
    /// Quickstart: Get immediate architecture insights (zero-key, deterministic)
    Quickstart {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().ok();
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Version => commands::version(),
        Commands::Scan { path, output } => commands::scan(&path, &output).await,
        Commands::Lint { file } => commands::lint(&file).await,
        Commands::Export {
            format,
            file,
            extended,
            view_level,
            target,
        } => commands::export(&format, &file, extended, view_level, target.as_deref()).await,
        Commands::Fmt { file, check } => commands::fmt(&file, check).await,
        Commands::Lsp { .. } => commands::lsp().await,
        Commands::Compile { file } => commands::compile(&file).await,
        Commands::Validate {
            file,
            constraints,
            fail_on_violations,
            format_json,
        } => commands::validate(&file, constraints, fail_on_violations, format_json).await,
        Commands::List { file } => commands::list_elements(&file).await,
        Commands::Tree { file } => commands::tree(&file).await,
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
        Commands::Why {
            question,
            repo,
            graph,
        } => commands::why(&question, &repo, graph.as_deref()).await,
        Commands::Drift {
            repo,
            architecture,
            format,
            violations_only,
        } => {
            commands::drift(
                &repo,
                architecture.as_deref(),
                &format,
                false,
                violations_only,
            )
            .await
        }
        Commands::Quickstart { path, format } => commands::quickstart(&path, &format).await,
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

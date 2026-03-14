//! Sruja CLI
//!
//! Command-line interface for the Sruja DSL tool.

mod commands;
mod compliance;
mod config;
mod context_detection;
mod modules;
pub mod selection;
mod utils;
mod views;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sruja")]
#[command(
    about = "Architecture-as-code and drift intelligence",
    long_about = None,
    after_help = "Common: sruja analyze -r .  |  sruja quickstart -r .  |  sruja drift -r ."
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
        /// Output format: text (default) or json (machine-readable for IDE/CI)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Export a Sruja file to various formats
    Export {
        /// Export format (json, mermaid, markdown, context, dsl)
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
        /// Fail with exit code 1 if specified violations found (comma-separated: cycles,layer-violations,god-modules,orphans,all)
        #[arg(long)]
        fail_on: Option<String>,
    },
    /// PR-scoped drift: detect only NEW violations in a PR
    DriftPr {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Base ref (e.g. main, origin/main)
        #[arg(long, short = 'b')]
        base: Option<String>,
        /// Head ref (defaults to HEAD)
        #[arg(long, short = 'H')]
        head: Option<String>,
        /// Output format (text, json, github-actions)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Quickstart: Get immediate architecture insights (zero-key, deterministic)
    Quickstart {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Generate a draft architecture.sruja baseline from scan
        #[arg(long)]
        generate_baseline: bool,
        /// Fail with exit code 1 if specified violations found (comma-separated: cycles,layer-violations,god-modules,orphans,all)
        #[arg(long)]
        fail_on: Option<String>,
    },
    /// Analyze structural complexity (treewidth, SCC, centrality, coupling)
    Complexity {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text, json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Include treewidth analysis
        #[arg(long)]
        treewidth: bool,
        /// Include SCC (strongly connected components) analysis
        #[arg(long)]
        scc: bool,
        /// Include centrality metrics
        #[arg(long)]
        centrality: bool,
        /// Include coupling metrics
        #[arg(long)]
        coupling: bool,
    },
    /// Smart component coverage selection (quality over quantity)
    SmartCoverage {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text, json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Target compression ratio (0.1 = 10%, default: 0.15)
        #[arg(long, short = 't')]
        target_ratio: Option<f64>,
    },
    /// Comprehensive analysis (structural + intent)
    Analyze {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Analysis view (cto, sre, devops, security, product, platform-engineer, tech-lead, or custom from .sruja.yaml)
        #[arg(long, short = 'v', default_value = "cto")]
        view: String,
        /// Path to intent directory (ADRs, .sruja files; defaults to repo/docs/architecture)
        #[arg(long, short = 'i')]
        intent: Option<String>,
        /// Output format (text, json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Compare declared architectural intent vs actual implementation
    Intent {
        #[command(subcommand)]
        cmd: IntentCommand,
    },
    /// Compliance report: structural drift + intent + policy violations (exit 1 if non-compliant)
    Compliance {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to baseline architecture (.sruja file)
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Path to intent directory (ADRs, .sruja files)
        #[arg(long, short = 'i')]
        intent: Option<String>,
        /// Output format (text, json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Export architecture context for AI tools (Cursor, Copilot, Claude)
    Context {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (cursor-rules, copilot-instructions, markdown, json)
        #[arg(long, short = 'f', default_value = "cursor-rules")]
        format: String,
        /// Output file (defaults to stdout)
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
    /// Analyze runtime traces (spans) for emergent cycles and hotspots
    Runtime {
        #[command(subcommand)]
        cmd: RuntimeCommand,
    },
    /// Discovery: question bank or repo context for intelligent capture (use with sruja-architecture-agent skill)
    Discover {
        /// Print repo context summary (structure, technologies, suggested areas) for contextual questions
        #[arg(long)]
        context: bool,
        /// Path to repository (for --context; default current dir)
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format for --context: text (default) or json (machine-readable for agents)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Generate a prompt (skill + repo context) for use with any LLM to produce architecture.sruja without Cursor CLI
    Generate {
        /// Path to repository
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to skill file (SKILL.md); else SRUJA_SKILL_PATH or ./SKILL.md or ./skills/sruja-architecture-agent/SKILL.md
        #[arg(long)]
        skill_path: Option<String>,
        /// Emit prompt only (no LLM call); write to -o or stdout
        #[arg(long)]
        prompt_only: bool,
        /// Output path for prompt (default: stdout if --prompt-only)
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum RuntimeCommand {
    /// Analyze trace/span JSON for emergent cycles and hotspots
    Analyze {
        /// Path to traces JSON file (array of spans with id, name, start, end, children)
        #[arg(long, short = 't')]
        traces: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
enum IntentCommand {
    /// Check intent vs reality and report drift
    Check {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to intent directory (ADRs, .sruja files)
        #[arg(long, short = 'i')]
        intent: Option<String>,
        /// Output format (text, json, markdown)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Propose ADR from detected drift
    Propose {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to intent directory
        #[arg(long, short = 'i')]
        intent: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::try_init().ok();
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Version => commands::version(),
        Commands::Scan { path, output } => commands::scan(&path, &output).await,
        Commands::Lint { file, format } => commands::lint(&file, &format).await,
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
            fail_on,
        } => {
            commands::drift(
                &repo,
                architecture.as_deref(),
                &format,
                false,
                violations_only,
                fail_on.as_deref(),
            )
            .await
        }
        Commands::DriftPr {
            repo,
            base,
            head,
            format,
        } => commands::drift_pr(&repo, base.as_deref(), head.as_deref(), &format).await,
        Commands::Quickstart {
            path,
            format,
            generate_baseline,
            fail_on,
        } => commands::quickstart(&path, &format, generate_baseline, fail_on.as_deref()).await,
        Commands::Complexity {
            repo,
            format,
            treewidth,
            scc,
            centrality,
            coupling,
        } => commands::complexity(&repo, &format, treewidth, scc, centrality, coupling).await,
        Commands::SmartCoverage {
            repo,
            format,
            target_ratio,
        } => commands::smart_coverage(&repo, &format, target_ratio).await,
        Commands::Analyze {
            repo,
            view,
            intent,
            format,
        } => {
            let intent_opt = intent.or_else(|| std::env::var("SRUJA_INTENT_PATH").ok());
            commands::analyze(&repo, &view, intent_opt.as_deref(), &format).await
        }
        Commands::Intent { cmd } => match cmd {
            IntentCommand::Check {
                repo,
                intent,
                format,
            } => {
                let intent_opt = intent.or_else(|| std::env::var("SRUJA_INTENT_PATH").ok());
                commands::intent_check(&repo, intent_opt.as_deref(), &format).await
            }
            IntentCommand::Propose { repo, intent } => {
                commands::intent_propose(&repo, intent.as_deref()).await
            }
        },
        Commands::Compliance {
            repo,
            architecture,
            intent,
            format,
        } => commands::compliance(&repo, architecture.as_deref(), intent.as_deref(), &format).await,
        Commands::Context {
            repo,
            format,
            output,
        } => commands::context_export(&repo, &format, output.as_deref()).await,
        Commands::Runtime { cmd } => match cmd {
            RuntimeCommand::Analyze { traces, format } => {
                commands::runtime_analyze(&traces, &format).await
            }
        },
        Commands::Discover { context, repo, format } => {
            if context {
                commands::discover_context(&repo, &format).await
            } else {
                commands::discover_questions()
            }
        }
        Commands::Generate {
            repo,
            skill_path,
            prompt_only,
            output,
        } => {
            if !prompt_only {
                eprintln!("Only --prompt-only is supported. Use: sruja generate -r . --prompt-only -o prompt.txt");
                eprintln!("Then use the prompt with any LLM; save output as architecture.sruja and run sruja lint.");
                std::process::exit(1);
            }
            commands::generate_prompt(&repo, skill_path.as_deref(), output.as_deref())
        }
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

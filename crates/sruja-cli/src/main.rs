//! Sruja CLI
//!
//! Command-line interface for the Sruja DSL tool.

mod commands;
mod compliance;
mod context_detection;
mod graph_store;
mod modules;
mod report;
pub mod selection;
mod utils;

use clap::{Parser, Subcommand};

#[derive(clap::ValueEnum, Clone, Debug)]
enum ContextIntent {
    AddFeature,
    Refactor,
    FixBug,
    AddTest,
}

impl ContextIntent {
    fn as_str(&self) -> &'static str {
        match self {
            ContextIntent::AddFeature => "add-feature",
            ContextIntent::Refactor => "refactor",
            ContextIntent::FixBug => "fix-bug",
            ContextIntent::AddTest => "add-test",
        }
    }
}

#[derive(Parser)]
#[command(name = "sruja")]
#[command(
     about = "Sruja – Context engineering for the AI era. Architecture-as-code with deterministic CLI primitives for evidence-backed discovery",
    long_about = None,
    after_help = "Stable: sruja quickstart -r .  |  sruja sync -r .  |  sruja status -r .  |  sruja lint  |  sruja drift -r .  |  sruja publish -r . -o repo.bundle.json  |  sruja compose -i <dir> -o system.index.json"
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
    /// Impact analysis: blast radius (upstream dependents + downstream dependencies)
    Impact {
        /// Node selector (exact id or substring match against id/label/path)
        target: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Max traversal depth (0 = none, 1 = direct neighbors)
        #[arg(long, default_value_t = 3)]
        depth: usize,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Ask an architecture question with deterministic evidence from the knowledge graph
    Why {
        /// Question to ask (e.g. "why did we choose PostgreSQL?")
        question: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Lint a Sruja file
    Lint {
        /// Path to .sruja file
        file: String,
        /// Output format: text (default), json, github-actions
        #[arg(long, default_value = "text")]
        format: String,
        /// Optional baseline JSON (ignore existing diagnostics; fail only on new ones)
        #[arg(long)]
        baseline: Option<String>,
        /// Write a baseline JSON snapshot of current diagnostics (exits 0 if parsing succeeds)
        #[arg(long)]
        write_baseline: Option<String>,
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
        /// Named view to export (for markdown format - uses view-driven export)
        #[arg(long)]
        view: Option<String>,
        /// Export all defined custom views in markdown (adds Custom views section)
        #[arg(long)]
        all_views: bool,
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
    /// Start MCP server (stdio)
    Mcp,
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

    /// Detect architectural drift in codebase
    Drift {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to .sruja architecture file (optional)
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output format (text, json, github-actions)
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
        /// Generate a draft repo.sruja baseline from scan
        #[arg(long)]
        generate_baseline: bool,
        /// Fail with exit code 1 if specified violations found (comma-separated: cycles,layer-violations,god-modules,orphans,all)
        #[arg(long)]
        fail_on: Option<String>,
    },
    /// Initialize Sruja in a repo: create .sruja/, run quickstart, optionally generate prompt for baseline
    Init {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Generate .sruja/init_prompt.txt for use with sruja-architecture skill
        #[arg(long)]
        prompt: bool,
    },
    /// Show repo health, baseline, and truth status (reviewed / drifted / unknown)
    Status {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Output format (text, json, github-actions)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Watch a directory for changes, continuously re-evaluating architecture
    Watch {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Clear screen between runs
        #[arg(long)]
        clear: bool,
    },
    /// Refresh evidence (write .sruja/context.json) and run drift
    Sync {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Review: refresh evidence, detect drift, propose updates or open questions
    Review {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Check: CI-focused drift check (always exits 0, outputs github-actions format)
    Check {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Output format (text, json, github-actions)
        #[arg(long, short = 'f', default_value = "github-actions")]
        format: String,
        /// Optional JSON baseline of pre-existing violations (generated by `sruja baseline`)
        #[arg(long = "baseline", short = 'b')]
        violations_baseline: Option<String>,
    },
    /// Baseline: snapshot current violations to ignore them in CI (use with `sruja check --baseline`)
    Baseline {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output path (default: .sruja/violations.baseline.json)
        #[arg(long, short = 'o', default_value = ".sruja/violations.baseline.json")]
        output: String,
    },
    /// Publish repo truth + evidence to repo.bundle.json (multi-repo federation)
    Publish {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        repo_id: Option<String>,
        /// Output path for bundle (default: repo.bundle.json)
        #[arg(long, short = 'o', default_value = "repo.bundle.json")]
        output: String,
    },
    /// Compose one or more repo bundles into system.index.json
    Compose {
        #[arg(long, short = 'i', action = clap::ArgAction::Append)]
        input: Vec<String>,
        #[arg(long)]
        recursive: bool,
        /// Output path for system index (default: system.index.json)
        #[arg(long, short = 'o', default_value = "system.index.json")]
        output: String,
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
        #[arg(long, short = 'r', action = clap::ArgAction::Append)]
        repo: Vec<String>,
        /// Output format (cursor-rules, copilot-instructions, markdown, repomap, json, for-ai)
        #[arg(long, short = 'f', default_value = "cursor-rules")]
        format: String,
        /// Output file (defaults to stdout)
        #[arg(long, short = 'o')]
        output: Option<String>,
        /// Optional file focus for task-scoped context (relative to repo root or absolute path)
        #[arg(long)]
        file: Option<String>,
        /// Optional intent hint for task-scoped context
        #[arg(long)]
        intent: Option<ContextIntent>,
        /// Max dependency traversal depth when --file is provided (0 = none, 1 = direct neighbors)
        #[arg(long, default_value_t = 2)]
        depth: usize,
        /// Max tokens to output (approximate)
        #[arg(long, default_value_t = 10000)]
        max_tokens: usize,
    },
    /// Analyze runtime traces (spans) for emergent cycles and hotspots
    Runtime {
        #[command(subcommand)]
        cmd: RuntimeCommand,
    },
    /// Discovery: question bank or repo context for intelligent capture (use with sruja-architecture skill)
    Discover {
        /// Print repo context summary (structure, technologies, suggested areas) for contextual questions
        #[arg(long)]
        context: bool,
        /// Explain what the scanner discovered, why it inferred that shape, and what to review next
        #[arg(long)]
        explain: bool,
        /// Generate a repository map with tree-sitter signatures for top files (for LLM context)
        #[arg(long)]
        repomap: bool,
        /// Path to repository (for --context; default current dir)
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format for --context/--explain: text (default) or json (machine-readable for agents)
        #[arg(long, default_value = "text")]
        format: String,
        /// Maximum number of files to include in repomap (default: 100)
        #[arg(long, default_value_t = 100)]
        max_files: usize,
        /// Maximum tokens for repomap (default: 5000)
        #[arg(long, default_value_t = 5000)]
        max_tokens: usize,
    },
    /// Component knowledge: list doc links, show doc for an element, or find gaps
    Knowledge {
        #[command(subcommand)]
        cmd: KnowledgeCommand,
    },
    /// List source bindings (OpenAPI, Kubernetes, docs) linked to architecture elements
    Sources {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to .sruja architecture file (optional)
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Element ID to show sources for (optional; shows all if omitted)
        element: Option<String>,
        /// Filter by source type (openapi, kubernetes, docs, etc.)
        #[arg(long, short = 't')]
        source_type: Option<String>,
        /// Validate that all source paths exist
        #[arg(long, short = 'v')]
        validate: bool,
        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Generate a prompt (skill + repo context) for use with any LLM to produce architecture.sruja without Cursor CLI
    Generate {
        /// Path to repository
        #[arg(long, short = 'r', action = clap::ArgAction::Append)]
        repo: Vec<String>,
        /// Path to skill file (SKILL.md); else SRUJA_SKILL_PATH or ./SKILL.md or ./skills/sruja-architecture/SKILL.md
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
pub enum KnowledgeCommand {
    /// List elements that have a doc link
    List {
        /// Path to repository root (for resolving default architecture file)
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to .sruja architecture file (optional; default: repo.sruja / architecture.sruja)
        #[arg(long, short = 'a')]
        architecture: Option<String>,
    },
    /// Show knowledge file content for an element
    Show {
        /// Element ID (e.g. PaymentService or Backend.API)
        element_id: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to .sruja architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
    },
    /// List elements that have no doc link (gaps)
    Gaps {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to .sruja architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
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
        #[arg(long, short = 'i', default_value = None)]
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
        #[arg(long, short = 'i', default_value = None)]
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
        Commands::Impact {
            repo,
            target,
            depth,
            format,
        } => commands::impact(&repo, &target, depth, &format).await,
        Commands::Why {
            question,
            repo,
            format,
        } => commands::why(&repo, &question, &format).await,
        Commands::Lint {
            file,
            format,
            baseline,
            write_baseline,
        } => {
            commands::lint(
                &file,
                &format,
                baseline.as_deref(),
                write_baseline.as_deref(),
            )
            .await
        }
        Commands::Export {
            format,
            file,
            extended,
            view_level,
            target,
            view,
            all_views,
        } => {
            commands::export(
                &format,
                &file,
                extended,
                view_level,
                target.as_deref(),
                view.as_deref(),
                all_views,
            )
            .await
        }
        Commands::Fmt { file, check } => commands::fmt(&file, check).await,
        Commands::Lsp { .. } => commands::lsp().await,
        Commands::Mcp => commands::mcp().await,
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
        Commands::Init { path, prompt } => commands::init(&path, prompt).await,
        Commands::Status { path, format } => commands::status(&path, &format).await,
        Commands::Watch { path, clear } => commands::watch(&path, clear).await,
        Commands::Sync { path, format } => commands::sync(&path, &format).await,
        Commands::Review { path, format } => commands::review(&path, &format).await,
        Commands::Check {
            path,
            format,
            violations_baseline,
        } => commands::check(&path, &format, violations_baseline.as_deref()).await,
        Commands::Baseline { repo, output } => commands::baseline(&repo, &output).await,
        Commands::Publish {
            repo,
            repo_id,
            output,
        } => commands::publish(&repo, repo_id.as_deref(), &output).await,
        Commands::Compose {
            input,
            recursive,
            output,
        } => commands::compose(&input, recursive, &output).await,
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
            file,
            intent,
            depth,
            max_tokens,
        } => {
            commands::context_export(
                &repo,
                &format,
                output.as_deref(),
                file.as_deref(),
                intent.as_ref().map(ContextIntent::as_str),
                depth,
                max_tokens,
            )
            .await
        }
        Commands::Runtime { cmd } => match cmd {
            RuntimeCommand::Analyze { traces, format } => {
                commands::runtime_analyze(&traces, &format).await
            }
        },
        Commands::Discover {
            context,
            explain,
            repomap,
            repo,
            format,
            max_files,
            max_tokens,
        } => {
            if repomap {
                commands::discover_repomap_cmd(&repo, max_files, max_tokens).await
            } else if explain {
                commands::discover_explain(&repo, &format).await
            } else if context {
                commands::discover_context(&repo, &format).await
            } else {
                commands::discover_questions()
            }
        }
        Commands::Knowledge { cmd } => commands::knowledge(cmd).await,
        Commands::Sources {
            repo,
            architecture,
            element,
            source_type,
            validate,
            format,
        } => {
            commands::sources(
                &repo,
                architecture.as_deref(),
                element.as_deref(),
                source_type.as_deref(),
                validate,
                &format,
            )
            .await
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
            std::process::exit(e.exit_code());
        }
    }
}

use crate::commands;
use clap::{Parser, Subcommand};

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ContextIntent {
    AddFeature,
    Refactor,
    FixBug,
    AddTest,
}

impl ContextIntent {
    pub fn as_str(&self) -> &'static str {
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
    about = "Architecture-as-code CLI for keeping repo context, drift checks, and AI guidance in sync",
    long_about = None,
    after_help = r#"Start Here:
  sruja start -r . --prompt   Set up .sruja/ and generate an AI-ready prompt
  sruja overview -r .         Get a quick structural read of the repo

Daily Loop:
  sruja daily -r .            Refresh evidence and review what changed
  sruja watch -r .            Keep architecture feedback live while coding
  sruja doctor -r .           Quick truth + health check

Docs & CI:
  sruja lint repo.sruja
  sruja export markdown repo.sruja
  sruja check -r . -f github-actions"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print version information
    Version,
    /// Propose architectural changes for review
    Propose {
        #[command(subcommand)]
        cmd: ProposeCommand,
    },
    /// Scan a repository and infer an architecture graph
    Scan {
        /// Path to repository root (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Output path for inferred graph JSON (use "-" for stdout)
        #[arg(long, default_value = "sruja.graph.json")]
        output: String,
    },
    /// Adversarial architectural critique of proposed changes
    Critique {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Changed file paths to critique
        #[arg(long, short = 'f')]
        files: Vec<String>,
        /// Description of what the change does (helps pattern matching)
        #[arg(long, short = 'd')]
        description: Option<String>,
        /// Proposal ID if this is an approved proposal
        #[arg(long, short = 'p')]
        proposal: Option<String>,
        /// Git base ref for diff-based critique
        #[arg(long)]
        base: Option<String>,
        /// Git head ref for diff-based critique
        #[arg(long)]
        head: Option<String>,
        /// Critique staged git changes
        #[arg(long)]
        staged: bool,
        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,
        /// Fail the command (exit 1) if findings of this level or higher are found
        #[arg(long)]
        fail_on: Option<String>,
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
        /// Export format (json, mermaid, markdown, context, dsl, d2)
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
        /// Inject the exported content into a file between `<!-- sruja:start -->` and `<!-- sruja:end -->` markers
        #[arg(long)]
        inject: Option<String>,
        /// Hydrate architecture elements with source code content (JSON only)
        #[arg(long)]
        hydrate: bool,
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
    Mcp {
        /// Default repository root used when MCP tool calls omit a path
        #[arg(long, short = 'r', default_value = ".")]
        root: String,
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
    /// Quick repo overview with immediate architecture insights
    #[command(visible_alias = "overview")]
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
    /// Set up Sruja in a repo and print the next best steps
    #[command(visible_alias = "start")]
    Init {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Generate .sruja/init_prompt.txt for use with sruja-architecture skill
        #[arg(long)]
        prompt: bool,
        /// Automatically detect architecture and generate repo.sruja (the "wow" factor)
        #[arg(long, short = 'a')]
        auto: bool,
        /// Overwrite repo.sruja if it already exists
        #[arg(long, short = 'f')]
        force: bool,
        /// Install a git pre-commit hook to run Sruja checks
        #[arg(long)]
        hook: bool,
        /// Install a GitHub Actions workflow for Sruja checks
        #[arg(long)]
        ci: bool,
        /// Do not write files, only show what would happen
        #[arg(long)]
        dry_run: bool,
    },
    /// Quick repo health check: baseline, truth status, and last evidence refresh
    #[command(visible_alias = "doctor")]
    Status {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Output format (text, json, github-actions)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Keep architecture feedback live while you code
    Watch {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Clear screen between runs
        #[arg(long)]
        clear: bool,
        /// Only watch specific paths (comma separated)
        #[arg(long)]
        focus: Option<String>,
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
    /// Daily review: refresh evidence, detect drift, and suggest next actions
    #[command(visible_alias = "daily")]
    Review {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Show all violations (default is capped at 5)
        #[arg(long, short)]
        verbose: bool,
        /// Include adversarial critique of unstaged changes
        #[arg(long)]
        critique: bool,
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
        /// Strict mode: fail if unproposed changes are detected
        #[arg(long)]
        strict: bool,
    },
    /// Export architecture context for AI tools (Cursor, Copilot, Claude)
    Context {
        /// Path to repository root
        #[arg(long, short = 'r', action = clap::ArgAction::Append)]
        repo: Vec<String>,
        /// Output format (cursor-rules, copilot-instructions, markdown, repomap, json, for-ai, legacy-json)
        #[arg(long, short = 'f', default_value = "cursor-rules")]
        format: String,
        /// Output file (defaults to stdout)
        #[arg(long, short = 'o')]
        output: Option<String>,
        /// Optional file focus for task-scoped context (relative to repo root or absolute path)
        #[arg(long)]
        file: Option<String>,
        /// Optional architecture element ID focus (e.g. MySystem.Api)
        #[arg(long)]
        element_id: Option<String>,
        /// Optional natural language query (semantic/recall fallback)
        #[arg(long)]
        query: Option<String>,
        /// Optional git base ref for PR scope (requires --head-ref)
        #[arg(long)]
        base_ref: Option<String>,
        /// Optional git head ref for PR scope (requires --base-ref)
        #[arg(long)]
        head_ref: Option<String>,
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
    /// Generate indices for architectural nodes
    Index {
        #[command(subcommand)]
        cmd: IndexCommand,
    },
    /// Query the architectural registry
    Query {
        /// Query string (e.g., "Checkout", "depends_on Payments")
        query: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Calculate and report architecture health score
    Health {
        /// Repository root to scan
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },

    /// Context Score: how well-equipped is an AI agent to work on this codebase? (0-100)
    #[command(name = "context-score")]
    ContextScore {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Fail with exit code 1 if score is below this threshold
        #[arg(long)]
        fail_under: Option<u8>,
    },
    /// Generate an interactive HTML/D3.js visualization of the architecture context
    #[command(name = "context-graph")]
    ContextGraph {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output path for the HTML file (default: context_graph.html)
        #[arg(long, short = 'o', default_value = "context_graph.html")]
        output: String,
        /// Open the browser after generation
        #[arg(long)]
        open: bool,
    },

    /// Focus: get a context briefing for a specific file or architecture element
    Focus {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// File path to focus on (relative to repo root)
        #[arg(long)]
        file: Option<String>,
        /// Architecture element ID to focus on (e.g. Auth.Handler)
        #[arg(long)]
        element_id: Option<String>,
        /// Output format (text, json, for-ai)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },

    /// Ingest external context (ADRs, design docs, API contracts) into .sruja/context/
    Ingest {
        /// Files or directories to ingest
        sources: Vec<String>,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Category tag (adr, design-doc, api-contract, runbook, note)
        #[arg(long, short = 'c')]
        category: Option<String>,
        /// Comma-separated architecture element IDs to link (e.g. Auth.Handler,Database.Users)
        #[arg(long, short = 'e')]
        elements: Option<String>,
    },
    /// Manage Agentic Memory (learnings, guardrails, and failed hypotheses)
    Agent {
        #[command(subcommand)]
        cmd: AgentCommand,
    },
}

#[derive(Subcommand)]
pub enum AgentCommand {
    /// Show architectural learning history and guardrails
    History {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Filter by architectural element ID
        #[arg(long, short = 'e')]
        element_id: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Manually record a learning or failed hypothesis
    Record {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Context of the learning (e.g. "Refactoring Auth")
        #[arg(long, short = 'c')]
        context: String,
        /// What was being tried
        #[arg(long, short = 'H')]
        hypothesis: String,
        /// Outcome (success or failed)
        #[arg(long, short = 'o', default_value = "failed")]
        outcome: String,
        /// Explicit advice for future agents (the "Guardrail")
        #[arg(long, short = 'g')]
        guardrail: String,
        /// Why it failed (optional)
        #[arg(long, short = 's')]
        reason: Option<String>,
        /// Comma-separated architectural element IDs affected
        #[arg(long, short = 'e')]
        elements: Option<String>,
    },
    /// Clear all agentic memory for this repository
    Clear {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum IndexCommand {
    /// Generate semantic embeddings for architectural nodes
    Semantic {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output path for vector index
        #[arg(long, short = 'o', default_value = ".sruja/vectors.json")]
        output: String,
    },
    /// Automatically discover architectural artifacts and update registry
    Registry {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Automatically apply discovered sources to the architecture file
        #[arg(long)]
        fix: bool,
        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Generate a visual dashboard for the architectural registry
    Dashboard {
        /// Path to repository root (or federated index)
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output file path (e.g., dashboard.md or dashboard.html)
        #[arg(long, short = 'o', default_value = "dashboard.md")]
        output: String,
    },
}

#[derive(Subcommand)]
pub enum ProposeCommand {
    /// Create a new architectural proposal
    Create {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Description of the change
        #[arg(long, short = 'd')]
        description: String,
        /// Add elements in format "id:kind:label[:tech]"
        #[arg(long, short = 'e')]
        add_elements: Vec<String>,
        /// Add relationships in format "source->target[:label]"
        #[arg(long, short = 'l')]
        add_relationships: Vec<String>,
        /// Remove elements by ID
        #[arg(long)]
        remove_elements: Vec<String>,
    },
    /// List all architectural proposals
    List {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    /// Approve and merge a proposal
    Approve {
        /// Proposal ID to approve
        proposal_id: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
}

#[derive(Subcommand)]
pub enum IntentCommand {
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
        /// Strict mode: fail if unproposed changes are detected
        #[arg(long)]
        strict: bool,
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

pub async fn run_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    let result = match command {
        Commands::Version => commands::version(),
        Commands::Propose { cmd } => match cmd {
            ProposeCommand::Create {
                repo,
                description,
                add_elements,
                add_relationships,
                remove_elements,
            } => {
                commands::propose_create(
                    &repo,
                    &description,
                    add_elements,
                    add_relationships,
                    remove_elements,
                )
                .await
            }
            ProposeCommand::List { repo } => commands::propose_list(&repo).await,
            ProposeCommand::Approve { proposal_id, repo } => {
                commands::propose_approve(&repo, &proposal_id).await
            }
        },
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
            inject,
            hydrate,
        } => {
            commands::export(
                &format,
                &file,
                commands::ExportOptions {
                    extended,
                    view_level,
                    target,
                    view_name: view,
                    all_views,
                    inject,
                    hydrate,
                },
            )
            .await
        }
        Commands::Fmt { file, check } => commands::fmt(&file, check).await,
        Commands::Lsp { .. } => commands::lsp().await,
        Commands::Mcp { root } => commands::mcp(&root).await,
        Commands::Critique {
            repo,
            files,
            description,
            proposal,
            base,
            head,
            staged,
            format,
            fail_on,
        } => {
            commands::critique(
                &repo,
                files,
                description,
                proposal,
                base,
                head,
                staged,
                &format,
                fail_on.as_deref(),
            )
            .await
        }
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
        Commands::Init {
            path,
            prompt,
            auto,
            force,
            hook,
            ci,
            dry_run,
        } => commands::init(&path, prompt, auto, force, hook, ci, dry_run).await,
        Commands::Status { path, format } => commands::status(&path, &format).await,
        Commands::Watch { path, clear, focus } => commands::watch(&path, clear, focus).await,
        Commands::Sync { path, format } => commands::sync(&path, &format).await,
        Commands::Review {
            path,
            format,
            verbose,
            critique,
        } => commands::review(&path, &format, verbose, critique).await,
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
                strict,
            } => {
                let intent_opt = intent.or_else(|| std::env::var("SRUJA_INTENT_PATH").ok());
                commands::intent_check(&repo, intent_opt.as_deref(), &format, strict).await
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
            strict,
        } => {
            commands::compliance(
                &repo,
                architecture.as_deref(),
                intent.as_deref(),
                &format,
                strict,
            )
            .await
        }
        Commands::Context {
            repo,
            format,
            output,
            file,
            element_id,
            query,
            base_ref,
            head_ref,
            intent,
            depth,
            max_tokens,
        } => {
            commands::context_export(
                &repo,
                &format,
                output.as_deref(),
                commands::ContextRequest {
                    file: file.as_deref(),
                    element_id: element_id.as_deref(),
                    query: query.as_deref(),
                    base_ref: base_ref.as_deref(),
                    head_ref: head_ref.as_deref(),
                    intent: intent.as_ref().map(ContextIntent::as_str),
                    depth,
                    max_tokens,
                },
            )
            .await
        }
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
        Commands::Index { cmd } => match cmd {
            IndexCommand::Semantic {
                repo,
                architecture,
                output,
            } => commands::semantic_index(&repo, architecture.as_deref(), &output).await,
            IndexCommand::Registry {
                repo,
                architecture,
                fix,
                format,
            } => commands::registry_index(&repo, architecture.as_deref(), fix, &format).await,
            IndexCommand::Dashboard { repo, output } => {
                commands::registry_dashboard(&repo, &output).await
            }
        },
        Commands::Query {
            query,
            repo,
            architecture,
            format,
        } => commands::query_registry(&repo, architecture.as_deref(), &query, &format).await,
        Commands::Completions { shell } => commands::completions(shell),
        Commands::Health {
            repo,
            architecture,
            format,
        } => commands::health(&repo, architecture.as_deref(), &format).await,
        Commands::ContextScore {
            repo,
            format,
            fail_under,
        } => commands::context_score(&repo, &format, fail_under).await,
        Commands::ContextGraph { repo, output, open } => {
            commands::context_graph(&repo, &output, open).await
        }
        Commands::Focus {
            repo,
            file,
            element_id,
            format,
        } => commands::focus(&repo, file.as_deref(), element_id.as_deref(), &format).await,
        Commands::Ingest {
            sources,
            repo,
            category,
            elements,
        } => commands::ingest(&repo, &sources, category.as_deref(), elements.as_deref()).await,
        Commands::Agent { cmd } => match cmd {
            AgentCommand::History {
                repo,
                element_id,
                format,
            } => commands::agent_history(&repo, element_id.as_deref(), &format).await,
            AgentCommand::Record {
                repo,
                context,
                hypothesis,
                outcome,
                guardrail,
                reason,
                elements,
            } => {
                commands::agent_record(
                    &repo,
                    &context,
                    &hypothesis,
                    &outcome,
                    &guardrail,
                    reason.as_deref(),
                    elements.as_deref(),
                )
                .await
            }
            AgentCommand::Clear { repo, force } => commands::agent_clear(&repo, force).await,
        },
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            e.report();
            std::process::exit(e.exit_code());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn context_intent_as_str_mappings() {
        assert_eq!(ContextIntent::AddFeature.as_str(), "add-feature");
        assert_eq!(ContextIntent::Refactor.as_str(), "refactor");
        assert_eq!(ContextIntent::FixBug.as_str(), "fix-bug");
        assert_eq!(ContextIntent::AddTest.as_str(), "add-test");
    }

    #[test]
    fn parses_context_defaults() {
        let cli = Cli::try_parse_from(["sruja", "context"]).expect("parse");
        match cli.command {
            Commands::Context {
                format,
                repo,
                output,
                file,
                element_id,
                query,
                base_ref,
                head_ref,
                intent,
                depth,
                max_tokens,
            } => {
                assert_eq!(format, "cursor-rules");
                assert!(repo.is_empty());
                assert!(output.is_none());
                assert!(file.is_none());
                assert!(element_id.is_none());
                assert!(query.is_none());
                assert!(base_ref.is_none());
                assert!(head_ref.is_none());
                assert!(intent.is_none());
                assert_eq!(depth, 2);
                assert_eq!(max_tokens, 10000);
            }
            _ => panic!("expected Context command"),
        }
    }
}

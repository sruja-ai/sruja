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

/// Check if a deprecated alias was used to invoke a command.
fn was_invoked_as(alias: &str) -> bool {
    std::env::args().nth(1).is_some_and(|arg| arg == alias)
}

#[derive(Parser)]
#[command(name = "sruja")]
#[command(
    about = "Architecture-as-code CLI for keeping repo context, drift checks, and AI guidance in sync",
    long_about = None,
    after_help = r#"Product loop (define truth → context → drift → review):
  Use the sruja-architecture skill + repo.sruja for reviewed intent; lint after edits;
  sync/review/drift for freshness; focus or ai before coding; MCP inside AI tools.

Start here:
  sruja start -r . --prompt   Set up .sruja/ and emit a skill-ready prompt
  sruja quickstart -r .       First structural read (optional --generate-baseline)
  sruja focus -r . --file <path>   Task-scoped blast radius before you edit

Daily loop:
  sruja review -r .           Evidence refresh + drift + next actions (alias: daily)
  sruja watch -r .            Live feedback while coding
  sruja status -r .           Truth freshness + baseline (alias: doctor)

Three different scores (do not confuse):
  sruja status                Truth sync / baseline signals
  sruja health                Structural violations vs repo.sruja
  sruja context-score         AI-readiness (0–100)

Docs & CI:
  sruja lint repo.sruja
  sruja export markdown repo.sruja
  sruja drift --ci -r .       CI drift (github-actions format; replaces hidden `check`)"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Increase logging verbosity (-v for info, -vv for debug, -vvv for trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Path to custom classification rules YAML file
    #[arg(long, global = true)]
    pub classification_rules: Option<String>,
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
        /// Repository root (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Output path for inferred graph JSON (use "-" for stdout)
        #[arg(long, default_value = "sruja.graph.json")]
        output: String,
    },
    /// Adversarial architectural critique of proposed changes (review-oriented, not context-oriented)
    ///
    /// Use `critique` to get an adversarial review of code changes against architecture.
    /// For context/briefing, use `ai` (task-specific) or `onboard` (full-repo).
    Critique {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
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
        /// Output format (text, json, for-ai)
        #[arg(long, default_value = "text")]
        format: String,
        /// Optional enrichment (cmd/openai) to add a narrative review grounded in the critique report.
        #[arg(long)]
        enrich: bool,
        /// Enrichment provider (cmd|openai). Can also be set via SRUJA_ENRICH_PROVIDER or .sruja/config.toml.
        #[arg(long, alias = "llm-provider")]
        enrich_provider: Option<String>,
        /// External enrichment command (reads JSON from stdin; writes markdown to stdout).
        #[arg(long)]
        enrich_cmd: Option<String>,
        /// Model name (used for provider=openai). Can also be set via SRUJA_ENRICH_MODEL.
        #[arg(long, alias = "llm-model")]
        enrich_model: Option<String>,
        /// Base URL (used for provider=openai). Can also be set via SRUJA_ENRICH_BASE_URL.
        #[arg(long, alias = "llm-base-url")]
        enrich_base_url: Option<String>,
        /// Timeout for enrichment in milliseconds (default: 15000)
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes to read from enrichment stdout (default: 20000)
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
        /// Fail the command (exit 1) if findings of this level or higher are found
        #[arg(long)]
        fail_on: Option<String>,
    },
    /// Impact analysis: blast radius (upstream dependents + downstream dependencies)
    ///
    /// Use after you know *where* you are working; for pre-task briefing and AI instructions use `focus`.
    Impact {
        /// Node selector (exact id or substring match against id/label/path)
        target: String,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Max traversal depth (0 = none, 1 = direct neighbors)
        #[arg(long, default_value_t = 3)]
        depth: usize,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Investigate architecture decisions with deterministic evidence
    ///
    /// Use this when asking "Why is this like this?" — finds rationale from the knowledge graph.
    /// For task briefing, use `focus`. For AI paste-ready brief, use `ai`.
    Why {
        /// Question to ask (e.g. "why did we choose PostgreSQL?")
        question: String,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Use reasoning-tree traversal (PageIndex-style) for traceable why explanations
        #[arg(long)]
        reasoned: bool,
        /// Use LLM-guided tree search for context-aware, question-relevant traversal
        #[arg(long)]
        llmguided: bool,
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
        /// Export from scan graph instead of .sruja file (for graphml, neo4j, obsidian)
        #[arg(long)]
        from_scan: bool,
        /// Repository path for scan-based export (requires --from-scan)
        #[arg(long, short = 'r')]
        repo: Option<String>,
        /// Output directory for file-based exports (obsidian)
        #[arg(long)]
        output_dir: Option<String>,
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
    ///
    /// Environment:
    /// - `SRUJA_MCP_READONLY=1` — expose only read/query tools; mutating calls return an error.
    /// - `SRUJA_MCP_LOG=1` — emit one JSON log line per `tools/call` on stderr (tool, repo, ms, ok).
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
        /// CI mode: defaults format to github-actions unless --format is explicitly set (equivalent to deprecated `sruja check`)
        #[arg(long)]
        ci: bool,
        /// Optional JSON baseline of pre-existing violations (for --ci mode; generated by `sruja baseline`)
        #[arg(long = "baseline", short = 'b')]
        violations_baseline: Option<String>,
        /// Baseline mode for scan vs DSL comparison (auto|overview|exhaustive)
        #[arg(long)]
        baseline_mode: Option<String>,
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
    /// First look: structural overview and optional baseline generation
    ///
    /// Use when asking "What is in this repo?" For a full-repo brief, use `onboard`. For an AI task brief, use `ai`.
    #[command(visible_alias = "overview")]
    Quickstart {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
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
    /// Set up Sruja in a repo: create .sruja/, .srujaignore, and optional baseline
    ///
    /// This is a setup command, not a briefing command.
    /// For repo overview, use `quickstart`. For full briefing, use `onboard`.
    #[command(visible_alias = "start")]
    Init {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Generate .sruja/init_prompt.txt for use with sruja-architecture skill
        #[arg(long, group = "init_mode")]
        prompt: bool,
        /// Automatically detect architecture and generate repo.sruja
        #[arg(long, short = 'a', group = "init_mode")]
        auto: bool,
        /// Overwrite repo.sruja if it already exists (only meaningful with --auto)
        #[arg(long, short = 'f')]
        force: bool,
        /// Install a git pre-commit hook to run Sruja checks
        #[arg(long, group = "init_mode")]
        hook: bool,
        /// Install a GitHub Actions workflow for Sruja checks
        #[arg(long, group = "init_mode")]
        ci: bool,
        /// Do not write files, only show what would happen
        #[arg(long)]
        dry_run: bool,
    },
    /// Truth freshness and baseline state
    ///
    /// Answers: "Is my `repo.sruja` current? When was evidence last refreshed?"
    /// For structural health, use `health`. For AI-readiness, use `context-score`.
    #[command(visible_alias = "doctor")]
    Status {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Output format (text, json, github-actions)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Show evolutionary health and metrics
        #[arg(long = "evolution", short = 'e')]
        evolution: bool,
    },
    /// Keep architecture feedback live while you code
    Watch {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
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
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Daily action list: refresh evidence, detect drift, suggest next steps (alias: `daily`)
    ///
    /// Use when asking "What should I tackle today?" For a one-time repo read use `quickstart`.
    /// For truth-at-a-glance without the action list, use `status`.
    #[command(visible_alias = "daily")]
    Review {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Show all violations (default is capped at 5)
        #[arg(long, short = 'a')]
        show_all: bool,
        /// Include adversarial critique of unstaged changes
        #[arg(long)]
        critique: bool,
    },
    /// Build scan evidence and learned-fact hypotheses under `.sruja/` (never edits `repo.sruja`)
    ///
    /// Output is evidence-backed inference for review — not the same as reviewed architecture.
    Learn {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Only include facts referencing this file (relative to repo root)
        #[arg(long)]
        file: Option<String>,
        /// Only include facts touching paths changed since this git ref (e.g. main)
        #[arg(long)]
        since: Option<String>,
        /// Do not write `.sruja/proposals/learn-*.json` bundles from proposed facts
        #[arg(long)]
        skip_proposals: bool,
        /// When set to `false`, do not write learn proposal files (design-doc alias of `--skip-proposals`)
        #[arg(long = "apply-proposals", default_value_t = true, action = clap::ArgAction::Set)]
        apply_proposals: bool,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// [deprecated: use `drift --ci`] CI-focused drift check
    #[command(hide = true)]
    Check {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Output format (text, json, github-actions)
        #[arg(long, short = 'f', default_value = "github-actions")]
        format: String,
        /// Optional JSON baseline of pre-existing violations (generated by `sruja baseline`)
        #[arg(long = "baseline", short = 'b')]
        violations_baseline: Option<String>,
    },
    /// Baseline: snapshot current violations to ignore them in CI (use with `sruja drift --ci --baseline`)
    Baseline {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output path (default: .sruja/violations.baseline.json)
        #[arg(long, short = 'o', default_value = ".sruja/violations.baseline.json")]
        output: String,
    },
    /// Publish repo truth + evidence to repo.bundle.json (multi-repo federation)
    Publish {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
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
    /// Structural drift + intent + policy gate (exit 1 when non-compliant)
    ///
    /// Use as an architecture aggregate gate—not a generic enterprise audit unless policies are concrete.
    Compliance {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
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
    /// Paste-ready briefing for an AI coding assistant
    ///
    /// Produces a task-specific brief combining worktree, architecture signals, and verification hints.
    /// For full-repo onboarding, use `onboard`. For structural overview, use `quickstart`.
    Ai {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Natural-language task or intent to give the AI coding assistant
        #[arg(long, short = 't')]
        task: Option<String>,
        /// File path to focus on (defaults to the first changed file when available)
        #[arg(long)]
        file: Option<String>,
        /// Architecture element ID to focus on (e.g. Auth.Handler)
        #[arg(long)]
        element_id: Option<String>,
        /// Natural-language query to find relevant architecture context
        #[arg(long)]
        query: Option<String>,
        /// Git base ref for diff-scoped context
        #[arg(long)]
        base_ref: Option<String>,
        /// Git head ref for diff-scoped context
        #[arg(long)]
        head_ref: Option<String>,
        /// Use staged changes instead of all changes against HEAD for changed-file detection
        #[arg(long)]
        staged: bool,
        /// Max tokens for the embedded task context
        #[arg(long, default_value_t = 8000)]
        max_tokens: usize,
        /// Output file (defaults to stdout)
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Optional enrichment (cmd/openai) to add a narrative plan grounded in the task context.
        #[arg(long)]
        enrich: bool,
        /// Enrichment provider (cmd|openai). Can also be set via SRUJA_ENRICH_PROVIDER or .sruja/config.toml.
        #[arg(long, alias = "llm-provider")]
        enrich_provider: Option<String>,
        /// External enrichment command (reads JSON from stdin; writes markdown to stdout).
        #[arg(long)]
        enrich_cmd: Option<String>,
        /// Model name (used for provider=openai). Can also be set via SRUJA_ENRICH_MODEL.
        #[arg(long, alias = "llm-model")]
        enrich_model: Option<String>,
        /// Base URL (used for provider=openai). Can also be set via SRUJA_ENRICH_BASE_URL.
        #[arg(long, alias = "llm-base-url")]
        enrich_base_url: Option<String>,
        /// Timeout for enrichment in milliseconds (default: 15000)
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes to read from enrichment stdout (default: 20000)
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
    },
    /// Complete architecture brief for human or AI reader
    ///
    /// Produces a full-repo onboarding brief with trust signals: truth status, drift counts, context score.
    /// For task-specific AI briefs, use `ai`. For quick structural overview, use `quickstart`.
    Onboard {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output format (markdown, json, github-actions)
        #[arg(long, short = 'f', default_value = "markdown")]
        format: String,
        /// Max number of items per section (entrypoints, elements, relationships)
        #[arg(long, default_value_t = 8)]
        max_items: usize,
        /// Optional LLM enrichment (adds a clearly-labeled narrative section; never changes grounded scan output)
        #[arg(long)]
        enrich: bool,
        /// Enrichment provider (cmd|openai). Can also be set via SRUJA_ENRICH_PROVIDER or .sruja/config.toml.
        #[arg(long, alias = "llm-provider")]
        enrich_provider: Option<String>,
        /// External enrichment command to run (reads onboard JSON from stdin; writes markdown to stdout).
        /// This is the recommended enterprise path because Sruja never handles API keys or network.
        ///
        /// Example: --enrich-cmd 'claude -p -'  (depending on your CLI)
        #[arg(long)]
        enrich_cmd: Option<String>,
        /// Model name (used for provider=openai). Can also be set via SRUJA_ENRICH_MODEL.
        #[arg(long, alias = "llm-model")]
        enrich_model: Option<String>,
        /// Base URL (used for provider=openai). Can also be set via SRUJA_ENRICH_BASE_URL.
        #[arg(long, alias = "llm-base-url")]
        enrich_base_url: Option<String>,
        /// Timeout for --enrich-cmd in milliseconds (default: 15000)
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes to read from --enrich-cmd stdout (default: 20000)
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
        /// Output file (defaults to stdout)
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
    /// Structured architecture context for AI editor integration (Cursor, Copilot, Claude)
    ///
    /// Use MCP tools inside your AI editor for the best experience.
    /// For CLI-based briefing, use `focus` or `ai`.
    #[command(name = "ai-context", alias = "context")]
    AiContext {
        /// Optional run ID for tracing (defaults to auto-generated)
        #[arg(long)]
        run_id: Option<String>,
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
    /// Scanner introspection for AI/debug: explain scan, repomap, discovery questions
    ///
    /// For a first human read use `quickstart`. For a full repo brief use `onboard`.
    /// For a coding task brief use `ai` or `focus`.
    Discover {
        #[command(subcommand)]
        cmd: Option<DiscoverCommand>,

        /// Path to repository (default current dir)
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,

        // --- Legacy flags (hidden, kept for backward compatibility) ---
        /// [deprecated: use `discover context`] Print repo context summary
        #[arg(long, hide = true)]
        context: bool,
        /// [deprecated: use `discover explain`] Explain scan results
        #[arg(long, hide = true)]
        explain: bool,
        /// [deprecated: use `discover repomap`] Generate a repository map
        #[arg(long, hide = true)]
        repomap: bool,

        // --- Shared args used by subcommands and legacy dispatch ---
        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        format: String,
        /// Maximum number of files to include in repomap (default: 100)
        #[arg(long, default_value_t = 100)]
        max_files: usize,
        /// Maximum tokens for repomap (default: 5000)
        #[arg(long, default_value_t = 5000)]
        max_tokens: usize,
        /// Export the explanation report to a markdown file (e.g. GRAPH_REPORT.md)
        #[arg(long)]
        export_report: Option<String>,
        /// Optional LLM enrichment (adds a clearly-labeled narrative section to explain; never changes grounded scan output)
        #[arg(long)]
        enrich: bool,
        /// Enrichment provider (cmd|openai). Can also be set via SRUJA_ENRICH_PROVIDER or .sruja/config.toml.
        #[arg(long, alias = "llm-provider")]
        enrich_provider: Option<String>,
        /// External enrichment command to run (reads explain JSON from stdin; writes markdown to stdout).
        #[arg(long)]
        enrich_cmd: Option<String>,
        /// Model name (used for provider=openai). Can also be set via SRUJA_ENRICH_MODEL.
        #[arg(long, alias = "llm-model")]
        enrich_model: Option<String>,
        /// Base URL (used for provider=openai). Can also be set via SRUJA_ENRICH_BASE_URL.
        #[arg(long, alias = "llm-base-url")]
        enrich_base_url: Option<String>,
        /// Timeout for --enrich-cmd in milliseconds (default: 15000)
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes to read from --enrich-cmd stdout (default: 20000)
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
        /// Run an incremental scan using AST caching (re-scan only modified files)
        #[arg(long, short = 'u', alias = "incremental")]
        update: bool,
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
    /// Query the architectural registry for elements and relationships
    ///
    /// Use this to find elements by name, type, or relationship pattern.
    /// For decision investigation, use `why`. For task briefing, use `focus`.
    Query {
        /// Query string (e.g., "Checkout", "depends_on Payments")
        query: String,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
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
    /// Architecture health score from structural violations (0-100)
    ///
    /// Answers: "Are there structural problems in the architecture graph?"
    /// For truth freshness, use `status`. For AI-readiness, use `context-score`.
    Health {
        /// Repository root to scan
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Path to architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },

    /// AI-readiness score (0-100): can an AI agent work effectively on this codebase?
    ///
    /// Answers: "Can AI work effectively here?"
    /// For structural health, use `health`. For truth freshness, use `status`.
    #[command(name = "context-score")]
    ContextScore {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
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
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output path for the HTML file (default: context_graph.html)
        #[arg(long, short = 'o', default_value = "context_graph.html")]
        output: String,
        /// Open the browser after generation
        #[arg(long)]
        open: bool,
    },

    /// Context briefing before starting a task: blast radius, decisions, AI instructions
    ///
    /// Use this before starting work on a file or element.
    /// For paste-ready AI brief, use `ai`. For investigation, use `why`.
    Focus {
        /// Optional run ID for tracing (defaults to auto-generated)
        #[arg(long)]
        run_id: Option<String>,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
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
        /// Optional enrichment (cmd/openai) to add a narrative plan grounded in the focus JSON.
        #[arg(long)]
        enrich: bool,
        /// Enrichment provider (cmd|openai). Can also be set via SRUJA_ENRICH_PROVIDER or .sruja/config.toml.
        #[arg(long, alias = "llm-provider")]
        enrich_provider: Option<String>,
        /// External enrichment command (reads JSON from stdin; writes markdown to stdout).
        #[arg(long)]
        enrich_cmd: Option<String>,
        /// Model name (used for provider=openai). Can also be set via SRUJA_ENRICH_MODEL.
        #[arg(long, alias = "llm-model")]
        enrich_model: Option<String>,
        /// Base URL (used for provider=openai). Can also be set via SRUJA_ENRICH_BASE_URL.
        #[arg(long, alias = "llm-base-url")]
        enrich_base_url: Option<String>,
        /// Timeout for enrichment in milliseconds (default: 15000)
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes to read from enrichment stdout (default: 20000)
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
        /// Git base ref for optional temporal context (use with `--head-ref`; if omitted, head defaults to `HEAD`)
        #[arg(long)]
        base_ref: Option<String>,
        /// Git head ref for optional temporal context (requires `--base-ref`)
        #[arg(long)]
        head_ref: Option<String>,
    },

    /// Ingest external context (ADRs, design docs, API contracts) into .sruja/context/
    Ingest {
        /// Files or directories to ingest
        sources: Vec<String>,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Category tag (adr, design-doc, api-contract, runbook, note)
        #[arg(long, short = 'c')]
        category: Option<String>,
        /// Comma-separated architecture element IDs to link (e.g. Auth.Handler,Database.Users)
        #[arg(long, short = 'e')]
        elements: Option<String>,
    },
    /// Agentic memory: learnings, guardrails, failed hypotheses (bounded to architecture work)
    Agent {
        #[command(subcommand)]
        cmd: AgentCommand,
    },
    /// Inspect and replay saved run snapshots under `.sruja/runs/`
    Run {
        #[command(subcommand)]
        cmd: RunCommand,
    },
    /// [deprecated: use `intent evaluate`] Evaluate fitness functions declared in .sruja files
    #[command(hide = true)]
    Evaluate {
        /// Path to .sruja file or directory
        #[arg(long, short = 'a', default_value = "repo.sruja")]
        architecture: String,
    },
    /// [deprecated: use `intent history`] Evolutionary history, log, and management
    #[command(hide = true)]
    Evolution {
        #[command(subcommand)]
        cmd: EvolutionCommand,
    },
}

#[derive(Subcommand)]
pub enum RunCommand {
    /// Show a saved run snapshot
    Show {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Run ID to show
        #[arg(long)]
        run_id: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum EvolutionCommand {
    /// Show evolution log/history of mutations and fitness scores
    Log {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum DiscoverCommand {
    /// Repo context summary for AI/debug (technologies, shape). For a human first read use `quickstart`.
    Context,
    /// Why the scanner inferred this graph and what to verify next. Pair with `lint` / `drift`, not a substitute for `onboard`.
    Explain,
    /// Token-oriented repomap for LLM context. Not a product brief — use `ai` or `onboard` for that.
    Repomap,
    /// Discovery question bank to drive architecture interviews or skill prompts.
    Questions,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
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
    /// Append a manual learning, failed hypothesis, or guardrail (reviewed architecture stays in `repo.sruja`)
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
    /// Show learning clusters (thematically linked groups) and tags
    Clusters {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Optional entry ID to show the cluster for
        #[arg(long, short = 'e')]
        entry_id: Option<String>,
        /// Optional tag to filter by
        #[arg(long, short = 't')]
        tag: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Architecture-bounded agent loop: observe → plan → (optional) apply → verify → record learnings
    ///
    /// Requires Sruja evidence and a reviewable plan; not a general-purpose coding agent.
    Run {
        /// Optional run ID for tracing (defaults to auto-generated)
        #[arg(long)]
        run_id: Option<String>,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Natural language goal (e.g. "Add agent loop to CLI")
        #[arg(long)]
        goal: String,
        /// File path focus (relative to repo root). Exactly one of --file/--element-id/--query is required.
        #[arg(long)]
        file: Option<String>,
        /// Architecture element ID focus. Exactly one of --file/--element-id/--query is required.
        #[arg(long)]
        element_id: Option<String>,
        /// Natural language query focus. Exactly one of --file/--element-id/--query is required.
        #[arg(long)]
        query: Option<String>,
        /// Execution mode (plan|apply). Default: plan
        #[arg(long, default_value = "plan")]
        mode: String,
        /// AI mode profile (standard|conservative|aggressive). Default: standard
        #[arg(long, default_value = "standard")]
        ai_mode: String,
        /// Output format (text|json|for-ai)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Max steps to execute in apply mode (bounded by config)
        #[arg(long)]
        max_steps: Option<usize>,
        /// Max runtime per step in milliseconds (bounded by config)
        #[arg(long)]
        max_runtime_ms_per_step: Option<u64>,
        /// Optional enrichment (cmd/openai) to add narrative grounded in gathered facts.
        #[arg(long)]
        enrich: bool,
        /// Enrichment provider (cmd|openai). Can also be set via SRUJA_ENRICH_PROVIDER or .sruja/config.toml.
        #[arg(long, alias = "llm-provider")]
        enrich_provider: Option<String>,
        /// External enrichment command (reads JSON from stdin; writes markdown to stdout).
        #[arg(long)]
        enrich_cmd: Option<String>,
        /// Model name (used for provider=openai). Can also be set via SRUJA_ENRICH_MODEL.
        #[arg(long, alias = "llm-model")]
        enrich_model: Option<String>,
        /// Base URL (used for provider=openai). Can also be set via SRUJA_ENRICH_BASE_URL.
        #[arg(long, alias = "llm-base-url")]
        enrich_base_url: Option<String>,
        /// Timeout for enrichment in milliseconds (default: 15000)
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes to read from enrichment stdout (default: 20000)
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
        /// Continue running verification even if an apply step fails
        #[arg(long)]
        continue_on_error: bool,
        /// Number of parallel sandbox trajectories for MaTTS self-contrast (minimum: 2)
        #[arg(long)]
        trajectories: Option<usize>,
    },
    /// Emit a reviewable plan JSON grounded in repo evidence (pair with `agent apply`)
    Plan {
        /// Optional run ID for tracing (defaults to auto-generated)
        #[arg(long)]
        run_id: Option<String>,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Natural language goal
        #[arg(long)]
        goal: String,
        /// File path focus (relative to repo root). Exactly one of --file/--element-id/--query is required.
        #[arg(long)]
        file: Option<String>,
        /// Architecture element ID focus. Exactly one of --file/--element-id/--query is required.
        #[arg(long)]
        element_id: Option<String>,
        /// Natural language query focus. Exactly one of --file/--element-id/--query is required.
        #[arg(long)]
        query: Option<String>,
        /// Output path for plan artifact (defaults to docs/plans/<run_id>-<slug>.json)
        #[arg(long)]
        out: Option<String>,
        /// Print the plan JSON to stdout (otherwise prints the plan file path)
        #[arg(long)]
        print: bool,
        /// AI mode profile (standard|conservative|aggressive). Default: standard
        #[arg(long, default_value = "standard")]
        ai_mode: String,
        /// Optional enrichment to add narrative grounded in gathered facts.
        #[arg(long)]
        enrich: bool,
        /// Enrichment provider (cmd|openai). Can also be set via SRUJA_ENRICH_PROVIDER or .sruja/config.toml.
        #[arg(long, alias = "llm-provider")]
        enrich_provider: Option<String>,
        /// External enrichment command (reads JSON from stdin; writes markdown to stdout).
        #[arg(long)]
        enrich_cmd: Option<String>,
        /// Model name (used for provider=openai). Can also be set via SRUJA_ENRICH_MODEL.
        #[arg(long, alias = "llm-model")]
        enrich_model: Option<String>,
        /// Base URL (used for provider=openai). Can also be set via SRUJA_ENRICH_BASE_URL.
        #[arg(long, alias = "llm-base-url")]
        enrich_base_url: Option<String>,
        /// Timeout for enrichment in milliseconds (default: 15000)
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes to read from enrichment stdout (default: 20000)
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
    },
    /// Apply a plan produced by `agent plan` only after human or CI review
    Apply {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to plan JSON created by `sruja agent plan`
        #[arg(long)]
        plan: String,
        /// Output format (json)
        #[arg(long, short = 'f', default_value = "json")]
        format: String,
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
    /// Evaluate fitness functions declared in .sruja files
    Evaluate {
        /// Path to .sruja file or directory
        #[arg(long, short = 'a', default_value = "repo.sruja")]
        architecture: String,
    },
    /// Show evolution history of mutations and fitness scores
    History {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
}

pub async fn run_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(ref rules_path) = cli.classification_rules {
        sruja_scan::set_classification_rules_path(Some(std::path::PathBuf::from(rules_path)));
    }

    let command = cli.command;
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
            reasoned,
            llmguided,
        } => commands::why(&repo, &question, &format, reasoned, llmguided).await,
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
            from_scan,
            repo,
            output_dir,
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
                    from_scan,
                    repo,
                    output_dir,
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
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
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
                enrich,
                enrich_provider.as_deref(),
                enrich_cmd.as_deref(),
                enrich_model.as_deref(),
                enrich_base_url.as_deref(),
                enrich_timeout_ms,
                enrich_max_bytes,
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
            ci,
            violations_baseline,
            baseline_mode,
        } => {
            if ci {
                let ci_format = if format == "text" {
                    "github-actions".to_string()
                } else {
                    format
                };
                commands::check(&repo, &ci_format, violations_baseline.as_deref()).await
            } else {
                commands::drift(
                    &repo,
                    architecture.as_deref(),
                    &format,
                    false,
                    violations_only,
                    fail_on.as_deref(),
                    baseline_mode.as_deref(),
                )
                .await
            }
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
        Commands::Status {
            path,
            format,
            evolution,
        } => commands::status(&path, &format, evolution).await,
        Commands::Watch { path, clear, focus } => commands::watch(&path, clear, focus).await,
        Commands::Sync { path, format } => commands::sync(&path, &format).await,
        Commands::Review {
            path,
            format,
            show_all,
            critique,
        } => commands::review(&path, &format, show_all, critique).await,
        Commands::Learn {
            path,
            file,
            since,
            skip_proposals,
            apply_proposals,
            format,
        } => {
            let skip = skip_proposals || !apply_proposals;
            commands::learn(&path, file.as_deref(), since.as_deref(), skip, &format).await
        }
        Commands::Check {
            path,
            format,
            violations_baseline,
        } => {
            eprintln!("warning: 'sruja check' is deprecated, use 'sruja drift --ci'");
            commands::check(&path, &format, violations_baseline.as_deref()).await
        }
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
            IntentCommand::Evaluate { architecture } => commands::evaluate(&architecture).await,
            IntentCommand::History { repo } => commands::evolution_log(&repo).await,
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
        Commands::Ai {
            repo,
            task,
            file,
            element_id,
            query,
            base_ref,
            head_ref,
            staged,
            max_tokens,
            output,
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
        } => {
            commands::ai_brief(commands::AiBriefOptions {
                repo: &repo,
                task: task.as_deref(),
                file: file.as_deref(),
                element_id: element_id.as_deref(),
                query: query.as_deref(),
                base_ref: base_ref.as_deref(),
                head_ref: head_ref.as_deref(),
                staged,
                max_tokens,
                output: output.as_deref(),
                enrich,
                enrich_provider: enrich_provider.as_deref(),
                enrich_cmd: enrich_cmd.as_deref(),
                enrich_model: enrich_model.as_deref(),
                enrich_base_url: enrich_base_url.as_deref(),
                enrich_timeout_ms,
                enrich_max_bytes,
            })
            .await
        }
        Commands::Onboard {
            repo,
            format,
            max_items,
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
            output,
        } => {
            commands::onboard(
                &repo,
                &format,
                max_items,
                enrich,
                enrich_provider.as_deref(),
                enrich_cmd.as_deref(),
                enrich_model.as_deref(),
                enrich_base_url.as_deref(),
                enrich_timeout_ms,
                enrich_max_bytes,
                commands::LlmConfig {
                    provider: enrich_provider.as_deref(),
                    model: enrich_model.as_deref(),
                    base_url: enrich_base_url.as_deref(),
                },
                output.as_deref(),
            )
            .await
        }
        Commands::AiContext {
            run_id,
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
            if was_invoked_as("context") {
                eprintln!("warning: 'sruja context' is deprecated, use 'sruja ai-context'");
            }
            commands::context_export(
                &repo,
                &format,
                output.as_deref(),
                commands::ContextRequest {
                    run_id: run_id.as_deref(),
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
            cmd,
            context,
            explain,
            repomap,
            repo,
            format,
            max_files,
            max_tokens,
            export_report,
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
            update,
        } => {
            // Resolve which mode to use: explicit subcommand takes priority,
            // then legacy flags (with deprecation warnings), then default.
            let effective = if let Some(ref sub) = cmd {
                sub.clone()
            } else if repomap {
                eprintln!(
                    "warning: 'discover --repomap' is deprecated, use 'sruja discover repomap'"
                );
                DiscoverCommand::Repomap
            } else if explain {
                eprintln!(
                    "warning: 'discover --explain' is deprecated, use 'sruja discover explain'"
                );
                DiscoverCommand::Explain
            } else if context {
                eprintln!(
                    "warning: 'discover --context' is deprecated, use 'sruja discover context'"
                );
                DiscoverCommand::Context
            } else {
                DiscoverCommand::Questions
            };

            match effective {
                DiscoverCommand::Repomap => {
                    commands::discover_repomap_cmd(&repo, max_files, max_tokens).await
                }
                DiscoverCommand::Explain => {
                    commands::discover_explain(
                        &repo,
                        &format,
                        export_report.as_deref(),
                        enrich,
                        enrich_provider.as_deref(),
                        enrich_cmd.as_deref(),
                        enrich_model.as_deref(),
                        enrich_base_url.as_deref(),
                        enrich_timeout_ms,
                        enrich_max_bytes,
                        update,
                    )
                    .await
                }
                DiscoverCommand::Context => {
                    if update {
                        let _ =
                            commands::scan_repo_cached_with_opts(std::path::Path::new(&repo), true);
                    }
                    commands::discover_context(&repo, &format).await
                }
                DiscoverCommand::Questions => {
                    if update {
                        let _ =
                            commands::scan_repo_cached_with_opts(std::path::Path::new(&repo), true);
                    }
                    commands::discover_questions()
                }
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
            run_id,
            repo,
            file,
            element_id,
            format,
            enrich,
            enrich_provider,
            enrich_cmd,
            enrich_model,
            enrich_base_url,
            enrich_timeout_ms,
            enrich_max_bytes,
            base_ref,
            head_ref,
        } => {
            commands::focus(
                &repo,
                file.as_deref(),
                element_id.as_deref(),
                &format,
                run_id.as_deref(),
                enrich,
                enrich_provider.as_deref(),
                enrich_cmd.as_deref(),
                enrich_model.as_deref(),
                enrich_base_url.as_deref(),
                enrich_timeout_ms,
                enrich_max_bytes,
                base_ref.as_deref(),
                head_ref.as_deref(),
            )
            .await
        }
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
            AgentCommand::Clusters {
                repo,
                entry_id,
                tag,
                format,
            } => {
                commands::agent_clusters(&repo, entry_id.as_deref(), tag.as_deref(), &format).await
            }
            AgentCommand::Run {
                run_id,
                repo,
                goal,
                file,
                element_id,
                query,
                mode,
                ai_mode,
                format,
                max_steps,
                max_runtime_ms_per_step,
                enrich,
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
                continue_on_error,
                trajectories,
            } => {
                commands::agent_run(commands::AgentRunOptions {
                    repo: &repo,
                    goal: &goal,
                    file: file.as_deref(),
                    element_id: element_id.as_deref(),
                    query: query.as_deref(),
                    mode: &mode,
                    ai_mode: &ai_mode,
                    format: &format,
                    run_id: run_id.as_deref(),
                    max_steps,
                    max_runtime_ms_per_step,
                    enrich,
                    enrich_provider: enrich_provider.as_deref(),
                    enrich_cmd: enrich_cmd.as_deref(),
                    enrich_model: enrich_model.as_deref(),
                    enrich_base_url: enrich_base_url.as_deref(),
                    enrich_timeout_ms,
                    enrich_max_bytes,
                    continue_on_error,
                    trajectories,
                })
                .await
            }
            AgentCommand::Plan {
                run_id,
                repo,
                goal,
                file,
                element_id,
                query,
                out,
                print,
                ai_mode,
                enrich,
                enrich_provider,
                enrich_cmd,
                enrich_model,
                enrich_base_url,
                enrich_timeout_ms,
                enrich_max_bytes,
            } => {
                let out_path = out.as_deref().map(std::path::Path::new);
                commands::agent_plan(
                    commands::AgentRunOptions {
                        repo: &repo,
                        goal: &goal,
                        file: file.as_deref(),
                        element_id: element_id.as_deref(),
                        query: query.as_deref(),
                        mode: "plan",
                        ai_mode: &ai_mode,
                        format: "json",
                        run_id: run_id.as_deref(),
                        max_steps: None,
                        max_runtime_ms_per_step: None,
                        enrich,
                        enrich_provider: enrich_provider.as_deref(),
                        enrich_cmd: enrich_cmd.as_deref(),
                        enrich_model: enrich_model.as_deref(),
                        enrich_base_url: enrich_base_url.as_deref(),
                        enrich_timeout_ms,
                        enrich_max_bytes,
                        continue_on_error: false,
                        trajectories: None,
                    },
                    out_path,
                    print,
                )
                .await
            }
            AgentCommand::Apply { repo, plan, format } => {
                commands::agent_apply(std::path::Path::new(&plan), &repo, &format).await
            }
        },
        Commands::Run { cmd } => match cmd {
            RunCommand::Show {
                repo,
                run_id,
                format,
            } => commands::run_show(&repo, &run_id, &format).await,
        },
        Commands::Evaluate { architecture } => {
            eprintln!("warning: 'sruja evaluate' is deprecated, use 'sruja intent evaluate'");
            commands::evaluate(&architecture).await
        }
        Commands::Evolution { cmd } => {
            eprintln!("warning: 'sruja evolution' is deprecated, use 'sruja intent history'");
            match cmd {
                EvolutionCommand::Log { repo } => commands::evolution_log(&repo).await,
            }
        }
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
    fn parses_ai_context_defaults() {
        std::thread::Builder::new()
            .name("clap_parse_large_stack".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let cli = Cli::try_parse_from(["sruja", "ai-context"]).expect("parse");
                match cli.command {
                    Commands::AiContext {
                        run_id: _,
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
                    _ => panic!("expected AiContext command"),
                }
            })
            .expect("spawn")
            .join()
            .expect("join");
    }

    #[test]
    fn parses_context_alias() {
        std::thread::Builder::new()
            .name("clap_parse_large_stack".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let cli = Cli::try_parse_from(["sruja", "context"]).expect("parse via alias");
                assert!(matches!(cli.command, Commands::AiContext { .. }));
            })
            .expect("spawn")
            .join()
            .expect("join");
    }

    #[test]
    fn parses_discover_subcommands() {
        std::thread::Builder::new()
            .name("clap_parse_large_stack".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let cli = Cli::try_parse_from(["sruja", "discover", "explain"]).expect("parse");
                match cli.command {
                    Commands::Discover { cmd, .. } => {
                        assert!(matches!(cmd, Some(DiscoverCommand::Explain)));
                    }
                    _ => panic!("expected Discover command"),
                }

                let cli2 = Cli::try_parse_from(["sruja", "discover", "repomap"]).expect("parse");
                match cli2.command {
                    Commands::Discover { cmd, .. } => {
                        assert!(matches!(cmd, Some(DiscoverCommand::Repomap)));
                    }
                    _ => panic!("expected Discover command"),
                }

                let cli3 = Cli::try_parse_from(["sruja", "discover"]).expect("parse bare");
                match cli3.command {
                    Commands::Discover { cmd, .. } => {
                        assert!(
                            cmd.is_none(),
                            "bare discover should have no subcommand (defaults to questions)"
                        );
                    }
                    _ => panic!("expected Discover command"),
                }

                let cli4 = Cli::try_parse_from(["sruja", "discover", "questions"])
                    .expect("parse questions");
                match cli4.command {
                    Commands::Discover { cmd, .. } => {
                        assert!(matches!(cmd, Some(DiscoverCommand::Questions)));
                    }
                    _ => panic!("expected Discover command"),
                }

                let cli5 =
                    Cli::try_parse_from(["sruja", "discover", "context"]).expect("parse context");
                match cli5.command {
                    Commands::Discover { cmd, .. } => {
                        assert!(matches!(cmd, Some(DiscoverCommand::Context)));
                    }
                    _ => panic!("expected Discover command"),
                }
            })
            .expect("spawn")
            .join()
            .expect("join");
    }

    #[test]
    fn parses_drift_ci_flag() {
        std::thread::Builder::new()
            .name("clap_parse_large_stack".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let cli = Cli::try_parse_from(["sruja", "drift", "--ci"]).expect("parse");
                match cli.command {
                    Commands::Drift { ci, .. } => assert!(ci),
                    _ => panic!("expected Drift command"),
                }

                let cli2 = Cli::try_parse_from(["sruja", "drift"]).expect("parse");
                match cli2.command {
                    Commands::Drift { ci, .. } => assert!(!ci),
                    _ => panic!("expected Drift command"),
                }
            })
            .expect("spawn")
            .join()
            .expect("join");
    }

    #[test]
    fn parses_intent_evaluate_and_history() {
        std::thread::Builder::new()
            .name("clap_parse_large_stack".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let cli = Cli::try_parse_from(["sruja", "intent", "evaluate"]).expect("parse");
                match cli.command {
                    Commands::Intent { cmd } => {
                        assert!(matches!(cmd, IntentCommand::Evaluate { .. }));
                    }
                    _ => panic!("expected Intent command"),
                }

                let cli2 = Cli::try_parse_from(["sruja", "intent", "history"]).expect("parse");
                match cli2.command {
                    Commands::Intent { cmd } => {
                        assert!(matches!(cmd, IntentCommand::History { .. }));
                    }
                    _ => panic!("expected Intent command"),
                }
            })
            .expect("spawn")
            .join()
            .expect("join");
    }

    #[test]
    fn parses_ai_brief_defaults() {
        std::thread::Builder::new()
            .name("clap_parse_large_stack".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let cli = Cli::try_parse_from(["sruja", "ai"]).expect("parse");
                match cli.command {
                    Commands::Ai {
                        repo,
                        task,
                        file,
                        element_id,
                        query,
                        base_ref,
                        head_ref,
                        staged,
                        max_tokens,
                        output,
                        enrich,
                        enrich_provider,
                        enrich_cmd,
                        enrich_model,
                        enrich_base_url,
                        enrich_timeout_ms,
                        enrich_max_bytes,
                    } => {
                        assert_eq!(repo, ".");
                        assert!(task.is_none());
                        assert!(file.is_none());
                        assert!(element_id.is_none());
                        assert!(query.is_none());
                        assert!(base_ref.is_none());
                        assert!(head_ref.is_none());
                        assert!(!staged);
                        assert_eq!(max_tokens, 8000);
                        assert!(output.is_none());
                        assert!(!enrich);
                        assert!(enrich_provider.is_none());
                        assert!(enrich_cmd.is_none());
                        assert!(enrich_model.is_none());
                        assert!(enrich_base_url.is_none());
                        assert_eq!(enrich_timeout_ms, 15000);
                        assert_eq!(enrich_max_bytes, 20000);
                    }
                    _ => panic!("expected Ai command"),
                }
            })
            .expect("spawn")
            .join()
            .expect("join");
    }

    #[test]
    fn parses_ai_brief_focus_options() {
        // Clap can use substantial stack for deeply-nested subcommands/args.
        // Keep this test stable by running parse on a larger stack.
        std::thread::Builder::new()
            .name("clap_parse_large_stack".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let cli = Cli::try_parse_from([
                    "sruja",
                    "ai",
                    "--task",
                    "Fix parser diagnostics",
                    "--file",
                    "crates/sruja-language/src/parser/mod.rs",
                    "--element-id",
                    "Sruja.Language",
                    "--query",
                    "parser",
                    "--base-ref",
                    "main",
                    "--head-ref",
                    "HEAD",
                    "--staged",
                    "--max-tokens",
                    "12000",
                    "-o",
                    "brief.md",
                ])
                .expect("parse");
                match cli.command {
                    Commands::Ai {
                        task,
                        file,
                        element_id,
                        query,
                        base_ref,
                        head_ref,
                        staged,
                        max_tokens,
                        output,
                        ..
                    } => {
                        assert_eq!(task.as_deref(), Some("Fix parser diagnostics"));
                        assert_eq!(
                            file.as_deref(),
                            Some("crates/sruja-language/src/parser/mod.rs")
                        );
                        assert_eq!(element_id.as_deref(), Some("Sruja.Language"));
                        assert_eq!(query.as_deref(), Some("parser"));
                        assert_eq!(base_ref.as_deref(), Some("main"));
                        assert_eq!(head_ref.as_deref(), Some("HEAD"));
                        assert!(staged);
                        assert_eq!(max_tokens, 12000);
                        assert_eq!(output.as_deref(), Some("brief.md"));
                    }
                    _ => panic!("expected Ai command"),
                }
            })
            .expect("spawn")
            .join()
            .expect("join");
    }

    #[test]
    fn parses_agent_run_defaults() {
        std::thread::Builder::new()
            .name("clap_parse_large_stack".to_string())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let cli = Cli::try_parse_from([
                    "sruja",
                    "agent",
                    "run",
                    "--goal",
                    "Add agent loop",
                    "--file",
                    "crates/sruja-cli/src/cli.rs",
                ])
                .expect("parse");
                match cli.command {
                    Commands::Agent { cmd } => match cmd {
                        AgentCommand::Run {
                            repo,
                            goal,
                            file,
                            element_id,
                            query,
                            mode,
                            ai_mode,
                            format,
                            max_steps,
                            max_runtime_ms_per_step,
                            enrich,
                            continue_on_error,
                            ..
                        } => {
                            assert_eq!(repo, ".");
                            assert_eq!(goal, "Add agent loop");
                            assert_eq!(file.as_deref(), Some("crates/sruja-cli/src/cli.rs"));
                            assert!(element_id.is_none());
                            assert!(query.is_none());
                            assert_eq!(mode, "plan");
                            assert_eq!(ai_mode, "standard");
                            assert_eq!(format, "text");
                            assert!(max_steps.is_none());
                            assert!(max_runtime_ms_per_step.is_none());
                            assert!(!enrich);
                            assert!(!continue_on_error);
                        }
                        _ => panic!("expected Agent run subcommand"),
                    },
                    _ => panic!("expected Agent command"),
                }
            })
            .expect("spawn")
            .join()
            .expect("join");
    }
}

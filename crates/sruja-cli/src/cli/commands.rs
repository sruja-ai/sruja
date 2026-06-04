use clap::Subcommand;

use super::app::ContextIntent;
use super::subcommands::{
    AgentCommand, AidlcCommand, AuthorCommand, DecisionCommand, DiscoverCommand, DslCommand,
    EventCommand, EvolutionCommand, FederationCommand, GraphCommand, GuardCommand, HumanCommand,
    IndexCommand, InspectCommand, IntentCommand, MemoryCommand, ProposeCommand, RunCommand,
    WorkflowCommand,
};
use crate::enrichment::EnrichmentArgs;

#[derive(Subcommand)]
pub enum Commands {
    /// Print version information
    Version,
    /// Workflow manifest + phase gates (Inception → Construction → Operations)
    Workflow {
        #[command(subcommand)]
        cmd: WorkflowCommand,
    },
    /// AI-DLC workflow: inception → construction → operations with phase gates
    ///
    /// Simplified entry point for AI-DLC users. Wraps `sruja workflow` with
    /// AI-DLC defaults pre-filled (--with-aidlc, --install-aidlc-rules).
    Aidlc {
        #[command(subcommand)]
        cmd: AidlcCommand,
    },
    /// Propose architectural changes for review
    #[command(hide = true)]
    Propose {
        #[command(subcommand)]
        cmd: ProposeCommand,
    },
    /// Grounded architecture authoring helpers (evidence bundle + proposal synthesis)
    Author {
        #[command(subcommand)]
        cmd: AuthorCommand,
    },
    /// Scan a repository and infer an architecture graph
    #[command(hide = true)]
    Scan {
        /// Repository root (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Output path for inferred graph JSON (use "-" for stdout)
        #[arg(long, default_value = "sruja.graph.json")]
        output: String,
    },
    /// Adversarial architectural critique of proposed changes (review-oriented, not context-oriented)
    #[command(hide = true)]
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
        /// Optional LLM enrichment to add a narrative review grounded in the critique report.
        #[command(flatten)]
        enrich: EnrichmentArgs,
        /// Fail the command (exit 1) if findings of this level or higher are found
        #[arg(long)]
        fail_on: Option<String>,
    },
    /// Impact analysis: blast radius (upstream dependents + downstream dependencies)
    #[command(hide = true)]
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
    #[command(hide = true)]
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
    #[command(hide = true)]
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
    #[command(hide = true)]
    Fmt {
        /// Path to .sruja file
        file: String,
        /// Check if file would be reformatted (CI mode, exits with error if changes needed)
        #[arg(long)]
        check: bool,
    },
    /// List elements from a file
    #[command(hide = true)]
    List {
        /// Path to .sruja file
        file: String,
    },
    /// Print architecture tree
    #[command(hide = true)]
    Tree {
        /// Path to .sruja file
        file: String,
    },
    /// Show differences between two architecture files
    #[command(hide = true)]
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
    #[command(hide = true)]
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
    #[command(hide = true)]
    Import {
        /// Format (json)
        format: String,
        /// File to import
        file: String,
    },
    /// Start LSP server (stdio)
    #[command(hide = true)]
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
    /// - `SRUJA_MCP_WATCH_DRIFT=1` — emit `notifications/drift_state` after MCP initialize.
    Mcp {
        /// Default repository root used when MCP tool calls omit a path
        #[arg(long, short = 'r', default_value = ".")]
        root: String,
    },
    /// Compile a Sruja file
    #[command(hide = true)]
    Compile {
        /// Path to .sruja file
        file: String,
    },
    /// Validate architecture against rules
    #[command(hide = true)]
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
        /// Compare scan only (ignore repo.sruja even if present)
        #[arg(long)]
        structural_only: bool,
        /// First-run friendly: always print scan summary; omit orphan info findings
        #[arg(long)]
        advisory: bool,
        /// Exclude barrel files (mod.rs, __init__.py, index.ts) from orphan and god-module checks
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        exclude_barrel_files: bool,
    },
    /// Structured drift payload for AI host injection (`drift_state/v1` JSON)
    #[command(name = "drift-state")]
    DriftState {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    /// PR-scoped drift: detect only NEW violations in a PR
    #[command(hide = true)]
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
    /// First look: structural overview and optional repo.sruja.draft (evidence, not reviewed truth)
    ///
    /// Use when asking "What is in this repo?" For a full-repo brief, use `onboard`. For an AI task brief, use `ai`.
    #[command(visible_alias = "overview", hide = true)]
    Quickstart {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Write repo.sruja.draft (workspace map evidence; not reviewed architecture)
        #[arg(long)]
        generate_baseline: bool,
        /// Fail with exit code 1 if specified violations found (comma-separated: cycles,layer-violations,god-modules,orphans,all)
        #[arg(long)]
        fail_on: Option<String>,
        /// First-run friendly: omit orphan info findings (same as `drift --advisory`)
        #[arg(long)]
        advisory: bool,
    },
    /// Set up Sruja in a repo: create .sruja/, .srujaignore, and optional repo.sruja.draft
    ///
    /// OSS hero: `sruja start -r .` then `sruja drift -r . --structural-only --advisory`.
    /// For full briefing, use `onboard` (hidden). For task briefs, use `focus` or `ai`.
    ///
    /// Quickstart: `sruja init --scan -r .` scans the repo, generates repo.sruja, shows
    /// architecture visualization, health score, and syncs IDE rules — all in under 60 seconds.
    #[command(visible_alias = "start")]
    Init {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Generate .sruja/init_prompt.txt for use with sruja-architecture skill
        #[arg(long, group = "init_mode")]
        prompt: bool,
        /// Scan workspace and write repo.sruja.draft (structural evidence; author repo.sruja separately)
        #[arg(long, short = 'a', group = "init_mode")]
        auto: bool,
        /// Full scan: generate repo.sruja, show architecture visualization, health score, and sync IDE rules
        #[arg(long, short = 's', group = "init_mode")]
        scan: bool,
        /// Overwrite repo.sruja if it already exists (only meaningful with --auto or --scan)
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
        /// Schema to use (architecture, compliance, business_process, knowledge)
        #[arg(long, default_value = "architecture")]
        schema: String,
        /// Sync IDE rules after scan (.cursorrules, copilot-instructions.md, llms-architecture.txt)
        #[arg(long)]
        sync_rules: bool,
    },
    /// Show current density tier and progression hints
    Density {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Truth freshness and baseline state
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
    #[command(hide = true)]
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
    /// Write editor rule files from validated architecture (cursor-rules, copilot, llms-architecture.txt)
    ///
    /// In `--check` mode, exits non-zero if on-disk IDE files differ from architecture-derived outputs.
    #[command(name = "sync-ide-rules")]
    SyncIdeRules {
        /// Path to repository root (single repo only)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Max tokens for generated rule bodies (approximate; matches `ai-context` default)
        #[arg(long, default_value_t = 10000)]
        max_tokens: usize,
        /// Exit non-zero if on-disk IDE files differ from architecture-derived outputs
        #[arg(long)]
        check: bool,
    },
    /// Generate .sruja/classification.json for a repository
    ///
    /// Scans the repository structure and produces a classification that describes
    /// the logical layers, boundaries, and forbidden patterns.
    /// Edit the generated file to customize, then run `sruja sync-ide-rules` to update IDE context.
    #[command(name = "classify")]
    Classify {
        /// Path to repository root (single repo only)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Overwrite existing classification.json
        #[arg(long)]
        force: bool,
    },
    /// Daily action list: refresh evidence, detect drift, suggest next steps (alias: `daily`)
    #[command(visible_alias = "daily", hide = true)]
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
    #[command(hide = true)]
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
    #[command(hide = true)]
    Baseline {
        /// Path to repository root (defaults to current directory)
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output path (default: .sruja/violations.baseline.json)
        #[arg(long, short = 'o', default_value = ".sruja/violations.baseline.json")]
        output: String,
    },
    /// Publish repo truth + evidence to repo.bundle.json (multi-repo federation)
    #[command(hide = true)]
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
    #[command(hide = true)]
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
    #[command(hide = true)]
    Intent {
        #[command(subcommand)]
        cmd: IntentCommand,
    },
    /// Structural drift + intent + policy gate (exit 1 when non-compliant)
    ///
    /// Use as an architecture aggregate gate—not a generic enterprise audit unless policies are concrete.
    #[command(hide = true)]
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
    #[command(hide = true)]
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

        /// Optional LLM enrichment to add a narrative plan grounded in the task context.
        #[command(flatten)]
        enrich: EnrichmentArgs,
    },
    /// Complete architecture brief for human or AI reader
    ///
    /// Produces a full-repo onboarding brief with trust signals: truth status, drift counts, context score.
    /// For task-specific AI briefs, use `ai`. For quick structural overview, use `quickstart`.
    #[command(hide = true)]
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
        /// Optional LLM enrichment to add a narrative section; never changes grounded scan output.
        #[command(flatten)]
        enrich: EnrichmentArgs,
        /// Output file (defaults to stdout)
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
    /// Structured architecture context for AI editor integration (Cursor, Copilot, Claude)
    ///
    /// Use MCP tools inside your AI editor for the best experience.
    /// For CLI-based briefing, use `focus` or `ai`.
    #[command(name = "ai-context", alias = "context", hide = true)]
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
        /// For `-f for-ai`: emit invariant/tools/volatile blocks for prompt-cache-friendly payloads
        #[arg(long)]
        cache_friendly: bool,
    },
    /// Scanner introspection for AI/debug: explain scan, repomap, discovery questions
    #[command(hide = true)]
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
        /// Optional LLM enrichment to add a narrative section to explain; never changes grounded scan output.
        #[command(flatten)]
        enrich: EnrichmentArgs,
        /// Run an incremental scan using AST caching (re-scan only modified files)
        #[arg(long, short = 'u', alias = "incremental")]
        update: bool,
    },
    /// Generate a prompt (skill + repo context) for use with any LLM to produce architecture.sruja without Cursor CLI
    #[command(hide = true)]
    Generate {
        /// Path to repository
        #[arg(long, short = 'r', action = clap::ArgAction::Append)]
        repo: Vec<String>,
        /// Path to skill file (SKILL.md); else SRUJA_SKILL_PATH or ./SKILL.md or ./skills/sruja-architecture/SKILL.md
        #[arg(long)]
        skill_path: Option<String>,
        /// Emit prompt only (no LLM call); write to -o or stdout
        #[arg(long, required = true)]
        prompt_only: bool,
        /// Output path for prompt (default: stdout if --prompt-only)
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
    /// Generate indices for architectural nodes
    #[command(hide = true)]
    Index {
        #[command(subcommand)]
        cmd: IndexCommand,
    },
    /// Query the architectural registry for elements and relationships
    ///
    /// Use this to find elements by name, type, or relationship pattern.
    /// For decision investigation, use `why`. For task briefing, use `focus`.
    #[command(hide = true)]
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
    #[command(hide = true)]
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
    #[command(name = "context-score", hide = true)]
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
    /// Generate the Architecture Explorer model (JSON) for the VS Code webview
    #[command(name = "explore")]
    Explore {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
    },

    /// Generate an interactive HTML/D3.js visualization of the architecture context
    #[command(name = "context-graph", hide = true)]
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
        /// Optional LLM enrichment to add a narrative plan grounded in the focus JSON.
        #[command(flatten)]
        enrich: EnrichmentArgs,
        /// Git base ref for optional temporal context (use with `--head-ref`; if omitted, head defaults to `HEAD`)
        #[arg(long)]
        base_ref: Option<String>,
        /// Git head ref for optional temporal context (requires `--base-ref`)
        #[arg(long)]
        head_ref: Option<String>,
        /// Only output active drift, failed learnings/guardrails, and boundary violations. Skip topology and enrichment.
        #[arg(long)]
        compact: bool,
    },

    /// Ingest external context (ADRs, design docs, API contracts) into .sruja/context/
    #[command(hide = true)]
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
    /// Append-only context lineage (intent, drift, proposals, decision traces)
    #[command(hide = true)]
    Event {
        #[command(subcommand)]
        cmd: EventCommand,
    },
    /// Indexed cross-session memory (SQLite + FTS5 under `.sruja/memory.sqlite`)
    #[command(hide = true)]
    Memory {
        #[command(subcommand)]
        cmd: MemoryCommand,
    },
    /// Decision Records (generalized ADRs) under `.sruja/decisions/`
    #[command(hide = true)]
    Decision {
        #[command(subcommand)]
        cmd: DecisionCommand,
    },
    /// Graph temporal queries (history, velocity, etc.)
    Graph {
        #[command(subcommand)]
        cmd: GraphCommand,
    },
    /// List and filter requirements from .sruja files
    Requirements {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Filter by priority (must, should, could, wont)
        #[arg(long)]
        priority: Option<String>,
        /// Filter by status (proposed, accepted, deprecated)
        #[arg(long)]
        status: Option<String>,
    },
    /// Agentic memory: learnings, guardrails, failed hypotheses (bounded to architecture work)
    Agent {
        #[command(subcommand)]
        cmd: AgentCommand,
    },
    /// Run verification steps for a task profile (coding, bugfix, review, arch)
    VerifyTask {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Verification profile: coding, bugfix, review, arch
        #[arg(long, short = 'p', default_value = "coding")]
        profile: String,
        /// File path focus (used by bugfix profile)
        #[arg(long)]
        file: Option<String>,
        /// Max runtime per step in milliseconds (default: 30000)
        #[arg(long)]
        max_runtime_ms: Option<u64>,
        /// Write an evidence pack folder under `.sruja/evidence-packs/<timestamp>/`.
        ///
        /// Use `--evidence-pack-dir` to override the output location.
        #[arg(long)]
        evidence_pack: bool,
        /// Override evidence pack output directory (implies `--evidence-pack`).
        #[arg(long)]
        evidence_pack_dir: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Post-AI-edit confidence report: what changed, what evidence was checked, what risks remain
    ///
    /// Advisory by default — exits successfully even if the report contains blockers.
    /// Only exits non-zero for fatal execution/input errors.
    Confidence {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Verification profile: review, coding, bugfix, arch
        #[arg(long, short = 'p', default_value = "review")]
        profile: String,
        /// File path focus (required for bugfix profile)
        #[arg(long)]
        file: Option<String>,
        /// Max runtime per step in milliseconds (default: 30000)
        #[arg(long)]
        max_runtime_ms: Option<u64>,
        /// Write an evidence pack folder under `.sruja/evidence-packs/<timestamp>/`.
        #[arg(long)]
        evidence_pack: bool,
        /// Override evidence pack output directory (implies `--evidence-pack`).
        #[arg(long)]
        evidence_pack_dir: Option<String>,
        /// Output format: md (default), text, json
        #[arg(long, short = 'f', default_value = "md")]
        format: String,
    },
    /// Inspect and replay saved run snapshots under `.sruja/runs/`
    #[command(hide = true)]
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
    /// DSL authoring tools: list, tree, explain, diff, import, compile, validate, generate
    Dsl {
        #[command(subcommand)]
        cmd: DslCommand,
    },
    /// Analysis & scores: health, impact, why, query, scores, onboard, watch, learn
    Inspect {
        #[command(subcommand)]
        cmd: InspectCommand,
    },
    /// Review & compliance gates: critique, compliance, baseline, drift-pr
    Guard {
        #[command(subcommand)]
        cmd: GuardCommand,
    },
    /// Multi-repo federation: publish, compose
    Federation {
        #[command(subcommand)]
        cmd: FederationCommand,
    },
    /// Human-centric system intelligence: trace, explain, map, before, daily, what-if
    Human {
        #[command(subcommand)]
        cmd: HumanCommand,
    },
}

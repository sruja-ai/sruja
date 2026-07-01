use clap::Subcommand;

use super::app::ContextIntent;
use super::subcommands::{
    AgentCommand, AidlcCommand, AuthorCommand, DecisionCommand, DiscoverCommand, DslCommand,
    EvalCommand, EventCommand, FederationCommand, GraphCommand, GuardCommand, HumanCommand,
    IndexCommand, InspectCommand, IntentCommand, MemoryCommand, ProposeCommand, RunCommand,
    WorkflowCommand,
};
use crate::enrichment::EnrichmentArgs;

#[derive(Subcommand)]
pub enum Commands {
    /// Print version information
    Version,
    /// Format a Sruja file in-place
    #[command(hide = true, name = "fmt")]
    Fmt {
        /// Check if file is already formatted (exit non-zero if changes needed)
        #[arg(long)]
        check: bool,
        /// Path to .sruja file
        file: String,
    },
    /// Export a Sruja file to another format
    #[command(hide = true, name = "export")]
    Export {
        /// Export format (json, mermaid)
        format: String,
        /// Path to .sruja file
        file: String,
        /// Export from a repository scan graph instead of a DSL file
        #[arg(long)]
        from_scan: bool,
        /// Path to repository root (used with --from-scan)
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output directory (used by some formats like obsidian; used with --from-scan)
        #[arg(long = "output-dir")]
        output_dir: Option<String>,
    },
    /// List elements from a Sruja file
    #[command(hide = true, name = "list")]
    List {
        /// Path to .sruja file
        file: String,
    },
    /// Print an architecture tree from a Sruja file
    #[command(hide = true, name = "tree")]
    Tree {
        /// Path to .sruja file
        file: String,
    },
    /// Show differences between two Sruja files
    #[command(hide = true, name = "diff")]
    Diff {
        /// First file
        file1: String,
        /// Second file
        file2: String,
        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Explain an element from a Sruja file
    #[command(hide = true, name = "explain")]
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
    /// Workflow manifest + phase gates (Inception → Construction → Operations)
    #[command(hide = true)]
    Workflow {
        #[command(subcommand)]
        cmd: WorkflowCommand,
    },
    /// AI-DLC workflow: inception → construction → operations with phase gates
    ///
    /// Simplified entry point for AI-DLC users. Wraps `sruja workflow` with
    /// AI-DLC defaults pre-filled (--with-aidlc, --install-aidlc-rules).
    /// Prefer `sruja workflow init --with-aidlc` for new projects.
    #[command(hide = true)]
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
    #[command(hide = true)]
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
    /// Investigate a question with deterministic evidence
    ///
    /// Use this for deep investigation. For the core workflow, prefer `focus` and `verify-task`.
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
    #[command(alias = "validate")]
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
    /// Start LSP server (stdio)

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

        /// Use the new rmcp-based server implementation (v2)
        #[arg(long, hide = true)]
        v2: bool,
    },

    /// Drift and structural checks (from code, optional baseline)
    ///
    /// Compares the repo's current structure against itself (structural-only) or against reviewed intent (`repo.sruja`).
    /// Use this as the primary structural verification step in the core loop.
    ///
    /// Examples:
    /// - `sruja drift -r . --structural-only --advisory`
    /// - `sruja drift -r . -a repo.sruja`
    /// - `sruja drift --ci -r . --baseline .sruja/violations.baseline.json -f github-actions`
    #[command(name = "drift", alias = "check")]
    Check {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Path to .sruja architecture file (optional)
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output format (text, json, github-actions, drift-state)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Only show violations, not suggestions
        #[arg(long)]
        violations_only: bool,
        /// Fail with exit code 1 if specified violations found (comma-separated: cycles,layer-violations,god-modules,orphans,all)
        #[arg(long)]
        fail_on: Option<String>,
        /// CI mode: defaults format to github-actions
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
        /// PR-scoped: detect only NEW violations (diff between base and head refs)
        #[arg(long)]
        pr: bool,
        /// Base ref for PR mode (e.g. main, origin/main)
        #[arg(long, short = 'B')]
        base: Option<String>,
        /// Head ref for PR mode (defaults to HEAD)
        #[arg(long, short = 'H')]
        head: Option<String>,
        /// Full compliance gate: structural drift + intent + policy violations
        #[arg(long)]
        compliance: bool,
        /// Path to intent directory (ADRs, .sruja files) for compliance mode
        #[arg(long, short = 'i')]
        intent: Option<String>,
        /// Strict mode: fail if unproposed changes are detected (compliance mode)
        #[arg(long)]
        strict: bool,
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
    /// Set up Sruja in a repo (.sruja/ and initial evidence)
    ///
    /// Recommended: `sruja start -r .` then `sruja drift -r . --structural-only --advisory`.
    #[command(name = "start", alias = "init")]
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
        /// Full scan: generate repo.sruja, show outputs, and sync IDE rules
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
    ///
    /// Focused density view. For a unified view with health, AI readiness, and more, use `status`.
    #[command(hide = true)]
    Density {
        /// Repository root (defaults to current directory)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Unified repo status: truth freshness, structural health, AI readiness, density, agent memory
    ///
    /// Single entry point for all repo health metrics. Includes health score, context score,
    /// density tier, and agent memory status. Use `status --health` or `status --ai-score`
    /// for focused metric views.
    #[command(visible_alias = "doctor", hide = true)]
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

    /// Refresh evidence files for context retrieval and reviewed intent workflows
    #[command(hide = true)]
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
    #[command(hide = true)]
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
    #[command(hide = true)]
    Classify {
        /// Path to repository root (single repo only)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Overwrite existing classification.json
        #[arg(long)]
        force: bool,
    },
    /// Generate a prompt for AI to extract procedural knowledge and create a project skill
    ///
    /// Collects sruja evidence (classification, context, graph) and formats it into a prompt
    /// that AI agents can use to extract procedural knowledge and generate a project-specific skill.
    #[command(name = "generate-skill")]
    #[command(hide = true)]
    GenerateSkill {
        /// Path to repository root (single repo only)
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output file for the prompt (default: stdout)
        #[arg(long, short = 'o')]
        output: Option<String>,
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
    /// Compare declared intent (decisions, ADRs) vs actual implementation
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
    /// Complete architecture brief for human or AI reader
    ///
    /// Produces a full-repo onboarding brief with trust signals: truth status, drift counts, context score.
    /// For task-specific AI briefs, use `focus`. For quick structural overview, use `sruja inspect quickstart`.
    /// Canonical path: `sruja inspect onboard`.
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
    #[command(name = "ai-context", hide = true)]
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
    /// Focused structural score. For a unified view with truth freshness, AI readiness, and more, use `status`.
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
    /// Focused AI-readiness score. For a unified view with truth freshness, structural health, and more, use `status`.
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
    #[command(hide = true)]
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

    /// Retrieve task context before editing
    ///
    /// Generates task-scoped or file-scoped context for AI editors and developers.
    /// Combines impact, linked decisions, constraints, and relevant evidence into a single briefing.
    ///
    /// Use `sruja focus -r . --file <path>` before editing a file.
    /// Use `sruja focus -r . --task "description"` for paste-ready AI briefs.
    /// Use `sruja focus -r . --format for-ai` for prompt-cache-friendly payloads.
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
        /// Natural-language task or intent to give the AI coding assistant
        #[arg(long, short = 't')]
        task: Option<String>,
        /// Natural-language query to find relevant architecture context
        #[arg(long)]
        query: Option<String>,
        /// Output format (text, json, for-ai, markdown)
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
        /// Use staged changes instead of all changes against HEAD for changed-file detection
        #[arg(long)]
        staged: bool,
        /// Max tokens for the embedded task context
        #[arg(long, default_value_t = 8000)]
        max_tokens: usize,
        /// Output file (defaults to stdout)
        #[arg(long, short = 'o')]
        output: Option<String>,
        /// For --format for-ai: emit invariant/tools/volatile blocks for prompt-cache-friendly payloads
        #[arg(long)]
        cache_friendly: bool,
    },
    /// Paste-ready AI coding brief (includes task context section)
    #[command(name = "ai")]
    Ai {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Natural-language task or intent for the AI assistant
        #[arg(long, short = 't')]
        task: Option<String>,
        /// File path focus (relative to repo root)
        #[arg(long)]
        file: Option<String>,
        /// Architecture element ID focus
        #[arg(long)]
        element_id: Option<String>,
        /// Natural-language query for context retrieval
        #[arg(long)]
        query: Option<String>,
        /// Git base ref for optional temporal context (use with --head-ref)
        #[arg(long)]
        base_ref: Option<String>,
        /// Git head ref for optional temporal context (requires --base-ref)
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
        /// Optional LLM enrichment to add a narrative plan grounded in the task context JSON.
        #[command(flatten)]
        enrich: EnrichmentArgs,
    },

    /// Ingest external context into `.sruja/context/`
    ///
    /// Supports files or directories; ingested docs are surfaced by `focus` and improve context retrieval.
    Ingest {
        /// Files or directories to ingest
        sources: Vec<String>,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Category tag
        #[arg(long, short = 'c')]
        category: Option<String>,
        /// Comma-separated architecture element IDs
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
    Decision {
        #[command(subcommand)]
        cmd: DecisionCommand,
    },
    /// Graph temporal queries (history, velocity, etc.)
    #[command(hide = true)]
    Graph {
        #[command(subcommand)]
        cmd: GraphCommand,
    },
    /// List and filter requirements from .sruja files
    #[command(hide = true)]
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
    #[command(hide = true)]
    Agent {
        #[command(subcommand)]
        cmd: AgentCommand,
    },
    /// Autonomous execution loop: comprehend → plan → execute → verify → learn.
    ///
    /// Example: sruja auto "add health check endpoint"
    Auto {
        /// What to do (e.g. "add health check endpoint")
        goal: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Maximum plan->execute->verify iterations (default: 3)
        #[arg(long)]
        max_iterations: Option<usize>,
        /// Plan preview only, no mutations
        #[arg(long)]
        dry_run: bool,
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
        /// Execute from saved pipeline YAML, or generate one
        #[arg(long)]
        pipeline: Option<String>,
        /// Resume from last checkpoint
        #[arg(long)]
        resume: bool,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Understand scope and produce a reviewable plan.
    ///
    /// Example: sruja plan "what does adding auth affect"
    Plan {
        /// What to understand (e.g. "what does adding auth affect")
        goal: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// File path focus (narrow scope to a specific file)
        #[arg(long)]
        file: Option<String>,
        /// Architecture element ID focus
        #[arg(long)]
        element_id: Option<String>,
        /// Natural language query focus
        #[arg(long)]
        query: Option<String>,
        /// Also emit editable pipeline YAML
        #[arg(long)]
        pipeline: bool,
        /// Save plan JSON to path
        #[arg(long)]
        output: Option<String>,
        /// JSON output (machine-readable)
        #[arg(long)]
        json: bool,
        /// Compact summary (no inline evidence)
        #[arg(long)]
        compact: bool,
    },
    /// Check architecture health: drift + lint + intent + confidence.
    ///
    /// Example: sruja verify
    Verify {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Verification profile (full, coding, bugfix, review, arch)
        #[arg(long, short = 'p', default_value = "full")]
        profile: String,
        /// File path focus
        #[arg(long)]
        file: Option<String>,
        /// Also compute confidence score
        #[arg(long)]
        confidence: bool,
        /// Run verification from saved plan
        #[arg(long)]
        plan: Option<String>,
        /// JSON output
        #[arg(long)]
        json: bool,
        /// Continue on error
        #[arg(long)]
        continue_on_error: bool,
    },
    /// Run verification steps for a task profile (coding, bugfix, review, arch)
    #[command(hide = true)]
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
    ///
    /// ⚠️ Deprecated: use `sruja verify --confidence` instead.
    #[command(hide = true)]
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
    /// DSL authoring tools: list, tree, explain, diff, import, compile, validate, generate
    #[command(hide = true)]
    Dsl {
        #[command(subcommand)]
        cmd: DslCommand,
    },
    /// Analysis & scores: health, impact, why, query, scores, onboard, watch, learn
    #[command(hide = true)]
    Inspect {
        #[command(subcommand)]
        cmd: InspectCommand,
    },
    /// Keep architecture feedback live while you code
    #[command(hide = true, name = "watch")]
    Watch {
        /// Repository root
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Clear screen between runs
        #[arg(long)]
        clear: bool,
        /// Only watch specific paths
        #[arg(long)]
        focus: Option<String>,
    },
    /// Build scan evidence and learned-fact hypotheses
    #[command(hide = true, name = "learn")]
    Learn {
        /// Repository root
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Only include facts referencing this file
        #[arg(long)]
        file: Option<String>,
        /// Only include facts since git ref
        #[arg(long)]
        since: Option<String>,
        /// Do not write proposals
        #[arg(long)]
        skip_proposals: bool,
        /// Apply proposals
        #[arg(long = "apply-proposals", default_value_t = true, action = clap::ArgAction::Set)]
        apply_proposals: bool,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Review & compliance gates: critique, compliance, baseline, drift-pr
    #[command(hide = true)]
    Guard {
        #[command(subcommand)]
        cmd: GuardCommand,
    },
    /// Adversarial architectural critique of proposed changes
    #[command(hide = true, name = "critique")]
    Critique {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Changed file paths to critique
        #[arg(long, short = 'f')]
        files: Vec<String>,
        /// Description of change
        #[arg(long, short = 'd')]
        description: Option<String>,
        /// Proposal ID
        #[arg(long, short = 'p')]
        proposal: Option<String>,
        /// Git base ref
        #[arg(long)]
        base: Option<String>,
        /// Git head ref
        #[arg(long)]
        head: Option<String>,
        /// Critique staged changes
        #[arg(long)]
        staged: bool,
        /// Output format
        #[arg(long, default_value = "text")]
        format: String,
        /// Optional LLM enrichment to add a narrative critique.
        #[command(flatten)]
        enrich: EnrichmentArgs,
        /// Fail the command on findings level
        #[arg(long)]
        fail_on: Option<String>,
    },
    /// Multi-repo federation: publish, compose
    #[command(hide = true)]
    Federation {
        #[command(subcommand)]
        cmd: FederationCommand,
    },
    /// Publish repo truth + evidence to repo.bundle.json
    #[command(hide = true, name = "publish")]
    Publish {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        #[arg(long)]
        repo_id: Option<String>,
        /// Output path for bundle
        #[arg(long, short = 'o', default_value = "repo.bundle.json")]
        output: String,
    },
    /// Compose one or more repo bundles into system.index.json
    #[command(hide = true, name = "compose")]
    Compose {
        #[arg(long, short = 'i', action = clap::ArgAction::Append)]
        input: Vec<String>,
        #[arg(long)]
        recursive: bool,
        /// Output path for system index
        #[arg(long, short = 'o', default_value = "system.index.json")]
        output: String,
    },
    /// Human-centric system intelligence: trace, explain, map, before, daily, what-if
    #[command(hide = true)]
    Human {
        #[command(subcommand)]
        cmd: HumanCommand,
    },
    /// Agent capability eval harness: run SWE-bench-flavored tasks against the agent
    #[command(hide = true)]
    Eval {
        #[command(subcommand)]
        cmd: EvalCommand,
    },
}

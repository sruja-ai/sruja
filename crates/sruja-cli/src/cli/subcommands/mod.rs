use clap::Subcommand;

use crate::enrichment::EnrichmentArgs;

pub mod agent;
pub mod human;
pub mod inspect;
pub mod workflow;

#[allow(unused_imports)]
pub use agent::AgentCommand;
#[allow(unused_imports)]
pub use human::HumanCommand;
#[allow(unused_imports)]
pub use inspect::InspectCommand;
#[allow(unused_imports)]
pub use workflow::WorkflowCommand;

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
    /// Export a trace bundle for a run_id (snapshots + agent artifacts + context events slice)
    Export {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Run ID to export
        #[arg(long)]
        run_id: String,
        /// Output directory (defaults to .sruja/run_exports/<run_id>)
        #[arg(long)]
        out: Option<String>,
        /// Max number of context events to include (newest-first)
        #[arg(long, default_value_t = 2000)]
        events_limit: usize,
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
pub enum AuthorCommand {
    /// Emit a capped, citeable evidence bundle for grounded architecture authoring
    Evidence {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (json)
        #[arg(long, short = 'f', default_value = "json")]
        format: String,
        /// Output path (defaults to .sruja/author_evidence.json)
        #[arg(long, short = 'o')]
        output: Option<String>,
        /// Do not print the JSON bundle to stdout (file is still written)
        #[arg(long, default_value_t = true)]
        quiet: bool,
    },
    /// Run an external enrichment command to synthesize a Proposal JSON from author evidence
    Propose {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// External enrichment command (reads JSON from stdin; writes Proposal JSON to stdout)
        #[arg(long)]
        enrich_cmd: String,
        /// Timeout for --enrich-cmd in milliseconds (default: 15000)
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes to read from --enrich-cmd stdout (default: 20000)
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
    },
}

#[derive(Subcommand)]
pub enum EventCommand {
    /// Append one JSON event line (use --json or pipe one line on stdin)
    Append {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// JSON object (otherwise first non-empty line from stdin)
        #[arg(long)]
        json: Option<String>,
    },
    /// List recent events (newest first)
    List {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        details_substring: Option<String>,
        #[arg(long)]
        decision_id: Option<String>,
        #[arg(long)]
        trace_id: Option<String>,
        /// Filter to events touching this architecture element id
        #[arg(long)]
        element_id: Option<String>,
        /// Only kinds used for decision/workflow lineage
        #[arg(long, default_value_t = false)]
        decision_lineage_only: bool,
    },
}

#[derive(Subcommand)]
pub enum MemoryCommand {
    /// Rebuild `.sruja/memory.sqlite` from learnings, events, and decisions
    Reindex {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    /// Full-text search indexed memory (hypothesis vs reviewed_truth labels)
    Search {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Search query
        query: String,
        #[arg(long)]
        element_id: Option<String>,
        #[arg(long)]
        decision_id: Option<String>,
        #[arg(long)]
        hitl_kind: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Chronological slice around an anchor id or timestamp
    Timeline {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        anchor_id: Option<String>,
        #[arg(long)]
        anchor_timestamp: Option<String>,
        #[arg(long, default_value_t = 10)]
        before: usize,
        #[arg(long, default_value_t = 10)]
        after: usize,
        #[arg(long)]
        decision_id: Option<String>,
        #[arg(long)]
        element_id: Option<String>,
    },
    /// Show per-skill effectiveness from context events
    SkillStats {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Archive stale learnings (decay score below threshold, older than min age)
    Archive {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Decay score threshold (entries below this are archived)
        #[arg(long, default_value_t = 0.15)]
        decay_threshold: f64,
        /// Minimum age in days before an entry can be archived
        #[arg(long, default_value_t = 30)]
        min_age_days: i64,
        /// Actually delete (requires confirmation)
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum DecisionCommand {
    /// Create a new proposed Decision Record file
    New {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long, short = 't')]
        title: String,
        /// architecture | product | operational | security | agent | exception
        #[arg(long)]
        typ: String,
        #[arg(long)]
        scope: Option<String>,
    },
    List {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    Show {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        id: String,
    },
    Trace {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        id: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Link {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        id: String,
        #[arg(long)]
        element: String,
    },
    Accept {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        id: String,
    },
    Supersede {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        id: String,
        /// Decision id that supersedes this record
        #[arg(long = "by")]
        by: String,
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
        /// Optional workflow id to link this proposal to
        #[arg(long)]
        workflow_id: Option<String>,
        /// Add elements in format "id:kind:label[:tech]"
        #[arg(long, short = 'e')]
        add_elements: Vec<String>,
        /// Add relationships in format "source->target[:label]"
        #[arg(long, short = 'l')]
        add_relationships: Vec<String>,
        /// Remove elements by ID
        #[arg(long)]
        remove_elements: Vec<String>,
        /// Remove relationships in format "source->target"
        #[arg(long)]
        remove_relationships: Vec<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// List all architectural proposals
    List {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Approve and merge a proposal
    Approve {
        /// Proposal ID to approve
        proposal_id: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Validate, show the merge plan, and exit without writing files
        #[arg(long)]
        dry_run: bool,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
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

#[derive(Subcommand)]
pub enum DslCommand {
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
    /// Generate a prompt for LLM-based architecture.sruja generation
    Generate {
        /// Path to repository
        #[arg(long, short = 'r', action = clap::ArgAction::Append)]
        repo: Vec<String>,
        /// Path to skill file
        #[arg(long)]
        skill_path: Option<String>,
        /// Emit prompt only
        #[arg(long, required = true)]
        prompt_only: bool,
        /// Output path for prompt
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum GuardCommand {
    /// Adversarial architectural critique of proposed changes
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
    /// Structural drift + intent + policy gate
    Compliance {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Path to baseline architecture
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Path to intent directory
        #[arg(long, short = 'i')]
        intent: Option<String>,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Strict mode
        #[arg(long)]
        strict: bool,
    },
    /// Baseline: snapshot current violations to ignore them in CI
    Baseline {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output path
        #[arg(long, short = 'o', default_value = ".sruja/violations.baseline.json")]
        output: String,
    },
    /// PR-scoped drift: detect only NEW violations in a PR
    #[command(name = "drift-pr")]
    DriftPr {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Base ref
        #[arg(long, short = 'b')]
        base: Option<String>,
        /// Head ref
        #[arg(long, short = 'H')]
        head: Option<String>,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum FederationCommand {
    /// Publish repo truth + evidence to repo.bundle.json
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
    Compose {
        #[arg(long, short = 'i', action = clap::ArgAction::Append)]
        input: Vec<String>,
        #[arg(long)]
        recursive: bool,
        /// Output path for system index
        #[arg(long, short = 'o', default_value = "system.index.json")]
        output: String,
    },
}

/// AI-DLC workflow commands: inception → construction → operations with phase gates.
///
/// Simplified entry point for AI-DLC users. Wraps `sruja workflow` with
/// AI-DLC defaults pre-filled (--with-aidlc, --install-aidlc-rules).
#[derive(Subcommand)]
pub enum AidlcCommand {
    /// Create an AI-DLC workflow with defaults pre-filled
    Init {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow title (e.g., "Add payment service")
        #[arg(long, short = 't')]
        title: String,
        /// Optional workflow id (defaults to random short id)
        #[arg(long)]
        id: Option<String>,
        /// AI-DLC profile: minimal (default) or full
        #[arg(long, default_value = "minimal")]
        profile: String,
        /// Scaffold template: minimal, feature, bugfix, e2e
        #[arg(long)]
        template: Option<String>,
        /// Target architecture element ids
        #[arg(long = "element", short = 'e')]
        target_elements: Vec<String>,
    },
    /// Show gate readiness + AI-DLC status
    Status {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
        /// Exit non-zero if the current phase gate fails
        #[arg(long)]
        check: bool,
    },
    /// Validate workflow + AI-DLC artifact checklist
    Validate {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
    },
    /// Show actionable next steps for current phase
    NextSteps {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
    },
    /// Copy vendored AIDLC rules into .aidlc/ for the editor host
    InstallRules {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    /// Show a beautiful end-to-end workflow summary
    Summary {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum GraphCommand {
    /// Show graph change history over time
    History {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Filter by time period (e.g., "7d", "30d", "90d")
        #[arg(long)]
        since: Option<String>,
        /// Filter by element ID
        #[arg(long)]
        element: Option<String>,
        /// Filter by delta kind (node_added, edge_added, etc.)
        #[arg(long)]
        kind: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum EvalCommand {
    /// Run an eval task instance against the agent
    Run {
        /// Task instance ID (directory name under evaluation/tasks/)
        #[arg(long)]
        instance: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Maximum iterations for the agent loop
        #[arg(long, default_value_t = 3)]
        max_iterations: usize,
        /// Dry-run mode: block all file mutations
        #[arg(long)]
        dry_run: bool,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// List available eval task instances
    List {
        /// Path to evaluation/tasks/ directory
        #[arg(long, default_value = "evaluation/tasks")]
        tasks_dir: String,
    },
}

use clap::Subcommand;

use crate::enrichment::EnrichmentArgs;

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
        /// HITL classification: precedent | exception | correction | guardrail
        #[arg(long)]
        hitl_kind: Option<String>,
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
    /// Review learnings for merge/delete suggestions (read-only)
    Curate {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Update an existing learning by id
    Update {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long, short = 'i')]
        id: String,
        #[arg(long, short = 'c')]
        context: Option<String>,
        #[arg(long, short = 'H')]
        hypothesis: Option<String>,
        #[arg(long, short = 'o')]
        outcome: Option<String>,
        #[arg(long, short = 'g')]
        guardrail: Option<String>,
        #[arg(long, short = 's')]
        reason: Option<String>,
    },
    /// Delete a learning by id
    Delete {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long, short = 'i')]
        id: String,
        #[arg(long, short = 'y')]
        force: bool,
    },
    /// Merge multiple learnings into one
    Merge {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Comma-separated learning ids to merge
        #[arg(long)]
        ids: String,
        #[arg(long, short = 'c')]
        context: String,
        #[arg(long, short = 'H')]
        hypothesis: String,
        #[arg(long, short = 'g')]
        guardrail: String,
        #[arg(long, short = 'o', default_value = "success")]
        outcome: String,
    },
    /// Record what worked (playbook) or failed after any agent completes a task.
    ///
    /// Standalone command for coding agents (Claude Code, Cursor, etc.) to
    /// auto-distill learnings without going through `sruja agent run`.
    Distill {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// What task was being performed
        #[arg(long, short = 'c')]
        goal: String,
        /// Outcome: success or failed
        #[arg(long, short = 'o', default_value = "success")]
        outcome: String,
        /// Comma-separated element IDs affected
        #[arg(long, short = 'e')]
        elements: Option<String>,
        /// Optional: what specifically worked or failed
        #[arg(long)]
        detail: Option<String>,
        /// Optional: what to do differently next time (for failures)
        #[arg(long, short = 'g')]
        guardrail: Option<String>,
    },
    /// Write a session handoff summary for the next agent session to consume.
    ///
    /// Coding agents call this at the end of a task so the next session
    /// (via `sruja focus`) picks up context automatically.
    SessionSummary {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// What task was performed
        #[arg(long, short = 'c')]
        goal: String,
        /// Whether the task succeeded
        #[arg(long)]
        success: bool,
        /// Optional element ID that was the focus
        #[arg(long, short = 'e')]
        element_id: Option<String>,
        /// Optional: brief summary of what happened
        #[arg(long, short = 's')]
        summary: Option<String>,
    },
    /// Propose a higher-level architectural fact for human review.
    ///
    /// Unlike scan-derived facts, these are agent-inferred observations
    /// (e.g., "the auth module is the most frequently changed component").
    ProposeFact {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Subject (e.g., "AuthModule")
        #[arg(long)]
        subject: String,
        /// Predicate (e.g., "has_change_frequency")
        #[arg(long)]
        predicate: String,
        /// Object (e.g., "high")
        #[arg(long)]
        object: String,
        /// Human-readable claim
        #[arg(long, short = 'c')]
        claim: String,
        /// Confidence 0.0-1.0
        #[arg(long, default_value_t = 0.7)]
        confidence: f64,
        /// Optional comma-separated evidence references
        #[arg(long)]
        evidence: Option<String>,
    },
    /// Interactive setup for LLM provider (configures .sruja/config.toml)
    Setup {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Provider id (skip interactive selection): openrouter, openai, zai, ximimo, groq, ollama
        #[arg(long, short = 'p')]
        provider: Option<String>,
        /// API key (skip interactive prompt; not needed for ollama)
        #[arg(long, short = 'k')]
        api_key: Option<String>,
        /// Model override (uses provider default if omitted)
        #[arg(long, short = 'm')]
        model: Option<String>,
    },
    /// Architecture-bounded agent loop: observe → plan → (optional) apply → verify → record learnings
    ///
    /// Requires Sruja evidence and a reviewable plan; not a general-purpose coding agent.
    #[command(hide = true)]
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
        /// File path focus (relative to repo root). Optional: narrow the scope to a specific file.
        #[arg(long)]
        file: Option<String>,
        /// Architecture element ID from repo.sruja. Optional: narrow the scope to an architecture element.
        #[arg(long)]
        element_id: Option<String>,
        /// Natural language query focus. Optional: narrow the scope with a free-text query.
        /// When none of --file/--element-id/--query are given, the goal text is used as the query.
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
        /// Optional LLM enrichment to add narrative grounded in gathered facts.
        #[command(flatten)]
        enrich: EnrichmentArgs,
        /// Continue running verification even if an apply step fails
        #[arg(long)]
        continue_on_error: bool,
        /// Force full sync even when cache is fresh (re-scans entire codebase)
        #[arg(long)]
        force_sync: bool,
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
        /// File path focus (relative to repo root). Optional: narrow the scope to a specific file.
        #[arg(long)]
        file: Option<String>,
        /// Architecture element ID from repo.sruja. Optional: narrow the scope to an architecture element.
        #[arg(long)]
        element_id: Option<String>,
        /// Natural language query focus. Optional: narrow the scope with a free-text query.
        /// When none of --file/--element-id/--query are given, the goal text is used as the query.
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
        /// Optional LLM enrichment to add narrative grounded in gathered facts.
        #[command(flatten)]
        enrich: EnrichmentArgs,
    },
    /// Suggest learnings from a facts_bundle.json (optional --write to record)
    Reflect {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        write: bool,
        #[arg(long, short = 'f', default_value = "json")]
        format: String,
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
    /// Autonomous coding loop: comprehend -> plan -> execute via tools -> critique -> replan until approved
    Loop {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Natural language goal (e.g. "Add a health check endpoint")
        #[arg(long)]
        goal: String,
        /// Maximum plan->execute->critique iterations (default: 3, or from .sruja/loop.toml)
        #[arg(long)]
        max_iterations: Option<usize>,
        /// Disable TDD mode (plans write tests before implementation by default)
        #[arg(long)]
        no_tdd: bool,
        /// Dry-run mode: block all file mutations, still run the loop
        #[arg(long)]
        dry_run: bool,
        /// Override the model name (default: from OPENAI_MODEL env or gpt-4o-mini)
        #[arg(long)]
        model: Option<String>,
        /// Override the API base URL (default: https://api.openai.com/v1)
        #[arg(long)]
        base_url: Option<String>,
        /// USD spend cap — abort the loop if estimated cost exceeds this
        #[arg(long)]
        spend_cap: Option<f64>,
        /// Disable oscillation detection (auto-terminate on repeated critique patterns)
        #[arg(long)]
        no_oscillation_detection: bool,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Force proceed past the calibration gate (override Ask verdict; no calibration DR written)
        #[arg(long, alias = "force-proceed")]
        yes: bool,
        /// Disable the default deterministic grader (sruja lint + drift); trust the LLM critic only
        #[arg(long, alias = "trust-critic")]
        no_default_grader: bool,
        /// Interactive steering: prompt between iterations to continue, stop, or view the live report
        #[arg(long)]
        steer: bool,
        /// Resume a previously interrupted loop from checkpoint (in .sruja/runs/<run_id>/)
        #[arg(long)]
        resume: bool,
        /// Show the plan preview before execution (even for Proceed* verdicts)
        #[arg(long)]
        show_plan: bool,
        /// Create a git checkpoint ref before execution for rollback safety
        #[arg(long)]
        checkpoint: bool,
        /// Disable auto git checkpoint (overrides default behavior for one-way-door goals)
        #[arg(long)]
        no_checkpoint: bool,
        /// Force writing a post-loop changelog (skipped for trivial runs by default)
        #[arg(long)]
        changelog: bool,
        /// Plan-only mode: produce a plan without making code changes (dry-run, skip verify).
        #[arg(long)]
        plan_only: bool,
        /// Show the resolved pipeline stages as JSON and exit (no execution).
        #[arg(long)]
        show_pipeline: bool,
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
pub enum WorkflowCommand {
    /// Create workflow manifest + phase directories under .sruja/workflows/<id>/
    Init {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow title
        #[arg(long)]
        title: String,
        /// Optional workflow id (defaults to random short id)
        #[arg(long)]
        id: Option<String>,
        /// Target architecture element ids (optional; used for record-impact and context)
        #[arg(long = "element", short = 'e')]
        target_elements: Vec<String>,
        /// Enforce strict gates (default: true)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        strict_gates: bool,
        /// Enable AI-DLC artifact dirs and manifest.aidlc
        #[arg(long)]
        with_aidlc: bool,
        /// AIDLC gate profile when --with-aidlc (minimal|full)
        #[arg(long, default_value = "minimal")]
        aidlc_profile: String,
        /// Run workflow install-rules during init
        #[arg(long)]
        install_aidlc_rules: bool,
        /// Workflow profile (minimal|full|e2e)
        #[arg(long, default_value = "minimal")]
        profile: String,
        /// Workflow scaffold template (e2e|feature|bugfix|minimal)
        #[arg(long)]
        template: Option<String>,
    },
    /// List workflows under .sruja/workflows/
    List {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    /// Show workflow phase and gate readiness
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
    /// Record impact.json for the workflow's target_elements
    RecordImpact {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id
        #[arg(long)]
        id: String,
        /// Impact traversal depth (default: 3)
        #[arg(long, default_value_t = 3)]
        depth: usize,
    },
    /// Approve a phase after verifying required artifacts are present
    Approve {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id
        #[arg(long)]
        id: String,
        /// Phase to approve (inception|construction|operations)
        #[arg(long)]
        phase: String,
        /// Actor name (defaults to "human")
        #[arg(long)]
        by: Option<String>,
    },
    /// Advance to the next phase if the current phase is approved (strict) or always (non-strict)
    Advance {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id
        #[arg(long)]
        id: String,
    },
    /// Copy vendored AIDLC rules into .aidlc/ for the editor host
    InstallRules {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
    },
    /// Validate workflow + optional AIDLC artifact checklist (same checks as status --check)
    Validate {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: Option<String>,
    },
    /// Append an audit event to workflow audit.jsonl
    Audit {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        event: String,
        #[arg(long)]
        by: Option<String>,
    },
    /// Generate traceability matrix from workflow aidlc-docs (requires aidlc-traceability Python package)
    Trace {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: String,
        #[arg(long, default_value = "markdown")]
        format: String,
        #[arg(long)]
        check: bool,
    },
    /// Optional headless AIDLC run via aidlc-evaluator (requires Python + AWS when not --dry-run)
    Run {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        vision: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Grounded design review for workflow inception (writes design-review.md)
    DesignReview {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        #[arg(long)]
        id: String,
        #[arg(long, short = 'o')]
        output: Option<String>,
        #[arg(long)]
        enrich_cmd: Option<String>,
    },
    /// Scaffold or capture requirements under .sruja/workflows/<id>/inception/requirements.md
    CaptureRequirements {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
        /// Optional issue URL or identifier to ingest from
        #[arg(long)]
        from_issue: Option<String>,
        /// Optional external enrichment command to run (reads JSON from stdin; writes markdown to stdout)
        #[arg(long)]
        enrich_cmd: Option<String>,
    },
    /// Record test verification results under .sruja/workflows/<id>/construction/test-results.json
    RecordTestResults {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
        /// Task verification profile (coding|bugfix|review|arch)
        #[arg(long)]
        profile: Option<String>,
        /// Path to a pre-recorded test output JSON file to copy
        #[arg(long)]
        from_file: Option<String>,
    },
    /// Record operations readiness checklist under .sruja/workflows/<id>/operations/readiness.json
    RecordReadiness {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
    },
    /// Show a beautiful end-to-end workflow summary and health dashboard
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
    /// Show actionable next steps for the current workflow phase
    NextSteps {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Workflow id (required when multiple exist)
        #[arg(long)]
        id: Option<String>,
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
pub enum InspectCommand {
    /// Architecture health score from structural violations (0-100)
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
    /// Impact analysis: blast radius
    Impact {
        /// Node selector
        target: String,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Max traversal depth
        #[arg(long, default_value_t = 3)]
        depth: usize,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Investigate architecture decisions with deterministic evidence
    Why {
        /// Question to ask
        question: String,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Use reasoning-tree traversal
        #[arg(long)]
        reasoned: bool,
        /// Use LLM-guided tree search
        #[arg(long)]
        llmguided: bool,
    },
    /// Query the architectural registry
    Query {
        /// Query string
        query: String,
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Path to architecture file
        #[arg(long, short = 'a')]
        architecture: Option<String>,
        /// Output format
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// AI-readiness score (0-100)
    #[command(name = "context-score")]
    ContextScore {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Fail with exit code 1 if score is below this threshold
        #[arg(long)]
        fail_under: Option<u8>,
    },
    /// Generate interactive HTML/D3.js visualization
    #[command(name = "context-graph")]
    ContextGraph {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output path for the HTML file
        #[arg(long, short = 'o', default_value = "context_graph.html")]
        output: String,
        /// Open the browser after generation
        #[arg(long)]
        open: bool,
    },
    /// Complete architecture brief for onboarding
    Onboard {
        /// Path to repository root
        #[arg(long, short = 'r', alias = "path", default_value = ".")]
        repo: String,
        /// Output format
        #[arg(long, short = 'f', default_value = "markdown")]
        format: String,
        /// Max number of items per section
        #[arg(long, default_value_t = 8)]
        max_items: usize,
        /// Optional LLM enrichment to add a narrative section.
        #[command(flatten)]
        enrich: EnrichmentArgs,
        /// Output file
        #[arg(long, short = 'o')]
        output: Option<String>,
    },
    /// Structural overview (first look structural brief)
    #[command(visible_alias = "overview")]
    Quickstart {
        /// Repository root
        #[arg(long = "repo", short = 'r', alias = "path", default_value = ".")]
        path: String,
        /// Output format
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Write repo.sruja.draft (workspace map evidence; not reviewed architecture)
        #[arg(long)]
        generate_baseline: bool,
        /// Fail on specified violations
        #[arg(long)]
        fail_on: Option<String>,
        /// First-run friendly: omit orphan info findings
        #[arg(long)]
        advisory: bool,
    },
    /// Keep architecture feedback live while you code
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
    /// Ingest external context into .sruja/context/
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
pub enum HumanCommand {
    /// "What happens when..." — trace a flow across all repos
    Trace {
        /// Natural language query (e.g. "user clicks checkout")
        query: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Max traversal depth
        #[arg(long, default_value_t = 3)]
        depth: usize,
        /// Filter by team/owner
        #[arg(long)]
        team: Option<String>,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// "What is this thing?" — human-centric element explanation
    Explain {
        /// Element name, ID, or alias
        target: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text, json, md)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Save explanation to docs/architecture/<element>.md
        #[arg(long)]
        persist: bool,
    },
    /// "How does the system work?" — compressed cognitive map
    Map {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Filter by team/owner
        #[arg(long)]
        team: Option<String>,
        /// Focus on a specific element neighborhood
        #[arg(long)]
        focus: Option<String>,
        /// Show all nodes (including modules, docs, assets)
        #[arg(long)]
        all: bool,
    },
    /// Pre-change impact check: "what will I break?"
    Before {
        /// File path to check impact for
        file: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// CI mode: exit non-zero if downstream count exceeds threshold
        #[arg(long)]
        ci: bool,
        /// Downstream threshold for CI mode (default: 10)
        #[arg(long, default_value_t = 10)]
        threshold: usize,
    },
    /// Morning intelligence briefing: what changed, what to care about
    Daily {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
    },
    /// Cognitive debt score: how well does the team understand the system?
    CognitiveDebt {
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// CI mode: exit non-zero if score below threshold
        #[arg(long)]
        ci: bool,
    },
    /// "What if I change X?" — safe change modeling
    WhatIf {
        /// Description of the change to model
        query: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// CI mode: exit non-zero if blast radius exceeds threshold
        #[arg(long)]
        ci: bool,
        /// Direct effects threshold for CI mode (default: 5)
        #[arg(long, default_value_t = 5)]
        threshold: usize,
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

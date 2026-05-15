use clap::Subcommand;

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
        /// Optional LLM enrichment
        #[arg(long)]
        enrich: bool,
        /// Enrichment provider
        #[arg(long, alias = "llm-provider")]
        enrich_provider: Option<String>,
        /// External enrichment command
        #[arg(long)]
        enrich_cmd: Option<String>,
        /// Model name
        #[arg(long, alias = "llm-model")]
        enrich_model: Option<String>,
        /// Base URL
        #[arg(long, alias = "llm-base-url")]
        enrich_base_url: Option<String>,
        /// Timeout for enrichment in milliseconds
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes to read from enrichment stdout
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
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
        /// Generate a draft repo.sruja baseline from scan
        #[arg(long)]
        generate_baseline: bool,
        /// Fail on specified violations
        #[arg(long)]
        fail_on: Option<String>,
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
        /// Optional enrichment
        #[arg(long)]
        enrich: bool,
        /// Enrichment provider
        #[arg(long, alias = "llm-provider")]
        enrich_provider: Option<String>,
        /// External enrichment command
        #[arg(long)]
        enrich_cmd: Option<String>,
        /// Model name
        #[arg(long, alias = "llm-model")]
        enrich_model: Option<String>,
        /// Base URL
        #[arg(long, alias = "llm-base-url")]
        enrich_base_url: Option<String>,
        /// Timeout
        #[arg(long, default_value_t = 15000)]
        enrich_timeout_ms: u64,
        /// Max bytes
        #[arg(long, default_value_t = 20000)]
        enrich_max_bytes: usize,
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

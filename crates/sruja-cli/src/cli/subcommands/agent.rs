use clap::Subcommand;

use crate::enrichment::EnrichmentArgs;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
#[command(after_help = r#"Quick start:
  sruja agent task "add health check"    Autonomous coding agent
  sruja agent setup                      Configure your LLM provider

Learning & memory:
  sruja agent history                    What the agent has learned
  sruja agent learn                      Record what worked or failed
  sruja agent session-summary            Handoff context between sessions

Advanced:
  sruja agent record / update / delete / merge / curate / clusters / clear
  sruja agent propose-fact / reflect

Tip: sruja auto "task" does the same thing as sruja agent task "task"."#)]
pub enum AgentCommand {
    /// Run the autonomous coding agent on a task.
    ///
    /// Example: sruja agent task "add a health check endpoint"
    ///
    /// The agent will plan, implement, and verify your changes automatically.
    /// Run `sruja agent setup` first to configure your LLM provider.
    #[command(visible_alias = "go")]
    Task {
        /// What do you want the agent to do?
        goal: String,
        /// Path to repository root
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// Maximum plan->execute->verify iterations (default: 3)
        #[arg(long)]
        max_iterations: Option<usize>,
        /// Plan preview only, no code changes
        #[arg(long)]
        dry_run: bool,
        /// Skip confirmation prompts
        #[arg(long)]
        yes: bool,
        /// Execute from saved pipeline TOML
        #[arg(long)]
        pipeline: Option<String>,
        /// Resume from last checkpoint
        #[arg(long)]
        resume: bool,
        /// Output format (text or json)
        #[arg(long, short = 'f', default_value = "text")]
        format: String,
        /// Show internal details (paths, tokens, costs)
        #[arg(long = "show-details")]
        show_details: bool,
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
    /// Show what the agent has learned from past tasks
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
    /// Record what the agent learned from a task (what worked, what failed).
    ///
    /// Use this after any coding session so future runs learn from past experience.
    Learn {
        #[arg(long, short = 'r', default_value = ".")]
        repo: String,
        /// What task was done
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
        /// Optional LLM enrichment to add a narrative grounded in gathered facts.
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
        /// Skip safety check confirmation prompts
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

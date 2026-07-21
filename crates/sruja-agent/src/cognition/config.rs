use serde::{Deserialize, Serialize};

use crate::llm::{DEFAULT_MODEL, PREMIUM_MODEL};
use crate::cognition::types::CritiquePersona;

/// How the critique ensemble dispatches: always run the full set of
/// personas, or run a single quick check first and skip the ensemble when
/// the quick check is confident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CritiqueMode {
    /// Always run the full persona ensemble (current behavior).
    Full,
    /// Run a single quick-check call first. If its score >= threshold and
    /// it approves, skip the full ensemble. Otherwise, fall through.
    #[default]
    QuickThenFull,
    /// Always run just the quick check (cheapest, least thorough).
    QuickOnly,
}

/// User-configured model names per complexity tier.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelMapping {
    pub cheap: String,
    pub mid: String,
    pub premium: String,
    pub review: String,
}

impl Default for ModelMapping {
    fn default() -> Self {
        Self {
            cheap: DEFAULT_MODEL.into(),
            mid: DEFAULT_MODEL.into(),
            premium: PREMIUM_MODEL.into(),
            review: PREMIUM_MODEL.into(),
        }
    }
}

/// Framework-wide configuration with opinionated defaults.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Per-complexity model routing + review model.
    pub models: ModelMapping,
    /// TDD mode: plans write tests before implementation (default: true).
    pub tdd: bool,
    /// Run the Critic after every change, using the review model (default: true).
    pub review_every_change: bool,
    /// USD spend cap (default: None = unlimited).
    pub spend_cap_usd: Option<f64>,
    /// Block all mutations (default: false).
    pub dry_run: bool,
    /// Max tool-call iterations before giving up (default: 8).
    pub max_tool_iterations: usize,
    /// Wall-clock timeout for the entire tool loop in seconds (default: 300 = 5 min).
    /// Prevents the agent from getting stuck if individual calls are slow.
    pub loop_timeout_secs: u64,
    /// Additional instructions appended to the comprehension system prompt.
    /// Use for context-specific nudges (e.g., "call sruja_focus first").
    pub system_hints: Vec<String>,
    /// The critic ensemble: one probe-bound persona per perspective. When
    /// non-empty, [`Agent::critique`] fans these out in parallel and unions
    /// their issues (AND semantics for approval). When empty, falls back to a
    /// single call with the legacy [`CRITIQUE_SYSTEM_PROMPT`] (backward
    /// compatible). Default is [`CritiquePersona::default_personas`].
    pub critique_personas: Vec<CritiquePersona>,
    /// When true, emit `tool_call` / `tool_result` context events for every
    /// agent->tool dispatch (requires `repo_path`, `run_id`, `trace_id` to be
    /// set on the agent).
    pub enable_tool_call_tracing: bool,
    /// Abort after N consecutive tool-only iterations (no text output).
    /// Default: 3. Set to 0 to disable.
    pub max_consecutive_tool_only: usize,
    /// Abort after N consecutive identical tool+arg signatures.
    /// Default: 3. Set to 0 to disable.
    pub max_consecutive_same_call: usize,
    /// Abort when non-converged fraction exceeds this threshold.
    /// Default: 0.5. Set to >1.0 to disable.
    pub max_non_converged_fraction: f64,
    /// Critique dispatch mode. When `QuickThenFull` (default), a single
    /// lightweight check runs first; the full ensemble is skipped if the
    /// check is confident (score >= `quick_critique_threshold`).
    pub critique_mode: CritiqueMode,
    /// Minimum score for the quick critique to short-circuit the full
    /// ensemble. Only used when `critique_mode` is `QuickThenFull`.
    /// Default: 0.9.
    pub quick_critique_threshold: f64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            models: ModelMapping::default(),
            tdd: true,
            review_every_change: true,
            spend_cap_usd: None,
            dry_run: false,
            // 7 tool-call iterations gives enough budget for: read 1-2 files ->
            // receive progress nudge at iteration 3 -> make edits by iteration 5
            // before the hard convergence cutoff. 5 was too tight — models that
            // read even 2 files had no budget left for edits.
            max_tool_iterations: 7,
            // 5-minute wall-clock timeout for the entire tool loop.
            loop_timeout_secs: 300,
            system_hints: Vec::new(),
            critique_personas: CritiquePersona::default_personas(),
            enable_tool_call_tracing: true,
            max_consecutive_tool_only: 3,
            max_consecutive_same_call: 3,
            max_non_converged_fraction: 0.5,
            critique_mode: CritiqueMode::QuickThenFull,
            quick_critique_threshold: 0.9,
        }
    }
}

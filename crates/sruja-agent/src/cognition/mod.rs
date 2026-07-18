//! Agent cognition — the Principal Engineer loop.
//!
//! ## TDD pipeline (default)
//!
//! ```text
//!  Comprehend → TestAuthor → TestReview(HITL) → Implement → Review → Reflect
//! ```
//!
//! Tests and code are **never in flux simultaneously** — the [`FileGuard`]
//! freezes one side while the other changes.
//!
//! ## Complexity routing
//!
//! Each subtask in a plan carries a [`TaskTier`] tag. The executor routes
//! the LLM call through the model configured for that tier, giving per-subtask
//! cost control.

pub mod changelog;
pub mod chat;
pub mod decision;
pub mod errors;
pub mod hook;
pub mod loop_event;
pub mod runbook;
pub mod subagent;
pub mod tool_tracing;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::llm::{
    CompletionRequest, CompletionResponse, LlmClient, LlmError, Message, ModelRouter, Usage,
    DEFAULT_MODEL, PREMIUM_MODEL,
};
use crate::tool::ToolSignal;

pub use crate::llm::TaskTier;
use crate::memory::AgenticMemory;
use crate::tool::{FileGuard, Phase, ToolRegistry};
use crate::verify::{
    run_verification_steps, VerifyOptions, VerifyResult, VerifyStatus, VerifyStep,
};
use crate::LearningEntry;

pub use decision::{DecisionRecord, DecisionStatus};
pub use errors::{AgentError, PlanParseError};
pub use hook::{Hook, HookAction, HookRegistry, Hooks, LoggingHook};
pub use loop_event::{LoopEvent, LoopPhase, PlanBrief};
pub use runbook::{Runbook, RunbookSeverity};
pub use tool_tracing::ToolCallTracer;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

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
    /// agent→tool dispatch (requires `repo_path`, `run_id`, `trace_id` to be
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
            // 7 tool-call iterations gives enough budget for: read 1-2 files →
            // receive progress nudge at iteration 3 → make edits by iteration 5
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

// ---------------------------------------------------------------------------
// Task complexity routing
// ---------------------------------------------------------------------------

/// Heuristic task complexity, determined from the goal statement and scope.
///
/// Controls prompt selection, TDD enforcement, tool-call budgets, and whether
/// post-loop artifacts (decision record, runbook) are generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskComplexity {
    /// One-line change: comment, typo, rename, format, whitespace.
    /// Skips TDD, skips post-loop artifacts, uses a minimal plan prompt.
    Trivial,
    /// Small change: 1-2 files, no architecture impact.
    /// Full review but lightweight planning.
    #[default]
    Simple,
    /// Multi-file change or moderate refactoring.
    /// Full TDD pipeline, full review.
    Moderate,
    /// Architecture-level: migration, system redesign, new module.
    /// Full pipeline, max iterations.
    Complex,
    /// Research/analysis: comprehend IS the output. No code changes produced.
    /// Pipeline: [Comprehend]. Recovery: Fail. Hard-capped at 1 iteration.
    Research,
}

impl TaskComplexity {
    /// Whether TDD should be enforced for this complexity level.
    pub fn enforce_tdd(self) -> bool {
        !matches!(self, TaskComplexity::Trivial | TaskComplexity::Research)
    }

    /// Whether post-loop artifacts (decision record, runbook) should be generated.
    pub fn generate_artifacts(self) -> bool {
        !matches!(self, TaskComplexity::Trivial)
    }

    /// Effective max tool iterations for this complexity level.
    pub fn max_tool_iterations(self, configured: usize) -> usize {
        match self {
            TaskComplexity::Trivial => configured.min(7),
            TaskComplexity::Simple => configured.min(7),
            TaskComplexity::Research => configured.min(10),
            _ => configured,
        }
    }
}

/// Classify task complexity from the goal statement and scope hints.
///
/// Uses keyword heuristics + scope (file/element count). Deterministic —
/// no LLM call — so it adds zero latency.
pub fn classify_task_complexity(
    goal: &str,
    target_files: &[String],
    target_elements: &[String],
) -> TaskComplexity {
    let goal_lower = goal.to_lowercase();
    let file_count = target_files.len();
    let element_count = target_elements.len();

    // Research heuristics: detect analysis/review-only goals BEFORE Complex so
    // "explain the migration system" → Research, not Complex (explaining is
    // research even about a complex topic).
    {
        let trimmed = goal_lower.trim();
        let starts_with_how = trimmed.starts_with("how to") || trimmed.starts_with("how do");
        // Use word-boundary matching via split_whitespace to avoid false positives
        // like "build" in "the build failing" or "change" in "what changed".
        let has_implementation_keywords = {
            let words: std::collections::HashSet<&str> = goal_lower.split_whitespace().collect();
            const IMPL_WORDS: &[&str] = &[
                "add",
                "create",
                "implement",
                "write",
                "edit",
                "fix",
                "refactor",
                "migrate",
                "delete",
                "remove",
                "modify",
            ];
            IMPL_WORDS.iter().any(|k| words.contains(k))
        };

        let is_question = trimmed.ends_with('?');
        let is_exploratory_prefix = [
            "what",
            "why",
            "explain",
            "analyze",
            "describe",
            "investigate",
            "evaluate",
            "review",
            "research",
        ]
        .iter()
        .any(|prefix| {
            let p = format!("{} ", prefix);
            goal_lower.starts_with(&p) || goal_lower == *prefix
        });

        if !starts_with_how
            && !has_implementation_keywords
            && (is_question || is_exploratory_prefix)
        {
            return TaskComplexity::Research;
        }
    }

    // Complex keywords: architecture-level work.
    // Check FIRST so "add a comment to migrate the database" → Complex, not Trivial.
    let complex_keywords = [
        "migrate",
        "migration",
        "architecture",
        "redesign",
        "restructure",
        "system-wide",
        "overhaul",
    ];
    let is_complex_keyword = complex_keywords.iter().any(|k| goal_lower.contains(k));
    if is_complex_keyword || element_count >= 3 || file_count >= 5 {
        return TaskComplexity::Complex;
    }

    // Trivial keywords: cosmetic / single-token changes.
    let trivial_keywords = [
        "comment",
        "doc comment",
        "add a comment",
        "add comment",
        "typo",
        "spelling",
        "whitespace",
        "reformat",
        "add a blank line",
        "add newline",
    ];
    let is_trivial_keyword = trivial_keywords.iter().any(|k| goal_lower.contains(k));
    if is_trivial_keyword && file_count <= 1 && element_count == 0 {
        return TaskComplexity::Trivial;
    }

    // Rename is trivial when scoped to one file.
    if goal_lower.contains("rename") && file_count <= 1 {
        return TaskComplexity::Trivial;
    }

    // Simple: small scope, no architecture keywords.
    if file_count <= 2 && element_count <= 1 {
        return TaskComplexity::Simple;
    }

    TaskComplexity::Moderate
}

// ---------------------------------------------------------------------------
// Cognition types
// ---------------------------------------------------------------------------

/// What kind of work a subtask represents (maps to TDD phases).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubtaskKind {
    Comprehend,
    TestAuthor,
    Implement,
    Verify,
    /// Adversarial test generation (U5): after implementation, generate a
    /// failing test that exposes a flaw in the implementation. If the test
    /// passes, the implementation is incomplete or the test is wrong.
    AdversarialTest,
    Review,
}

impl SubtaskKind {
    pub fn phase(&self) -> Phase {
        match self {
            Self::Comprehend => Phase::Comprehend,
            Self::TestAuthor => Phase::TestAuthor,
            Self::Implement => Phase::Implement,
            Self::Verify => Phase::Implement,
            Self::AdversarialTest => Phase::TestAuthor,
            Self::Review => Phase::Review,
        }
    }
}

/// A single step in a plan, tagged with complexity for model routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub id: String,
    pub description: String,
    /// Complexity tier — determines which model handles this subtask.
    pub tier: TaskTier,
    pub kind: SubtaskKind,
    /// Files this subtask will touch (for guard enforcement).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_criteria: Vec<String>,
}

/// A plan produced by the Planner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Backward-compatible goal string (kept for serialized JSON compat).
    pub goal: String,
    /// Typed goal statement — the free-text part of the goal.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub goal_statement: String,
    /// Acceptance criteria carried through from the [`GoalSpec`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<String>,
    pub subtasks: Vec<Subtask>,
    pub tdd: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risks: Vec<String>,
    /// Schema version for forward/backward compatibility. Old serialized
    /// plans without this field deserialize via `#[serde(default)]`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub schema_version: String,
    /// Task complexity classification — used to bound tool-loop iterations.
    #[serde(default)]
    pub complexity: TaskComplexity,
}

/// Result of executing a subtask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub subtask_id: String,
    pub status: StepStatus,
    pub output: String,
    pub usage: Usage,
    /// Per-tool-call structural signals for the grader (U4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_signals: Vec<ToolSignal>,
    /// Whether the tool loop converged naturally (model stopped calling tools)
    /// vs hitting the iteration fallback. When false, the output may be
    /// incomplete or garbled.
    #[serde(default)]
    pub converged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Ok,
    Failed,
    Skipped,
}

/// Deep understanding of a goal, grounded in deterministic context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comprehension {
    pub goal: String,
    pub summary: String,
    /// Architecture element IDs cited from sruja tools (proves grounding).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cited_elements: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_findings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risks: Vec<String>,
    pub usage: Usage,
    /// IDs of past learnings retrieved during comprehension (U3 observability).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retrieved_learning_ids: Vec<String>,
    /// Heuristic complexity classification for this goal.
    /// Controls prompt selection, TDD enforcement, and artifact generation.
    #[serde(default)]
    pub complexity: TaskComplexity,
    /// Actionable pre-condition directives derived from error history.
    /// Injected into execute prompts to prevent repeated failures.
    /// E.g., "Run cargo check before editing — high rate of compilation errors."
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_conditions: Vec<String>,
}

/// The Critic's assessment.
///
/// When produced by an ensemble ([`Agent::critique`] with a non-empty persona
/// set), `approved` is the AND of all personas, `issues` is the union, and
/// `score` is the MIN across personas (a blocking persona drags the score
/// down rather than being averaged away). `persona_breakdown` carries the
/// per-persona result for telemetry; `injected_learning_ids` records which
/// past guardrails were injected as blind-spot probes (the compounding loop).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Critique {
    pub approved: bool,
    pub score: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
    pub usage: Usage,
    /// Per-persona results when the critique was produced by an ensemble.
    /// Empty for the single-critic fallback path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub persona_breakdown: Vec<PersonaResult>,
    /// IDs of guardrail learnings injected into the critique prompt as
    /// blind-spot probes. Feeds the retrieval-counter accounting so the
    /// memory loop closes back into review, not just planning.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub injected_learning_ids: Vec<String>,
    /// Per-criterion coverage matrix (U3). Each acceptance criterion gets
    /// addressed|partial|missing; approval requires all `addressed`.
    /// Empty when no acceptance criteria are defined.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<CriterionStatus>,
    /// Source of this critique: "quick_check" when the tiered mode's fast
    /// path sufficed, "ensemble" when the full persona set ran, or empty
    /// for the legacy single-critic fallback.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
}

/// Result of a single persona critic within an ensemble.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonaResult {
    /// The persona's id (e.g. "correctness", "spec_coverage").
    pub id: String,
    pub approved: bool,
    pub score: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
}

/// Status of a single acceptance criterion in the coverage matrix (U3).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CriterionStatus {
    /// 1-based index matching the numbered criterion list.
    pub index: usize,
    /// The criterion text for traceability.
    pub criterion: String,
    /// Whether the change addresses this criterion.
    pub status: CriterionVerdict,
    /// One-line justification from the spec_coverage persona.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reason: String,
}

/// Verdict for a single acceptance criterion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CriterionVerdict {
    Addressed,
    Partial,
    Missing,
}

/// A specialized critic perspective. Each persona asks **one sharp probe**
/// rather than a generic "is this good?" verdict — different probes catch
/// different issues, and the union approaches enumerable coverage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiquePersona {
    /// Short stable id, used to tag issues (`[correctness] ...`).
    pub id: String,
    /// Human-readable focus area.
    pub focus: String,
    /// The system prompt — must contain the marker "reviewing a change" so
    /// callers can recognize critic-class requests.
    pub system_prompt: String,
    /// Optional model override; when `None`, routes to `models.review`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl CritiquePersona {
    /// Construct a persona. The marker "reviewing a change" is appended to the
    /// system prompt if missing, so critic-class requests stay recognizable.
    pub fn new(
        id: impl Into<String>,
        focus: impl Into<String>,
        system_prompt: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let focus = focus.into();
        let mut system_prompt = system_prompt.into();
        if !system_prompt.contains("reviewing a change") {
            system_prompt.push_str(" You are reviewing a change.");
        }
        Self {
            id,
            focus,
            system_prompt,
            model: None,
        }
    }

    /// The default ensemble: four probe-bound perspectives. Each asks one
    /// sharp question. The set is intentionally small (cost = N× review-model
    /// calls per critique); extend via `AgentConfig::critique_personas` or a
    /// loop manifest.
    pub fn default_personas() -> Vec<Self> {
        vec![
            Self::new(
                "correctness",
                "correctness failures and edge inputs",
                CORRECTNESS_PERSONA_PROMPT,
            ),
            Self::new(
                "spec_coverage",
                "unaddressed acceptance criteria",
                SPEC_COVERAGE_PERSONA_PROMPT,
            ),
            Self::new(
                "boundary",
                "architectural boundary crossings and drift",
                BOUNDARY_PERSONA_PROMPT,
            ),
            Self::new(
                "regression",
                "regressions of previously-working behavior",
                REGRESSION_PERSONA_PROMPT,
            ),
            Self::new(
                "adversarial_test",
                "adversarial test generation to expose implementation flaws",
                ADVERSARIAL_TEST_PERSONA_PROMPT,
            ),
        ]
    }
}

/// Error classification for pattern learning across runs.
///
/// The classifier uses deterministic pattern matching on critic issues and
/// tool output to categorize failures. This enables cross-run learning:
/// "In this repo, 40% of failures are type errors — run cargo check first."
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    /// Code doesn't compile (syntax, borrow checker, missing imports)
    Compilation,
    /// Type mismatch, trait bound not satisfied, lifetime issue
    Type,
    /// Test assertion failed (logic is wrong)
    Test,
    /// Lint/rustfmt failure (style only)
    Lint,
    /// Runtime panic, unwrap on None, index out of bounds
    Runtime,
    /// Architectural boundary violation (critic-detected)
    Architecture,
    /// Spec criterion not addressed (critic-detected)
    SpecGap,
    /// Other / unclassified
    #[default]
    Other,
}

/// Returns `true` if `content` contains common error indicators
/// (case-insensitive): `error`, `panic`, `FAILED`, `backtrace`, `stack trace`.
/// Check whether model output contains meaningful content beyond just surface text.
///
/// A model can produce text without tools (step_converged=true) but the output
/// may still be a refusal, a placeholder, or gibberish — all of which should
/// count as non-converged for quality purposes.
pub fn content_has_quality(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.len() < 30 {
        return false;
    }
    // Refusal/failure indicators that suggest the model gave up rather than
    // producing a real answer.
    let low_quality_patterns = [
        "i cannot",
        "i can't",
        "i am unable",
        "i'm unable",
        "i apologize",
        "i'm sorry",
        "i cannot complete",
        "unable to complete",
        "cannot fulfill",
    ];
    let lower = trimmed.to_lowercase();
    if low_quality_patterns.iter().any(|p| lower.contains(p)) {
        return false;
    }
    true
}

/// Structured quality gate that uses step convergence + tool signal data
/// already computed at every call site, instead of raw string heuristics alone.
///
/// This replaces ad-hoc string checks like `content.contains("ERROR")` with
/// structural signals: did the model converge naturally? Did all tools succeed?
/// Is the output non-trivial and free of refusal patterns?
pub fn step_has_quality(
    converged: bool,
    tool_signals: &[crate::tool::ToolSignal],
    content: &str,
) -> bool {
    if !converged {
        return false;
    }
    if tool_signals.iter().any(|s| !s.ok) {
        return false;
    }
    content_has_quality(content)
}

/// Prioritizes tool output (shell/stderr) for low-level errors, then critic issues for
/// high-level architectural/spec problems.
pub fn classify_error(critique_issues: &[String], step_results: &[StepResult]) -> ErrorClass {
    // Collect all tool output from step results — this is where error text lives.
    let tool_output: String = step_results
        .iter()
        .map(|s| s.output.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    // Check for Rust compilation errors (error[E0XXX] or generic "error:")
    if tool_output.contains("error[E0") || tool_output.contains("error:") {
        if tool_output.contains("mismatched types")
            || tool_output.contains("trait bound")
            || tool_output.contains("lifetime")
            || tool_output.contains("borrow checker")
        {
            return ErrorClass::Type;
        }
        return ErrorClass::Compilation;
    }

    // Test failures — check BEFORE runtime panics since test output often
    // contains "panicked" / "unwrap on None" from the test assertion itself.
    if tool_output.contains("test ... FAILED")
        || tool_output.contains("assertion failed")
        || tool_output.contains("test result: FAILED")
    {
        return ErrorClass::Test;
    }

    // Runtime panics (not from test execution).
    if tool_output.contains("panicked") || tool_output.contains("unwrap on None") {
        return ErrorClass::Runtime;
    }
    if tool_output.contains("index out of bounds") {
        return ErrorClass::Runtime;
    }

    // Critic-detected errors (check issues text for architectural / spec gaps)
    let issues_text = critique_issues.join(" ").to_lowercase();
    if issues_text.contains("boundary") || issues_text.contains("drift") {
        return ErrorClass::Architecture;
    }
    if issues_text.contains("criterion") || issues_text.contains("not addressed") {
        return ErrorClass::SpecGap;
    }

    // Check for lint failures via tool signals (sruja tool failures on lint ops)
    if step_results
        .iter()
        .any(|s| s.tool_signals.iter().any(|t| t.tool == "sruja" && !t.ok))
        && tool_output.contains("warning")
    {
        return ErrorClass::Lint;
    }

    ErrorClass::Other
}

/// Tracks failed approaches across iterations to prevent repeating mistakes.
///
/// Accumulates `(approach_summary, failure_reason, iteration, error_class)` pairs
/// and injects them into replanning prompts so the agent tries genuinely different strategies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FailureTracker {
    /// (approach_summary, failure_reason, iteration, error_class)
    pub failures: Vec<(String, String, usize, ErrorClass)>,
    /// Number of times the same approach has been tried consecutively.
    pub consecutive_same_approach: usize,
    /// The approach summary from the last iteration.
    pub last_approach: Option<String>,
    /// Optional diagnostic output from a `DiagnoseThenRetry` recovery cycle.
    pub diagnostic: Option<String>,
}

impl FailureTracker {
    pub fn record(
        &mut self,
        approach: String,
        reason: String,
        iteration: usize,
        error_class: ErrorClass,
    ) {
        if self.last_approach.as_deref() == Some(approach.as_str()) {
            self.consecutive_same_approach += 1;
        } else {
            self.consecutive_same_approach = 1;
        }
        self.last_approach = Some(approach.clone());
        self.failures
            .push((approach, reason, iteration, error_class));
    }

    /// Format failures for injection into replanning prompt.
    ///
    /// When `diagnostic` is set (from `DiagnoseThenRetry` recovery), the
    /// diagnostic is prefixed and the prompt asks the model to address it.
    pub fn format_for_prompt(&self) -> String {
        if self.failures.is_empty() {
            return String::new();
        }
        let mut out = if let Some(ref diag) = self.diagnostic {
            format!("\n\n## Diagnostic Analysis\n{diag}\n\n")
        } else {
            String::new()
        };
        out.push_str("## Previously Failed Approaches\n\n");
        out.push_str("The following approaches have been tried and failed. You MUST try a DIFFERENT strategy:\n\n");
        for (i, (approach, reason, iter, error_class)) in self.failures.iter().enumerate() {
            out.push_str(&format!(
                "{}. **Iteration {}**: {}\n   Failure reason: {}\n   Error class: {:?}\n\n",
                i + 1,
                iter,
                approach,
                reason,
                error_class
            ));
        }
        if self.consecutive_same_approach >= 2 {
            out.push_str(&format!(
                "⚠️ You have tried the same approach {} times in a row. \
                 You MUST fundamentally change your strategy — different file, \
                 different pattern, different level of abstraction.\n",
                self.consecutive_same_approach
            ));
        }
        out
    }
}

/// Runtime scope drift detection.
///
/// Tracks tool call volumes during execution and detects when the actual
/// scope exceeds the initial `TaskComplexity` classification. When drift
/// is detected, the pipeline can be escalated mid-loop (e.g., Simple → Moderate
/// adds Plan and Critique stages).
///
/// Since `ToolSignal` doesn't carry file paths, we approximate scope by
/// counting tool calls: a Simple task making 20+ tool calls is likely
/// doing more than expected.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeDrift {
    /// Total tool calls observed across the iteration.
    pub total_tool_calls: usize,
    /// Whether scope exceeded the original classification.
    pub exceeded: bool,
    /// Whether escalation has already been applied.
    pub escalated: bool,
}

impl ScopeDrift {
    /// Thresholds per complexity: max tool calls before drift is detected.
    fn threshold(complexity: TaskComplexity) -> usize {
        match complexity {
            TaskComplexity::Trivial => 3,
            TaskComplexity::Simple => 10,
            TaskComplexity::Research => 15,
            TaskComplexity::Moderate => 20,
            TaskComplexity::Complex => 40,
        }
    }

    /// Check if scope has drifted beyond the original classification.
    pub fn detect(&mut self, initial: TaskComplexity) -> bool {
        let max_calls = Self::threshold(initial);
        self.exceeded = self.total_tool_calls > max_calls;
        self.exceeded
    }

    /// Return an escalated pipeline that adds Plan and/or Critique if missing.
    pub fn escalated_stages(
        &self,
        current: &[crate::manifest::StageKind],
    ) -> Vec<crate::manifest::StageKind> {
        use crate::manifest::StageKind;
        let mut stages: Vec<StageKind> = current.to_vec();
        if !stages.contains(&StageKind::Plan) {
            if let Some(pos) = stages.iter().position(|s| *s == StageKind::Comprehend) {
                stages.insert(pos + 1, StageKind::Plan);
            }
        }
        if !stages.contains(&StageKind::Critique) {
            stages.push(StageKind::Critique);
        }
        stages
    }

    /// Record tool signals to accumulate scope metrics.
    pub fn record_tool_signals(&mut self, signals: &[ToolSignal]) {
        self.total_tool_calls += signals.len();
    }
}

/// Result of direct execution (bypasses plan/critique).
/// Complete result of a full agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunResult {
    pub goal: String,
    pub comprehension: Comprehension,
    pub plan: Plan,
    pub step_results: Vec<StepResult>,
    pub critique: Option<Critique>,
    /// Decision record explaining WHY this change was made.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<DecisionRecord>,
    /// Runbook for handling failures related to this change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runbook: Option<Runbook>,
    pub total_usage: Usage,
}

// ---------------------------------------------------------------------------
// Checkpoint: persist state for crash-resume on long-running tasks
// ---------------------------------------------------------------------------

/// Persisted state for resuming a long-running agent loop after timeout or crash.
///
/// Written to `.sruja/runs/<run_id>/checkpoint.json` after each iteration.
/// On resume, the agent loads this file and continues from the next iteration.
/// Cleaned up on successful convergence.
/// Checkpoint for saving and resuming agent loop state.
///
/// Captures the full state of a running agent loop so it can be resumed
/// after interruption (crash, timeout, user cancel). Includes the goal,
/// comprehension, plan, step results, and all tracking state.
///
/// Checkpoints are written to `.sruja/runs/<run_id>/checkpoint.json`
/// after each iteration and cleaned up on successful convergence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCheckpoint {
    /// The goal statement (for display and verification on resume).
    pub goal: String,
    /// The comprehension from the initial run (carried forward).
    pub comprehension: Comprehension,
    /// Iterations completed so far.
    pub iterations: Vec<LoopIteration>,
    /// The last plan produced (may be rejected, but needed for replanning).
    pub last_plan: Option<Plan>,
    /// Step results from the last iteration.
    pub last_steps: Vec<StepResult>,
    /// Critique from the last iteration.
    pub last_critique: Option<Critique>,
    /// Failure tracker state (what approaches failed and why).
    pub failure_tracker: FailureTracker,
    /// Total token usage accumulated so far.
    pub total_usage: Usage,
    /// Whether the loop converged.
    pub converged: bool,
    /// Termination reason.
    pub termination: LoopTermination,
    /// Issue signatures seen so far (for oscillation detection).
    pub seen_signatures: Vec<String>,
    /// Checkpoint timestamp (ISO 8601).
    pub timestamp: String,
}

impl RunCheckpoint {
    /// Save checkpoint to disk.
    pub fn write(&self, run_dir: &std::path::Path) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(run_dir)?;
        let path = run_dir.join("checkpoint.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&path, json)?;
        tracing::debug!(path = %path.display(), "checkpoint: saved");
        Ok(())
    }

    /// Load checkpoint from disk.
    pub fn load(run_dir: &std::path::Path) -> Result<Self, std::io::Error> {
        let path = run_dir.join("checkpoint.json");
        let json = std::fs::read_to_string(&path)?;
        let checkpoint: Self = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(checkpoint)
    }

    /// Delete checkpoint file (called on successful convergence).
    pub fn cleanup(run_dir: &std::path::Path) -> Result<(), std::io::Error> {
        let path = run_dir.join("checkpoint.json");
        if path.exists() {
            std::fs::remove_file(&path)?;
            tracing::debug!("checkpoint: cleaned up");
        }
        Ok(())
    }

    /// Check if a checkpoint exists for a run directory.
    pub fn exists(run_dir: &std::path::Path) -> bool {
        run_dir.join("checkpoint.json").exists()
    }
}

// ---------------------------------------------------------------------------
// Outer ReAct loop: critique -> replan until approved or budget exhausted
// ---------------------------------------------------------------------------

/// Configuration for the deterministic verifier that runs inside the loop.
///
/// The verifier is the **independent grader**: it runs after `execute` in every
/// iteration, and a failing step vetoes convergence regardless of the LLM
/// critic's verdict. Failures are injected into the next replan so the loop
/// addresses them. The workdir is supplied here (not assumed from the agent)
/// because the agent crate is intentionally repo-agnostic.
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    pub steps: Vec<VerifyStep>,
    pub options: VerifyOptions,
    pub workdir: std::path::PathBuf,
}

/// Configuration for the outer loop (`Agent::run_loop`).
///
/// `AgentConfig.max_tool_iterations` caps tool calls *within* one LLM step;
/// this caps whole plan->execute->critique *iterations* of the actor/reviewer
/// loop — the "loop engineering" spine.
#[derive(Debug, Clone)]
pub struct LoopConfig {
    /// Maximum plan->execute->critique iterations (default: 3).
    pub max_iterations: usize,
    /// Stop as soon as the critic approves (default: true).
    pub stop_on_approval: bool,
    /// Re-plan using critique feedback when the critic rejects (default: true).
    /// If false, the loop terminates after the first non-approving critique.
    pub replan_on_failure: bool,
    /// USD spend cap. The loop terminates with [`LoopTermination::SpendCapExceeded`]
    /// if the estimated cost exceeds this amount (default: None = unlimited).
    pub spend_cap_usd: Option<f64>,
    /// Detect repeated critique patterns and terminate with
    /// [`LoopTermination::Oscillation`] to avoid flailing (default: true).
    pub detect_oscillation: bool,
    /// Pipeline configuration — the explicit workflow model.
    ///
    /// Controls which stages run, in what order, and how to recover from
    /// failures. The default (three-stage: comprehend → implement → verify)
    /// matches the current `run_loop()` behavior exactly.
    pub pipeline: crate::manifest::PipelineConfig,
    /// Optional deterministic verifier — the independent grader. When set, its
    /// steps run after `execute` in every iteration. Any failure vetoes
    /// convergence (overrides the LLM critic) and feeds failures into the next
    /// replan. `None` = no deterministic gate (critic-only convergence).
    ///
    /// Note: verify failures are appended to the critique issues, so a
    /// *persistent* failure produces a stable issue signature. With the default
    /// `detect_oscillation: true` this can terminate the loop as
    /// [`LoopTermination::Oscillation`] rather than exhausting `max_iterations`
    /// — usually desirable (no point retrying a stuck failure), but set
    /// `detect_oscillation: false` if you want full retries.
    pub verifier: Option<VerifierConfig>,
    /// Directory for writing checkpoint files (resume on crash/timeout).
    /// `None` = no checkpointing (default). Set to `.sruja/runs/<run_id>/`
    /// to enable crash-resume for long-running tasks.
    pub checkpoint_dir: Option<std::path::PathBuf>,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            stop_on_approval: true,
            replan_on_failure: true,
            spend_cap_usd: None,
            detect_oscillation: true,
            pipeline: crate::manifest::PipelineConfig::default(),
            verifier: None,
            checkpoint_dir: None,
        }
    }
}

/// Why the outer loop terminated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoopTermination {
    /// The independent critic (and optional verify gate) approved.
    Approved,
    /// The iteration budget was exhausted without approval.
    MaxIterations,
    /// `replan_on_failure` was false and the critic did not approve.
    NoReplan,
    /// Spend cap exceeded — the estimated cost at termination.
    SpendCapExceeded(f64),
    /// Detected repeated critique patterns (the loop is oscillating).
    Oscillation,
    /// The model(s) failed to converge — kept calling tools without producing
    /// final answers across most subtasks. Further iterations would waste
    /// tokens. The number is the fraction of steps that did not converge.
    ModelNotConverging(f64),
    /// A hard error aborted the loop.
    Aborted(String),
}

/// Per-iteration evidence — the telemetry a host needs to detect convergence,
/// oscillation, and flailing loops.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopIteration {
    /// 1-based iteration index.
    pub iteration: usize,
    /// True if this iteration re-planned from prior critique feedback.
    pub replanned: bool,
    pub plan_goal: String,
    pub subtask_count: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub critique_approved: bool,
    pub critique_score: f64,
    /// Critic issues carried forward into the next iteration's re-plan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub critique_issues: Vec<String>,
    /// Deterministic-verify failures recorded this iteration (independent
    /// grader). Non-empty means convergence was vetoed even if the LLM critic
    /// approved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verify_failed: Vec<String>,
    /// Guardrail learning IDs injected into the critique prompt this
    /// iteration (the compounding loop: misses → memory → future review).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub injected_learning_ids: Vec<String>,
    pub usage: Usage,
    /// Plan parse error that occurred during this iteration (if any).
    /// Non-empty means the plan was malformed; the retry may have recovered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_parse_error: Option<String>,
    /// Structural incorporation gap: the replan produced a plan identical
    /// (or nearly identical) to the prior one despite non-empty critique
    /// issues. `Some(description)` means the actor ignored the critic's
    /// feedback structurally. `None` means incorporation was plausible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incorporation_gap: Option<String>,
    /// Estimated USD cost for this iteration, computed from per-model
    /// pricing when available, falling back to default flat rates.
    #[serde(default)]
    pub cost_usd: f64,
}

/// Result of `Agent::run_loop`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopResult {
    pub goal: String,
    pub iterations: Vec<LoopIteration>,
    /// True iff the critic approved before the budget ran out.
    pub converged: bool,
    pub termination: LoopTermination,
    pub total_usage: Usage,
    /// Provenance of the in-loop grader: "default" | "manifest" | "none".
    pub grader_source: String,
    /// The final, most-developed single-pass result (comprehension, last plan,
    /// last step results, last critique, decision, runbook). Present even when
    /// the loop did not converge so the caller can inspect partial progress.
    pub final_result: AgentRunResult,
}

impl LoopResult {
    /// Number of iterations actually executed.
    pub fn iteration_count(&self) -> usize {
        self.iterations.len()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

// Errors from parsing a [`Plan`] from the LLM response.
// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// The programmable agent. Holds an LLM brain, tool hands, optional memory,
/// lifecycle hooks, and a file guard enforcing the TDD pipeline.
pub struct Agent {
    llm: Arc<dyn LlmClient>,
    tools: ToolRegistry,
    guard: FileGuard,
    hooks: HookRegistry,
    config: AgentConfig,
    /// Repo root for resolving `.sruja/` paths (decisions, runbooks, memory).
    repo_root: Option<std::path::PathBuf>,
    /// Pluggable memory backend (in-memory JSON, FTS5+BM25, etc.).
    memory: Option<std::sync::Arc<dyn crate::memory::Memory + Send + Sync>>,
    #[cfg(feature = "mcp-client")]
    #[allow(dead_code)]
    mcp_manager: Option<crate::tool::mcp::McpClientManager>,
    /// Tool-call tracer for context event attribution (U5).
    tool_call_tracer: Option<Box<dyn ToolCallTracer>>,
    /// Trace context for tool-call event attribution (U5).
    trace_run_id: Option<String>,
    trace_id: Option<String>,
    /// Pre-loaded target file contents, keyed by path.
    /// Injected into the comprehension user prompt to avoid redundant file_read
    /// tool calls when --file is specified on the CLI.
    preloaded_files: std::collections::HashMap<String, String>,
    /// Pre-loaded architecture context (repomap, topology).
    /// Injected into the comprehension user prompt to avoid redundant MCP tool calls.
    preloaded_arch_context: String,
}

impl Agent {
    /// Start building an agent.
    pub fn builder() -> AgentBuilder {
        AgentBuilder::default()
    }

    /// The file guard (for external phase control, e.g. HITL gates).
    pub fn guard(&self) -> &FileGuard {
        &self.guard
    }

    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Validate that the agent's intermediate state is consistent and ready to continue.
    ///
    /// Checks include:
    /// - Phase guard is properly initialized
    /// - Tool registry is not empty (if tools are expected)
    /// - LLM client is available
    /// - Memory backend (if present) is properly configured
    ///
    /// Returns `Ok(())` if validation passes, or an `AgentError` describing the issue.
    /// Validate that the agent is in a consistent state before proceeding.
    ///
    /// Checks:
    /// 1. Tool registry is not empty (in non-dry-run mode)
    /// 2. Memory is accessible (if enabled)
    pub fn validate_intermediate_state(&self) -> Result<(), AgentError> {
        // Check tool registry
        if !self.config.dry_run && self.tools.names().is_empty() {
            return Err(AgentError::Other(
                "Tool registry is empty in non-dry-run mode".into(),
            ));
        }

        // If memory is enabled, verify it's accessible
        if let Some(ref mem) = self.memory {
            let _ = mem.search("", 0, None);
        }

        Ok(())
    }

    // --- Phase 0: Comprehension (read-only, grounded) ---

    /// Deeply understand a goal using available tools, then produce a
    /// grounded summary citing architecture element IDs.
    ///
    /// If memory is enabled, relevant past learnings are injected into the
    /// context — the agent learns from its own history.
    pub async fn comprehend(
        &self,
        goal: &crate::goal::GoalSpec,
    ) -> Result<Comprehension, AgentError> {
        self.guard.set_phase(Phase::Comprehend);
        self.hooks.on_phase_change(Phase::Comprehend).await;

        let goal_str = goal.statement.as_str();

        // Retrieve relevant memories (token-budget capped).
        let (memory_context, retrieved_learning_ids) = if let Some(ref mem) = self.memory {
            let learnings = mem.search(goal_str, 5, None);
            if learnings.is_empty() {
                (String::new(), Vec::new())
            } else {
                let ids: Vec<String> = learnings.iter().map(|l| l.id.clone()).collect();
                let entries: Vec<String> = learnings
                    .iter()
                    .map(|l| {
                        let kind = l.kind.map(|k| format!("{k:?}")).unwrap_or_default();
                        let utility = l
                            .utility_ratio()
                            .map(|u| format!("{:.0}%", u * 100.0))
                            .unwrap_or_default();
                        format!(
                            "- [{kind}] {} (utility: {utility}, retrieved {} times)\n  Advice: {}",
                            l.context, l.retrieval_count, l.guardrail_advice
                        )
                    })
                    .collect();
                (
                    format!(
                        "\n\n## Past Learnings (from previous runs)\n\
                         The following lessons were learned from earlier tasks. \
                         Use them to avoid repeating mistakes and replicate successes:\n{}",
                        entries.join("\n")
                    ),
                    ids,
                )
            }
        } else {
            (String::new(), Vec::new())
        };

        let hints = if self.config.system_hints.is_empty() {
            String::new()
        } else {
            format!(
                "\n\n## Additional Instructions\n{}",
                self.config
                    .system_hints
                    .iter()
                    .map(|h| format!("- {h}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        // Retrieve error frequency history for this repo.
        // Retrieve error frequency history for this repo.
        let (error_history, pre_conditions) = if let Some(ref mem) = self.memory {
            if let Some(repo_path) = &self.repo_root {
                let repo_path_str = repo_path.display().to_string();
                if let Ok(frequencies) = mem.search_error_history(&repo_path_str) {
                    if frequencies.is_empty() {
                        (String::new(), Vec::new())
                    } else {
                        let total: usize = frequencies.iter().map(|f| f.count).sum();
                        let mut percentages = Vec::new();
                        let mut preconds = Vec::new();
                        for f in &frequencies {
                            let pct = if total > 0 {
                                (f.count as f64 / total as f64 * 100.0) as u32
                            } else {
                                0
                            };
                            let (advice, precond) = match f.error_class {
                                ErrorClass::Compilation => (
                                    "(run cargo check first)",
                                    if pct >= 20 {
                                        Some("Run `cargo check` before editing — high rate of compilation errors in this repo.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Type => (
                                    "(check type annotations before tests)",
                                    if pct >= 20 {
                                        Some("Check type annotations and trait bounds carefully — type errors are common here.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Test => (
                                    "(verify logic against acceptance criteria)",
                                    if pct >= 20 {
                                        Some("Verify test assertions against acceptance criteria before implementing.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Runtime => (
                                    "(check for unwrap/None, bounds)",
                                    if pct >= 20 {
                                        Some("Check for unwrap/None and bounds — runtime panics are frequent in this repo.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Lint => (
                                    "(run cargo clippy)",
                                    if pct >= 20 {
                                        Some("Run `cargo clippy --fix` after changes.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Architecture => (
                                    "(check boundary crossings)",
                                    if pct >= 20 {
                                        Some("Run `sruja drift` before verification — boundary violations are common.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::SpecGap => (
                                    "(verify all criteria are addressed)",
                                    if pct >= 20 {
                                        Some("Verify all acceptance criteria are addressed before submitting.".to_string())
                                    } else {
                                        None
                                    },
                                ),
                                ErrorClass::Other => ("(investigate carefully)", None),
                            };
                            percentages.push(format!("{}% {:?} {}", pct, f.error_class, advice));
                            if let Some(pc) = precond {
                                preconds.push(pc);
                            }
                        }
                        let history = format!(
                            "\n\n## Error History for This Repo\n\
                             This repo's past agent runs had these failure patterns:\n\
                             - {}\n\
                             Focus your attention accordingly.",
                            percentages.join("\n- ")
                        );
                        (history, preconds)
                    }
                } else {
                    (String::new(), Vec::new())
                }
            } else {
                (String::new(), Vec::new())
            }
        } else {
            (String::new(), Vec::new())
        };

        let system = format!("{COMPREHENSION_SYSTEM_PROMPT}{memory_context}{error_history}{hints}");

        let preloaded_section = if self.preloaded_files.is_empty() {
            String::new()
        } else {
            let mut sections = Vec::new();
            for (path, content) in &self.preloaded_files {
                sections.push(format!("### {path}\n```\n{content}\n```"));
            }
            format!(
                "\n\n## Pre-loaded Target Files\n\
                 The following files have been provided for your reference. \
                 Do NOT call file_read for these — the content is already here.\n\n{}",
                sections.join("\n\n")
            )
        };

        // Include pre-loaded architecture context if available
        let arch_context_section = if self.preloaded_arch_context.is_empty() {
            String::new()
        } else {
            self.preloaded_arch_context.clone()
        };

        let user = format!(
            "## Goal\n{goal_str}\n\n\
             ## Instructions\n\
             Use the available tools to explore the codebase. \
             Cite architecture element IDs in your findings. \
             Produce a concise, grounded understanding.{preloaded_section}{arch_context_section}"
        );

        // Delegate exploration to an isolated Reader sub-agent.
        // The Reader gets a fresh context window with only read-only tools,
        // preventing exploration noise from poisoning later phases.
        tracing::info!("comprehend: delegating exploration to Reader sub-agent");
        let report = self
            .delegate(crate::cognition::subagent::SubAgentSpec {
                role: crate::cognition::subagent::Role::Reader,
                goal: goal.clone(),
                inject: Vec::new(),
                budget: crate::cognition::subagent::SubAgentBudget {
                    max_iterations: Some(self.config.max_tool_iterations),
                    max_summary_chars: 8000,
                },
                system_prompt: Some(system),
                user_prompt: Some(user),
            })
            .await?;

        let cited_elements = extract_element_ids(&report.summary);

        let complexity =
            classify_task_complexity(goal_str, &goal.target_files, &goal.target_elements);
        tracing::info!(?complexity, "comprehend: classified task complexity");

        let summary = report.summary;
        let final_usage = crate::llm::Usage::default();

        Ok(Comprehension {
            goal: goal.to_string(),
            summary,
            cited_elements,
            key_findings: Vec::new(),
            risks: Vec::new(),
            usage: final_usage,
            retrieved_learning_ids,
            complexity,
            pre_conditions,
        })
    }

    // --- Tool-calling loop (shared by all phases) ---

    /// Runs the main LLM tool-calling loop, repeatedly invoking the LLM and
    /// dispatching tool calls until the model stops requesting tools or the
    /// configured iteration limit is reached.
    ///
    /// # When to use
    ///
    /// This is the primary entry-point for non-streaming phases (`comprehend`,
    /// `plan`, `execute`, `reflect`, etc.).  Each phase builds a
    /// [`CompletionRequest`], hands it here, and consumes the returned
    /// response to decide what to do next.
    ///
    /// If you need a lower iteration cap for a lightweight or low-stakes task
    /// (e.g. a quick comment-only edit), call [`run_tool_loop_with_limit`]
    /// directly with a custom limit instead.
    ///
    /// # Relationship to [`run_tool_loop_with_limit`]
    ///
    /// `run_tool_loop` is a thin convenience wrapper around
    /// [`run_tool_loop_with_limit`](Self::run_tool_loop_with_limit) that
    /// forwards `self.config.max_tool_iterations` as the limit.  All
    /// iteration, convergence-pressure, and graceful-degradation logic lives
    /// in the `_with_limit` variant; this method simply picks the default.
    ///
    /// # Returns
    ///
    /// A tuple of:
    /// * **`CompletionResponse`** — the final LLM response whose content is
    ///   the answer (tool-calling responses are consumed inside the loop).
    /// * **`Usage`** — cumulative prompt, completion, and total token counts
    ///   across every LLM call made during the loop.
    /// * **`Vec<ToolSignal>`** — a per-call list of [`ToolSignal`] outcomes
    ///   (ok, empty, error, etc.) that downstream executors fold into
    ///   [`StepResult`].
    pub async fn run_tool_loop(
        &self,
        req: CompletionRequest,
    ) -> Result<(CompletionResponse, Usage, Vec<ToolSignal>), AgentError> {
        let (response, usage, signals, _converged) = self
            .run_tool_loop_with_limit(req, self.config.max_tool_iterations)
            .await?;
        Ok((response, usage, signals))
    }

    /// Run the LLM tool-calling loop with an explicit iteration limit.
    ///
    /// Use this to cap iterations for trivial tasks (e.g. max 3 for a comment
    /// change). Pass `0` or omit to use the agent's configured default.
    ///
    /// The loop is also bounded by a wall-clock timeout (`config.loop_timeout_secs`)
    /// to prevent indefinite hangs when tools or LLM calls are slow.
    pub async fn run_tool_loop_with_limit(
        &self,
        req: CompletionRequest,
        max_iterations: usize,
    ) -> Result<(CompletionResponse, Usage, Vec<ToolSignal>, bool), AgentError> {
        let timeout = std::time::Duration::from_secs(self.config.loop_timeout_secs);
        match tokio::time::timeout(timeout, self.run_tool_loop_inner(req, max_iterations)).await {
            Ok(result) => result,
            Err(_) => {
                tracing::warn!(
                    timeout_secs = self.config.loop_timeout_secs,
                    "tool_loop: wall-clock timeout exceeded"
                );
                Err(AgentError::Timeout(self.config.loop_timeout_secs))
            }
        }
    }

    /// Stream an LLM completion and dispatch tool calls the instant they arrive,
    /// overlapping tool execution with the tail of generation.
    ///
    /// This is the Claude Code "streaming tool execution" pattern: instead of
    /// blocking on a single `complete()` call and only then running tools, we
    /// open the stream, accumulate tool calls as their argument JSON completes,
    /// and run each one concurrently while the model keeps generating.
    ///
    /// Model-agnostic: every [`LlmClient`] implements `complete_stream` (the
    /// default buffers non-streaming providers into a correct event stream), so
    /// this path works on any OpenAI-compatible endpoint.
    ///
    /// Returns the fully reassembled final [`CompletionResponse`] plus the
    /// accumulated tool results keyed by tool-call id in arrival order.
    async fn stream_and_dispatch(
        &self,
        req: CompletionRequest,
    ) -> Result<
        (
            CompletionResponse,
            Usage,
            Vec<(
                String,
                String,
                serde_json::Value,
                Result<(String, crate::tool::ToolCallRecord), crate::tool::ToolError>,
            )>,
        ),
        AgentError,
    > {
        use crate::llm::stream::StreamEvent;
        use futures::StreamExt;

        let model = req.model.clone().unwrap_or_default();
        let mut stream = self.llm.complete_stream(&req);

        // Accumulators for reassembly.
        let mut content = String::new();
        let mut usage = Usage::default();
        let mut finish_reason = crate::llm::FinishReason::Stop;

        // Tool-call assembly buffers (args arrive as JSON-string fragments).
        let mut accs: std::collections::BTreeMap<usize, (Option<String>, Option<String>, String)> =
            Default::default();
        // Dispatched tasks keyed by call id.
        let mut dispatched: std::collections::HashMap<
            String,
            (
                String,
                serde_json::Value,
                tokio::task::JoinHandle<
                    Result<(String, crate::tool::ToolCallRecord), crate::tool::ToolError>,
                >,
            ),
        > = std::collections::HashMap::new();

        while let Some(event) = stream.next().await {
            let event = event.map_err(|e| AgentError::Other(e.to_string()))?;
            match event {
                StreamEvent::ContentDelta(s) => content.push_str(&s),
                StreamEvent::ToolCallStart { index, id, name } => {
                    let entry = accs.entry(index).or_default();
                    entry.0 = Some(id);
                    entry.1 = Some(name);
                }
                StreamEvent::ToolCallArguments { index, fragment } => {
                    let entry = accs.entry(index).or_default();
                    entry.2.push_str(&fragment);
                    // Dispatch the moment args parse as valid complete JSON.
                    if let Some((id, name, buf)) = accs.get(&index).map(|e| {
                        (
                            e.0.clone().unwrap_or_default(),
                            e.1.clone().unwrap_or_default(),
                            e.2.clone(),
                        )
                    }) {
                        if !dispatched.contains_key(&id) {
                            if let Ok(args) = serde_json::from_str::<serde_json::Value>(&buf) {
                                let name_c = name.clone();
                                let args_c = args.clone();
                                let id_c = id.clone();
                                let tools = self.tools.clone();
                                let handle = tokio::spawn(async move {
                                    tools.dispatch_record(&name_c, args_c).await
                                });
                                dispatched.insert(id_c, (name, args, handle));
                            }
                        }
                    }
                }
                StreamEvent::Usage(u) => usage = u,
                StreamEvent::Finish { finish_reason: fr } => {
                    finish_reason = fr;
                    // Finalize any tool call whose args never parsed mid-stream.
                    for (id, name, buf) in accs.values() {
                        let id = id.clone().unwrap_or_default();
                        if dispatched.contains_key(&id) {
                            continue;
                        }
                        let args = serde_json::from_str(buf).unwrap_or(serde_json::json!({}));
                        let name_c = name.clone().unwrap_or_default();
                        let args_c = args.clone();
                        let id_c = id.clone();
                        let tools = self.tools.clone();
                        let name_in_closure = name_c.clone();
                        let handle = tokio::spawn(async move {
                            tools.dispatch_record(&name_in_closure, args_c).await
                        });
                        dispatched.insert(id_c, (name_c, args, handle));
                    }
                }
            }
        }

        // Wait for all dispatched tool tasks.
        let mut tool_results = Vec::new();
        for (id, (_name, args, handle)) in dispatched.into_iter() {
            let name = _name;
            let result = handle
                .await
                .map_err(|e| AgentError::Other(format!("tool task join: {e}")))?;
            tool_results.push((id, name, args, result));
        }

        // Reassemble tool_calls in index order for the final response.
        let mut ordered: Vec<(usize, crate::llm::ToolCall)> = Vec::new();
        for (idx, (id, name, buf)) in accs {
            let arguments = serde_json::from_str(&buf).unwrap_or(serde_json::json!({}));
            ordered.push((
                idx,
                crate::llm::ToolCall {
                    id: id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                    arguments,
                },
            ));
        }
        ordered.sort_by_key(|(idx, _)| *idx);
        let tool_calls = ordered.into_iter().map(|(_, tc)| tc).collect();

        let response = CompletionResponse {
            content,
            tool_calls,
            usage: usage.clone(),
            model,
            finish_reason,
        };

        Ok((response, usage, tool_results))
    }

    /// Inner implementation of the tool loop (extracted for timeout wrapping).
    ///
    /// Returns `(response, usage, tool_signals, converged)`. When `converged`
    /// is false, the model never stopped calling tools on its own and the
    /// fallback path was taken — the output may be incomplete.
    async fn run_tool_loop_inner(
        &self,
        mut req: CompletionRequest,
        max_iterations: usize,
    ) -> Result<(CompletionResponse, Usage, Vec<ToolSignal>, bool), AgentError> {
        let mut total_usage = Usage::default();
        let mut tool_signals: Vec<ToolSignal> = Vec::new();
        let mut last_response: Option<CompletionResponse> = None;
        let mut soft_sent = false;
        let mut hard_sent = false;
        // Circuit breaker: track consecutive tool errors and inject recovery.
        let mut consecutive_errors: usize = 0;
        let mut recovery_sent = false;
        // Track consecutive iterations where the model only called tools
        // (no meaningful content). After 3, abort early — the model is
        // stuck in a tool-calling loop it won't exit.
        let mut consecutive_tool_only: usize = 0;
        // Track repeated tool+arg signatures to detect stuck loops
        // even when the model produces some text. Same tool with same
        // args 3+ times = loop.
        let mut last_tool_signature: Option<String> = None;
        let mut consecutive_same_tool_call: usize = 0;
        // Progress tracking: track actual changes made
        let mut file_changes: usize = 0;
        let mut successful_tool_calls: usize = 0;
        let mut last_file_change_iteration: Option<usize> = None;

        for iteration in 0..max_iterations {
            // Stream the completion and dispatch tool calls as they arrive,
            // overlapping tool execution with the tail of generation.
            let (response, _usage, tool_results) = self.stream_and_dispatch(req.clone()).await?;
            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            total_usage.total_tokens += response.usage.total_tokens;

            let tool_names: Vec<&str> = response
                .tool_calls
                .iter()
                .map(|c| c.name.as_str())
                .collect();
            let content_preview: String = response.content.chars().take(120).collect();
            tracing::info!(
                iteration,
                finish_reason = ?response.finish_reason,
                tool_calls = tool_names.len(),
                tool_names = ?tool_names,
                content_preview = %content_preview,
                "tool_loop: LLM response"
            );
            // Termination: stop when the model stops requesting tools.
            // Also stop when finish_reason is Stop — some OpenAI-compatible
            // servers emit tool_calls alongside a "stop" finish_reason.
            if !response.wants_tools() || response.finish_reason == crate::llm::FinishReason::Stop {
                if response.wants_tools() {
                    tracing::warn!(
                        iteration,
                        tool_names = ?tool_names,
                        "tool_loop: finish_reason=Stop but tool_calls present — \
                         treating content as final answer (server quirk)"
                    );
                    let mut response = response;
                    response.tool_calls.clear();
                    return Ok((response, total_usage, tool_signals, true));
                }
                return Ok((response, total_usage, tool_signals, true));
            }

            // Track tool-only iterations for early-abort.
            if response.content.trim().is_empty() {
                consecutive_tool_only += 1;
            } else {
                consecutive_tool_only = 0;
            }
            if self.config.max_consecutive_tool_only > 0
                && consecutive_tool_only >= self.config.max_consecutive_tool_only
            {
                // Check if we're making progress (file changes)
                let recent_file_change = last_file_change_iteration
                    .map(|last| iteration.saturating_sub(last) <= 2)
                    .unwrap_or(false);

                if recent_file_change || file_changes > 0 {
                    // We're making progress, don't abort
                    tracing::info!(
                        iteration,
                        consecutive_tool_only,
                        file_changes,
                        "tool_loop: model making progress despite tool-only iterations"
                    );
                    consecutive_tool_only = 0; // Reset counter
                } else {
                    tracing::warn!(
                        iteration,
                        consecutive_tool_only,
                        file_changes,
                        "tool_loop: model has called tools 3+ times with no output — aborting early"
                    );
                    let mut fallback = last_response.unwrap_or_else(|| {
                        CompletionResponse::text(
                            "ERROR: model stuck in tool-calling loop with no output.",
                        )
                    });
                    fallback.tool_calls.clear();
                    fallback.finish_reason = crate::llm::FinishReason::Stop;
                    return Ok((fallback, total_usage, tool_signals, false));
                }
            }

            last_response = Some(response.clone());

            // Push the assistant's tool-call message.
            req.messages.push(Message {
                role: crate::llm::MessageRole::Assistant,
                content: response.content.clone(),
                tool_calls: response.tool_calls.clone(),
                tool_call_id: None,
            });

            // Tool calls were already dispatched (concurrently, as they streamed
            // in). Map the streamed results into the per-call handling below.
            let call_id_to_result: std::collections::HashMap<
                String,
                Result<(String, crate::tool::ToolCallRecord), crate::tool::ToolError>,
            > = tool_results
                .into_iter()
                .map(|(id, _name, _args, res)| (id, res))
                .collect();

            for (_call_idx, call) in response.tool_calls.iter().enumerate() {
                let result = call_id_to_result.get(&call.id);
                let (result, record) = match result {
                    Some(Ok(ok)) => ok.clone(),
                    Some(Err(e)) => {
                        let record = crate::tool::ToolCallRecord {
                            ok: false,
                            empty: false,
                            elapsed_ms: 0,
                            source: crate::tool::ToolRegistry::classify_source(&call.name),
                            truncated: false,
                            payload: e.to_string(),
                        };
                        (format!("ERROR: {e}"), record)
                    }
                    None => {
                        let record = crate::tool::ToolCallRecord {
                            ok: false,
                            empty: false,
                            elapsed_ms: 0,
                            source: crate::tool::ToolRegistry::classify_source(&call.name),
                            truncated: false,
                            payload: "streamed tool result missing".to_string(),
                        };
                        ("ERROR: streamed tool result missing".to_string(), record)
                    }
                };
                tracing::debug!(
                    tool = %call.name,
                    args_preview = %call.arguments.to_string().chars().take(200).collect::<String>(),
                    "tool_loop: processing tool result"
                );
                // U5: emit tool_call event before dispatch (when tracing enabled).
                if self.config.enable_tool_call_tracing {
                    if let (
                        Some(ref tracer),
                        Some(ref repo),
                        Some(ref run_id),
                        Some(ref trace_id),
                    ) = (
                        &self.tool_call_tracer,
                        &self.repo_root,
                        &self.trace_run_id,
                        &self.trace_id,
                    ) {
                        let args_keys: Vec<String> = call
                            .arguments
                            .as_object()
                            .map(|m| {
                                let mut keys = m.keys().cloned().collect::<Vec<_>>();
                                keys.sort();
                                keys
                            })
                            .unwrap_or_default();
                        tracer.on_tool_call(repo, run_id, trace_id, &call.name, &args_keys);
                    }
                }

                let truncated_text = truncate(&result, 8_000);
                let was_truncated = result.len() > 8_000;
                let mut record = record;
                if was_truncated {
                    record.truncated = true;
                }
                tracing::debug!(
                    tool = %call.name,
                    result_len = truncated_text.len(),
                    result_preview = %truncated_text.chars().take(120).collect::<String>(),
                    "tool_loop: tool result"
                );
                tool_signals.push(ToolSignal {
                    tool: call.name.clone(),
                    ok: record.ok,
                    empty: record.empty,
                    elapsed_ms: record.elapsed_ms,
                    source: record.source,
                });

                // Track consecutive errors for circuit breaker.
                if !record.ok {
                    consecutive_errors += 1;
                } else {
                    consecutive_errors = 0;
                    successful_tool_calls += 1;
                }

                // Track file changes for progress detection
                if record.ok
                    && (call.name == "file_write"
                        || call.name == "file_edit"
                        || call.name == "diff_edit")
                {
                    file_changes += 1;
                    last_file_change_iteration = Some(iteration);
                    tracing::debug!(iteration, file_changes, "tool_loop: file change detected");
                }

                // U5: emit tool_result event after dispatch (when tracing enabled).
                if self.config.enable_tool_call_tracing {
                    if let (
                        Some(ref tracer),
                        Some(ref repo),
                        Some(ref run_id),
                        Some(ref trace_id),
                    ) = (
                        &self.tool_call_tracer,
                        &self.repo_root,
                        &self.trace_run_id,
                        &self.trace_id,
                    ) {
                        tracer.on_tool_result(
                            repo,
                            run_id,
                            trace_id,
                            &call.name,
                            record.ok,
                            record.empty,
                            record.elapsed_ms,
                        );
                    }
                }

                req.messages
                    .push(Message::tool_result(&call.id, truncated_text));
            }

            // ── Tool call signature tracking ────────────────────────────
            // Detect repeated tool+arg patterns to distinguish productive
            // exploration from stuck loops. Build a signature from (tool_name, arg_keys)
            // pairs — if the same signature appears 3+ times in a row, abort.
            {
                let sig: String = response
                    .tool_calls
                    .iter()
                    .map(|call| {
                        let keys: Vec<String> = call
                            .arguments
                            .as_object()
                            .map(|m| {
                                let mut k: Vec<String> = m.keys().cloned().collect();
                                k.sort();
                                k
                            })
                            .unwrap_or_default();
                        format!("{}:{:?}", call.name, keys)
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                if last_tool_signature.as_deref() == Some(&sig) {
                    consecutive_same_tool_call += 1;
                } else {
                    consecutive_same_tool_call = 0;
                }
                last_tool_signature = Some(sig);
                if self.config.max_consecutive_same_call > 0
                    && consecutive_same_tool_call >= self.config.max_consecutive_same_call
                {
                    // Check if we're making progress (file changes)
                    let recent_file_change = last_file_change_iteration
                        .map(|last| iteration.saturating_sub(last) <= 2)
                        .unwrap_or(false);

                    if recent_file_change || file_changes > 0 {
                        // We're making progress, don't abort
                        tracing::info!(
                            iteration,
                            consecutive_same_tool_call,
                            file_changes,
                            "tool_loop: same tool+args called but making progress"
                        );
                        consecutive_same_tool_call = 0; // Reset counter
                    } else {
                        tracing::warn!(
                            iteration,
                            consecutive_same_tool_call,
                            "tool_loop: same tool+args called 3+ times in a row — aborting"
                        );
                        let mut fallback = last_response.unwrap_or_else(|| {
                            CompletionResponse::text(
                                "ERROR: model stuck repeating the same tool call with same arguments.",
                            )
                        });
                        fallback.tool_calls.clear();
                        fallback.finish_reason = crate::llm::FinishReason::Stop;
                        return Ok((fallback, total_usage, tool_signals, false));
                    }
                }
            }

            // ── Circuit breaker: consecutive error recovery ──────────────
            // Self-Harness paper: "tool-error-triggered recovery injection"
            // When the model hits 3 consecutive tool failures, inject a
            // redirect: diagnose, try different approach, don't abandon work.
            if consecutive_errors >= 3 && !recovery_sent {
                recovery_sent = true;
                tracing::warn!(
                    iteration,
                    consecutive_errors,
                    "tool_loop: injecting error-recovery message (circuit breaker)"
                );
                req.messages.push(Message::user(
                    "You have had 3 consecutive tool errors. STOP and diagnose:\n\
                     1. What exactly is going wrong? Read the error message carefully.\n\
                     2. Is the file path correct? Does the file exist?\n\
                     3. Try a completely different approach.\n\
                     Do NOT retry the same command. Do NOT delete files.\n\
                     If you cannot fix the error, make your best attempt with \
                     what you know and write your final answer.",
                ));
                // Reset counter so recovery gets a fair chance.
                consecutive_errors = 0;
            }

            // ── Progress-based recovery injection ──────────────────────
            // If the model has been calling tools for a while without making
            // file changes, inject a message to help it get unstuck.
            if iteration >= 3 && file_changes == 0 && successful_tool_calls >= 3 {
                let tool_names: Vec<&str> = tool_signals
                    .iter()
                    .skip(tool_signals.len().saturating_sub(3))
                    .map(|s| s.tool.as_str())
                    .collect();
                let has_read = tool_names.iter().any(|t| t.contains("read"));
                let has_write = tool_names
                    .iter()
                    .any(|t| t.contains("write") || t.contains("edit"));

                if has_read && !has_write {
                    tracing::info!(
                        iteration,
                        successful_tool_calls,
                        file_changes,
                        "tool_loop: injecting progress recovery message"
                    );
                    req.messages.push(Message::user(
                        "You have been reading files but haven't made any changes yet. \
                         The goal requires code changes. Please:\n\
                         1. Identify the file(s) that need to be modified\n\
                         2. Use diff_edit, file_edit, or file_write to make the changes\n\
                         3. Don't just keep reading — take action!\n\
                         If you're unsure, make your best attempt and move on.",
                    ));
                }
            }

            // ── Tiered convergence pressure ─────────────────────────────
            // Two-stage pressure: soft reminder at 50% remaining, hard
            // cutoff at 25% remaining. This gives models a chance to wrap
            // up gradually instead of a one-shot ultimatum.
            let remaining = max_iterations - iteration - 1;
            let quarter = (max_iterations / 4).max(1);
            let half = (max_iterations / 2).max(1);
            if remaining > 0 && remaining <= quarter && !hard_sent {
                hard_sent = true;
                tracing::warn!(
                    iteration,
                    remaining,
                    "tool_loop: injecting hard convergence message"
                );
                req.messages.push(Message::user(
                    prompts::CONVERGENCE_HARD,
                ));
            } else if remaining > 0 && remaining <= half && !soft_sent && !hard_sent {
                soft_sent = true;
                tracing::info!(
                    iteration,
                    remaining,
                    "tool_loop: injecting soft convergence reminder"
                );
                req.messages.push(Message::user(
                    prompts::CONVERGENCE_SOFT,
                ));
            }
        }

        // Graceful degradation: the model didn't self-terminate.
        tracing::warn!(
            max_iterations,
            "tool_loop: model did not converge — returning last response as fallback"
        );
        let mut fallback = last_response.unwrap_or_else(|| {
            CompletionResponse::text(
                "ERROR: tool loop exhausted without any response from the model.",
            )
        });
        fallback.tool_calls.clear();
        fallback.finish_reason = crate::llm::FinishReason::Stop;
        Ok((fallback, total_usage, tool_signals, false))
    }

    /// Resolve the model name configured for a complexity tier.
    fn model_for_tier(&self, tier: TaskTier) -> &str {
        match tier {
            TaskTier::Cheap => &self.config.models.cheap,
            TaskTier::Mid => &self.config.models.mid,
            TaskTier::Premium => &self.config.models.premium,
        }
    }

    /// Route a request to the model configured for a specific tier.
    pub async fn complete_tiered(
        &self,
        tier: TaskTier,
        req: CompletionRequest,
    ) -> Result<CompletionResponse, AgentError> {
        let model = self.model_for_tier(tier);
        let req = CompletionRequest {
            model: Some(model.to_string()),
            ..req
        };
        Ok(self.llm.complete(&req).await?)
    }

    // --- Plan: complexity-tagged subtask decomposition ---

    /// Produce a plan with complexity-tagged subtasks.
    ///
    /// If `config.tdd` is true, test subtasks always precede implementation
    /// subtasks (TDD pipeline).
    pub async fn plan(
        &self,
        goal: &crate::goal::GoalSpec,
        comprehension: &Comprehension,
    ) -> Result<Plan, AgentError> {
        let goal_str = goal.statement.as_str();
        if let HookAction::Abort(reason) = self.hooks.before_plan(goal_str).await {
            return Err(AgentError::HookAborted(reason));
        }

        // Complexity-aware: skip TDD for trivial tasks even if config.tdd is on.
        let enforce_tdd = self.config.tdd && comprehension.complexity.enforce_tdd();

        let tdd_note = if enforce_tdd {
            "\n\nTDD MODE IS ON: You MUST emit test_author subtasks BEFORE any implement subtasks. \
             The framework enforces this — tests are written first, reviewed, then code is written \
             to pass the frozen tests. Tests and code are NEVER in flux simultaneously."
        } else {
            ""
        };

        // Complexity-aware prompt selection.
        let (system_prompt, plan_instructions) = match comprehension.complexity {
            TaskComplexity::Trivial => (
                PLAN_TRIVIAL_SYSTEM_PROMPT,
                "This is a trivial change (e.g. comment, typo, rename, format). \
                 Output a SINGLE implement subtask that directly makes the change. \
                 Do NOT add test, verify, or review subtasks. \
                 Do NOT call any tools — just output the plan JSON.\n",
            ),
            TaskComplexity::Simple => (
                PLAN_SYSTEM_PROMPT,
                "Break this goal into 1-2 concrete subtasks. Keep it minimal.\n",
            ),
            _ => (
                PLAN_SYSTEM_PROMPT,
                "Break this goal into concrete subtasks. Each subtask must specify:\n\
                 - `id`: a short unique identifier (e.g. \"s1\", \"s2\")\n\
                 - `description`: what to do (concise, actionable)\n\
                 - `tier`: cheap (classification/extraction), mid (standard coding), \
                   or premium (hard architecture reasoning)\n\
                 - `kind`: test_author, implement, verify, or review\n\
                 - `files`: list of files this subtask touches\n\
                 - `acceptance_criteria`: how to verify completion\n\n",
            ),
        };

        let user = format!(
            "## Goal\n{goal_str}\n\n\
             ## Comprehension\n{}\n\n\
             ## Architecture Elements Cited\n{:?}\n\n\
             ## Instructions\n\
             {plan_instructions}\
             Output a JSON object with `subtasks` array and `risks` array.\n\
             {tdd_note}",
            comprehension.summary, comprehension.cited_elements,
        );

        let mut req = CompletionRequest::prompt(system_prompt, user);
        // For trivial plans, do not attach tools — the prompt says "Do NOT
        // call any tools" and attaching schemas causes the model to ignore
        // that instruction and explore indefinitely.
        if !matches!(comprehension.complexity, TaskComplexity::Trivial) {
            req = req.with_tools(self.tools.schemas());
        }

        let max_iters = match comprehension.complexity {
            TaskComplexity::Trivial => 3,
            TaskComplexity::Simple => 5,
            _ => self.config.max_tool_iterations,
        };
        let (response, _usage, _signals, _converged) =
            self.run_tool_loop_with_limit(req, max_iters).await?;

        // Parse the plan from the LLM response.
        match parse_plan_from_response(&response.content, goal, enforce_tdd) {
            Ok(plan) => {
                tracing::warn!(
                    response_len = response.content.len(),
                    subtask_count = plan.subtasks.len(),
                    risks = plan.risks.len(),
                    "plan:parsed"
                );
                if plan.subtasks.is_empty() {
                    tracing::warn!(
                        raw_response = %response.content.chars().take(2000).collect::<String>(),
                        "plan:empty — model returned 0 subtasks"
                    );
                }

                let mut plan = plan;
                plan.complexity = comprehension.complexity;
                if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
                    return Err(AgentError::HookAborted(reason));
                }

                Ok(plan)
            }
            Err(parse_err) => {
                // Format-correction retry: issue one re-prompt with the parse reason.
                tracing::warn!(error = %parse_err, "plan:parse_failed — issuing correction re-prompt");
                let correction_user = format!(
                    "## Previous plan (rejected)\n{}\n\n\
                     ## Parse error\n{parse_err}\n\n\
                     ## Instructions\n\
                     The plan JSON above was rejected because: {parse_err}.\n\
                     Re-emit a VALID plan JSON. Each subtask MUST have `id`, `description`, \
                     `tier`, and `kind` fields. Output a JSON object with `subtasks` array \
                     and `risks` array.",
                    response.content,
                );
                let correction_req = CompletionRequest::prompt(system_prompt, correction_user);
                let correction_req = if matches!(comprehension.complexity, TaskComplexity::Trivial)
                {
                    correction_req
                } else {
                    correction_req.with_tools(self.tools.schemas())
                };
                let (retry_response, _retry_usage, _signals, _converged) = self
                    .run_tool_loop_with_limit(correction_req, max_iters)
                    .await?;
                let plan = parse_plan_from_response(&retry_response.content, goal, enforce_tdd)
                    .map_err(AgentError::PlanParseFailed)?;

                tracing::warn!(
                    response_len = retry_response.content.len(),
                    subtask_count = plan.subtasks.len(),
                    risks = plan.risks.len(),
                    "plan:parsed_after_correction"
                );

                let mut plan = plan;
                plan.complexity = comprehension.complexity;
                if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
                    return Err(AgentError::HookAborted(reason));
                }

                Ok(plan)
            }
        }
    }

    /// Re-plan using the prior critique as feedback.
    ///
    /// This is the feedback edge that closes the outer ReAct loop: when the
    /// independent critic rejects a change, its `issues` and `suggestions`
    /// are injected into a new plan rather than discarded.
    ///
    /// The `failure_tracker` accumulates failed approaches across iterations
    /// and injects them into the replanning prompt so the agent tries
    /// genuinely different strategies instead of repeating mistakes.
    pub async fn replan(
        &self,
        goal: &crate::goal::GoalSpec,
        comprehension: &Comprehension,
        critique: &Critique,
        convergence_pressure: Option<&str>,
        failure_tracker: &FailureTracker,
    ) -> Result<Plan, AgentError> {
        let goal_str = goal.statement.as_str();
        if let HookAction::Abort(reason) = self.hooks.before_plan(goal_str).await {
            return Err(AgentError::HookAborted(reason));
        }

        let tdd_note = if self.config.tdd && comprehension.complexity.enforce_tdd() {
            "\n\nTDD MODE IS ON: keep test_author subtasks BEFORE implement subtasks."
        } else {
            ""
        };

        let pressure_note = if let Some(pressure) = convergence_pressure {
            format!(
                "\n\n## CONVERGENCE PRESSURE\n\
                 The previous replan was flagged: {pressure}.\n\
                 You MUST change the subtasks or risks to address the critic's issues. \
                 Emitting an identical plan will be flagged again."
            )
        } else {
            String::new()
        };

        let plan_instructions: &str = match comprehension.complexity {
            TaskComplexity::Trivial => {
                "This is a trivial change (e.g. comment, typo, rename, format). \
                 Output a SINGLE implement subtask that directly makes the change. \
                 Do NOT add test, verify, or review subtasks. \
                 Do NOT call any tools — just output the plan JSON.\n"
            }
            _ => "Produce a revised plan that addresses the critic's feedback.\n",
        };
        let system_prompt: &str = match comprehension.complexity {
            TaskComplexity::Trivial => PLAN_TRIVIAL_SYSTEM_PROMPT,
            _ => PLAN_SYSTEM_PROMPT,
        };

        let failure_context = failure_tracker.format_for_prompt();

        // Structured critique JSON instead of flat text soup.
        let critique_json = serde_json::to_string_pretty(&serde_json::json!({
            "approved": critique.approved,
            "score": critique.score,
            "issues": critique.issues,
            "suggestions": critique.suggestions,
            "persona_breakdown": critique.persona_breakdown.iter().map(|p| {
                serde_json::json!({
                    "persona_id": p.id,
                    "approved": p.approved,
                    "issues": p.issues,
                })
            }).collect::<Vec<_>>(),
            "criteria_matrix": critique.criteria.iter().map(|c| {
                serde_json::json!({
                    "index": c.index,
                    "criterion": c.criterion,
                    "status": c.status,
                    "reason": c.reason,
                })
            }).collect::<Vec<_>>(),
        }))
        .unwrap_or_else(|_| "{}".to_string());

        let user = format!(
            "## Goal\n{goal_str}\n\n\
             ## Comprehension\n{}\n\n\
             ## Prior Review Outcome (Structured)\n\
             The independent critic REJECTED the previous attempt (score: {:.0}%).\n\
             Below is the full structured critique — each issue is tagged with\n\
             its originating persona, and the criteria matrix shows which\n\
             acceptance criteria are missing or partial.\n\n\
             ```json\n{critique_json}\n```\n\
             {failure_context}\
             ## Instructions\n\
             {plan_instructions}\
             Output a JSON object with `subtasks` array and `risks` array. \
             Do not repeat failed approaches. Try a DIFFERENT strategy. \
             Each criterion marked `missing` or `partial` in the criteria matrix \
             MUST be addressed by new subtasks.{tdd_note}{pressure_note}",
            comprehension.summary,
            critique.score * 100.0,
        );

        let mut req = CompletionRequest::prompt(system_prompt, user);
        if !matches!(comprehension.complexity, TaskComplexity::Trivial) {
            req = req.with_tools(self.tools.schemas());
        }

        let max_iters = match comprehension.complexity {
            TaskComplexity::Trivial => 3,
            TaskComplexity::Simple => 5,
            _ => self.config.max_tool_iterations,
        };
        let (response, _usage, _signals, _converged) =
            self.run_tool_loop_with_limit(req, max_iters).await?;

        match parse_plan_from_response(
            &response.content,
            goal,
            self.config.tdd && comprehension.complexity.enforce_tdd(),
        ) {
            Ok(plan) => {
                tracing::warn!(
                    response_len = response.content.len(),
                    subtask_count = plan.subtasks.len(),
                    "replan:parsed"
                );
                if plan.subtasks.is_empty() {
                    tracing::warn!(
                        raw_response = %response.content.chars().take(2000).collect::<String>(),
                        "replan:empty — model returned 0 subtasks"
                    );
                }

                let mut plan = plan;
                plan.complexity = comprehension.complexity;
                if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
                    return Err(AgentError::HookAborted(reason));
                }
                Ok(plan)
            }
            Err(parse_err) => {
                // Format-correction retry: issue one re-prompt with the parse reason.
                tracing::warn!(error = %parse_err, "replan:parse_failed — issuing correction re-prompt");
                let correction_user = format!(
                    "## Previous plan (rejected)\n{}\n\n\
                     ## Parse error\n{parse_err}\n\n\
                     ## Instructions\n\
                     The plan JSON above was rejected because: {parse_err}.\n\
                     Re-emit a VALID plan JSON. Each subtask MUST have `id`, `description`, \
                     `tier`, and `kind` fields. Output a JSON object with `subtasks` array \
                     and `risks` array.",
                    response.content,
                );
                let correction_req = CompletionRequest::prompt(system_prompt, correction_user);
                let correction_req = if matches!(comprehension.complexity, TaskComplexity::Trivial)
                {
                    correction_req
                } else {
                    correction_req.with_tools(self.tools.schemas())
                };
                let (retry_response, _retry_usage, _signals, _converged) = self
                    .run_tool_loop_with_limit(correction_req, max_iters)
                    .await?;
                let plan = parse_plan_from_response(
                    &retry_response.content,
                    goal,
                    self.config.tdd && comprehension.complexity.enforce_tdd(),
                )
                .map_err(AgentError::PlanParseFailed)?;

                tracing::warn!(
                    response_len = retry_response.content.len(),
                    subtask_count = plan.subtasks.len(),
                    "replan:parsed_after_correction"
                );

                let mut plan = plan;
                plan.complexity = comprehension.complexity;
                if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
                    return Err(AgentError::HookAborted(reason));
                }
                Ok(plan)
            }
        }
    }

    // --- Execute: run subtasks with phase enforcement ---

    /// Execute a plan, enforcing the TDD phase pipeline.
    ///
    /// Each subtask transitions the [`FileGuard`] to the appropriate phase,
    /// ensuring tests and code are never in flux simultaneously.
    pub async fn execute(&self, plan: &Plan) -> Result<Vec<StepResult>, AgentError> {
        let mut results = Vec::new();

        for step in &plan.subtasks {
            // Transition the file guard to this subtask's phase.
            let phase = step.kind.phase();
            self.guard.set_phase(phase);
            self.hooks.on_phase_change(phase).await;

            // Hook gate.
            match self.hooks.before_step(step).await {
                HookAction::Skip => {
                    results.push(StepResult {
                        subtask_id: step.id.clone(),
                        status: StepStatus::Skipped,
                        output: "skipped by hook".into(),
                        usage: Usage::default(),
                        tool_signals: Vec::new(),
                        converged: true,
                    });
                    continue;
                }
                HookAction::Abort(reason) => {
                    return Err(AgentError::HookAborted(reason));
                }
                HookAction::Continue => {}
            }

            // Route to the appropriate model tier. The tool loop runs on the
            // tiered model so mutations respect the configured complexity
            // routing — no separate one-shot call beforehand.
            let tier = step.tier;
            let system = EXECUTION_SYSTEM_PROMPT;
            let execute_instruction = match plan.complexity {
                TaskComplexity::Trivial => {
                    "Make the change directly. Read the file, make the edit, and stop. \
                     Do NOT read other files. Do NOT explore."
                }
                _ => {
                    "Execute this subtask using the available tools. \
                     Be precise. Cite evidence."
                }
            };
            let user = format!(
                "## Subtask: {}\n\n\
                 ## Description\n{}\n\n\
                 ## Acceptance Criteria\n{}\n\n\
                 ## Phase\n{:?}\n\n\
                 {execute_instruction}",
                step.id,
                step.description,
                step.acceptance_criteria.join("\n"),
                phase,
            );

            let mut req = CompletionRequest::prompt(system, user).with_tools(self.tools.schemas());
            req.model = Some(self.model_for_tier(tier).to_string());

            let max_iters = match plan.complexity {
                TaskComplexity::Trivial => 5,
                TaskComplexity::Simple => 8,
                _ => self.config.max_tool_iterations,
            };
            let (response, tool_usage, tool_signals, step_converged) =
                self.run_tool_loop_with_limit(req, max_iters).await?;

            let status = if response.content.contains("ERROR")
                || tool_signals.iter().any(|s| !s.ok || s.empty)
            {
                StepStatus::Failed
            } else {
                StepStatus::Ok
            };

            let result = StepResult {
                subtask_id: step.id.clone(),
                status,
                output: response.content,
                usage: tool_usage,
                tool_signals,
                converged: step_converged,
            };

            self.hooks.after_step(step, &result).await;
            results.push(result);
        }

        // After all subtasks, reset to Comprehend phase.
        self.guard.set_phase(Phase::Comprehend);

        Ok(results)
    }

    // --- Critique: review every change via the review model ---

    /// Review changes via the configured critic ensemble.
    ///
    /// When `config.critique_personas` is non-empty, this fans out N
    /// probe-bound persona critics in parallel and merges them:
    /// - `approved` = AND of all personas (one blocker vetoes).
    /// - `issues`   = union, deduped, sorted (deterministic output).
    /// - `score`    = MIN across personas (a blocking persona drags the score
    ///   down, never averaged away).
    ///
    /// When `config.critique_personas` is empty, falls back to a single call
    /// with the legacy [`CRITIQUE_SYSTEM_PROMPT`] (backward compatible).
    ///
    /// Past guardrail learnings from agentic memory are injected into every
    /// persona's prompt as a "Known blind spots to probe for" section — the
    /// compounding loop that turns past misses into permanent probes.
    pub async fn critique(
        &self,
        plan: &Plan,
        results: &[StepResult],
    ) -> Result<Critique, AgentError> {
        if let HookAction::Abort(reason) = self.hooks.before_review().await {
            return Err(AgentError::HookAborted(reason));
        }

        let step_summary: Vec<String> = results
            .iter()
            .map(|r| {
                format!(
                    "- [{}] {:?}: {}",
                    r.subtask_id,
                    r.status,
                    truncate(&r.output, 200)
                )
            })
            .collect();

        // --- U2: diff-grounded critic (fixes G2) ---
        // Obtain the real git diff instead of relying solely on the actor's
        // self-report. The diff is the ground truth; step_summary is reframed
        // as "what the actor claims it did" — divergence between claims and
        // diff is itself a finding.
        let git_diff = self.get_git_diff().await;

        // --- U4: memory injection (compounding loop) ---
        // Retrieve past GUARDRAIL learnings and render them as blind-spot
        // probes in the critic prompt. Playbooks are excluded — they inform
        // planning, not review, and would bias the critic toward the actor's
        // prior successes. Retrievals are recorded so `retrieval_count` /
        // utility counters stay accurate for the critique path, not just
        // comprehension.
        let mut injected_learning_ids: Vec<String> = Vec::new();
        let blind_spots = if let Some(ref mem) = self.memory {
            let learnings = mem.search(&plan.goal_statement, 5, None);
            let guardrails: Vec<&LearningEntry> = learnings
                .iter()
                .filter(|l| l.kind == Some(crate::LearningKind::Guardrail))
                .collect();
            if guardrails.is_empty() {
                String::new()
            } else {
                injected_learning_ids = guardrails.iter().map(|g| g.id.clone()).collect();
                let ids: Vec<&str> = guardrails.iter().map(|g| g.id.as_str()).collect();
                mem.record_retrievals(&ids);
                let body = guardrails
                    .iter()
                    .map(|g| {
                        let util = g
                            .utility_ratio()
                            .map(|u| format!("{:.0}%", u * 100.0))
                            .unwrap_or_else(|| "?".to_string());
                        format!(
                            "- [retrieved {}x, util {}] {}\n  Probe: {}",
                            g.retrieval_count, util, g.context, g.guardrail_advice
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "\n\n## Known blind spots to actively probe for\n\
                     The following guardrails were learned from earlier runs. For EACH, \
                     state whether this change is exposed to it. If yes, that is a \
                     blocking issue.\n{}",
                    body
                )
            }
        } else {
            String::new()
        };

        // --- U2: assemble shared context with diff as ground truth ---
        let diff_section = match &git_diff {
            Some(diff) => format!("\n\n## Actual Diff (ground truth)\n```diff\n{}\n```", diff),
            None => "\n\n## Actual Diff\n[diff-unavailable: not a git repository or diff failed]"
                .to_string(),
        };

        let shared_user = format!(
            "## Goal\n{}\n\n\
             ## Plan\n{}\n\n\
             ## What the actor claims it did (self-report, may be inaccurate)\n{}{}\n{}",
            plan.goal_statement,
            plan.subtasks
                .iter()
                .map(|s| format!("- [{}] {} ({:?})", s.id, s.description, s.tier))
                .collect::<Vec<_>>()
                .join("\n"),
            step_summary.join("\n"),
            diff_section,
            blind_spots,
        );

        let personas = self.config.critique_personas.clone();

        let mut critique = if personas.is_empty() {
            // Backward-compatible single-critic fallback (KD7).
            let user = format!(
                "{}\n\n## Instructions\n\
                 Review this change as a senior architect. Check:\n\
                 1. Does the output match the goal?\n\
                 2. Are acceptance criteria met?\n\
                 3. Any architectural violations or risks?\n\
                 4. Should this be approved or rejected?\n\n\
                 Respond with JSON: {{\"approved\": bool, \"score\": 0.0-1.0, \
                 \"issues\": [...], \"suggestions\": [...]}}",
                shared_user,
            );
            let req = CompletionRequest::prompt(CRITIQUE_SYSTEM_PROMPT, &user)
                .with_model(&self.config.models.review);
            let response = self.llm.complete(&req).await?;
            let mut c = parse_critique_from_response(&response.content, response.usage.clone());
            c.source = "legacy".to_string();
            c
        } else if self.config.critique_mode == CritiqueMode::Full {
            // Full ensemble mode: always run all personas.
            self.run_persona_ensemble(personas, &shared_user).await?
        } else {
            // Tiered mode: run quick check first.
            let quick_req = CompletionRequest::prompt(QUICK_CRITIQUE_PROMPT, &shared_user)
                .with_model(&self.config.models.review);
            let quick_resp = self.llm.complete(&quick_req).await?;
            let quick_critique =
                parse_critique_from_response(&quick_resp.content, quick_resp.usage.clone());

            if self.config.critique_mode == CritiqueMode::QuickOnly {
                let mut c = quick_critique;
                c.source = "quick_check".to_string();
                c
            } else if quick_critique.approved
                && quick_critique.score >= self.config.quick_critique_threshold
            {
                // Quick check passed with high confidence — skip the ensemble.
                let mut c = quick_critique;
                c.source = "quick_check".to_string();
                c
            } else {
                // Quick check didn't clear the threshold — run full ensemble.
                let mut c = self.run_persona_ensemble(personas, &shared_user).await?;
                c.source = "ensemble".to_string();
                c
            }
        };

        critique.injected_learning_ids = injected_learning_ids;

        if let HookAction::Abort(reason) = self.hooks.after_review(&critique).await {
            return Err(AgentError::HookAborted(reason));
        }

        Ok(critique)
    }

    /// Obtain the real git diff via the shell tool (U2: diff-grounded critic).
    ///
    /// Returns `Some(diff_text)` on success, `None` if not a git repo or on
    /// error (graceful degradation — the critic falls back to step_summary).
    /// The diff is truncated to a token budget to avoid overwhelming the prompt.
    async fn get_git_diff(&self) -> Option<String> {
        let params = serde_json::json!({
            "command": "git",
            "args": ["diff", "HEAD"],
            "timeout_ms": 10_000,
        });
        match self.tools.dispatch("shell", params).await {
            Ok(output) => {
                // Parse the shell output format: "exit: {code}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
                let stdout = output
                    .split("--- stdout ---\n")
                    .nth(1)?
                    .split("\n--- stderr ---")
                    .next()?
                    .trim();
                if stdout.is_empty() {
                    None
                } else {
                    // Truncate to 12k chars (matching existing token budget patterns)
                    Some(truncate(stdout, 12_000))
                }
            }
            Err(_) => None,
        }
    }

    /// Fan out the persona ensemble in parallel and merge results.
    ///
    /// Each persona runs as an independent task with its own prompt + the
    /// shared context. Independence of *perspective* (separate prompts) is the
    /// point; parallel execution is a latency win. Errors from any persona
    /// abort the critique (a partial ensemble would silently weaken the gate).
    async fn run_persona_ensemble(
        &self,
        personas: Vec<CritiquePersona>,
        shared_user: &str,
    ) -> Result<Critique, AgentError> {
        let llm = self.llm.clone();
        let review_model = self.config.models.review.clone();

        let mut set = tokio::task::JoinSet::new();
        for persona in personas {
            let llm = llm.clone();
            let user = shared_user.to_string();
            let model = persona
                .model
                .clone()
                .unwrap_or_else(|| review_model.clone());
            set.spawn(async move {
                let req = CompletionRequest::prompt(&persona.system_prompt, user).with_model(model);
                let response = llm.complete(&req).await?;
                let parsed = parse_critique_from_response(&response.content, response.usage);
                Ok::<(CritiquePersona, Critique), LlmError>((persona, parsed))
            });
        }

        let mut persona_results: Vec<PersonaResult> = Vec::new();
        let mut approved = true;
        let mut score = 1.0_f64;
        let mut issues: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut suggestions: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut all_criteria: std::collections::HashMap<usize, CriterionStatus> =
            std::collections::HashMap::new();
        let mut usage = Usage::default();

        while let Some(join_res) = set.join_next().await {
            let (persona, parsed) = match join_res {
                Ok(Ok(pair)) => pair,
                Ok(Err(e)) => return Err(AgentError::Llm(e)),
                Err(e) => return Err(AgentError::Other(format!("critique task panicked: {e}"))),
            };
            persona_results.push(PersonaResult {
                id: persona.id,
                approved: parsed.approved,
                score: parsed.score,
                issues: parsed.issues.clone(),
            });
            approved &= parsed.approved;
            score = score.min(parsed.score);
            for issue in parsed.issues {
                issues.insert(issue);
            }
            for s in parsed.suggestions {
                suggestions.insert(s);
            }
            // U3: Merge coverage matrix from spec_coverage persona
            for criterion in parsed.criteria {
                // Use the worst verdict for each criterion (missing > partial > addressed)
                let entry = all_criteria
                    .entry(criterion.index)
                    .or_insert(criterion.clone());
                if criterion.status == CriterionVerdict::Missing
                    || (criterion.status == CriterionVerdict::Partial
                        && entry.status == CriterionVerdict::Addressed)
                {
                    *entry = criterion;
                }
            }
            usage.accumulate(&parsed.usage);
        }

        // U3: Check coverage matrix — any missing or partial criterion blocks approval
        let mut criteria_vec: Vec<CriterionStatus> = all_criteria.into_values().collect();
        criteria_vec.sort_by_key(|c| c.index);
        for criterion in &criteria_vec {
            if criterion.status == CriterionVerdict::Missing {
                approved = false;
                score = score.min(0.0);
                issues.insert(format!(
                    "criterion #{} '{}': missing",
                    criterion.index, criterion.criterion
                ));
            } else if criterion.status == CriterionVerdict::Partial {
                approved = false;
                score = score.min(0.5);
                issues.insert(format!(
                    "criterion #{} '{}': partial",
                    criterion.index, criterion.criterion
                ));
            }
        }

        let mut issues_vec: Vec<String> = issues.into_iter().collect();
        issues_vec.sort();
        let mut suggestions_vec: Vec<String> = suggestions.into_iter().collect();
        suggestions_vec.sort();

        Ok(Critique {
            approved,
            score,
            issues: issues_vec,
            suggestions: suggestions_vec,
            usage,
            persona_breakdown: persona_results,
            injected_learning_ids: Vec::new(),
            criteria: criteria_vec,
            source: "ensemble".to_string(),
        })
    }

    // --- Reflect: extract learnings from a completed run ---

    /// Extract lessons learned from a completed run.
    ///
    /// Each run produces learnings that future runs retrieve — the compound
    /// self-learning loop that makes the agent improve over time.
    pub async fn reflect(
        &self,
        comprehension: &Comprehension,
        plan: &Plan,
        results: &[StepResult],
        critique: Option<&Critique>,
    ) -> Result<Vec<LearningEntry>, AgentError> {
        let successes = results
            .iter()
            .filter(|r| r.status == StepStatus::Ok)
            .count();
        let failures = results
            .iter()
            .filter(|r| r.status == StepStatus::Failed)
            .count();

        let user = format!(
            "## Goal\n{}\n\n\
             ## What happened\n\
             - {} subtasks succeeded, {} failed\n\
             - Comprehension cited elements: {:?}\n\
             - Critique: {}\n\n\
             ## Instructions\n\
             Extract 1-3 learnings from this run. For each, produce JSON:\n\
             {{\"context\": \"...\", \"hypothesis\": \"...\", \"guardrail_advice\": \"...\", \
             \"kind\": \"playbook|guardrail\"}}\n\
             Playbooks = what worked. Guardrails = what to avoid.",
            plan.goal_statement,
            successes,
            failures,
            comprehension.cited_elements,
            critique
                .map(|c| format!("approved={}, score={}", c.approved, c.score))
                .unwrap_or_else(|| "skipped".into()),
        );

        let req = CompletionRequest::prompt(REFLECTION_SYSTEM_PROMPT, &user)
            .with_model(&self.config.models.cheap);

        let (response, _usage, _signals) = self.run_tool_loop(req).await?;

        let learnings = parse_learnings_from_response(&response.content);

        for entry in &learnings {
            self.hooks.on_learning(entry).await;
        }

        // Persist learnings to memory.
        if let Some(ref mem) = self.memory {
            for entry in &learnings {
                if let Err(e) = mem.record(entry.clone()) {
                    tracing::warn!(error = %e, "failed to record learning to memory");
                }
            }
            if let Some(ref repo) = self.repo_root {
                if let Err(e) = mem.save_to_path(repo) {
                    tracing::warn!(error = %e, "failed to persist learnings to disk");
                }
            }
        }

        Ok(learnings)
    }

    /// Helper: best-effort emit a LoopEvent. Logs but ignores errors.
    fn emit_event(events: Option<&mpsc::Sender<LoopEvent>>, event: LoopEvent) {
        if let Some(sender) = events {
            if let Err(e) = sender.try_send(event) {
                tracing::warn!(error = %e, "loop_event: failed to send (receiver closed?)");
            }
        }
    }

    /// Run the outer ReAct loop: comprehend once, then iterate
    /// Simplified agent loop: trust the model, give it tools, verify results.
    ///
    /// Flow:
    /// 1. Classify complexity (deterministic)
    /// 2. Model drives via tool loop (read files, edit, run commands)
    /// 3. Deterministic verification (lint, test, drift)
    /// 4. If verification fails, feed errors back for one retry
    ///
    /// No planning phase. No critique ensemble. No TDD enforcement.
    /// The model decides what to do. Deterministic checks catch mistakes.
    pub async fn run_loop(
        &self,
        goal: &crate::goal::GoalSpec,
        loop_config: &LoopConfig,
        events: Option<&mpsc::Sender<LoopEvent>>,
        _calibration: Option<&crate::calibration::AskPlan>,
    ) -> Result<LoopResult, AgentError> {
        let max_iterations = loop_config.max_iterations.max(1);

        Self::emit_event(events, LoopEvent::PhaseChanged(LoopPhase::Comprehend));

        // Validate target element IDs before comprehension when available.
        if !goal.target_elements.is_empty() {
            if let Err(unknown) = goal.validate(None) {
                return Err(AgentError::Validation(format!(
                    "unknown target element IDs: {}",
                    unknown.join(", ")
                )));
            }
        }

        // Classify complexity (deterministic, no LLM call).
        let complexity =
            classify_task_complexity(&goal.statement, &goal.target_files, &goal.target_elements);

        // Build synthetic comprehension for backward compatibility.
        let mut comprehension = Comprehension {
            goal: goal.statement.clone(),
            summary: goal.statement.clone(),
            cited_elements: goal.target_elements.clone(),
            key_findings: vec![],
            risks: vec![],
            usage: Usage::default(),
            retrieved_learning_ids: vec![],
            complexity,
            pre_conditions: vec![],
        };

        Self::emit_event(
            events,
            LoopEvent::Started {
                goal: goal.statement.clone(),
                max_iterations,
            },
        );

        // --- Build initial request ---
        let system = crate::cognition::prompts::AGENT_LOOP_SYSTEM_PROMPT;
        let pre_condition_section = if !comprehension.pre_conditions.is_empty() {
            format!(
                "\n\n## Pre-conditions from Error History\n\
                 This repo has patterns of recurring failures. Address these proactively:\n{}\n",
                comprehension
                    .pre_conditions
                    .iter()
                    .map(|p| format!("- {p}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };
        let initial_user = format!("{}{}", goal.statement, pre_condition_section);
        let mut req =
            CompletionRequest::prompt(system, &initial_user).with_tools(self.tools.schemas());
        req.model = Some(self.config.models.mid.clone());

        // Research tasks use the premium model for deeper analysis.
        if complexity == TaskComplexity::Research {
            req.model = Some(self.config.models.premium.clone());
        }

        let pipeline = &loop_config.pipeline;
        let mut pipeline_stages = pipeline.stages.clone();
        let mut total_usage = Usage::default();
        let mut iterations: Vec<LoopIteration> = Vec::new();
        let mut converged = false;
        let mut termination = LoopTermination::MaxIterations;
        let mut _last_output = String::new();
        let mut step_results: Vec<StepResult> = Vec::new();
        let mut non_converged_count: usize = 0;
        let mut seen_signatures: Vec<String> = Vec::new();
        let mut failure_tracker: FailureTracker = FailureTracker::default();
        let mut scope_drift: ScopeDrift = ScopeDrift::default();
        let mut current_plan: Option<Plan> = None;
        let mut current_critique: Option<Critique> = None;

        // Research tasks hard-cap at 1 iteration — comprehension IS the output.
        let effective_iterations = if complexity == TaskComplexity::Research {
            1
        } else {
            max_iterations
        };

        // --- Write initial checkpoint after comprehension ---
        // This enables resume from comprehension state if something goes wrong
        if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
            let checkpoint = RunCheckpoint {
                goal: goal.statement.clone(),
                comprehension: comprehension.clone(),
                iterations: Vec::new(),
                last_plan: None,
                last_steps: Vec::new(),
                last_critique: None,
                failure_tracker: FailureTracker::default(),
                total_usage: Usage::default(),
                converged: false,
                termination: LoopTermination::MaxIterations,
                seen_signatures: Vec::new(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            if let Err(e) = checkpoint.write(checkpoint_dir) {
                tracing::warn!(error = %e, "checkpoint: failed to write initial checkpoint");
            }
        }

        for iteration in 1..=effective_iterations {
            Self::emit_event(
                events,
                LoopEvent::IterationStarted {
                    n: iteration,
                    reason: if iteration > 1 {
                        Some("Addressing feedback from previous iteration".into())
                    } else {
                        None
                    },
                },
            );

            let mut iteration_verify_failed: Vec<String> = Vec::new();

            let stages_this_iteration: Vec<crate::manifest::StageKind> = pipeline_stages.clone();
            for &stage_kind in &stages_this_iteration {
                // Skip "comprehend" after the first iteration — understood once.
                if stage_kind == crate::manifest::StageKind::Comprehend && iteration > 1 {
                    continue;
                }

                // Set file permissions for this stage.
                let phase = stage_kind.to_file_guard_phase();
                if self.guard.phase() != phase {
                    self.guard.set_phase(phase);
                    self.hooks.on_phase_change(phase).await;
                }
                Self::emit_event(events, LoopEvent::PhaseChanged(stage_kind.to_loop_phase()));

                match stage_kind {
                    crate::manifest::StageKind::Comprehend
                    | crate::manifest::StageKind::TestReview => {
                        // Use full LLM-based comprehension for all non-trivial tasks.
                        // Only Trivial tasks keep the synthetic comprehension.
                        if stage_kind == crate::manifest::StageKind::Comprehend
                            && iteration == 1
                            && complexity != TaskComplexity::Trivial
                        {
                            let real_comprehension = self.comprehend(goal).await?;
                            total_usage.accumulate(&real_comprehension.usage);
                            _last_output = real_comprehension.summary.clone();
                            comprehension = real_comprehension;

                            step_results.push(StepResult {
                                subtask_id: "research_comprehend".into(),
                                status: StepStatus::Ok,
                                output: _last_output.clone(),
                                usage: comprehension.usage.clone(),
                                tool_signals: vec![],
                                converged: true,
                            });
                        }
                        // else: common comprehend / TestReview are no-ops.
                    }

                    crate::manifest::StageKind::Plan => {
                        let mut plan = self.plan(goal, &comprehension).await?;
                        // Notify hooks about the plan.
                        self.hooks.after_plan(&mut plan).await;
                        current_plan = Some(plan);
                    }

                    crate::manifest::StageKind::Critique => {
                        if let Some(ref plan) = current_plan {
                            let critique = self.critique(plan, &step_results).await?;
                            self.hooks.after_review(&critique).await;
                            current_critique = Some(critique);
                        }
                    }

                    crate::manifest::StageKind::Fix => {
                        // Targeted fix: if critique has file-level issues, run a
                        // focused tool loop to apply targeted edits instead of
                        // regenerating the full plan.
                        if let Some(ref critique) = current_critique {
                            if !critique.approved && !critique.issues.is_empty() {
                                let file_refs = crate::cognition::parsing::extract_file_references(
                                    &critique.issues,
                                );
                                if !file_refs.is_empty() {
                                    let git_diff = self.get_git_diff().await.unwrap_or_default();
                                    let critique_json =
                                        serde_json::to_string_pretty(&serde_json::json!({
                                            "approved": critique.approved,
                                            "score": critique.score,
                                            "issues": critique.issues,
                                            "suggestions": critique.suggestions,
                                            "file_references": file_refs.iter().map(|(f, lines)| {
                                                serde_json::json!({"file": f, "lines": lines})
                                            }).collect::<Vec<_>>(),
                                        }))
                                        .unwrap_or_default();

                                    let fix_user = format!(
                                        "## Current Diff\n```diff\n{}\n```\n\n\
                                         ## Critique Issues\n```json\n{}\n```\n\n\
                                         ## Instructions\n\
                                         Fix the issues above by editing the flagged files.\n\
                                         Only modify files referenced in the critique.",
                                        git_diff, critique_json,
                                    );
                                    let mut fix_req = CompletionRequest::prompt(
                                        crate::cognition::prompts::FIX_SYSTEM_PROMPT,
                                        &fix_user,
                                    )
                                    .with_tools(self.tools.schemas());
                                    fix_req.model = Some(self.config.models.premium.clone());

                                    let (response, usage, tool_signals, step_converged) =
                                        self.run_tool_loop_with_limit(fix_req, 5).await?;
                                    total_usage.accumulate(&usage);
                                    scope_drift.record_tool_signals(&tool_signals);

                                    let status = if step_has_quality(
                                        step_converged,
                                        &tool_signals,
                                        &response.content,
                                    ) {
                                        StepStatus::Ok
                                    } else {
                                        StepStatus::Failed
                                    };
                                    let output = response.content;
                                    let result = StepResult {
                                        subtask_id: format!("{iteration}_fix"),
                                        status,
                                        output: output.clone(),
                                        usage,
                                        tool_signals,
                                        converged: step_converged,
                                    };
                                    // Pass a placeholder subtask for the hook
                                    let fix_subtask = Subtask {
                                        id: format!("{iteration}_fix"),
                                        description: "targeted fix from critique feedback".into(),
                                        tier: TaskTier::Premium,
                                        kind: SubtaskKind::Implement,
                                        files: file_refs.iter().map(|(f, _)| f.clone()).collect(),
                                        acceptance_criteria: vec![],
                                    };
                                    self.hooks.after_step(&fix_subtask, &result).await;
                                    step_results.push(result);
                                    _last_output = output;
                                } else {
                                    tracing::info!(
                                        "fix stage: no file-level references — skipping"
                                    );
                                }
                            } else {
                                tracing::info!("fix stage: critique approved or empty — skipping");
                            }
                        } else {
                            tracing::info!("fix stage: no critique available — skipping");
                        }
                    }

                    crate::manifest::StageKind::Reflect => {
                        let reflect_plan = current_plan.clone().unwrap_or_else(|| Plan {
                            goal: goal.to_string(),
                            goal_statement: goal.statement.clone(),
                            criteria: goal.acceptance_criteria.clone(),
                            subtasks: vec![Subtask {
                                id: "research".into(),
                                description: goal.statement.clone(),
                                tier: TaskTier::Mid,
                                kind: SubtaskKind::Comprehend,
                                files: goal.target_files.clone(),
                                acceptance_criteria: goal.acceptance_criteria.clone(),
                            }],
                            tdd: false,
                            risks: vec![],
                            schema_version: "1.0".into(),
                            complexity,
                        });
                        let _ = self
                            .reflect(
                                &comprehension,
                                &reflect_plan,
                                &step_results,
                                current_critique.as_ref(),
                            )
                            .await;
                    }

                    crate::manifest::StageKind::Implement
                    | crate::manifest::StageKind::TestAuthor => {
                        // When a plan exists, use structured per-subtask execution
                        // with phase enforcement and TDD support.
                        if let Some(ref plan) = current_plan {
                            let exec_results = self.execute(plan).await?;
                            let mut usage_sum = crate::llm::Usage::default();
                            for r in &exec_results {
                                usage_sum.accumulate(&r.usage);
                            }
                            total_usage.accumulate(&usage_sum);
                            if let Some(last) = exec_results.last() {
                                _last_output = last.output.clone();
                                if !last.converged {
                                    non_converged_count += 1;
                                }
                                scope_drift.record_tool_signals(&last.tool_signals);
                            }
                            step_results.extend(exec_results);

                            Self::emit_event(
                                events,
                                LoopEvent::StepProgress {
                                    step: iteration,
                                    total: max_iterations,
                                    description: format!(
                                        "{} ({} subtask(s))",
                                        stage_kind.user_friendly_description(),
                                        plan.subtasks.len(),
                                    ),
                                },
                            );
                        } else {
                            // Fallback: raw tool loop when no plan is available.
                            let max_iters =
                                complexity.max_tool_iterations(self.config.max_tool_iterations);
                            let (response, usage, tool_signals, step_converged) = self
                                .run_tool_loop_with_limit(req.clone(), max_iters)
                                .await?;
                            total_usage.accumulate(&usage);
                            scope_drift.record_tool_signals(&tool_signals);
                            _last_output = response.content.clone();

                            if !step_converged {
                                non_converged_count += 1;
                            }

                            let status = if !step_has_quality(
                                step_converged,
                                &tool_signals,
                                &response.content,
                            ) {
                                StepStatus::Failed
                            } else {
                                StepStatus::Ok
                            };
                            step_results.push(StepResult {
                                subtask_id: format!("{iteration}_{stage_kind:?}"),
                                status,
                                output: response.content.clone(),
                                usage: usage.clone(),
                                tool_signals,
                                converged: step_converged,
                            });
                        }
                    }

                    crate::manifest::StageKind::Replan => {
                        // Structured replan: use critique feedback to generate
                        // a revised plan instead of a raw tool loop.
                        if let (Some(ref critique), Some(ref _plan)) =
                            (&current_critique, &current_plan)
                        {
                            let pressure = if non_converged_count > 0 {
                                Some(format!(
                                    "{} of {} iterations failed to converge. \
                                     Change the approach significantly.",
                                    non_converged_count, iteration
                                ))
                            } else {
                                None
                            };
                            let new_plan = self
                                .replan(
                                    goal,
                                    &comprehension,
                                    critique,
                                    pressure.as_deref(),
                                    &failure_tracker,
                                )
                                .await?;
                            current_plan = Some(new_plan);
                        } else {
                            // Fallback: raw tool loop when critique or plan missing.
                            let max_iters =
                                complexity.max_tool_iterations(self.config.max_tool_iterations);
                            let (response, usage, tool_signals, step_converged) = self
                                .run_tool_loop_with_limit(req.clone(), max_iters)
                                .await?;
                            total_usage.accumulate(&usage);
                            scope_drift.record_tool_signals(&tool_signals);
                            _last_output = response.content.clone();

                            if !step_converged {
                                non_converged_count += 1;
                            }

                            let status = if !step_has_quality(
                                step_converged,
                                &tool_signals,
                                &response.content,
                            ) {
                                StepStatus::Failed
                            } else {
                                StepStatus::Ok
                            };
                            step_results.push(StepResult {
                                subtask_id: format!("{iteration}_{stage_kind:?}"),
                                status,
                                output: response.content.clone(),
                                usage: usage.clone(),
                                tool_signals,
                                converged: step_converged,
                            });
                        }
                    }

                    crate::manifest::StageKind::Verify => {
                        // --- Deterministic verification ---
                        Self::emit_event(events, LoopEvent::PhaseChanged(LoopPhase::Verify));
                        iteration_verify_failed = if let Some(vconf) = &loop_config.verifier {
                            let results = run_verification_steps(
                                &vconf.steps,
                                &vconf.options,
                                &vconf.workdir,
                            )
                            .await;
                            for r in &results {
                                Self::emit_event(
                                    events,
                                    LoopEvent::VerifyResult {
                                        step: r.step_id.clone(),
                                        ok: r.status.is_pass(),
                                    },
                                );
                            }
                            summarize_verify_failures(&results)
                        } else {
                            Vec::new()
                        };

                        // --- Scope drift detection (escalate pipeline if needed) ---
                        if !scope_drift.escalated && scope_drift.detect(complexity) {
                            let new_stages = scope_drift.escalated_stages(&pipeline_stages);
                            tracing::info!(
                                from = ?pipeline_stages,
                                to = ?new_stages,
                                "pipeline: escalating due to scope drift"
                            );
                            pipeline_stages = new_stages;
                            scope_drift.escalated = true;
                        }
                    }
                }

                // --- Write checkpoint after each stage ---
                // This enables resume from last good state if something goes wrong
                if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                    let checkpoint = RunCheckpoint {
                        goal: goal.statement.clone(),
                        comprehension: comprehension.clone(),
                        iterations: iterations.clone(),
                        last_plan: current_plan.clone(),
                        last_steps: step_results.clone(),
                        last_critique: current_critique.clone(),
                        failure_tracker: failure_tracker.clone(),
                        total_usage: total_usage.clone(),
                        converged: false,
                        termination: LoopTermination::MaxIterations, // placeholder
                        seen_signatures: seen_signatures.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    if let Err(e) = checkpoint.write(checkpoint_dir) {
                        tracing::warn!(error = %e, "checkpoint: failed to write after stage");
                    }
                }
            }

            // Determine approval from the last work stage result.
            let last_work = step_results.last();
            let step_converged = last_work.is_some_and(|r| r.converged);
            let last_tool_signals = last_work.map(|r| r.tool_signals.as_slice()).unwrap_or(&[]);
            let critique_approved = current_critique
                .as_ref()
                .map(|c| c.approved)
                .unwrap_or(true);
            let approved = if complexity == TaskComplexity::Research {
                // For Research, comprehension IS the deliverable — no verify gate needed.
                step_has_quality(step_converged, last_tool_signals, &_last_output)
            } else {
                iteration_verify_failed.is_empty()
                    && step_has_quality(step_converged, last_tool_signals, &_last_output)
                    && critique_approved
            };

            // Record iteration.
            iterations.push(LoopIteration {
                iteration,
                replanned: iteration > 1,
                plan_goal: goal.statement.clone(),
                subtask_count: pipeline_stages.len(),
                succeeded: if approved { 1 } else { 0 },
                failed: if approved { 0 } else { 1 },
                critique_approved: approved,
                critique_score: if approved { 1.0 } else { 0.0 },
                critique_issues: iteration_verify_failed.clone(),
                verify_failed: iteration_verify_failed.clone(),
                injected_learning_ids: vec![],
                usage: total_usage.clone(),
                cost_usd: total_usage.estimated_cost_usd(),
                plan_parse_error: None,
                incorporation_gap: None,
            });

            // --- Write checkpoint after each iteration ---
            // This enables resume from last good state if something goes wrong
            if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                let checkpoint = RunCheckpoint {
                    goal: goal.statement.clone(),
                    comprehension: comprehension.clone(),
                    iterations: iterations.clone(),
                    last_plan: current_plan.clone(),
                    last_steps: step_results.clone(),
                    last_critique: current_critique.clone(),
                    failure_tracker: failure_tracker.clone(),
                    total_usage: total_usage.clone(),
                    converged: false,
                    termination: LoopTermination::MaxIterations, // placeholder
                    seen_signatures: seen_signatures.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Err(e) = checkpoint.write(checkpoint_dir) {
                    tracing::warn!(error = %e, "checkpoint: failed to write after iteration");
                }
            }

            // --- Spend cap (check BEFORE convergence to enforce budget) ---
            if let Some(cap) = loop_config.spend_cap_usd {
                let cost: f64 = iterations.iter().map(|i| i.cost_usd).sum();
                if cost >= cap {
                    termination = LoopTermination::SpendCapExceeded(cost);
                    break;
                }
            }

            // --- Convergence check ---
            if approved && loop_config.stop_on_approval {
                converged = true;
                termination = LoopTermination::Approved;
                break;
            }

            // --- Non-convergence fail-fast ---
            if iterations.len() >= 2 {
                let non_converged_fraction = non_converged_count as f64 / iterations.len() as f64;
                if self.config.max_non_converged_fraction <= 1.0
                    && non_converged_fraction > self.config.max_non_converged_fraction
                {
                    termination = LoopTermination::ModelNotConverging(non_converged_fraction);
                    break;
                }
            }

            // --- Oscillation detection ---
            if loop_config.detect_oscillation {
                let signature = iteration_verify_failed.join("|");
                if !signature.is_empty() {
                    if seen_signatures.last() == Some(&signature) {
                        tracing::warn!(
                            iteration,
                            "oscillation: same verify_failed pattern repeated consecutively"
                        );
                        termination = LoopTermination::Oscillation;
                        break;
                    }
                    seen_signatures.push(signature);
                }
            }

            // --- Recovery strategy: structured error feedback ---
            if !iteration_verify_failed.is_empty() {
                let error_class = classify_error(&iteration_verify_failed, &step_results);
                failure_tracker.record(
                    format!("iteration {iteration}"),
                    iteration_verify_failed.join("; "),
                    iteration,
                    error_class,
                );

                let retries_remaining = pipeline
                    .max_retries
                    .saturating_sub(failure_tracker.failures.len());

                match pipeline.recovery {
                    crate::manifest::RecoveryStrategy::Retry if retries_remaining > 0 => {
                        let mut feedback = format!(
                            "Verification failed:\n{}",
                            iteration_verify_failed.join("\n")
                        );
                        feedback.push_str(&failure_tracker.format_for_prompt());
                        req.messages.push(Message::user(&feedback));
                    }
                    crate::manifest::RecoveryStrategy::DiagnoseThenRetry
                        if retries_remaining > 0 =>
                    {
                        let mut feedback = String::from(
                            "[Diagnostic mode]\nAnalyze the failure before retrying.\n",
                        );
                        feedback.push_str(&format!(
                            "Failure:\n{}\n",
                            iteration_verify_failed.join("\n")
                        ));
                        failure_tracker.diagnostic = Some(iteration_verify_failed.join(", "));
                        feedback.push_str(&failure_tracker.format_for_prompt());
                        req.messages.push(Message::user(&feedback));
                    }
                    crate::manifest::RecoveryStrategy::Escalate => {
                        tracing::info!("recovery: escalate — stopping for human input");
                        termination = LoopTermination::NoReplan;
                        break;
                    }
                    crate::manifest::RecoveryStrategy::Fail => {
                        tracing::info!("recovery: fail — stopping pipeline");
                        termination = LoopTermination::NoReplan;
                        break;
                    }
                    _ => {
                        // No retries remaining — stop.
                        tracing::info!(
                            retries_remaining,
                            "recovery: no retries remaining — stopping"
                        );
                        termination = LoopTermination::NoReplan;
                        break;
                    }
                }
            }
        }

        // --- Cleanup checkpoint on convergence ---
        if converged {
            if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                if let Err(e) = RunCheckpoint::cleanup(checkpoint_dir) {
                    tracing::warn!(error = %e, "checkpoint: cleanup failed");
                }
            }
        } else {
            // --- Write final checkpoint for non-converged runs ---
            // This enables resume from last state
            if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                let checkpoint = RunCheckpoint {
                    goal: goal.statement.clone(),
                    comprehension: comprehension.clone(),
                    iterations: iterations.clone(),
                    last_plan: current_plan.clone(),
                    last_steps: step_results.clone(),
                    last_critique: current_critique.clone(),
                    failure_tracker: failure_tracker.clone(),
                    total_usage: total_usage.clone(),
                    converged: false,
                    termination: termination.clone(),
                    seen_signatures: seen_signatures.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Err(e) = checkpoint.write(checkpoint_dir) {
                    tracing::warn!(error = %e, "checkpoint: failed to write final checkpoint");
                }
            }
        }

        // --- Build Plan for the result ---
        // Use the real plan if Plan stage ran, otherwise fall back to synthetic.
        let plan = current_plan.unwrap_or_else(|| Plan {
            goal: goal.to_string(),
            goal_statement: goal.statement.clone(),
            criteria: goal.acceptance_criteria.clone(),
            subtasks: vec![Subtask {
                id: "work".into(),
                description: goal.statement.clone(),
                tier: TaskTier::Mid,
                kind: SubtaskKind::Implement,
                files: goal.target_files.clone(),
                acceptance_criteria: goal.acceptance_criteria.clone(),
            }],
            tdd: false,
            risks: vec![],
            schema_version: "1.0".into(),
            complexity,
        });

        let final_result = AgentRunResult {
            goal: goal.statement.clone(),
            comprehension,
            plan,
            step_results,
            critique: current_critique,
            decision: None,
            runbook: None,
            total_usage: total_usage.clone(),
        };

        let outcome_summary = if converged {
            format!(
                "Completed successfully in {} iteration(s)",
                iterations.len()
            )
        } else {
            format!(
                "Stopped after {} iteration(s) - {}",
                iterations.len(),
                match termination {
                    crate::cognition::LoopTermination::Approved => "approved",
                    crate::cognition::LoopTermination::MaxIterations => "max iterations reached",
                    crate::cognition::LoopTermination::NoReplan => "no replan",
                    crate::cognition::LoopTermination::SpendCapExceeded(_) => "budget exceeded",
                    crate::cognition::LoopTermination::Oscillation => "oscillation detected",
                    crate::cognition::LoopTermination::ModelNotConverging(_) =>
                        "model not converging",
                    crate::cognition::LoopTermination::Aborted(_) => "aborted",
                }
            )
        };
        Self::emit_event(events, LoopEvent::Done { outcome_summary });

        Ok(LoopResult {
            goal: goal.statement.clone(),
            iterations,
            converged,
            termination,
            total_usage,
            grader_source: "simple".into(),
            final_result,
        })
    }

    pub async fn resume_loop(
        &self,
        goal: &crate::goal::GoalSpec,
        loop_config: &LoopConfig,
    ) -> Result<LoopResult, AgentError> {
        let checkpoint_dir = loop_config.checkpoint_dir.as_ref().ok_or_else(|| {
            AgentError::Checkpoint("no checkpoint_dir configured for resume".into())
        })?;

        let checkpoint = RunCheckpoint::load(checkpoint_dir)
            .map_err(|e| AgentError::Checkpoint(format!("failed to load checkpoint: {e}")))?;

        tracing::info!(
            goal = %checkpoint.goal,
            iteration = checkpoint.iterations.len(),
            timestamp = %checkpoint.timestamp,
            "resume_loop: loaded checkpoint"
        );

        if checkpoint.goal != goal.statement {
            tracing::warn!(
                checkpoint_goal = %checkpoint.goal,
                requested_goal = %goal.statement,
                "resume_loop: goal mismatch — checkpoint goal differs from requested goal"
            );
        }

        // If converged, return checkpoint result directly.
        if checkpoint.converged {
            tracing::info!("resume_loop: checkpoint already converged — nothing to resume");
            let final_result = AgentRunResult {
                goal: checkpoint.goal,
                comprehension: checkpoint.comprehension,
                plan: checkpoint.last_plan.unwrap_or_else(|| Plan {
                    goal: String::new(),
                    goal_statement: goal.statement.clone(),
                    criteria: Vec::new(),
                    subtasks: Vec::new(),
                    tdd: false,
                    risks: Vec::new(),
                    schema_version: "1.0".into(),
                    complexity: TaskComplexity::Simple,
                }),
                step_results: checkpoint.last_steps,
                critique: checkpoint.last_critique,
                decision: None,
                runbook: None,
                total_usage: checkpoint.total_usage.clone(),
            };
            return Ok(LoopResult {
                goal: goal.statement.clone(),
                iterations: checkpoint.iterations,
                converged: true,
                termination: LoopTermination::Approved,
                total_usage: checkpoint.total_usage,
                grader_source: "checkpoint".to_string(),
                final_result,
            });
        }

        // Stale checkpoint from old pipeline: clean up and start fresh.
        tracing::info!("resume_loop: checkpoint is non-converged — cleaning up and starting fresh");
        if let Err(e) = RunCheckpoint::cleanup(checkpoint_dir) {
            tracing::warn!(error = %e, "resume_loop: failed to clean up stale checkpoint");
        }
        self.run_loop(goal, loop_config, None, None).await
    }
}

/// Summarize failing verification steps into human-readable critique issues.
///
/// Used by [`Agent::run_loop`] to (a) veto convergence and (b) feed the
/// failures into the next replan so the loop addresses them.
fn summarize_verify_failures(results: &[VerifyResult]) -> Vec<String> {
    results
        .iter()
        .filter(|r| !matches!(r.status, VerifyStatus::Ok))
        .map(|r| {
            let exit = r
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "none".to_string());
            let status_str = match r.status {
                VerifyStatus::Failed => "failed",
                VerifyStatus::Skipped => "skipped",
                VerifyStatus::Ok => unreachable!(),
            };

            // For drift failures, parse the JSON to extract actionable violation details.
            if r.step_id == "grader_drift" {
                if let Some(lines) = extract_violation_lines_from_drift_json(&r.stdout) {
                    return format!(
                        "verify '{}' {} (exit={}):\n{}",
                        r.step_id,
                        status_str,
                        exit,
                        lines.join("\n")
                    );
                }
            }

            let detail = if r.stderr.trim().is_empty() {
                r.stdout.trim()
            } else {
                r.stderr.trim()
            };
            format!(
                "verify '{}' {} (exit={}): {}",
                r.step_id, status_str, exit, detail
            )
        })
        .collect()
}

/// Extract human-readable violation lines from a `sruja drift` JSON output.
///
/// Parses the stdout of `sruja drift -f json --structural-only --fail-on ...`
/// and returns one line per violation that triggered the failure. Returns
/// `None` if the JSON can't be parsed (fall back to raw output).
fn extract_violation_lines_from_drift_json(stdout: &str) -> Option<Vec<String>> {
    let parsed: serde_json::Value = serde_json::from_str(stdout).ok()?;
    let violations = parsed.get("violations")?.as_array()?;

    if violations.is_empty() {
        return Some(vec!["No specific violations listed.".into()]);
    }

    // Show up to 12 violations so the model gets context without flooding.
    let mut lines: Vec<String> = violations
        .iter()
        .filter(|v| v.get("suppressed").and_then(|s| s.as_bool()) != Some(true))
        .take(12)
        .map(|v| {
            let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("?");
            let severity = v.get("severity").and_then(|s| s.as_str()).unwrap_or("?");
            let message = v.get("message").and_then(|m| m.as_str()).unwrap_or("?");
            let location = v.get("location").and_then(|l| l.as_str()).unwrap_or("");
            let file = v
                .get("sources")
                .and_then(|s| s.as_array())
                .and_then(|arr| arr.first())
                .and_then(|src| src.get("file"))
                .and_then(|f| f.as_str())
                .unwrap_or("");
            let baseline = v
                .get("baseline_delta")
                .and_then(|b| b.as_str())
                .unwrap_or("");
            let tag = if baseline == "new" { " [NEW]" } else { "" };
            if file.is_empty() {
                format!("  {kind}({severity}){tag}: {message} ({location})")
            } else {
                format!("  {kind}({severity}){tag}: {message} — {file}")
            }
        })
        .collect();

    let total = violations.len();
    let suppressed = violations
        .iter()
        .filter(|v| v.get("suppressed").and_then(|s| s.as_bool()) == Some(true))
        .count();
    let summary = format!(
        "{} new violation(s) ({} total, {} pre-existing suppressed):",
        total - suppressed,
        total,
        suppressed
    );
    lines.insert(0, summary);

    Some(lines)
}

/// Structural check: did the replan actually incorporate the prior critique?
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...\n(truncated, {} total chars)", &s[..max], s.len())
    }
}

pub(crate) fn extract_element_ids(text: &str) -> Vec<String> {
    // Match patterns like Element.Id or System.Container.Component
    let mut ids = Vec::new();
    for word in text.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
        if cleaned.contains('.') && cleaned.chars().filter(|c| *c == '.').count() >= 1 {
            let parts: Vec<&str> = cleaned.split('.').collect();
            if parts
                .iter()
                .all(|p| !p.is_empty() && p.chars().next().is_some_and(|c| c.is_uppercase()))
            {
                ids.push(cleaned.to_string());
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

mod prompts;
pub(crate) use prompts::*;

mod parsing;
pub use parsing::parse_plan_from_response;
use parsing::{parse_critique_from_response, parse_learnings_from_response};
// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct AgentBuilder {
    llm: Option<Arc<dyn LlmClient>>,
    tools: ToolRegistry,
    guard: FileGuard,
    hooks: Vec<Box<dyn Hook>>,
    config: AgentConfig,
    repo_root: Option<std::path::PathBuf>,
    memory: Option<std::sync::Arc<dyn crate::memory::Memory + Send + Sync>>,
    #[cfg(feature = "mcp-client")]
    mcp_manager: Option<crate::tool::mcp::McpClientManager>,
    tool_call_tracer: Option<Box<dyn ToolCallTracer>>,
    trace_run_id: Option<String>,
    trace_id: Option<String>,
    preloaded_files: std::collections::HashMap<String, String>,
    preloaded_arch_context: String,
}

impl AgentBuilder {
    /// Set the LLM client (the brain).
    pub fn llm(mut self, llm: Arc<dyn LlmClient>) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Set the tool registry (the hands).
    pub fn tools(mut self, tools: ToolRegistry) -> Self {
        self.tools = tools;
        self
    }

    /// Attach the file guard (created automatically if not set).
    pub fn guard(mut self, guard: FileGuard) -> Self {
        self.guard = guard;
        self
    }

    /// Register a lifecycle hook.
    pub fn hook(mut self, hook: Box<dyn Hook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Set the agent configuration.
    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable memory: provide the repo root where `.sruja/agent_memory.json` lives.
    ///
    /// This constructs the default in-memory backend. For indexed search
    /// (FTS5+BM25), use [`memory_backend`] instead.
    pub fn memory(mut self, repo_root: impl Into<std::path::PathBuf>) -> Self {
        let repo = repo_root.into();
        let mem = crate::memory::AgenticMemory::load(&repo).unwrap_or_default();
        self.memory = Some(std::sync::Arc::new(std::sync::Mutex::new(mem)));
        self.repo_root = Some(repo);
        self
    }

    /// Set a custom memory backend (e.g. FTS5+BM25 indexed search).
    ///
    /// The `repo_root` is used for resolving `.sruja/` paths (decisions,
    /// runbooks, and as the write target for memory persistence).
    pub fn memory_backend(
        mut self,
        repo_root: impl Into<std::path::PathBuf>,
        backend: std::sync::Arc<dyn crate::memory::Memory + Send + Sync>,
    ) -> Self {
        self.repo_root = Some(repo_root.into());
        self.memory = Some(backend);
        self
    }

    /// Register MCP tools from a loop manifest.
    ///
    /// Connects to all enabled MCP servers, lists their tools,
    /// and registers them with the tool registry. Returns a
    /// future that resolves on successful tool registration.
    ///
    /// This is an async builder step; await the future before calling `build`.
    #[cfg(feature = "mcp-client")]
    pub async fn with_mcp(
        mut self,
        manifest: &crate::manifest::LoopManifest,
        repo_root: impl Into<std::path::PathBuf>,
    ) -> Result<Self, AgentError> {
        use crate::tool::mcp::McpClientManager;

        let repo_root = repo_root.into();
        let (manager, mcp_tools) = McpClientManager::from_manifest(manifest, &repo_root)
            .await
            .map_err(|e| AgentError::Mcp(format!("initialization failed: {}", e)))?;

        for tool in mcp_tools {
            self.tools.register(tool);
        }

        self.mcp_manager = Some(manager);
        Ok(self)
    }

    /// Set trace context for tool-call event attribution (U5).
    ///
    /// When all three are provided and `config.enable_tool_call_tracing` is
    /// true, every agent→tool dispatch emits `tool_call`/`tool_result`
    /// context events to `context_events.jsonl`.
    pub fn trace_context(mut self, run_id: impl Into<String>, trace_id: impl Into<String>) -> Self {
        self.trace_run_id = Some(run_id.into());
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Set the tool-call tracer for context event attribution (U5).
    ///
    /// The tracer is called before and after every tool dispatch when
    /// `config.enable_tool_call_tracing` is true and trace context is set.
    pub fn tool_call_tracer(mut self, tracer: Box<dyn ToolCallTracer>) -> Self {
        self.tool_call_tracer = Some(tracer);
        self
    }

    /// Set pre-loaded target file contents.
    ///
    /// When `--file <path>` is specified on the CLI, the file is read once
    /// and injected into the comprehension user prompt. This eliminates
    /// redundant `file_read` tool calls that models often repeat on the
    /// same file in small chunks.
    pub fn preloaded_files(mut self, files: std::collections::HashMap<String, String>) -> Self {
        self.preloaded_files = files;
        self
    }

    /// Set pre-loaded architecture context.
    ///
    /// When architecture context (repomap, topology) is pre-loaded, it's
    /// injected into the comprehension prompt so the agent doesn't need to
    /// call MCP tools for basic architecture context. Saves tokens and
    /// makes the agent more efficient.
    pub fn preloaded_arch_context(mut self, context: String) -> Self {
        self.preloaded_arch_context = context;
        self
    }

    /// Build the agent.
    pub fn build(self) -> Result<Agent, AgentError> {
        let llm_arc = self.llm.ok_or(AgentError::NoLlm)?;

        // Wrap in ModelRouter if a spend cap is configured.
        let llm: Arc<dyn LlmClient> = if let Some(cap) = self.config.spend_cap_usd {
            let rc = crate::llm::router::RouterConfig {
                spend_cap_usd: Some(cap),
                ..Default::default()
            };
            Arc::new(ModelRouter::with_config(llm_arc, rc))
        } else {
            llm_arc
        };

        // Wrap in circuit breaker for per-model failure detection and
        // fast-fail. This prevents cascading failures when a provider is
        // unhealthy — the circuit opens after 3 consecutive failures for
        // a model and rejects further calls for 30s.
        let llm: Arc<dyn LlmClient> = Arc::new(crate::llm::CircuitBreakerClient::new(llm));

        // Wire the guard and dry_run into the tools.
        let mut tools = self.tools;
        tools.set_guard(self.guard.clone());
        if self.config.dry_run {
            tools.set_dry_run(true);
        }

        // Memory: use the provided backend, or fall back to in-memory JSON.
        let memory = self.memory.or_else(|| {
            self.repo_root.as_ref().map(|repo| {
                let mem = AgenticMemory::load(repo).unwrap_or_default();
                std::sync::Arc::new(std::sync::Mutex::new(mem))
                    as std::sync::Arc<dyn crate::memory::Memory + Send + Sync>
            })
        });

        #[cfg(feature = "mcp-client")]
        let mcp_manager = self.mcp_manager;

        Ok(Agent {
            llm,
            tools,
            guard: self.guard,
            hooks: HookRegistry::new(self.hooks),
            config: self.config,
            repo_root: self.repo_root,
            memory,
            #[cfg(feature = "mcp-client")]
            mcp_manager,
            tool_call_tracer: self.tool_call_tracer,
            trace_run_id: self.trace_run_id,
            trace_id: self.trace_id,
            preloaded_files: self.preloaded_files,
            preloaded_arch_context: self.preloaded_arch_context,
        })
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_loop_event;

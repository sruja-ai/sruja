use serde::{Deserialize, Serialize};

use crate::llm::TaskTier;
use crate::tool::{Phase, ToolSignal};
use crate::manifest::StageKind;
use crate::cognition::prompts::{
    CORRECTNESS_PERSONA_PROMPT, SPEC_COVERAGE_PERSONA_PROMPT,
    BOUNDARY_PERSONA_PROMPT, REGRESSION_PERSONA_PROMPT, ADVERSARIAL_TEST_PERSONA_PROMPT,
};
use crate::cognition::decision::DecisionRecord;
use crate::cognition::runbook::Runbook;
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
    pub complexity: crate::cognition::complexity::TaskComplexity,
}

/// Result of executing a subtask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub subtask_id: String,
    pub status: StepStatus,
    pub output: String,
    pub usage: crate::llm::Usage,
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
    pub usage: crate::llm::Usage,
    /// IDs of past learnings retrieved during comprehension (U3 observability).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retrieved_learning_ids: Vec<String>,
    /// Heuristic complexity classification for this goal.
    /// Controls prompt selection, TDD enforcement, and artifact generation.
    #[serde(default)]
    pub complexity: crate::cognition::complexity::TaskComplexity,
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
    pub usage: crate::llm::Usage,
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
/// is detected, the pipeline can be escalated mid-loop (e.g., Simple -> Moderate
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
    fn threshold(complexity: crate::cognition::complexity::TaskComplexity) -> usize {
        match complexity {
            crate::cognition::complexity::TaskComplexity::Trivial => 3,
            crate::cognition::complexity::TaskComplexity::Simple => 10,
            crate::cognition::complexity::TaskComplexity::Research => 15,
            crate::cognition::complexity::TaskComplexity::Moderate => 20,
            crate::cognition::complexity::TaskComplexity::Complex => 40,
        }
    }

    /// Check if scope has drifted beyond the original classification.
    pub fn detect(&mut self, initial: crate::cognition::complexity::TaskComplexity) -> bool {
        let max_calls = Self::threshold(initial);
        self.exceeded = self.total_tool_calls > max_calls;
        self.exceeded
    }

    /// Return an escalated pipeline that adds Plan and/or Critique if missing.
    pub fn escalated_stages(
        &self,
        current: &[StageKind],
    ) -> Vec<StageKind> {
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
    pub total_usage: crate::llm::Usage,
}

/// Configuration for the deterministic verifier that runs inside the loop.
///
/// The verifier is the **independent grader**: it runs after `execute` in every
/// iteration, and a failing step vetoes convergence regardless of the LLM
/// critic's verdict. Failures are injected into the next replan so the loop
/// addresses them. The workdir is supplied here (not assumed from the agent)
/// because the agent crate is intentionally repo-agnostic.
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    pub steps: Vec<crate::verify::VerifyStep>,
    pub options: crate::verify::VerifyOptions,
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
    /// failures. The default (three-stage: comprehend -> implement -> verify)
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
    /// iteration (the compounding loop: misses -> memory -> future review).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub injected_learning_ids: Vec<String>,
    pub usage: crate::llm::Usage,
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
    pub total_usage: crate::llm::Usage,
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

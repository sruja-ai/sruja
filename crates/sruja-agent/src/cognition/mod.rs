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

pub mod chat;
pub mod decision;
pub mod hook;
pub mod loop_event;
pub mod runbook;
pub mod tool_tracing;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::llm::{
    CompletionRequest, CompletionResponse, LlmClient, LlmError, Message, MessageRole, ModelRouter, Usage,
    DEFAULT_MODEL, PREMIUM_MODEL,
};
use crate::tool::ToolSignal;

pub use crate::llm::TaskTier;
use crate::memory::AgenticMemory;
use crate::tool::{FileGuard, Phase, ToolError, ToolRegistry};
use crate::verify::{
    all_passed, run_verification_steps, VerifyOptions, VerifyResult, VerifyStatus, VerifyStep,
};
use crate::LearningEntry;

pub use decision::{DecisionRecord, DecisionStatus};
pub use hook::{Hook, HookAction, HookRegistry, Hooks, LoggingHook};
pub use loop_event::{LoopEvent, LoopPhase, PlanBrief};
pub use runbook::{Runbook, RunbookSeverity};
pub use tool_tracing::ToolCallTracer;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

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
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            models: ModelMapping::default(),
            tdd: true,
            review_every_change: true,
            spend_cap_usd: None,
            dry_run: false,
            // Comprehension prompt says "3-5 tool calls" but models often ignore
            // it. 8 is the safety net: soft guard at 4th call (iter 3), hard at 6th call (iter 5).
            // Keeps total loop time under ~60s per phase at 5s/call.
            max_tool_iterations: 8,
            // 5-minute wall-clock timeout for the entire tool loop.
            loop_timeout_secs: 300,
            system_hints: Vec::new(),
            critique_personas: CritiquePersona::default_personas(),
            enable_tool_call_tracing: false,
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
}

impl TaskComplexity {
    /// Whether TDD should be enforced for this complexity level.
    pub fn enforce_tdd(self) -> bool {
        !matches!(self, TaskComplexity::Trivial)
    }

    /// Whether post-loop artifacts (decision record, runbook) should be generated.
    pub fn generate_artifacts(self) -> bool {
        !matches!(self, TaskComplexity::Trivial)
    }

    /// Effective max tool iterations for this complexity level.
    pub fn max_tool_iterations(self, configured: usize) -> usize {
        match self {
            TaskComplexity::Trivial => configured.min(3),
            TaskComplexity::Simple => configured.min(5),
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

/// Compresses old tool results in the message history to save context tokens.
///
/// Preserves: role, tool_call_id, tool_calls fields. Only rewrites content.
/// Never compresses: most recent tool results (after the last assistant message),
/// system prompt, user goal message, file_write/file_edit confirmations.
///
/// Compresses: file_read and shell outputs older than the most recent assistant
/// turn, only if > 500 chars and > 6 lines.
fn compress_tool_results(messages: &mut Vec<Message>) -> Vec<Message> {
    // Build a map: tool_call_id -> tool_name (to detect file_write/file_edit).
    let mut call_id_to_tool: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for msg in messages.iter() {
        if msg.role == MessageRole::Assistant {
            for call in &msg.tool_calls {
                call_id_to_tool.insert(call.id.clone(), call.name.clone());
            }
        }
    }

    // Find the index of the last assistant message. Tool messages after it
    // are the "most recent" results — the model needs them for its next step.
    let most_recent_threshold = match messages.iter().rposition(|m| m.role == MessageRole::Assistant) {
        Some(idx) => idx,
        None => return std::mem::take(messages),
    };

    let mut compressed = Vec::new();

    for (idx, msg) in messages.drain(..).enumerate() {
        if msg.role == MessageRole::Tool {
            let tool_call_id = msg.tool_call_id.clone().unwrap_or_default();
            let tool_name = call_id_to_tool
                .get(&tool_call_id)
                .map(|s| s.as_str())
                .unwrap_or("");

            // Never compress: most recent tool results (after last assistant message).
            if idx > most_recent_threshold {
                compressed.push(msg);
                continue;
            }

            // Never compress: file_write/file_edit confirmations.
            if tool_name == "file_write" || tool_name == "file_edit" {
                compressed.push(msg);
                continue;
            }

            // Compress if > 500 chars and > 6 lines.
            let compressed_content = if msg.content.len() > 500 {
                let lines: Vec<&str> = msg.content.lines().collect();
                if lines.len() <= 6 {
                    msg.content.clone()
                } else {
                    let summary = lines.first().copied().unwrap_or("");
                    let last_lines = lines
                        .iter()
                        .rev()
                        .take(3)
                        .rev()
                        .copied()
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!(
                        "{}\n[... {} lines compressed ...]\n{}",
                        summary,
                        lines.len().saturating_sub(4),
                        last_lines
                    )
                }
            } else {
                msg.content.clone()
            };

            compressed.push(Message {
                role: msg.role,
                content: compressed_content,
                tool_calls: msg.tool_calls,
                tool_call_id: msg.tool_call_id,
            });
        } else {
            compressed.push(msg);
        }
    }

    compressed
}

/// Classifies errors from critic issues and tool output using deterministic pattern matching.
///
/// Returns the most specific error class that matches the available signals.
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
    if step_results.iter().any(|s| {
        s.tool_signals
            .iter()
            .any(|t| t.tool == "sruja" && !t.ok)
    }) && tool_output.contains("warning") {
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
}

impl FailureTracker {
    pub fn record(&mut self, approach: String, reason: String, iteration: usize, error_class: ErrorClass) {
        if self.last_approach.as_deref() == Some(approach.as_str()) {
            self.consecutive_same_approach += 1;
        } else {
            self.consecutive_same_approach = 1;
        }
        self.last_approach = Some(approach.clone());
        self.failures.push((approach, reason, iteration, error_class));
    }

    /// Format failures for injection into replanning prompt.
    pub fn format_for_prompt(&self) -> String {
        if self.failures.is_empty() {
            return String::new();
        }
        let mut out = String::from("\n\n## Previously Failed Approaches\n\n");
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

/// Result of direct execution (bypasses plan/critique).
#[derive(Debug, Clone)]
pub struct DirectResult {
    /// The model's final response (summary of what it changed).
    pub output: String,
    /// Token usage from the direct execution LLM call(s).
    pub usage: Usage,
    /// Per-tool-call signals for telemetry.
    pub tool_signals: Vec<ToolSignal>,
}

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

/// Errors from parsing a [`Plan`] from the LLM response.
///
/// These are *recoverable* — the caller may issue a format-correction
/// re-prompt on the first failure before hard-failing.
#[derive(Debug, thiserror::Error)]
pub enum PlanParseError {
    #[error("malformed JSON: {0}")]
    MalformedJson(String),
    #[error("missing required field `{field}` on subtask {subtask_index}")]
    MissingRequiredField { field: String, subtask_index: usize },
    #[error("plan contains no subtasks")]
    NoSubtasks,
    #[error("empty plan (JSON had no subtasks or risks)")]
    EmptyPlan,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),
    #[error("tool error: {0}")]
    Tool(#[from] ToolError),
    #[error("no LLM client configured")]
    NoLlm,
    #[error("max tool iterations ({0}) exceeded")]
    MaxIterations(usize),
    #[error("aborted by hook: {0}")]
    HookAborted(String),
    #[error("plan parse failed (unrecoverable after retry): {0}")]
    PlanParseFailed(#[source] PlanParseError),
    #[error("{0}")]
    Other(String),
}

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
            let learnings = mem.search(goal_str, 5);
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
        let error_history = if let Some(ref mem) = self.memory {
            if let Some(repo_path) = &self.repo_root {
                let repo_path_str = repo_path.display().to_string();
                if let Ok(frequencies) = mem.search_error_history(&repo_path_str) {
                    if frequencies.is_empty() {
                        String::new()
                    } else {
                        let total: usize = frequencies.iter().map(|f| f.count).sum();
                        let percentages: Vec<String> = frequencies
                            .iter()
                            .map(|f| {
                                let pct = if total > 0 {
                                    (f.count as f64 / total as f64 * 100.0) as u32
                                } else {
                                    0
                                };
                                let advice = match f.error_class {
                                    ErrorClass::Compilation => "(run cargo check first)",
                                    ErrorClass::Type => "(check type annotations before tests)",
                                    ErrorClass::Test => "(verify logic against acceptance criteria)",
                                    ErrorClass::Runtime => "(check for unwrap/None, bounds)",
                                    ErrorClass::Lint => "(run cargo clippy)",
                                    ErrorClass::Architecture => "(check boundary crossings)",
                                    ErrorClass::SpecGap => "(verify all criteria are addressed)",
                                    ErrorClass::Other => "(investigate carefully)",
                                };
                                format!("{}% {:?} {}", pct, f.error_class, advice)
                            })
                            .collect();
                        format!(
                            "\n\n## Error History for This Repo\n\
                             This repo's past agent runs had these failure patterns:\n\
                             - {}\n\
                             Focus your attention accordingly.",
                            percentages.join("\n- ")
                        )
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
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

        let user = format!(
            "## Goal\n{goal_str}\n\n\
             ## Instructions\n\
             Use the available tools to explore the codebase. \
             Cite architecture element IDs in your findings. \
             Produce a concise, grounded understanding.{preloaded_section}"
        );

        let req = CompletionRequest::prompt(&system, user).with_tools(self.tools.schemas());

        let (response, usage, _signals) = self.run_tool_loop(req).await?;

        let cited_elements = extract_element_ids(&response.content);

        let complexity = classify_task_complexity(
            goal_str,
            &goal.target_files,
            &goal.target_elements,
        );
        tracing::info!(?complexity, "comprehend: classified task complexity");

        Ok(Comprehension {
            goal: goal.to_string(),
            summary: response.content,
            cited_elements,
            key_findings: Vec::new(),
            risks: Vec::new(),
            usage,
            retrieved_learning_ids,
            complexity,
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
        self.run_tool_loop_with_limit(req, self.config.max_tool_iterations)
            .await
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
    ) -> Result<(CompletionResponse, Usage, Vec<ToolSignal>), AgentError> {
        let timeout = std::time::Duration::from_secs(self.config.loop_timeout_secs);
        match tokio::time::timeout(timeout, self.run_tool_loop_inner(req, max_iterations)).await {
            Ok(result) => result,
            Err(_) => {
                tracing::warn!(
                    timeout_secs = self.config.loop_timeout_secs,
                    "tool_loop: wall-clock timeout exceeded"
                );
                Err(AgentError::Other(format!(
                    "Agent loop timed out after {} seconds",
                    self.config.loop_timeout_secs
                )))
            }
        }
    }

    /// Inner implementation of the tool loop (extracted for timeout wrapping).
    async fn run_tool_loop_inner(
        &self,
        mut req: CompletionRequest,
        max_iterations: usize,
    ) -> Result<(CompletionResponse, Usage, Vec<ToolSignal>), AgentError> {
        let mut total_usage = Usage::default();
        let mut tool_signals: Vec<ToolSignal> = Vec::new();
        let mut last_response: Option<CompletionResponse> = None;
        let mut soft_sent = false;
        let mut hard_sent = false;
        // Circuit breaker: track consecutive tool errors and inject recovery.
        // Validated technique from the Self-Harness paper — prevents the agent
        // from blindly retrying the same failing command.
        let mut consecutive_errors: usize = 0;
        let mut recovery_sent = false;

        for iteration in 0..max_iterations {
            let response = self.llm.complete(&req).await?;
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
                    // Clear tool_calls so callers don't dispatch "final answer"
                    // tool calls that the loop intentionally chose not to run.
                    let mut response = response;
                    response.tool_calls.clear();
                    return Ok((response, total_usage, tool_signals));
                }
                return Ok((response, total_usage, tool_signals));
            }

            last_response = Some(response.clone());

            // Push the assistant's tool-call message.
            req.messages.push(Message {
                role: crate::llm::MessageRole::Assistant,
                content: response.content.clone(),
                tool_calls: response.tool_calls.clone(),
                tool_call_id: None,
            });

            // Execute each requested tool and feed results back.
            for call in &response.tool_calls {
                tracing::debug!(
                    tool = %call.name,
                    args_preview = %call.arguments.to_string().chars().take(200).collect::<String>(),
                    "tool_loop: dispatching tool"
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

                let (result, mut record) = match self
                    .tools
                    .dispatch_record(&call.name, call.arguments.clone())
                    .await
                {
                    Ok(out) => out,
                    Err(e) => {
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
                };
                let truncated_text = truncate(&result, 8_000);
                let was_truncated = result.len() > 8_000;
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

            // ── Progressive compression (every 3 iterations) ─────────────
            // Compress old tool results to save context tokens.
            // Only compress after 3 iterations have passed (not iteration 0).
            if iteration > 0 && iteration % 3 == 0 {
                let original_len = req.messages.len();
                req.messages = compress_tool_results(&mut req.messages);
                tracing::debug!(
                    iteration,
                    original_len,
                    compressed_len = req.messages.len(),
                    "tool_loop: compressed message history"
                );
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
                     what you know and write your final answer."
                ));
                // Reset counter so recovery gets a fair chance.
                consecutive_errors = 0;
            }

            // ── Convergence pressure ─────────────────────────────────────
            // Some models (especially smaller OpenAI-compatible ones) never
            // stop calling tools on their own — they explore indefinitely.
            // When the tool-call budget is running low, inject a user message
            // that forces the model to produce a final answer. Each tier is
            // injected at most once to avoid flooding the context.
            let remaining = max_iterations - iteration - 1;
            let quarter = (max_iterations / 4).max(1);
            let half = max_iterations / 2;
            if remaining > 0 && remaining <= quarter && !hard_sent {
                hard_sent = true;
                tracing::warn!(
                    iteration,
                    remaining,
                    "tool_loop: injecting hard convergence message"
                );
                req.messages.push(Message::user(
                    "CRITICAL: You have very few tool calls remaining. \
                     You MUST produce your final answer now as plain text. \
                     Do NOT call any more tools. Synthesize what you have \
                     learned and write your answer.",
                ));
            } else if remaining > 0 && remaining <= half && !soft_sent {
                soft_sent = true;
                tracing::info!(
                    iteration,
                    remaining,
                    "tool_loop: injecting soft convergence reminder"
                );
                req.messages.push(Message::user(
                    "REMINDER: You have used more than half your tool budget. \
                     Wrap up exploration and produce your final answer soon.",
                ));
            }
        }

        // Graceful degradation: the model didn't self-terminate, but it may
        // have produced useful content in its last response. Return that
        // content (with tool_calls cleared) rather than crashing the entire
        // agent. Downstream phases may still fail to parse it, but at least
        // the error message will be meaningful instead of "max iterations".
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
        Ok((fallback, total_usage, tool_signals))
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
        let (response, _usage, _signals) = self.run_tool_loop_with_limit(req, max_iters).await?;

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
                let correction_req = if matches!(comprehension.complexity, TaskComplexity::Trivial) {
                    correction_req
                } else {
                    correction_req.with_tools(self.tools.schemas())
                };
                let (retry_response, _retry_usage, _signals) =
                    self.run_tool_loop_with_limit(correction_req, max_iters).await?;
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

        let issues = if critique.issues.is_empty() {
            "(none reported)".to_string()
        } else {
            critique
                .issues
                .iter()
                .map(|i| format!("- {i}"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let suggestions = if critique.suggestions.is_empty() {
            "(none reported)".to_string()
        } else {
            critique
                .suggestions
                .iter()
                .map(|s| format!("- {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

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
            _ => {
                "Produce a revised plan that addresses the critic's feedback.\n"
            }
        };
        let system_prompt: &str = match comprehension.complexity {
            TaskComplexity::Trivial => PLAN_TRIVIAL_SYSTEM_PROMPT,
            _ => PLAN_SYSTEM_PROMPT,
        };

        let failure_context = failure_tracker.format_for_prompt();

        let user = format!(
            "## Goal\n{goal_str}\n\n\
             ## Comprehension\n{}\n\n\
             ## Prior review outcome\n\
             The independent critic REJECTED the previous attempt (score: {:.0}%).\n\
             You must produce a revised plan that addresses the feedback.\n\n\
             ### Critic issues\n{issues}\n\n\
             ### Critic suggestions\n{suggestions}\n\n\
             {failure_context}\
             ## Instructions\n\
             {plan_instructions}\
             Output a JSON object with `subtasks` array and `risks` array. \
             Do not repeat failed approaches. Try a DIFFERENT strategy.{tdd_note}{pressure_note}",
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
        let (response, _usage, _signals) = self.run_tool_loop_with_limit(req, max_iters).await?;

        match parse_plan_from_response(&response.content, goal, self.config.tdd && comprehension.complexity.enforce_tdd()) {
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
                let correction_req = if matches!(comprehension.complexity, TaskComplexity::Trivial) {
                    correction_req
                } else {
                    correction_req.with_tools(self.tools.schemas())
                };
                let (retry_response, _retry_usage, _signals) =
                    self.run_tool_loop_with_limit(correction_req, max_iters).await?;
                let plan = parse_plan_from_response(&retry_response.content, goal, self.config.tdd && comprehension.complexity.enforce_tdd())
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
            let (response, tool_usage, tool_signals) = self.run_tool_loop_with_limit(req, max_iters).await?;

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
            };

            self.hooks.after_step(step, &result).await;
            results.push(result);
        }

        // After all subtasks, reset to Comprehend phase.
        self.guard.set_phase(Phase::Comprehend);

        // Auto-format: fix indentation/style issues from model edits.
        self.auto_format().await;

        Ok(results)
    }

    // --- Direct execution: bypass plan/critique for simple tasks ---
    // Inspired by Self-Harness paper: the harness (prompt + loop policy)
    // matters more than model capability. For simple tasks, the overhead
    // of plan → critique → replan wastes tokens and time. Just make the change.

    /// Try direct execution: skip plan/critique, give the model the goal +
    /// context + tools, let it make the change in one shot.
    ///
    /// Returns `Ok(Some(result))` if the change was made successfully,
    /// `Ok(None)` if no changes were produced (caller should fall back to
    /// the full pipeline), or `Err` on LLM failure.
    async fn try_direct_execution(
        &self,
        goal: &crate::goal::GoalSpec,
        comprehension: &Comprehension,
    ) -> Result<Option<DirectResult>, AgentError> {
        let goal_str = goal.statement.as_str();

        // Build context: comprehension summary + preloaded files.
        let mut context = String::new();
        if !comprehension.summary.is_empty() {
            context.push_str(&format!(
                "## Understanding\n{}\n\n",
                truncate(&comprehension.summary, 2000)
            ));
        }
        if !self.preloaded_files.is_empty() {
            context.push_str("## Target Files (content provided — do NOT call file_read for these)\n");
            for (path, content) in &self.preloaded_files {
                context.push_str(&format!(
                    "### {path}\n```\n{}\n```\n\n",
                    truncate(content, 8000)
                ));
            }
        }

        let user = format!(
            "## Goal\n{goal_str}\n\n{context}\n\
             Make the change described in the goal. Use the edit tool to modify \
             the target file. Then verify your change. Then write a one-line summary."
        );

        let req = CompletionRequest::prompt(DIRECT_EXECUTION_PROMPT, user)
            .with_tools(self.tools.schemas());

        let max_iters = match comprehension.complexity {
            TaskComplexity::Trivial => 5,
            TaskComplexity::Simple => 8,
            _ => 10,
        };

        self.guard.set_phase(Phase::Implement);
        self.hooks.on_phase_change(Phase::Implement).await;

        let (response, usage, tool_signals) =
            self.run_tool_loop_with_limit(req, max_iters).await?;

        self.guard.set_phase(Phase::Comprehend);

        // Verify: did the model actually produce changes?
        let has_diff = self.has_git_diff().await;
        if !has_diff {
            tracing::info!("direct_execution: no git diff produced — falling back to pipeline");
            return Ok(None);
        }

        // Check for errors in tool signals.
        let has_errors = tool_signals.iter().any(|s| !s.ok);
        if has_errors && !tool_signals.iter().any(|s| s.ok) {
            tracing::info!("direct_execution: all tool calls failed — falling back");
            return Ok(None);
        }

        tracing::info!("direct_execution: changes verified via git diff");

        // Auto-format: fix indentation/style issues from model edits.
        self.auto_format().await;

        Ok(Some(DirectResult {
            output: response.content,
            usage,
            tool_signals,
        }))
    }

    /// Check whether direct execution should be attempted for this goal.
    ///
    /// Auto-learns from past outcomes: if similar goals have consistently
    /// failed via direct execution, skip it and use the full pipeline.
    fn should_try_direct(&self, goal: &crate::goal::GoalSpec, complexity: TaskComplexity) -> bool {
        // Only attempt direct for Trivial and Simple tasks.
        if !matches!(complexity, TaskComplexity::Trivial | TaskComplexity::Simple) {
            return false;
        }

        // Check past outcomes for similar goals.
        if let Some(ref mem) = self.memory {
            let pattern = goal_pattern(&goal.statement);
            let outcomes = mem.search(&pattern, 10);
            let routing_outcomes: Vec<_> = outcomes
                .iter()
                .filter(|l| l.tags.iter().any(|t| t == "routing"))
                .collect();

            if routing_outcomes.len() >= 3 {
                let direct_successes = routing_outcomes
                    .iter()
                    .filter(|l| {
                        l.tags.iter().any(|t| t == "direct")
                            && l.outcome == crate::ExperimentOutcome::Success
                    })
                    .count();
                let direct_failures = routing_outcomes
                    .iter()
                    .filter(|l| {
                        l.tags.iter().any(|t| t == "direct")
                            && l.outcome != crate::ExperimentOutcome::Success
                    })
                    .count();
                let total = direct_successes + direct_failures;
                if total >= 3 {
                    let success_rate = direct_successes as f64 / total as f64;
                    if success_rate < 0.4 {
                        tracing::info!(
                            success_rate,
                            total,
                            "adaptive_routing: skipping direct (poor success rate for similar goals)"
                        );
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Record task outcome to agent memory for future routing decisions.
    ///
    /// This is the self-learning loop: the agent tracks what worked and what
    /// didn't, and uses that data to make better routing decisions next time.
    /// No explicit configuration needed — it just works.
    async fn record_task_outcome(
        &self,
        goal: &crate::goal::GoalSpec,
        complexity: TaskComplexity,
        path: &str,
        success: bool,
        iterations: usize,
    ) {
        let Some(ref mem) = self.memory else {
            return;
        };

        let pattern = goal_pattern(&goal.statement);
        let outcome = if success {
            "succeeded"
        } else {
            "failed"
        };

        let entry = LearningEntry {
            id: crate::generate_entry_id(),
            kind: None,
            timestamp: chrono::Utc::now(),
            run_id: None,
            repo: None,
            selector: None,
            context: pattern,
            hypothesis: format!(
                "classified as {complexity:?}, routed to {path} path, {outcome} after {iterations} iterations"
            ),
            outcome: if success {
                crate::ExperimentOutcome::Success
            } else {
                crate::ExperimentOutcome::Failed
            },
            reason: None,
            guardrail_advice: if success {
                format!("{path} path effective for {complexity:?} tasks like this")
            } else {
                format!("{path} path failed for {complexity:?} tasks like this — try full pipeline")
            },
            affected_elements: vec![],
            evidence_refs: vec![],
            confidence: None,
            tags: vec![
                "routing".into(),
                path.into(),
                outcome.into(),
                format!("{complexity:?}").to_lowercase(),
            ],
            hitl_kind: None,
            related_ids: vec![],
            retrieval_count: 0,
            task_success_after: if success { 1 } else { 0 },
            task_total_after: 1,
        };

        if let Err(e) = mem.record(entry.clone()) {
            tracing::warn!(error = %e, "failed to record task outcome");
        }
        if let Some(ref repo) = self.repo_root {
            if let Err(e) = mem.save_to_path(repo) {
                tracing::warn!(error = %e, "failed to persist task outcome");
            }
        }
    }

    /// Check if there are uncommitted changes (git diff).
    async fn has_git_diff(&self) -> bool {
        // Helper to extract stdout from shell output
        let extract_stdout = |output: &str| -> String {
            output
                .split("--- stdout ---\n")
                .nth(1)
                .unwrap_or("")
                .split("\n--- stderr ---")
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        };

        // Try git diff --stat HEAD first (works when repo has commits)
        let params = serde_json::json!({
            "command": "git",
            "args": ["diff", "--stat", "HEAD"],
            "timeout_ms": 5_000,
        });
        if let Ok(output) = self.tools.dispatch("shell", params).await {
            if !extract_stdout(&output).is_empty() {
                return true;
            }
        }

        // Fallback: git diff --stat (no HEAD — works for repos without commits)
        let params = serde_json::json!({
            "command": "git",
            "args": ["diff", "--stat"],
            "timeout_ms": 5_000,
        });
        if let Ok(output) = self.tools.dispatch("shell", params).await {
            if !extract_stdout(&output).is_empty() {
                return true;
            }
        }

        // Fallback: git status --porcelain (catches untracked files)
        let params = serde_json::json!({
            "command": "git",
            "args": ["status", "--porcelain"],
            "timeout_ms": 5_000,
        });
        if let Ok(output) = self.tools.dispatch("shell", params).await {
            return !extract_stdout(&output).is_empty();
        }

        false
    }

    /// Auto-format changed files to fix indentation/style issues from model edits.
    ///
    /// Tries `cargo fmt` for Rust projects, then `prettier` for JS/TS.
    /// Failures are silently ignored — formatting is best-effort.
    async fn auto_format(&self) {
        // Try cargo fmt (Rust)
        let params = serde_json::json!({
            "command": "cargo",
            "args": ["fmt"],
            "timeout_ms": 30_000,
        });
        if let Ok(output) = self.tools.dispatch("shell", params).await {
            let stderr = output
                .split("--- stderr ---\n")
                .nth(1)
                .unwrap_or("")
                .trim();
            if stderr.is_empty() {
                tracing::info!("auto_format: cargo fmt succeeded");
                return;
            }
        }

        // Try prettier on changed files (JS/TS)
        let params = serde_json::json!({
            "command": "git",
            "args": ["diff", "--name-only"],
            "timeout_ms": 5_000,
        });
        if let Ok(output) = self.tools.dispatch("shell", params).await {
            let stdout = output
                .split("--- stdout ---\n")
                .nth(1)
                .unwrap_or("")
                .split("\n--- stderr ---")
                .next()
                .unwrap_or("")
                .trim();
            let js_files: Vec<&str> = stdout
                .lines()
                .filter(|l| l.ends_with(".js") || l.ends_with(".ts") || l.ends_with(".tsx"))
                .collect();
            if !js_files.is_empty() {
                let mut args = vec!["prettier".to_string(), "--write".to_string()];
                args.extend(js_files.iter().map(|s| s.to_string()));
                let params = serde_json::json!({
                    "command": "npx",
                    "args": args,
                    "timeout_ms": 30_000,
                });
                if self.tools.dispatch("shell", params).await.is_ok() {
                    tracing::info!("auto_format: prettier succeeded");
                }
            }
        }
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
            let learnings = mem.search(&plan.goal_statement, 5);
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
            parse_critique_from_response(&response.content, response.usage.clone())
        } else {
            self.run_persona_ensemble(personas, &shared_user).await?
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

    // --- Run: the full end-to-end Principal Engineer loop ---

    /// Run the full agent loop: comprehend → plan → execute → critique → reflect.
    ///
    /// After critique, the agent generates:
    /// - A **decision record** explaining WHY the change was made
    /// - A **runbook** for handling failures related to the change
    ///
    /// These are written to `.sruja/decisions/` and `.sruja/runbooks/` so that
    /// a human Debugging at 3AM has immediate, actionable context.
    pub async fn run(&self, goal: &crate::goal::GoalSpec) -> Result<AgentRunResult, AgentError> {
        let comprehension = self.comprehend(goal).await?;
        let complexity = comprehension.complexity;

        // ── Adaptive routing: try direct execution for simple tasks ──────
        if self.should_try_direct(goal, complexity) {
            if let Some(direct) = self.try_direct_execution(goal, &comprehension).await? {
                self.record_task_outcome(goal, complexity, "direct", true, 0)
                    .await;
                return Ok(AgentRunResult {
                    goal: goal.statement.clone(),
                    comprehension,
                    plan: Plan {
                        goal: goal.to_string(),
                        goal_statement: goal.statement.clone(),
                        criteria: goal.acceptance_criteria.clone(),
                        subtasks: vec![Subtask {
                            id: "direct".into(),
                            description: goal.statement.clone(),
                            tier: TaskTier::Mid,
                            kind: SubtaskKind::Implement,
                            files: goal.target_files.clone(),
                            acceptance_criteria: goal.acceptance_criteria.clone(),
                        }],
                        tdd: false,
                        risks: Vec::new(),
                        schema_version: "1.0".into(),
                        complexity,
                    },
                    step_results: vec![StepResult {
                        subtask_id: "direct".into(),
                        status: StepStatus::Ok,
                        output: direct.output,
                        usage: direct.usage,
                        tool_signals: direct.tool_signals,
                    }],
                    critique: None,
                    decision: None,
                    runbook: None,
                    total_usage: Usage::default(),
                });
            }
            self.record_task_outcome(goal, complexity, "direct", false, 0)
                .await;
        }

        // ── Full pipeline ────────────────────────────────────────────────
        let plan = self.plan(goal, &comprehension).await?;
        let step_results = self.execute(&plan).await?;

        let critique = if self.config.review_every_change {
            Some(self.critique(&plan, &step_results).await?)
        } else {
            None
        };

        // Complexity-aware: skip artifacts for trivial tasks.
        let generate_artifacts = comprehension.complexity.generate_artifacts();

        let _learnings = if generate_artifacts {
            self.reflect(&comprehension, &plan, &step_results, critique.as_ref())
                .await?
        } else {
            Vec::new()
        };

        let decision = if generate_artifacts {
            self.generate_decision(&plan, &step_results, critique.as_ref())
                .await
        } else {
            None
        };
        let runbook = if generate_artifacts {
            self.generate_runbook(&plan, &step_results).await
        } else {
            None
        };

        // Write artifacts to disk if a repo root is available.
        if let Some(ref repo) = self.repo_root {
            if let Some(ref d) = decision {
                self.write_decision(repo, d).await;
            }
            if let Some(ref r) = runbook {
                self.write_runbook(repo, r).await;
            }
        }

        Ok(AgentRunResult {
            goal: goal.statement.clone(),
            comprehension,
            plan,
            step_results,
            critique,
            decision,
            runbook,
            total_usage: Usage::default(),
        })
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
    /// (re)plan -> execute -> critique until the independent critic approves
    /// or the iteration budget is exhausted.
    ///
    /// This closes the loop that `run` leaves open: a rejected critique feeds
    /// back into a re-plan via [`Agent::replan`], embodying "loop engineering"
    /// — the actor iterates against an independent grader until a verifiable
    /// condition is met.
    ///
    /// Per-iteration evidence is captured in [`LoopResult::iterations`] so a
    /// host can detect convergence, oscillation, and flailing.
    ///
    /// **Events** are sent to `events` in order: `Started` after comprehension,
    /// `PlanReady` once the plan + calibration `AskPlan` are available,
    /// `PhaseChanged` at each phase boundary, `IterationStarted` at each
    /// replan loop, `StepProgress` per executed subtask, `VerifyResult` per
    /// verify step, and `Done` with a one-line summary before returning.
    /// The method performs **no terminal I/O** — the host renders events.
    ///
    /// Emissions are best-effort: a closed receiver must not fail the loop.
    /// When `events` is `None`, the method behaves exactly as before.
    pub async fn run_loop(
        &self,
        goal: &crate::goal::GoalSpec,
        loop_config: &LoopConfig,
        events: Option<&mpsc::Sender<LoopEvent>>,
    ) -> Result<LoopResult, AgentError> {
        let max_iterations = loop_config.max_iterations.max(1);

        Self::emit_event(events, LoopEvent::PhaseChanged(LoopPhase::Comprehend));
        let comprehension = self.comprehend(goal).await?;

        Self::emit_event(events, LoopEvent::Started {
            goal: goal.statement.clone(),
            max_iterations,
        });

        // ── Adaptive routing: try direct execution for simple tasks ──────
        // Self-Harness insight: for simple/trivial tasks, the overhead of
        // plan → critique → replan wastes tokens and time. Try making the
        // change directly first. Falls back to full pipeline on failure.
        // Auto-learns from past outcomes when to use direct vs pipeline.
        if self.should_try_direct(goal, comprehension.complexity) {
            tracing::info!(
                complexity = ?comprehension.complexity,
                "adaptive_routing: attempting direct execution"
            );
            match self.try_direct_execution(goal, &comprehension).await {
                Ok(Some(direct)) => {
                    tracing::info!("adaptive_routing: direct execution succeeded");
                    self.record_task_outcome(
                        goal,
                        comprehension.complexity,
                        "direct",
                        true,
                        0,
                    )
                    .await;

                    // Construct synthetic results for the caller.
                    let direct_usage = direct.usage.clone();
                    let step = StepResult {
                        subtask_id: "direct".into(),
                        status: StepStatus::Ok,
                        output: direct.output,
                        usage: direct.usage,
                        tool_signals: direct.tool_signals,
                    };
                    let plan = Plan {
                        goal: goal.to_string(),
                        goal_statement: goal.statement.clone(),
                        criteria: goal.acceptance_criteria.clone(),
                        subtasks: vec![Subtask {
                            id: "direct".into(),
                            description: goal.statement.clone(),
                            tier: TaskTier::Mid,
                            kind: SubtaskKind::Implement,
                            files: goal.target_files.clone(),
                            acceptance_criteria: goal.acceptance_criteria.clone(),
                        }],
                        tdd: false,
                        risks: Vec::new(),
                        schema_version: "1.0".into(),
                        complexity: comprehension.complexity,
                    };
                    let final_result = AgentRunResult {
                        goal: goal.statement.clone(),
                        comprehension,
                        plan,
                        step_results: vec![step.clone()],
                        critique: None,
                        decision: None,
                        runbook: None,
                        total_usage: direct_usage.clone(),
                    };
                    Self::emit_event(events, LoopEvent::Done {
                        outcome_summary: "Direct execution succeeded".into(),
                    });
                    return Ok(LoopResult {
                        goal: goal.statement.clone(),
                        iterations: vec![LoopIteration {
                            iteration: 1,
                            replanned: false,
                            plan_goal: goal.statement.clone(),
                            subtask_count: 1,
                            succeeded: 1,
                            failed: 0,
                            critique_approved: true,
                            critique_score: 1.0,
                            critique_issues: Vec::new(),
                            verify_failed: Vec::new(),
                            injected_learning_ids: Vec::new(),
                            usage: direct_usage,
                            plan_parse_error: None,
                            incorporation_gap: None,
                        }],
                        converged: true,
                        termination: LoopTermination::Approved,
                        total_usage: final_result.total_usage.clone(),
                        grader_source: "direct".into(),
                        final_result,
                    });
                }
                Ok(None) => {
                    tracing::info!("adaptive_routing: direct produced no changes — falling back to pipeline");
                    self.record_task_outcome(
                        goal,
                        comprehension.complexity,
                        "direct",
                        false,
                        0,
                    )
                    .await;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "adaptive_routing: direct failed — falling back to pipeline");
                    self.record_task_outcome(
                        goal,
                        comprehension.complexity,
                        "direct",
                        false,
                        0,
                    )
                    .await;
                }
            }
        }

        let mut iterations: Vec<LoopIteration> = Vec::new();
        let mut total_usage = Usage::default();
        let mut last_plan: Option<Plan> = None;
        let mut last_steps: Vec<StepResult> = Vec::new();
        let mut last_critique: Option<Critique> = None;
        let mut converged = false;
        let mut termination = LoopTermination::MaxIterations;
        let mut seen_signatures: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut convergence_pressure: Option<String> = None;
        let mut failure_tracker = FailureTracker::default();

        for iteration in 1..=max_iterations {
            self.hooks.before_iteration(iteration, max_iterations).await;
            let replanned = iteration > 1 && last_critique.is_some();

            // --- PLAN (or re-plan from critique feedback) ---
            let phase = if replanned { LoopPhase::Replan } else { LoopPhase::Plan };
            Self::emit_event(events, LoopEvent::PhaseChanged(phase));
            Self::emit_event(events, LoopEvent::IterationStarted {
                n: iteration,
                reason: if replanned { Some("critique feedback".into()) } else { None },
            });

            let (plan, plan_parse_error) = if replanned {
                match self
                    .replan(
                        goal,
                        &comprehension,
                        last_critique.as_ref().unwrap(),
                        convergence_pressure.as_deref(),
                        &failure_tracker,
                    )
                    .await
                {
                    Ok(p) => (p, None),
                    Err(AgentError::PlanParseFailed(e)) => {
                        // Already retried once inside replan — record and abort.
                        return Err(AgentError::PlanParseFailed(e));
                    }
                    Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                        termination = LoopTermination::SpendCapExceeded(spent);
                        break;
                    }
                    Err(e) => return Err(e),
                }
            } else {
                match self.plan(goal, &comprehension).await {
                    Ok(p) => (p, None),
                    Err(AgentError::PlanParseFailed(e)) => {
                        // Already retried once inside plan — record and abort.
                        return Err(AgentError::PlanParseFailed(e));
                    }
                    Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                        termination = LoopTermination::SpendCapExceeded(spent);
                        break;
                    }
                    Err(e) => return Err(e),
                }
            };

            // --- INCORPORATION CHECK (U3) ---
            // Before executing, check whether the replan structurally addressed
            // the prior critique. If not, record the gap for convergence pressure.
            let incorporation_gap = if replanned {
                if let (Some(ref prev), Some(ref prev_critique)) = (&last_plan, &last_critique) {
                    check_incorporation(prev, &plan, &prev_critique.issues)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(ref gap) = incorporation_gap {
                tracing::warn!(gap = %gap, "replan:incorporation_gap — structurally identical plan");
                convergence_pressure = Some(gap.clone());
            } else if replanned {
                // Clear pressure when the replan did change (incorporation worked).
                convergence_pressure = None;
            }

            // --- EXECUTE ---
            Self::emit_event(events, LoopEvent::PhaseChanged(LoopPhase::Execute));
            let step_results = match self.execute(&plan).await {
                Ok(r) => {
                    for (i, r) in r.iter().enumerate() {
                        Self::emit_event(events, LoopEvent::StepProgress {
                            step: i + 1,
                            total: plan.subtasks.len(),
                            description: r.subtask_id.clone(),
                        });
                    }
                    r
                },
                Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                    termination = LoopTermination::SpendCapExceeded(spent);
                    break;
                }
                Err(e) => return Err(e),
            };
            for r in &step_results {
                total_usage.accumulate(&r.usage);
            }

            // --- CRITIQUE ---
            Self::emit_event(events, LoopEvent::PhaseChanged(LoopPhase::Critique));
            let mut critique = if self.config.review_every_change {
                match self.critique(&plan, &step_results).await {
                    Ok(c) => c,
                    Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                        termination = LoopTermination::SpendCapExceeded(spent);
                        break;
                    }
                    Err(e) => return Err(e),
                }
            } else {
                let all_ok = step_results.iter().all(|r| r.status != StepStatus::Failed);
                Critique {
                    approved: all_ok,
                    score: if all_ok { 1.0 } else { 0.0 },
                    issues: Vec::new(),
                    suggestions: Vec::new(),
                    usage: Usage::default(),
                    persona_breakdown: Vec::new(),
                    injected_learning_ids: Vec::new(),
                    criteria: Vec::new(),
                }
            };
            total_usage.accumulate(&critique.usage);

            // --- DETERMINISTIC GRADER (independent of the LLM critic) ---
            // Runs the verifier, if configured. A failing step vetoes
            // convergence even when the critic approved, and the failures are
            // injected into the critique so the next replan addresses them.
            Self::emit_event(events, LoopEvent::PhaseChanged(LoopPhase::Verify));
            let verify_failed = if let Some(vconf) = &loop_config.verifier {
                let results =
                    run_verification_steps(&vconf.steps, &vconf.options, &vconf.workdir).await;
                for r in &results {
                    Self::emit_event(events, LoopEvent::VerifyResult {
                        step: r.step_id.clone(),
                        ok: r.status.is_pass(),
                    });
                }
                let failed = summarize_verify_failures(&results);
                if !all_passed(&results) {
                    critique.approved = false;
                    critique.issues.extend(failed.clone());
                }
                failed
            } else {
                Vec::new()
            };

            let approved = critique.approved;
            let issue_sig = critique_signature(&critique.issues);
            let succeeded = step_results
                .iter()
                .filter(|r| r.status == StepStatus::Ok)
                .count();
            let failed = step_results
                .iter()
                .filter(|r| r.status == StepStatus::Failed)
                .count();

            // Record failure for self-correction: track what approach was tried
            // and why it failed, so the next replan tries a different strategy.
            if !approved {
                let approach = format!(
                    "subtasks: [{}]",
                    plan.subtasks
                        .iter()
                        .map(|s| format!("{}({:?})", s.id, s.kind))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let reason = if critique.issues.is_empty() {
                    "critic rejected (no specific issues)".to_string()
                } else {
                    critique.issues.join("; ")
                };
                failure_tracker.record(
                    approach,
                    reason,
                    iteration,
                    classify_error(&critique.issues, &step_results),
                );
            }

            // Record the iteration evidence BEFORE guardrail checks so the
            // caller always sees what happened, even on the triggering iteration.
            iterations.push(LoopIteration {
                iteration,
                replanned,
                plan_goal: plan.goal_statement.clone(),
                subtask_count: plan.subtasks.len(),
                succeeded,
                failed,
                critique_approved: approved,
                critique_score: critique.score,
                critique_issues: critique.issues.clone(),
                verify_failed: verify_failed.clone(),
                injected_learning_ids: critique.injected_learning_ids.clone(),
                usage: critique.usage.clone(),
                plan_parse_error: plan_parse_error
                    .as_ref()
                    .map(|e: &PlanParseError| e.to_string()),
                incorporation_gap,
            });

            last_plan = Some(plan);
            last_steps = step_results.clone();
            last_critique = Some(critique);

            self.hooks
                .after_iteration(iteration, max_iterations, iterations.last().unwrap())
                .await;

            // --- CHECKPOINT: persist state for crash-resume ---
            if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                let checkpoint = RunCheckpoint {
                    goal: goal.statement.clone(),
                    comprehension: comprehension.clone(),
                    iterations: iterations.clone(),
                    last_plan: last_plan.clone(),
                    last_steps: last_steps.clone(),
                    last_critique: last_critique.clone(),
                    failure_tracker: failure_tracker.clone(),
                    total_usage: total_usage.clone(),
                    converged,
                    termination: termination.clone(),
                    seen_signatures: seen_signatures.iter().cloned().collect(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Err(e) = checkpoint.write(checkpoint_dir) {
                    tracing::warn!(error = %e, "checkpoint: failed to save");
                }
            }

            // --- GUARDRAIL: spend cap (loop-level estimate) ---
            if let Some(cap) = loop_config.spend_cap_usd {
                let cost = total_usage.estimated_cost_usd();
                if cost >= cap {
                    termination = LoopTermination::SpendCapExceeded(cost);
                    break;
                }
            }

            // --- GUARDRAIL: oscillation detection ---
            if loop_config.detect_oscillation && !approved && !seen_signatures.insert(issue_sig) {
                termination = LoopTermination::Oscillation;
                break;
            }

            if approved && loop_config.stop_on_approval {
                converged = true;
                termination = LoopTermination::Approved;
                break;
            }
            if !approved && !loop_config.replan_on_failure {
                termination = LoopTermination::NoReplan;
                break;
            }
        }

        // --- CHECKPOINT: cleanup on convergence, keep on failure for resume ---
        if converged {
            if let Some(ref checkpoint_dir) = loop_config.checkpoint_dir {
                if let Err(e) = RunCheckpoint::cleanup(checkpoint_dir) {
                    tracing::warn!(error = %e, "checkpoint: cleanup failed");
                }
            }
        }

        let plan = last_plan.ok_or(AgentError::Other("loop produced no plan".into()))?;
        let critique = last_critique;

        // Complexity-aware: skip reflect + artifacts for trivial tasks.
        // A comment typo doesn't need a decision record or runbook.
        let generate_artifacts = comprehension.complexity.generate_artifacts();

        let _learnings = if generate_artifacts {
            self.reflect(&comprehension, &plan, &last_steps, critique.as_ref())
                .await?
        } else {
            Vec::new()
        };
        let decision = if generate_artifacts {
            self.generate_decision(&plan, &last_steps, critique.as_ref())
                .await
        } else {
            None
        };
        let runbook = if generate_artifacts {
            self.generate_runbook(&plan, &last_steps).await
        } else {
            None
        };
        if let Some(ref repo) = self.repo_root {
            if let Some(ref d) = decision {
                self.write_decision(repo, d).await;
            }
            if let Some(ref r) = runbook {
                self.write_runbook(repo, r).await;
            }
        }

        let final_result = AgentRunResult {
            goal: goal.statement.clone(),
            comprehension,
            plan,
            step_results: last_steps,
            critique,
            decision,
            runbook,
            total_usage: total_usage.clone(),
        };

        // Record outcome for adaptive routing (self-learning).
        self.record_task_outcome(
            goal,
            final_result.comprehension.complexity,
            "pipeline",
            converged,
            iterations.len(),
        )
        .await;

        let outcome_summary = if converged {
            format!("Converged in {} iteration(s)", iterations.len())
        } else {
            format!("Not converged after {} iteration(s) - {:?}", iterations.len(), termination)
        };
        Self::emit_event(events, LoopEvent::Done { outcome_summary });

        Ok(LoopResult {
            goal: goal.statement.clone(),
            iterations,
            converged,
            termination,
            total_usage,
            grader_source: "unknown".to_string(),
            final_result,
        })
    }

    /// Resume a previously interrupted agent loop from a checkpoint.
    ///
    /// Loads the checkpoint from `loop_config.checkpoint_dir`, restores all
    /// state (iterations, failure tracker, last plan/critique), and continues
    /// the plan→execute→critique loop from the next iteration.
    ///
    /// Returns the same `LoopResult` as `run_loop`, but with the iterations
    /// from the original run prepended.
    pub async fn resume_loop(
        &self,
        goal: &crate::goal::GoalSpec,
        loop_config: &LoopConfig,
    ) -> Result<LoopResult, AgentError> {
        let checkpoint_dir = loop_config
            .checkpoint_dir
            .as_ref()
            .ok_or_else(|| AgentError::Other("no checkpoint_dir configured for resume".into()))?;

        let checkpoint = RunCheckpoint::load(checkpoint_dir).map_err(|e| {
            AgentError::Other(format!("failed to load checkpoint: {e}"))
        })?;

        tracing::info!(
            goal = %checkpoint.goal,
            iteration = checkpoint.iterations.len(),
            failures = checkpoint.failure_tracker.failures.len(),
            timestamp = %checkpoint.timestamp,
            "resume_loop: loaded checkpoint"
        );

        // Verify the goal matches.
        if checkpoint.goal != goal.statement {
            tracing::warn!(
                checkpoint_goal = %checkpoint.goal,
                requested_goal = %goal.statement,
                "resume_loop: goal mismatch — checkpoint goal differs from requested goal"
            );
        }

        // If already converged, return the checkpoint result directly.
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

        // Restore state from checkpoint and continue the loop.
        let mut iterations = checkpoint.iterations;
        let mut total_usage = checkpoint.total_usage;
        let mut last_plan = checkpoint.last_plan;
        let mut last_steps = checkpoint.last_steps;
        let mut last_critique = checkpoint.last_critique;
        let mut failure_tracker = checkpoint.failure_tracker;
        let mut converged = checkpoint.converged;
        let mut termination = checkpoint.termination;
        let mut seen_signatures: std::collections::HashSet<String> =
            checkpoint.seen_signatures.into_iter().collect();
        let comprehension = checkpoint.comprehension;
        let max_iterations = loop_config.max_iterations.max(1);
        let start_iteration = iterations.len() + 1;

        tracing::info!(
            start_iteration,
            max_iterations,
            "resume_loop: continuing from iteration {start_iteration}"
        );

        for iteration in start_iteration..=max_iterations {
            self.hooks.before_iteration(iteration, max_iterations).await;
            let replanned = iteration > 1 && last_critique.is_some();

            // --- PLAN (or re-plan from critique feedback) ---
            let (plan, plan_parse_error) = if replanned {
                match self
                    .replan(
                        goal,
                        &comprehension,
                        last_critique.as_ref().unwrap(),
                        None, // convergence_pressure — fresh start after resume
                        &failure_tracker,
                    )
                    .await
                {
                    Ok(p) => (p, None),
                    Err(AgentError::PlanParseFailed(e)) => {
                        return Err(AgentError::PlanParseFailed(e));
                    }
                    Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                        termination = LoopTermination::SpendCapExceeded(spent);
                        break;
                    }
                    Err(e) => return Err(e),
                }
            } else {
                match self.plan(goal, &comprehension).await {
                    Ok(p) => (p, None),
                    Err(AgentError::PlanParseFailed(e)) => {
                        return Err(AgentError::PlanParseFailed(e));
                    }
                    Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                        termination = LoopTermination::SpendCapExceeded(spent);
                        break;
                    }
                    Err(e) => return Err(e),
                }
            };

            // --- EXECUTE ---
            self.guard.set_phase(Phase::Implement);
            self.hooks.on_phase_change(Phase::Implement).await;

            let step_results = match self.execute(&plan).await {
                Ok(r) => r,
                Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                    termination = LoopTermination::SpendCapExceeded(spent);
                    break;
                }
                Err(e) => return Err(e),
            };

            self.guard.set_phase(Phase::Comprehend);

            for step in &step_results {
                total_usage.accumulate(&step.usage);
            }

            // --- CRITIQUE ---
            let critique = match self
                .critique(&plan, &step_results)
                .await
            {
                Ok(c) => c,
                Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                    termination = LoopTermination::SpendCapExceeded(spent);
                    break;
                }
                Err(e) => return Err(e),
            };
            total_usage.accumulate(&critique.usage);

            let approved = critique.approved;
            let issue_sig = critique_signature(&critique.issues);
            let succeeded = step_results
                .iter()
                .filter(|r| r.status == StepStatus::Ok)
                .count();
            let failed = step_results
                .iter()
                .filter(|r| r.status == StepStatus::Failed)
                .count();

            // Record failure for self-correction.
            if !approved {
                let approach = format!(
                    "subtasks: [{}]",
                    plan.subtasks
                        .iter()
                        .map(|s| format!("{}({:?})", s.id, s.kind))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let reason = if critique.issues.is_empty() {
                    "critic rejected (no specific issues)".to_string()
                } else {
                    critique.issues.join("; ")
                };
                failure_tracker.record(
                    approach,
                    reason,
                    iteration,
                    classify_error(&critique.issues, &step_results),
                );
            }

            iterations.push(LoopIteration {
                iteration,
                replanned,
                plan_goal: plan.goal_statement.clone(),
                subtask_count: plan.subtasks.len(),
                succeeded,
                failed,
                critique_approved: approved,
                critique_score: critique.score,
                critique_issues: critique.issues.clone(),
                verify_failed: Vec::new(),
                injected_learning_ids: critique.injected_learning_ids.clone(),
                usage: critique.usage.clone(),
                plan_parse_error: plan_parse_error
                    .as_ref()
                    .map(|e: &PlanParseError| e.to_string()),
                incorporation_gap: None,
            });

            last_plan = Some(plan);
            last_steps = step_results.clone();
            last_critique = Some(critique);

            self.hooks
                .after_iteration(iteration, max_iterations, iterations.last().unwrap())
                .await;

            // --- CHECKPOINT: persist state for crash-resume ---
            if let Some(ref cp_dir) = loop_config.checkpoint_dir {
                let checkpoint = RunCheckpoint {
                    goal: goal.statement.clone(),
                    comprehension: comprehension.clone(),
                    iterations: iterations.clone(),
                    last_plan: last_plan.clone(),
                    last_steps: last_steps.clone(),
                    last_critique: last_critique.clone(),
                    failure_tracker: failure_tracker.clone(),
                    total_usage: total_usage.clone(),
                    converged,
                    termination: termination.clone(),
                    seen_signatures: seen_signatures.iter().cloned().collect(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Err(e) = checkpoint.write(cp_dir) {
                    tracing::warn!(error = %e, "checkpoint: failed to save");
                }
            }

            // --- GUARDRAIL: spend cap ---
            if let Some(cap) = loop_config.spend_cap_usd {
                let cost = total_usage.estimated_cost_usd();
                if cost >= cap {
                    termination = LoopTermination::SpendCapExceeded(cost);
                    break;
                }
            }

            // --- GUARDRAIL: oscillation detection ---
            if loop_config.detect_oscillation && !approved && !seen_signatures.insert(issue_sig) {
                termination = LoopTermination::Oscillation;
                break;
            }

            if approved && loop_config.stop_on_approval {
                converged = true;
                termination = LoopTermination::Approved;
                break;
            }
            if !approved && !loop_config.replan_on_failure {
                termination = LoopTermination::NoReplan;
                break;
            }
        }

        // --- CHECKPOINT: cleanup on convergence ---
        if converged {
            if let Some(ref cp_dir) = loop_config.checkpoint_dir {
                if let Err(e) = RunCheckpoint::cleanup(cp_dir) {
                    tracing::warn!(error = %e, "checkpoint: cleanup failed");
                }
            }
        }

        let plan = last_plan.ok_or(AgentError::Other("resume_loop produced no plan".into()))?;
        let critique = last_critique;

        let final_result = AgentRunResult {
            goal: goal.statement.clone(),
            comprehension,
            plan,
            step_results: last_steps,
            critique,
            decision: None,
            runbook: None,
            total_usage: total_usage.clone(),
        };

        Ok(LoopResult {
            goal: goal.statement.clone(),
            iterations,
            converged,
            termination,
            total_usage,
            grader_source: "checkpoint-resume".to_string(),
            final_result,
        })
    }

    /// Generate a decision record explaining WHY this change was made.
    async fn generate_decision(
        &self,
        plan: &Plan,
        results: &[StepResult],
        critique: Option<&Critique>,
    ) -> Option<DecisionRecord> {
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
             ## What was done\n\
             {} subtasks succeeded, {} failed\n\
             Critique: {}\n\n\
             ## Instructions\n\
             Generate a decision record. Output JSON:\n\
             {{\"title\": \"...\", \"context\": \"...\", \"decision\": \"...\", \
             \"consequences\": [...], \"alternatives\": [...]}}\n\
             Be concise but thorough. This record will be read at 3AM by someone \
             who has no context on why this change was made.",
            plan.goal_statement,
            successes,
            failures,
            critique
                .map(|c| format!("approved={}", c.approved))
                .unwrap_or_else(|| "skipped".into()),
        );

        let req = CompletionRequest::prompt(DECISION_SYSTEM_PROMPT, &user)
            .with_model(&self.config.models.cheap);

        let (response, _usage, _signals) = self.run_tool_loop(req).await.ok()?;
        let json_str = extract_json(&response.content);
        let value: serde_json::Value = serde_json::from_str(&json_str).ok()?;

        let title = value.get("title")?.as_str()?.to_string();
        let context = value.get("context")?.as_str()?.to_string();
        let decision_text = value.get("decision")?.as_str()?.to_string();

        let mut record = DecisionRecord::new(title, context, decision_text)
            .with_status(DecisionStatus::Accepted)
            .with_elements(comprehension_cited_elements(plan));

        if let Some(c) = critique {
            record = record.with_consequence(format!("Critic score: {:.0}%", c.score * 100.0));
        }

        if let Some(arr) = value.get("consequences").and_then(|v| v.as_array()) {
            for c in arr {
                if let Some(s) = c.as_str() {
                    record = record.with_consequence(s);
                }
            }
        }

        if let Some(arr) = value.get("alternatives").and_then(|v| v.as_array()) {
            for a in arr {
                if let Some(s) = a.as_str() {
                    record = record.with_alternative(s);
                }
            }
        }

        Some(record)
    }

    /// Generate a runbook for handling failures related to this change.
    async fn generate_runbook(&self, plan: &Plan, _results: &[StepResult]) -> Option<Runbook> {
        let user = format!(
            "## Goal\n{}\n\n\
             ## Subtasks\n{}\n\n\
             ## Instructions\n\
             Generate a runbook for handling failures. Output JSON:\n\
             {{\"title\": \"...\", \"trigger\": \"...\", \"severity\": \"critical|high|medium|low\", \
             \"symptoms\": [...], \"diagnosis\": [...], \"resolution\": [...], \
             \"rollback\": [...], \"verification\": [...]}}\n\
             Be practical. This will be read at 3AM.",
            plan.goal_statement,
            plan.subtasks.iter()
                .map(|s| format!("- [{}] {}", s.id, s.description))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let req = CompletionRequest::prompt(RUNBOOK_SYSTEM_PROMPT, &user)
            .with_model(&self.config.models.cheap);

        let (response, _usage, _signals) = self.run_tool_loop(req).await.ok()?;
        let json_str = extract_json(&response.content);
        let value: serde_json::Value = serde_json::from_str(&json_str).ok()?;

        let title = value.get("title")?.as_str()?.to_string();
        let trigger = value.get("trigger")?.as_str()?.to_string();

        let mut rb = Runbook::new(title, trigger)
            .with_elements(plan.subtasks.iter().flat_map(|s| s.files.clone()).collect());

        if let Some(sev) = value.get("severity").and_then(|v| v.as_str()) {
            rb.severity = match sev {
                "critical" => runbook::RunbookSeverity::Critical,
                "medium" => runbook::RunbookSeverity::Medium,
                "low" => runbook::RunbookSeverity::Low,
                _ => runbook::RunbookSeverity::High,
            };
        }

        if let Some(arr) = value.get("symptoms").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    rb = rb.with_symptom(s);
                }
            }
        }
        if let Some(arr) = value.get("diagnosis").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    rb = rb.with_diagnosis_step(s);
                }
            }
        }
        if let Some(arr) = value.get("resolution").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    rb = rb.with_resolution_step(s);
                }
            }
        }
        if let Some(arr) = value.get("rollback").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    rb = rb.with_rollback_step(s);
                }
            }
        }
        if let Some(arr) = value.get("verification").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    rb = rb.with_verification_step(s);
                }
            }
        }

        Some(rb)
    }

    /// Write a decision record to `.sruja/decisions/`.
    async fn write_decision(&self, repo: &std::path::Path, record: &DecisionRecord) {
        let dir = repo.join(".sruja/decisions");
        let _ = tokio::fs::create_dir_all(&dir).await;
        let path = dir.join(record.filename());
        let _ = tokio::fs::write(&path, record.to_markdown()).await;
        tracing::info!(path = %path.display(), "decision:written");
    }

    /// Write a runbook to `.sruja/runbooks/`.
    async fn write_runbook(&self, repo: &std::path::Path, runbook: &Runbook) {
        let dir = repo.join(".sruja/runbooks");
        let _ = tokio::fs::create_dir_all(&dir).await;
        let path = dir.join(runbook.filename());
        let _ = tokio::fs::write(&path, runbook.to_markdown()).await;
        tracing::info!(path = %path.display(), "runbook:written");
    }

}

fn comprehension_cited_elements(plan: &Plan) -> Vec<String> {
    plan.subtasks.iter().flat_map(|s| s.files.clone()).collect()
}

/// Extract a normalized pattern from a goal statement for routing matching.
///
/// Takes the first few words (lowercased) as a pattern key so similar goals
/// cluster together for adaptive routing decisions.
/// e.g. "Add a comment to mod.rs" → "add a comment to mod.rs"
/// e.g. "Add a comment to lib.rs" → "add a comment to lib.rs"
/// These will match because they share the "add a comment" prefix.
fn goal_pattern(goal: &str) -> String {
    let lower = goal.to_lowercase();
    // Take first 5 words as the pattern — enough to capture the verb + object
    // without being too specific (file names, line numbers).
    lower
        .split_whitespace()
        .take(5)
        .collect::<Vec<_>>()
        .join(" ")
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
            let detail = if r.stderr.trim().is_empty() {
                r.stdout.trim()
            } else {
                r.stderr.trim()
            };
            let exit = r
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "none".to_string());
            let status_str = match r.status {
                VerifyStatus::Failed => "failed",
                VerifyStatus::Skipped => "skipped",
                VerifyStatus::Ok => unreachable!(),
            };
            format!(
                "verify '{}' {} (exit={}): {}",
                r.step_id, status_str, exit, detail
            )
        })
        .collect()
}

/// Build a normalised signature from critique issues to detect oscillation.
/// Sorts and joins the issues so re-ordering doesn't produce a false negative.
fn critique_signature(issues: &[String]) -> String {
    let mut sorted: Vec<&str> = issues.iter().map(|s| s.as_str()).collect();
    sorted.sort();
    sorted.join("\x00")
}

/// Structural check: did the replan actually incorporate the prior critique?
///
/// Returns `None` when incorporation is plausible (critique was empty, or the
/// new plan differs from the old one). Returns `Some(description)` when the
/// critique raised issues but the new plan is structurally identical to the
/// old one — meaning the actor likely re-emitted the same plan without changes.
fn check_incorporation(
    last_plan: &Plan,
    new_plan: &Plan,
    critique_issues: &[String],
) -> Option<String> {
    if critique_issues.is_empty() {
        return None;
    }

    // Build a structural fingerprint: sorted (subtask_id, description) pairs.
    let fingerprint = |plan: &Plan| -> Vec<(String, String)> {
        let mut pairs: Vec<(String, String)> = plan
            .subtasks
            .iter()
            .map(|s| (s.id.clone(), s.description.clone()))
            .collect();
        pairs.sort();
        pairs
    };

    let old_fp = fingerprint(last_plan);
    let new_fp = fingerprint(new_plan);

    if old_fp == new_fp {
        // Also check risks — if risks changed at least something moved.
        let mut old_risks = last_plan.risks.clone();
        let mut new_risks = new_plan.risks.clone();
        old_risks.sort();
        new_risks.sort();
        if old_risks == new_risks {
            let issue_count = critique_issues.len();
            return Some(format!(
                "replan incorporated none of {issue_count} critique issue(s); \
                 subtasks and risks are structurally identical to prior plan"
            ));
        }
    }

    None
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...\n(truncated, {} total chars)", &s[..max], s.len())
    }
}

fn extract_element_ids(text: &str) -> Vec<String> {
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
use parsing::{extract_json, parse_critique_from_response, parse_learnings_from_response};
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
            .map_err(|e| AgentError::Other(format!("MCP initialization failed: {}", e)))?;

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
        })
    }
}


#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_loop_event;

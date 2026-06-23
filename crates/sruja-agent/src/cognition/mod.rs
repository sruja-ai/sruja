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

pub mod decision;
pub mod hook;
pub mod runbook;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::llm::{
    CompletionRequest, CompletionResponse, LlmClient, LlmError, Message, ModelRouter, Usage,
};

pub use crate::llm::TaskTier;
use crate::memory::{AgenticMemory, Memory};
use crate::tool::{FileGuard, Phase, ToolError, ToolRegistry};
use crate::verify::{
    all_passed, run_verification_steps, VerifyOptions, VerifyResult, VerifyStatus, VerifyStep,
};
use crate::LearningEntry;

pub use decision::{DecisionRecord, DecisionStatus};
pub use hook::{Hook, HookAction, HookRegistry, Hooks, LoggingHook};
pub use runbook::{Runbook, RunbookSeverity};

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
            cheap: "gpt-4o-mini".into(),
            mid: "gpt-4o-mini".into(),
            premium: "gpt-4o".into(),
            review: "gpt-4o".into(),
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
    /// Additional instructions appended to the comprehension system prompt.
    /// Use for context-specific nudges (e.g., "call sruja_focus first").
    pub system_hints: Vec<String>,
    /// The critic ensemble: one probe-bound persona per perspective. When
    /// non-empty, [`Agent::critique`] fans these out in parallel and unions
    /// their issues (AND semantics for approval). When empty, falls back to a
    /// single call with the legacy [`CRITIQUE_SYSTEM_PROMPT`] (backward
    /// compatible). Default is [`CritiquePersona::default_personas`].
    pub critique_personas: Vec<CritiquePersona>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            models: ModelMapping::default(),
            tdd: true,
            review_every_change: true,
            spend_cap_usd: None,
            dry_run: false,
            max_tool_iterations: 15,
            system_hints: Vec::new(),
            critique_personas: CritiquePersona::default_personas(),
        }
    }
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
    Review,
}

impl SubtaskKind {
    pub fn phase(&self) -> Phase {
        match self {
            Self::Comprehend => Phase::Comprehend,
            Self::TestAuthor => Phase::TestAuthor,
            Self::Implement => Phase::Implement,
            Self::Verify => Phase::Implement,
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
    pub goal: String,
    pub subtasks: Vec<Subtask>,
    pub tdd: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risks: Vec<String>,
}

/// Result of executing a subtask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub subtask_id: String,
    pub status: StepStatus,
    pub output: String,
    pub usage: Usage,
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
        ]
    }
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
    memory: Option<(std::path::PathBuf, std::sync::Mutex<AgenticMemory>)>,
    #[cfg(feature = "mcp-client")]
    #[allow(dead_code)]
    mcp_manager: Option<crate::tool::mcp::McpClientManager>,
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
    pub async fn comprehend(&self, goal: &str) -> Result<Comprehension, AgentError> {
        self.guard.set_phase(Phase::Comprehend);
        self.hooks.on_phase_change(Phase::Comprehend).await;

        // Retrieve relevant memories (token-budget capped).
        let memory_context = if let Some((_, ref mem)) = self.memory {
            let learnings = mem.search(goal, 5);
            if learnings.is_empty() {
                String::new()
            } else {
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
                format!(
                    "\n\n## Past Learnings (from previous runs)\n\
                     The following lessons were learned from earlier tasks. \
                     Use them to avoid repeating mistakes and replicate successes:\n{}",
                    entries.join("\n")
                )
            }
        } else {
            String::new()
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
        let system = format!("{COMPREHENSION_SYSTEM_PROMPT}{memory_context}{hints}");
        let user = format!(
            "## Goal\n{goal}\n\n\
             ## Instructions\n\
             Use the available tools to explore the codebase. \
             Cite architecture element IDs in your findings. \
             Produce a concise, grounded understanding."
        );

        let req = CompletionRequest::prompt(&system, user).with_tools(self.tools.schemas());

        let (response, usage) = self.run_tool_loop(req).await?;

        let cited_elements = extract_element_ids(&response.content);

        Ok(Comprehension {
            goal: goal.to_string(),
            summary: response.content,
            cited_elements,
            key_findings: Vec::new(),
            risks: Vec::new(),
            usage,
        })
    }

    // --- Tool-calling loop (shared by all phases) ---

    /// Run the LLM tool-calling loop until the model stops requesting tools
    /// or the iteration limit is hit.
    pub async fn run_tool_loop(
        &self,
        mut req: CompletionRequest,
    ) -> Result<(CompletionResponse, Usage), AgentError> {
        let mut total_usage = Usage::default();
        let mut last_response: Option<CompletionResponse> = None;
        let mut soft_sent = false;
        let mut hard_sent = false;

        for iteration in 0..self.config.max_tool_iterations {
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
                    return Ok((response, total_usage));
                }
                return Ok((response, total_usage));
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
                let result = match self
                    .tools
                    .dispatch(&call.name, call.arguments.clone())
                    .await
                {
                    Ok(out) => out,
                    Err(e) => format!("ERROR: {e}"),
                };
                let truncated = truncate(&result, 8_000);
                tracing::debug!(
                    tool = %call.name,
                    result_len = truncated.len(),
                    result_preview = %truncated.chars().take(120).collect::<String>(),
                    "tool_loop: tool result"
                );
                req.messages.push(Message::tool_result(&call.id, truncated));
            }

            // ── Convergence pressure ─────────────────────────────────────
            // Some models (especially smaller OpenAI-compatible ones) never
            // stop calling tools on their own — they explore indefinitely.
            // When the tool-call budget is running low, inject a user message
            // that forces the model to produce a final answer. Each tier is
            // injected at most once to avoid flooding the context.
            let remaining = self.config.max_tool_iterations - iteration - 1;
            let quarter = (self.config.max_tool_iterations / 4).max(1);
            let half = self.config.max_tool_iterations / 2;
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
            max_iterations = self.config.max_tool_iterations,
            "tool_loop: model did not converge — returning last response as fallback"
        );
        let mut fallback = last_response.unwrap_or_else(|| {
            CompletionResponse::text(
                "ERROR: tool loop exhausted without any response from the model.",
            )
        });
        fallback.tool_calls.clear();
        fallback.finish_reason = crate::llm::FinishReason::Stop;
        Ok((fallback, total_usage))
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
        goal: &str,
        comprehension: &Comprehension,
    ) -> Result<Plan, AgentError> {
        if let HookAction::Abort(reason) = self.hooks.before_plan(goal).await {
            return Err(AgentError::HookAborted(reason));
        }

        let tdd_note = if self.config.tdd {
            "\n\nTDD MODE IS ON: You MUST emit test_author subtasks BEFORE any implement subtasks. \
             The framework enforces this — tests are written first, reviewed, then code is written \
             to pass the frozen tests. Tests and code are NEVER in flux simultaneously."
        } else {
            ""
        };

        let user = format!(
            "## Goal\n{goal}\n\n\
             ## Comprehension\n{}\n\n\
             ## Architecture Elements Cited\n{:?}\n\n\
             ## Instructions\n\
             Break this goal into concrete subtasks. Each subtask must specify:\n\
             - `id`: a short unique identifier (e.g. \"s1\", \"s2\")\n\
             - `description`: what to do (concise, actionable)\n\
             - `tier`: cheap (classification/extraction), mid (standard coding), \
               or premium (hard architecture reasoning)\n\
             - `kind`: test_author, implement, verify, or review\n\
             - `files`: list of files this subtask touches\n\
             - `acceptance_criteria`: how to verify completion\n\n\
             Output a JSON object with `subtasks` array and `risks` array.\n\
             {tdd_note}",
            comprehension.summary, comprehension.cited_elements,
        );

        let req =
            CompletionRequest::prompt(PLAN_SYSTEM_PROMPT, user).with_tools(self.tools.schemas());

        let (response, _usage) = self.run_tool_loop(req).await?;

        // Parse the plan from the LLM response.
        let plan = parse_plan_from_response(&response.content, goal, self.config.tdd);

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
        if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
            return Err(AgentError::HookAborted(reason));
        }

        Ok(plan)
    }

    /// Re-plan using the prior critique as feedback.
    ///
    /// This is the feedback edge that closes the outer ReAct loop: when the
    /// independent critic rejects a change, its `issues` and `suggestions`
    /// are injected into a new plan rather than discarded.
    pub async fn replan(
        &self,
        goal: &str,
        comprehension: &Comprehension,
        critique: &Critique,
    ) -> Result<Plan, AgentError> {
        if let HookAction::Abort(reason) = self.hooks.before_plan(goal).await {
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

        let tdd_note = if self.config.tdd {
            "\n\nTDD MODE IS ON: keep test_author subtasks BEFORE implement subtasks."
        } else {
            ""
        };

        let user = format!(
            "## Goal\n{goal}\n\n\
             ## Comprehension\n{}\n\n\
             ## Prior review outcome\n\
             The independent critic REJECTED the previous attempt (score: {:.0}%).\n\
             You must produce a revised plan that addresses the feedback.\n\n\
             ### Critic issues\n{issues}\n\n\
             ### Critic suggestions\n{suggestions}\n\n\
             ## Instructions\n\
             Output a REVISED plan as a JSON object with `subtasks` and `risks` arrays. \
             Each subtask needs `id` (short unique string like \"s1\"), `description`, `tier` (cheap|mid|premium), \
             `kind` (test_author|implement|verify|review), `files`, and \
             `acceptance_criteria`. Do not repeat failed approaches.{tdd_note}",
            comprehension.summary,
            critique.score * 100.0,
        );

        let req =
            CompletionRequest::prompt(PLAN_SYSTEM_PROMPT, user).with_tools(self.tools.schemas());

        let (response, _usage) = self.run_tool_loop(req).await?;
        let mut plan = parse_plan_from_response(&response.content, goal, self.config.tdd);

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

        if let HookAction::Abort(reason) = self.hooks.after_plan(&mut plan).await {
            return Err(AgentError::HookAborted(reason));
        }
        Ok(plan)
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
            let user = format!(
                "## Subtask: {}\n\n\
                 ## Description\n{}\n\n\
                 ## Acceptance Criteria\n{}\n\n\
                 ## Phase\n{:?}\n\n\
                 Execute this subtask using the available tools. \
                 Be precise. Cite evidence.",
                step.id,
                step.description,
                step.acceptance_criteria.join("\n"),
                phase,
            );

            let mut req = CompletionRequest::prompt(system, user).with_tools(self.tools.schemas());
            req.model = Some(self.model_for_tier(tier).to_string());

            let (response, tool_usage) = self.run_tool_loop(req).await?;

            let status = if response.content.contains("ERROR") {
                StepStatus::Failed
            } else {
                StepStatus::Ok
            };

            let result = StepResult {
                subtask_id: step.id.clone(),
                status,
                output: response.content,
                usage: tool_usage,
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

        // --- U4: memory injection (compounding loop) ---
        // Retrieve past GUARDRAIL learnings and render them as blind-spot
        // probes in the critic prompt. Playbooks are excluded — they inform
        // planning, not review, and would bias the critic toward the actor's
        // prior successes. Retrievals are recorded so `retrieval_count` /
        // utility counters stay accurate for the critique path, not just
        // comprehension.
        let mut injected_learning_ids: Vec<String> = Vec::new();
        let blind_spots = if let Some((_, ref mem)) = self.memory {
            let learnings = mem.search(&plan.goal, 5);
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

        let shared_user = format!(
            "## Goal\n{}\n\n\
             ## Plan\n{}\n\n\
             ## Execution Results\n{}{}",
            plan.goal,
            plan.subtasks
                .iter()
                .map(|s| format!("- [{}] {} ({:?})", s.id, s.description, s.tier))
                .collect::<Vec<_>>()
                .join("\n"),
            step_summary.join("\n"),
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
            usage.accumulate(&parsed.usage);
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
            plan.goal,
            successes,
            failures,
            comprehension.cited_elements,
            critique
                .map(|c| format!("approved={}, score={}", c.approved, c.score))
                .unwrap_or_else(|| "skipped".into()),
        );

        let req = CompletionRequest::prompt(REFLECTION_SYSTEM_PROMPT, &user)
            .with_model(&self.config.models.cheap);

        let (response, _usage) = self.run_tool_loop(req).await?;

        let learnings = parse_learnings_from_response(&response.content);

        for entry in &learnings {
            self.hooks.on_learning(entry).await;
        }

        // Persist learnings to memory.
        if let Some((ref repo, ref mem)) = self.memory {
            {
                let mut mem = mem.lock().unwrap();
                for entry in &learnings {
                    mem.add_learning(entry.clone());
                }
            }
            // Save to disk.
            let mem = mem.lock().unwrap();
            if let Err(e) = mem.save(repo) {
                tracing::warn!(error = %e, "failed to save learnings to memory");
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
    pub async fn run(&self, goal: &str) -> Result<AgentRunResult, AgentError> {
        let comprehension = self.comprehend(goal).await?;
        let plan = self.plan(goal, &comprehension).await?;
        let step_results = self.execute(&plan).await?;

        let critique = if self.config.review_every_change {
            Some(self.critique(&plan, &step_results).await?)
        } else {
            None
        };

        let _learnings = self
            .reflect(&comprehension, &plan, &step_results, critique.as_ref())
            .await?;

        // Generate decision record and runbook.
        let decision = self
            .generate_decision(&plan, &step_results, critique.as_ref())
            .await;
        let runbook = self.generate_runbook(&plan, &step_results).await;

        // Write artifacts to disk if a repo root is available.
        if let Some((ref repo, _)) = self.memory {
            if let Some(ref d) = decision {
                self.write_decision(repo, d).await;
            }
            if let Some(ref r) = runbook {
                self.write_runbook(repo, r).await;
            }
        }

        Ok(AgentRunResult {
            goal: goal.to_string(),
            comprehension,
            plan,
            step_results,
            critique,
            decision,
            runbook,
            total_usage: Usage::default(),
        })
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
    pub async fn run_loop(
        &self,
        goal: &str,
        loop_config: &LoopConfig,
    ) -> Result<LoopResult, AgentError> {
        let max_iterations = loop_config.max_iterations.max(1);
        let comprehension = self.comprehend(goal).await?;

        let mut iterations: Vec<LoopIteration> = Vec::new();
        let mut total_usage = Usage::default();
        let mut last_plan: Option<Plan> = None;
        let mut last_steps: Vec<StepResult> = Vec::new();
        let mut last_critique: Option<Critique> = None;
        let mut converged = false;
        let mut termination = LoopTermination::MaxIterations;
        let mut seen_signatures: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for iteration in 1..=max_iterations {
            self.hooks.before_iteration(iteration, max_iterations).await;
            let replanned = iteration > 1 && last_critique.is_some();

            // --- PLAN (or re-plan from critique feedback) ---
            let plan = if replanned {
                match self
                    .replan(goal, &comprehension, last_critique.as_ref().unwrap())
                    .await
                {
                    Ok(p) => p,
                    Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                        termination = LoopTermination::SpendCapExceeded(spent);
                        break;
                    }
                    Err(e) => return Err(e),
                }
            } else {
                match self.plan(goal, &comprehension).await {
                    Ok(p) => p,
                    Err(AgentError::Llm(LlmError::BudgetExceeded { spent, .. })) => {
                        termination = LoopTermination::SpendCapExceeded(spent);
                        break;
                    }
                    Err(e) => return Err(e),
                }
            };

            // --- EXECUTE ---
            let step_results = match self.execute(&plan).await {
                Ok(r) => r,
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
                }
            };
            total_usage.accumulate(&critique.usage);

            // --- DETERMINISTIC GRADER (independent of the LLM critic) ---
            // Runs the verifier, if configured. A failing step vetoes
            // convergence even when the critic approved, and the failures are
            // injected into the critique so the next replan addresses them.
            let verify_failed = if let Some(vconf) = &loop_config.verifier {
                let results =
                    run_verification_steps(&vconf.steps, &vconf.options, &vconf.workdir).await;
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

            // Record the iteration evidence BEFORE guardrail checks so the
            // caller always sees what happened, even on the triggering iteration.
            iterations.push(LoopIteration {
                iteration,
                replanned,
                plan_goal: plan.goal.clone(),
                subtask_count: plan.subtasks.len(),
                succeeded,
                failed,
                critique_approved: approved,
                critique_score: critique.score,
                critique_issues: critique.issues.clone(),
                verify_failed: verify_failed.clone(),
                injected_learning_ids: critique.injected_learning_ids.clone(),
                usage: critique.usage.clone(),
            });

            last_plan = Some(plan);
            last_steps = step_results.clone();
            last_critique = Some(critique);

            self.hooks
                .after_iteration(iteration, max_iterations, iterations.last().unwrap())
                .await;

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

        let plan = last_plan.ok_or(AgentError::Other("loop produced no plan".into()))?;
        let critique = last_critique;

        // Reflect + generate decision/runbook once, on the final state.
        let _learnings = self
            .reflect(&comprehension, &plan, &last_steps, critique.as_ref())
            .await?;
        let decision = self
            .generate_decision(&plan, &last_steps, critique.as_ref())
            .await;
        let runbook = self.generate_runbook(&plan, &last_steps).await;
        if let Some((ref repo, _)) = self.memory {
            if let Some(ref d) = decision {
                self.write_decision(repo, d).await;
            }
            if let Some(ref r) = runbook {
                self.write_runbook(repo, r).await;
            }
        }

        let final_result = AgentRunResult {
            goal: goal.to_string(),
            comprehension,
            plan,
            step_results: last_steps,
            critique,
            decision,
            runbook,
            total_usage: total_usage.clone(),
        };

        Ok(LoopResult {
            goal: goal.to_string(),
            iterations,
            converged,
            termination,
            total_usage,
            grader_source: "unknown".to_string(),
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
            plan.goal,
            successes,
            failures,
            critique
                .map(|c| format!("approved={}", c.approved))
                .unwrap_or_else(|| "skipped".into()),
        );

        let req = CompletionRequest::prompt(DECISION_SYSTEM_PROMPT, &user)
            .with_model(&self.config.models.cheap);

        let (response, _usage) = self.run_tool_loop(req).await.ok()?;
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
            plan.goal,
            plan.subtasks.iter()
                .map(|s| format!("- [{}] {}", s.id, s.description))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let req = CompletionRequest::prompt(RUNBOOK_SYSTEM_PROMPT, &user)
            .with_model(&self.config.models.cheap);

        let (response, _usage) = self.run_tool_loop(req).await.ok()?;
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

    // --- Convenience methods for DLC and pair programming ---

    /// Plan from a goal string (comprehends first, then plans).
    pub async fn plan_simple(&self, goal: &str) -> Result<Plan, AgentError> {
        let comprehension = self.comprehend(goal).await?;
        self.plan(goal, &comprehension).await
    }

    /// Execute a single subtask (for pair programming).
    pub async fn execute_step(
        &self,
        subtask: &Subtask,
        _comprehension: &Comprehension,
    ) -> Result<crate::pair::StepResult, AgentError> {
        let phase = subtask.kind.phase();
        self.guard.set_phase(phase);

        let system = EXECUTION_SYSTEM_PROMPT;
        let user = format!(
            "## Subtask: {}\n\n\
             ## Description\n{}\n\n\
             ## Acceptance Criteria\n{}\n\n\
             ## Phase\n{:?}\n\n\
             Execute this subtask. Be precise.",
            subtask.id,
            subtask.description,
            subtask.acceptance_criteria.join("\n"),
            phase,
        );

        let req = CompletionRequest::prompt(system, user).with_tools(self.tools.schemas());

        let response = self.complete_tiered(subtask.tier, req).await?;
        let (response, _tool_usage) = self
            .run_tool_loop(
                CompletionRequest::prompt(EXECUTION_SYSTEM_PROMPT, &response.content)
                    .with_tools(self.tools.schemas()),
            )
            .await?;

        Ok(crate::pair::StepResult {
            output: response.content,
            files_affected: subtask.files.clone(),
        })
    }

    /// Review a single change (for pair programming navigator).
    pub async fn review_change(
        &self,
        description: &str,
        output: &str,
    ) -> Result<crate::pair::ReviewResult, AgentError> {
        let user = format!(
            "## Change\n{}\n\n## Output\n{}\n\n\
             Review this change. Output JSON:\n\
             {{\"approved\": true/false, \"feedback\": \"...\"}}",
            description, output
        );

        let req =
            CompletionRequest::prompt("You are a code reviewer. Be concise and practical.", &user)
                .with_model(&self.config.models.cheap);

        let (response, _usage) = self.run_tool_loop(req).await?;
        let json_str = extract_json(&response.content);
        let value: serde_json::Value = serde_json::from_str(&json_str)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        Ok(crate::pair::ReviewResult {
            approved: value
                .get("approved")
                .and_then(|a| a.as_bool())
                .unwrap_or(true),
            feedback: value
                .get("feedback")
                .and_then(|f| f.as_str())
                .unwrap_or("No feedback")
                .to_string(),
        })
    }

    /// Suggest a fix for a rejected review (for pair programming).
    pub async fn suggest_fix(
        &self,
        review: &crate::pair::ReviewResult,
    ) -> Result<String, AgentError> {
        let user = format!(
            "## Review Feedback\n{}\n\n\
             Suggest a specific fix for this issue. Be concise.",
            review.feedback
        );

        let req = CompletionRequest::prompt(
            "You are suggesting a code fix. Be specific and actionable.",
            &user,
        )
        .with_model(&self.config.models.cheap);

        let (response, _usage) = self.run_tool_loop(req).await?;
        Ok(response.content)
    }

    /// Suggest cleanup steps (for pair programming navigator driving).
    pub async fn suggest_cleanup(
        &self,
        results: &[crate::pair::StepResult],
    ) -> Result<Vec<String>, AgentError> {
        let summaries: Vec<String> = results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let end = r
                    .output
                    .char_indices()
                    .nth(100)
                    .map(|(i, _)| i)
                    .unwrap_or(r.output.len());
                format!(
                    "{}. {} (files: {:?})",
                    i + 1,
                    &r.output[..end],
                    r.files_affected
                )
            })
            .collect();

        let user = format!(
            "## Changes made\n{}\n\n\
             Suggest cleanup steps. Output a JSON array of strings.",
            summaries.join("\n")
        );

        let req = CompletionRequest::prompt(
            "You are suggesting cleanup improvements. Be practical.",
            &user,
        )
        .with_model(&self.config.models.cheap);

        let (response, _usage) = self.run_tool_loop(req).await?;
        let json_str = extract_json(&response.content);
        let suggestions: Vec<String> =
            serde_json::from_str::<Vec<String>>(&json_str).unwrap_or_default();

        Ok(suggestions)
    }
}

fn comprehension_cited_elements(plan: &Plan) -> Vec<String> {
    plan.subtasks.iter().flat_map(|s| s.files.clone()).collect()
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

const COMPREHENSION_SYSTEM_PROMPT: &str = "\
You are a Principal Engineer with deep architectural expertise. \
Your job is to understand codebases thoroughly before recommending changes.\n\n\
Rules:\n\
1. Use tools to ground your understanding — never guess. \
   BUT limit yourself to 3-5 tool calls. After that, STOP calling tools and \
   produce your understanding as plain text.\n\
2. Cite architecture element IDs (e.g. Sruja.CLI, Sruja.Graph) in your findings.\n\
3. Assess blast radius and risks.\n\
4. Be concise. Cite evidence, not speculation.\n\n\
IMPORTANT: Once you have enough context (usually after 2-4 tool calls), you MUST \
stop calling tools and write your final answer as plain text in your response. \
Do NOT keep calling tools indefinitely.";

pub(crate) const PLAN_SYSTEM_PROMPT: &str = "\
You are a Principal Engineer decomposing work into concrete subtasks.\n\n\
Rules:\n\
1. Each subtask must have: id (short unique string like \"s1\"), description, tier (cheap/mid/premium), kind (test_author/implement/verify/review), files, acceptance_criteria.\n\
2. If TDD mode: test_author subtasks MUST come before implement subtasks.\n\
3. Tag complexity accurately: classification/extraction = cheap, standard coding = mid, hard architecture = premium.\n\
4. Identify risks and edge cases.\n\
5. Output a JSON object: {\"subtasks\": [...], \"risks\": [...]}.";

const EXECUTION_SYSTEM_PROMPT: &str = "\
You are a Principal Engineer executing a specific subtask.\n\n\
Rules:\n\
1. Use tools to accomplish the task — never guess file contents. \
   Read the target file(s) first, then make your edits, then STOP.\n\
2. Be precise and minimal — make the smallest change that satisfies acceptance criteria.\n\
3. If in TestAuthor phase: write tests only, do not touch implementation.\n\
4. If in Implement phase: write code to pass the frozen tests, do not modify tests.\n\
5. Cite evidence for every decision.\n\n\
IMPORTANT: After making your edits, you MUST stop calling tools and write a \
summary of what you changed as plain text. Do NOT keep calling tools after \
your edits are complete.";

const CRITIQUE_SYSTEM_PROMPT: &str = "\
You are a senior architect reviewing a change. Be adversarial but fair.\n\n\
Check:\n\
1. Does the change match the stated goal?\n\
2. Are acceptance criteria satisfied?\n\
3. Any architectural violations or boundary crossings?\n\
4. Is test coverage adequate?\n\
5. What is the blast radius?\n\n\
Respond with JSON: {\"approved\": bool, \"score\": 0.0-1.0, \"issues\": [...], \"suggestions\": [...]}";

const CORRECTNESS_PERSONA_PROMPT: &str = "You are a senior engineer reviewing a change for correctness failures. You are reviewing a change.\n\nAsk ONE question: what inputs or states break this?\nProbe specifically:\n- empty / nil / zero / max-boundary inputs\n- error and failure paths (does the change handle them or silently drop them?)\n- off-by-one, sign-flip, and partial-state cases\n- assumptions the change makes that could be false\n\nDo not give a generic verdict. For each concrete failure you can name, emit an issue. If you cannot name a specific input that breaks, approve.\n\nRespond with JSON: {\"approved\": bool, \"score\": 0.0-1.0, \"issues\": [...], \"suggestions\": [...]}";

const SPEC_COVERAGE_PERSONA_PROMPT: &str = "You are a senior engineer reviewing a change against its stated acceptance criteria. You are reviewing a change.\n\nAsk ONE question: which acceptance criterion is NOT addressed by this change?\nFor each numbered criterion, decide: addressed | partial | missing, with a one-line reason.\nAny 'missing' or 'partial' criterion is a blocking issue that names the criterion.\nIf no criteria are stated, or all are addressed, approve.\n\nRespond with JSON: {\"approved\": bool, \"score\": 0.0-1.0, \"issues\": [...], \"suggestions\": [...]}";

const BOUNDARY_PERSONA_PROMPT: &str = "You are a senior architect reviewing a change for boundary and drift violations. You are reviewing a change.\n\nAsk ONE question: what architectural boundary does this change cross?\nProbe specifically:\n- layering / dependency-direction violations (lower tier depending on higher)\n- forbidden dependencies and declared policy breaches\n- scope creep beyond the stated goal\n\nDo not restate metadata. Only emit an issue for a concrete, named crossing. If none, approve.\n\nRespond with JSON: {\"approved\": bool, \"score\": 0.0-1.0, \"issues\": [...], \"suggestions\": [...]}";

const REGRESSION_PERSONA_PROMPT: &str = "You are a senior engineer reviewing a change for regressions. You are reviewing a change.\n\nAsk ONE question: what previously-working behavior does this change break?\nProbe specifically:\n- callers of any modified signature\n- behavior other code depends on that is now altered\n- tests that would now fail (and whether new tests cover the new path)\n\nIf you cannot name a concrete regression path, approve.\n\nRespond with JSON: {\"approved\": bool, \"score\": 0.0-1.0, \"issues\": [...], \"suggestions\": [...]}";

const REFLECTION_SYSTEM_PROMPT: &str = "\
You are extracting lessons from a completed task.\n\n\
For each learning, produce JSON:\n\
{\"context\": \"what happened\", \"hypothesis\": \"why\", \"guardrail_advice\": \"what to do/not do next time\", \"kind\": \"playbook|guardrail\"}\n\
- playbook = what worked, do again\n\
- guardrail = what failed, don't repeat\n\
Output a JSON array of learnings.";

const DECISION_SYSTEM_PROMPT: &str = "\
You are writing a decision record for a code change.\n\n\
This record will be read at 3AM by someone with zero context.\n\
Be clear, concise, and thorough. Explain:\n\
1. WHY this change was needed (context)\n\
2. WHAT was decided (decision)\n\
3. What FOLLOWS from this decision (consequences)\n\
4. What ELSE was considered and why it was rejected (alternatives)\n\n\
Output JSON: {\"title\": \"...\", \"context\": \"...\", \"decision\": \"...\", \"consequences\": [...], \"alternatives\": [...]}";

const RUNBOOK_SYSTEM_PROMPT: &str = "\
You are writing a runbook for handling production failures.\n\n\
This runbook will be read at 3AM by someone who is tired and stressed.\n\
Be practical, specific, and actionable. Include:\n\
1. What would trigger this runbook (trigger)\n\
2. How to detect the problem (symptoms)\n\
3. Step-by-step diagnosis\n\
4. Step-by-step resolution\n\
5. How to roll back if needed\n\
6. How to verify the fix worked\n\n\
Output JSON: {\"title\": \"...\", \"trigger\": \"...\", \"severity\": \"critical|high|medium|low\", \"symptoms\": [...], \"diagnosis\": [...], \"resolution\": [...], \"rollback\": [...], \"verification\": [...]}";

// ---------------------------------------------------------------------------
// Response parsing helpers
// ---------------------------------------------------------------------------

pub fn parse_plan_from_response(content: &str, goal: &str, tdd: bool) -> Plan {
    // Try to extract JSON from the response (may be wrapped in markdown).
    let json_str = extract_json(content);

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let subtasks: Vec<Subtask> = value
            .get("subtasks")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .enumerate()
                    .filter_map(|(idx, st)| {
                        // `id` is optional — synthesize if the model omits it.
                        let id = st
                            .get("id")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| format!("s{}", idx + 1));

                        // `description`, `tier`, `kind` are required — drop with a
                        // diagnostic log if missing so failures are traceable.
                        let description = match st.get("description").and_then(|v| v.as_str()) {
                            Some(d) => d.to_string(),
                            None => {
                                tracing::warn!(
                                    index = idx,
                                    "plan subtask dropped: missing or invalid 'description'"
                                );
                                return None;
                            }
                        };
                        let tier_str = match st.get("tier").and_then(|v| v.as_str()) {
                            Some(t) => t,
                            None => {
                                tracing::warn!(
                                    index = idx,
                                    "plan subtask dropped: missing or invalid 'tier'"
                                );
                                return None;
                            }
                        };
                        let kind_str = match st.get("kind").and_then(|v| v.as_str()) {
                            Some(k) => k,
                            None => {
                                tracing::warn!(
                                    index = idx,
                                    "plan subtask dropped: missing or invalid 'kind'"
                                );
                                return None;
                            }
                        };

                        Some(Subtask {
                            id,
                            description,
                            tier: parse_tier(tier_str),
                            kind: parse_kind(kind_str),
                            files: st
                                .get("files")
                                .and_then(|f| f.as_array())
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default(),
                            acceptance_criteria: st
                                .get("acceptance_criteria")
                                .and_then(|a| a.as_array())
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let risks: Vec<String> = value
            .get("risks")
            .and_then(|r| r.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // If no subtasks survived parsing, fall back to a single subtask
        // so the agent still attempts the goal rather than silently doing nothing.
        if subtasks.is_empty() {
            tracing::warn!(
                "plan parsed successfully but contained 0 subtasks — \
                 falling back to single-subtask plan"
            );
            return Plan {
                goal: goal.to_string(),
                subtasks: vec![Subtask {
                    id: "s1".into(),
                    description: goal.to_string(),
                    tier: TaskTier::Mid,
                    kind: SubtaskKind::Implement,
                    files: Vec::new(),
                    acceptance_criteria: Vec::new(),
                }],
                tdd,
                risks,
            };
        }

        return Plan {
            goal: goal.to_string(),
            subtasks,
            tdd,
            risks,
        };
    }

    // Fallback: single subtask plan.
    Plan {
        goal: goal.to_string(),
        subtasks: vec![Subtask {
            id: "s1".into(),
            description: goal.to_string(),
            tier: TaskTier::Mid,
            kind: SubtaskKind::Implement,
            files: Vec::new(),
            acceptance_criteria: Vec::new(),
        }],
        tdd,
        risks: Vec::new(),
    }
}

fn parse_critique_from_response(content: &str, usage: Usage) -> Critique {
    let json_str = extract_json(content);

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
        return Critique {
            approved: value
                .get("approved")
                .and_then(|a| a.as_bool())
                .unwrap_or(false),
            score: value.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0),
            issues: value
                .get("issues")
                .and_then(|i| i.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            suggestions: value
                .get("suggestions")
                .and_then(|s| s.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            usage,
            persona_breakdown: Vec::new(),
            injected_learning_ids: Vec::new(),
        };
    }

    // Fallback: check for approve/reject keywords.
    let lower = content.to_lowercase();
    let is_approved = lower.contains("\napproved") || lower.starts_with("approved");

    Critique {
        approved: is_approved,
        score: if is_approved { 0.8 } else { 0.3 },
        issues: vec!["could not parse structured critique".into()],
        suggestions: Vec::new(),
        usage,
        persona_breakdown: Vec::new(),
        injected_learning_ids: Vec::new(),
    }
}

fn parse_learnings_from_response(content: &str) -> Vec<LearningEntry> {
    let json_str = extract_json(content);

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let arr = match &value {
            serde_json::Value::Array(a) => a.clone(),
            serde_json::Value::Object(_) => vec![value.clone()],
            _ => return Vec::new(),
        };

        return arr
            .iter()
            .filter_map(|v| {
                let context = v.get("context")?.as_str()?;
                let hypothesis = v.get("hypothesis")?.as_str()?;
                let advice = v.get("guardrail_advice")?.as_str()?;
                let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("playbook");

                Some(match kind {
                    "guardrail" => LearningEntry::guardrail(context, hypothesis, advice),
                    _ => LearningEntry::playbook(context, hypothesis, advice),
                })
            })
            .collect();
    }

    Vec::new()
}

/// Extract JSON from a response that may contain markdown code fences.
fn extract_json(content: &str) -> String {
    // Try to find JSON in code fences.
    if let Some(start) = content.find("```json") {
        let rest = &content[start + 7..];
        if let Some(end) = rest.find("```") {
            return rest[..end].trim().to_string();
        }
    }
    if let Some(start) = content.find("```") {
        let rest = &content[start + 3..];
        // Skip optional language tag.
        let rest = rest.lines().skip(1).collect::<Vec<_>>().join("\n");
        if let Some(end) = rest.find("```") {
            return rest[..end].trim().to_string();
        }
    }
    // Try to find a JSON object or array directly.
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            return content[start..=end].to_string();
        }
    }
    if let Some(start) = content.find('[') {
        if let Some(end) = content.rfind(']') {
            return content[start..=end].to_string();
        }
    }
    content.to_string()
}

fn parse_tier(s: &str) -> TaskTier {
    match s.to_lowercase().as_str() {
        "cheap" | "low" | "simple" => TaskTier::Cheap,
        "premium" | "high" | "complex" | "hard" => TaskTier::Premium,
        _ => TaskTier::Mid,
    }
}

fn parse_kind(s: &str) -> SubtaskKind {
    match s.to_lowercase().as_str() {
        "test_author" | "test" | "write_test" | "testing" => SubtaskKind::TestAuthor,
        "implement" | "code" | "implementing" => SubtaskKind::Implement,
        "verify" | "verification" | "check" => SubtaskKind::Verify,
        "review" | "critique" => SubtaskKind::Review,
        _ => SubtaskKind::Comprehend,
    }
}

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
    memory_repo: Option<std::path::PathBuf>,
    #[cfg(feature = "mcp-client")]
    mcp_manager: Option<crate::tool::mcp::McpClientManager>,
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
    pub fn memory(mut self, repo_root: impl Into<std::path::PathBuf>) -> Self {
        self.memory_repo = Some(repo_root.into());
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

        // Load memory if a repo root was provided.
        let memory = self.memory_repo.map(|repo| {
            let mem = AgenticMemory::load(&repo).unwrap_or_default();
            (repo, std::sync::Mutex::new(mem))
        });

        #[cfg(feature = "mcp-client")]
        let mcp_manager = self.mcp_manager;

        Ok(Agent {
            llm,
            tools,
            guard: self.guard,
            hooks: HookRegistry::new(self.hooks),
            config: self.config,
            memory,
            #[cfg(feature = "mcp-client")]
            mcp_manager,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::sync::Mutex;

    // --- Ensemble critic tests (U1) ---
    /// Helper for scripted ensemble tests: a mock that returns different
    /// responses based on which persona's system prompt substring it matches.

    struct DropGuard(Arc<AtomicUsize>);
    impl Drop for DropGuard {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }

    struct PersonaScriptedLlm {
        responses: Vec<PersonaResponse>,
        max_concurrent: Arc<AtomicUsize>,
        active: Arc<AtomicUsize>,
        last_system_prompt: Arc<Mutex<String>>,
        last_user_prompt: Arc<Mutex<String>>,
    }

    #[derive(Debug, Clone)]
    struct PersonaResponse {
        system_prompt_contains: &'static str,
        approved: bool,
        score: f64,
        issues: Vec<String>,
    }

    impl PersonaScriptedLlm {
        fn new(responses: Vec<PersonaResponse>) -> Arc<Self> {
            Arc::new(Self {
                responses,
                max_concurrent: Arc::new(AtomicUsize::new(0)),
                active: Arc::new(AtomicUsize::new(0)),
                last_system_prompt: Arc::new(Mutex::new(String::new())),
                last_user_prompt: Arc::new(Mutex::new(String::new())),
            })
        }

        #[allow(dead_code)]
        fn max_concurrent(&self) -> usize {
            self.max_concurrent.load(Ordering::SeqCst)
        }

        /// Returns the last system prompt the mock received, for test assertions.
        fn received_system_prompt(&self) -> String {
            self.last_system_prompt.lock().unwrap().clone()
        }

        /// Returns the last user prompt the mock received, for test assertions.
        fn received_user_prompt(&self) -> String {
            self.last_user_prompt.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl LlmClient for PersonaScriptedLlm {
        async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            let prev_max = self.max_concurrent.load(Ordering::SeqCst);
            if active > prev_max {
                self.max_concurrent.store(active, Ordering::SeqCst);
            }
            let _guard = DropGuard(self.active.clone());

            // Yield to allow other spawned tasks to start, proving parallelism.
            tokio::task::yield_now().await;

            let sys = req
                .messages
                .first()
                .map(|m| m.content.as_str())
                .unwrap_or("");
            *self.last_system_prompt.lock().unwrap() = sys.to_string();
            let user = req
                .messages
                .get(1)
                .map(|m| m.content.as_str())
                .unwrap_or("");
            *self.last_user_prompt.lock().unwrap() = user.to_string();
            let content = sys
                .lines()
                .find_map(|_l| {
                    self.responses
                        .iter()
                        .find(|r| sys.contains(r.system_prompt_contains))
                        .map(|r| {
                            format!(
                                r#"{{"approved":{},"score":{},"issues":{:?},"suggestions":[]}}"#,
                                r.approved, r.score, r.issues
                            )
                        })
                })
                .unwrap_or_else(|| {
                    r#"{"approved":false,"score":0.0,"issues":[],"suggestions":[]}"#.to_string()
                });

            Ok(CompletionResponse {
                content,
                tool_calls: Vec::new(),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                model: "scripted-ensemble".into(),
                finish_reason: crate::llm::FinishReason::Stop,
            })
        }

        fn default_model(&self) -> &str {
            "scripted-ensemble"
        }
    }

    #[test]
    fn default_config_is_tdd_and_review() {
        let config = AgentConfig::default();
        assert!(config.tdd);
        assert!(config.review_every_change);
    }

    #[test]
    fn extract_ids_works() {
        let ids = extract_element_ids("See Sruja.CLI and Sruja.Graph.KnowledgeGraph for details.");
        assert!(ids.contains(&"Sruja.CLI".to_string()));
        assert!(ids.contains(&"Sruja.Graph.KnowledgeGraph".to_string()));
    }

    #[test]
    fn parse_plan_synthesizes_id_when_missing() {
        // Simulate a real LLM that follows the prompt but omits `id`
        // (the prompt didn't ask for it before the fix).
        let raw = r#"{"subtasks":[
            {"description":"write add()","tier":"mid","kind":"implement","files":["src/main.rs"]},
            {"description":"test add()","tier":"cheap","kind":"test_author","files":["src/main.rs"]}
        ],"risks":[]}"#;
        let plan = parse_plan_from_response(raw, "add function", false);
        assert_eq!(plan.subtasks.len(), 2, "both subtasks should survive");
        assert_eq!(plan.subtasks[0].id, "s1", "first id synthesized");
        assert_eq!(plan.subtasks[1].id, "s2", "second id synthesized");
        assert_eq!(plan.subtasks[0].description, "write add()");
    }

    #[test]
    fn parse_plan_drops_subtask_missing_required_field() {
        // A subtask missing `tier` should be dropped (not crash), while
        // valid siblings survive.
        let raw = r#"{"subtasks":[
            {"description":"ok","tier":"mid","kind":"implement"},
            {"description":"no tier here","kind":"verify"}
        ],"risks":[]}"#;
        let plan = parse_plan_from_response(raw, "test", false);
        assert_eq!(plan.subtasks.len(), 1, "malformed subtask dropped");
        assert_eq!(plan.subtasks[0].description, "ok");
    }

    #[test]
    fn parse_plan_preserves_explicit_ids() {
        let raw = r#"{"subtasks":[
            {"id":"custom-id","description":"task","tier":"premium","kind":"review"}
        ],"risks":[]}"#;
        let plan = parse_plan_from_response(raw, "test", false);
        assert_eq!(plan.subtasks.len(), 1);
        assert_eq!(plan.subtasks[0].id, "custom-id");
    }

    #[test]
    fn parse_plan_empty_array_falls_back_to_single_subtask() {
        // Model returned valid JSON but with an empty subtasks array
        // (e.g. it hallucinated the work was already done).
        let raw = r#"{"subtasks":[],"risks":["nothing to do"]}"#;
        let plan = parse_plan_from_response(raw, "add the function", false);
        assert_eq!(plan.subtasks.len(), 1, "fallback produces single subtask");
        assert_eq!(plan.subtasks[0].id, "s1");
        assert_eq!(plan.subtasks[0].description, "add the function");
        assert_eq!(plan.subtasks[0].kind, SubtaskKind::Implement);
        assert_eq!(plan.risks, vec!["nothing to do"]);
    }

    #[test]
    fn parse_critique_i_do_not_approve_does_not_flip_to_approved() {
        let raw = "I do not approve this plan; it's missing tests.";
        let critique = parse_critique_from_response(raw, Usage::default());
        assert!(!critique.approved, "'I do not approve' should be rejected");
        assert_eq!(critique.score, 0.3);
        assert!(critique
            .issues
            .contains(&"could not parse structured critique".to_string()));
    }

    #[test]
    fn parse_critique_approved_keyword_at_line_start_passes() {
        let raw = "Approved - the plan looks solid.";
        let critique = parse_critique_from_response(raw, Usage::default());
        assert!(critique.approved, "'Approved' at start should pass");
        assert_eq!(critique.score, 0.8);
    }

    #[test]
    fn parse_critique_approved_keyword_on_new_line_passes() {
        let raw = "I reviewed this.\nApproved - all good.";
        let critique = parse_critique_from_response(raw, Usage::default());
        assert!(critique.approved, "'\\nApproved' should pass");
        assert_eq!(critique.score, 0.8);
    }

    #[test]
    fn parse_critique_do_not_approve_fails() {
        let raw = "do not approve - tests are missing.";
        let critique = parse_critique_from_response(raw, Usage::default());
        assert!(!critique.approved, "'do not approve' should fail");
        assert_eq!(critique.score, 0.3);
    }

    #[tokio::test]
    async fn verify_veto_when_critic_approves_but_fails_allowlisted_command() {
        use crate::verify;
        use verify::{VerifyOptions, VerifyStatus, VerifyStep};

        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();

        let steps = vec![VerifyStep {
            id: "test1".into(),
            command: "cargo".into(),
            args: vec!["test".into(), "--nonexistent-flag-xyz123".into()],
            expected: None,
        }];

        let opts = VerifyOptions {
            allowed_executables: vec!["cargo".into()],
            continue_on_error: false,
            timeout_ms: 5000,
        };

        let results = verify::run_verification_steps(&steps, &opts, repo).await;

        assert!(
            !verify::all_passed(&results),
            "failing cargo test should not pass"
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, VerifyStatus::Failed);
    }

    // --- Loop spine tests (critique -> replan closure) ---

    use async_trait::async_trait;

    /// A scripted LLM that routes by system-prompt content so the outer loop
    /// can be driven without a real provider. The critic flips to `approved`
    /// after `reject_first` rejections.
    struct ScriptedLlm {
        critique_calls: AtomicUsize,
        reject_first: usize,
    }

    impl ScriptedLlm {
        fn approve_after(reject_first: usize) -> Arc<Self> {
            Arc::new(Self {
                critique_calls: AtomicUsize::new(0),
                reject_first,
            })
        }
    }

    #[async_trait]
    impl LlmClient for ScriptedLlm {
        async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let sys = req
                .messages
                .first()
                .map(|m| m.content.as_str())
                .unwrap_or("");
            let content = if sys.contains("reviewing a change") {
                let n = self.critique_calls.fetch_add(1, Ordering::SeqCst);
                let approved = n >= self.reject_first;
                if approved {
                    r#"{"approved":true,"score":0.9,"issues":[],"suggestions":[]}"#.to_string()
                } else {
                    r#"{"approved":false,"score":0.2,"issues":["tests missing"],"suggestions":["add tests"]}"#
                        .to_string()
                }
            } else if sys.contains("decomposing work into concrete subtasks") {
                r#"{"subtasks":[{"id":"s1","description":"implement feature","tier":"mid","kind":"implement","files":[],"acceptance_criteria":["it works"]}],"risks":[]}"#
                    .to_string()
            } else if sys.contains("executing a specific subtask") {
                "done".to_string()
            } else if sys.contains("understand codebases thoroughly") {
                "Understood the goal.".to_string()
            } else {
                "{}".to_string()
            };

            Ok(CompletionResponse {
                content,
                tool_calls: Vec::new(),
                usage: Usage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                model: "scripted".into(),
                finish_reason: crate::llm::FinishReason::Stop,
            })
        }

        fn default_model(&self) -> &str {
            "scripted"
        }
    }

    fn loop_test_agent(llm: Arc<dyn LlmClient>) -> Agent {
        let config = AgentConfig {
            tdd: false, // keep execution single-phase for the loop test
            ..Default::default()
        };
        Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds")
    }

    #[tokio::test]
    async fn run_loop_converges_on_second_critique() {
        // Critic rejects once, then approves -> converge in 2 iterations.
        let llm = ScriptedLlm::approve_after(1);
        let agent = loop_test_agent(llm);
        let result = agent
            .run_loop("ship the feature", &LoopConfig::default())
            .await
            .expect("loop runs");

        assert!(result.converged);
        assert_eq!(result.termination, LoopTermination::Approved);
        assert_eq!(result.iteration_count(), 2);
        // Iteration 1 rejected, iteration 2 approved.
        assert!(!result.iterations[0].critique_approved);
        assert!(result.iterations[1].critique_approved);
        // The feedback edge fired: iteration 2 was a re-plan.
        assert!(result.iterations[1].replanned);
        assert!(!result.iterations[0].replanned);
    }

    #[tokio::test]
    async fn run_loop_exhausts_budget_without_convergence() {
        // Critic never approves, and oscillation detection is off so we
        // actually exhaust the iteration budget.
        let llm = ScriptedLlm::approve_after(usize::MAX);
        let agent = loop_test_agent(llm);
        let cfg = LoopConfig {
            max_iterations: 2,
            detect_oscillation: false,
            ..Default::default()
        };
        let result = agent
            .run_loop("ship the feature", &cfg)
            .await
            .expect("loop runs");

        assert!(!result.converged);
        assert_eq!(result.termination, LoopTermination::MaxIterations);
        assert_eq!(result.iteration_count(), 2);
        // Last iteration's critique issues carried forward.
        assert!(!result.iterations[1].critique_issues.is_empty());
    }

    #[tokio::test]
    async fn run_loop_no_replan_terminates_after_first_rejection() {
        let llm = ScriptedLlm::approve_after(usize::MAX);
        let agent = loop_test_agent(llm);
        let cfg = LoopConfig {
            max_iterations: 5,
            replan_on_failure: false,
            ..Default::default()
        };
        let result = agent
            .run_loop("ship the feature", &cfg)
            .await
            .expect("loop runs");

        assert!(!result.converged);
        assert_eq!(result.termination, LoopTermination::NoReplan);
        assert_eq!(result.iteration_count(), 1);
    }

    #[test]
    fn loop_config_defaults_are_sane() {
        let c = LoopConfig::default();
        assert!(c.stop_on_approval);
        assert!(c.replan_on_failure);
        assert!(c.max_iterations >= 1);
        assert!(c.detect_oscillation);
        assert!(c.spend_cap_usd.is_none());
    }

    #[test]
    fn loop_result_iteration_count_counts_records() {
        let result = LoopResult {
            goal: "g".into(),
            iterations: vec![
                LoopIteration {
                    iteration: 1,
                    replanned: false,
                    plan_goal: "g".into(),
                    subtask_count: 1,
                    succeeded: 1,
                    failed: 0,
                    critique_approved: false,
                    critique_score: 0.2,
                    critique_issues: vec!["x".into()],
                    verify_failed: Vec::new(),
                    injected_learning_ids: Vec::new(),
                    usage: Usage::default(),
                },
                LoopIteration {
                    iteration: 2,
                    replanned: true,
                    plan_goal: "g".into(),
                    subtask_count: 1,
                    succeeded: 1,
                    failed: 0,
                    critique_approved: true,
                    critique_score: 0.9,
                    critique_issues: vec![],
                    verify_failed: Vec::new(),
                    injected_learning_ids: Vec::new(),
                    usage: Usage::default(),
                },
            ],
            converged: true,
            termination: LoopTermination::Approved,
            total_usage: Usage::default(),
            grader_source: "test".to_string(),
            final_result: AgentRunResult {
                goal: "g".into(),
                comprehension: Comprehension {
                    goal: "g".into(),
                    summary: String::new(),
                    cited_elements: Vec::new(),
                    key_findings: Vec::new(),
                    risks: Vec::new(),
                    usage: Usage::default(),
                },
                plan: Plan {
                    goal: "g".into(),
                    subtasks: Vec::new(),
                    tdd: false,
                    risks: Vec::new(),
                },
                step_results: Vec::new(),
                critique: None,
                decision: None,
                runbook: None,
                total_usage: Usage::default(),
            },
        };
        assert_eq!(result.iteration_count(), 2);
    }

    // --- ACT phase test: the loop mutates files via tools ---

    /// A scripted LLM that issues a `file_write` tool call the first time it
    /// is asked to execute, then terminates the tool loop with plain text.
    /// Critic approves immediately so the loop converges in one iteration.
    struct ActingLlm {
        execute_calls: AtomicUsize,
    }

    #[async_trait]
    impl LlmClient for ActingLlm {
        async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let sys = req
                .messages
                .first()
                .map(|m| m.content.as_str())
                .unwrap_or("");
            let usage = Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            };
            let content = if sys.contains("reviewing a change") {
                r#"{"approved":true,"score":0.9,"issues":[],"suggestions":[]}"#.to_string()
            } else if sys.contains("executing a specific subtask") {
                let n = self.execute_calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    return Ok(CompletionResponse {
                        content: "Writing the file.".into(),
                        tool_calls: vec![crate::llm::ToolCall {
                            id: "call_1".into(),
                            name: "file_write".into(),
                            arguments: serde_json::json!({
                                "path": "src/hello.rs",
                                "content": "fn main() { println!(\"hello from the loop\"); }\n"
                            }),
                        }],
                        usage: usage.clone(),
                        model: "acting".into(),
                        finish_reason: crate::llm::FinishReason::ToolCalls,
                    });
                }
                "done".to_string()
            } else if sys.contains("decomposing work into concrete subtasks") {
                r#"{"subtasks":[{"id":"s1","description":"write hello module","tier":"mid","kind":"implement","files":["src/hello.rs"],"acceptance_criteria":["file exists"]}],"risks":[]}"#
                    .to_string()
            } else if sys.contains("understand codebases thoroughly") {
                "Understood.".to_string()
            } else {
                "{}".to_string()
            };

            Ok(CompletionResponse {
                content,
                tool_calls: Vec::new(),
                usage,
                model: "acting".into(),
                finish_reason: crate::llm::FinishReason::Stop,
            })
        }

        fn default_model(&self) -> &str {
            "acting"
        }
    }

    #[tokio::test]
    async fn run_loop_actually_mutates_files_via_tools() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();

        let llm = Arc::new(ActingLlm {
            execute_calls: AtomicUsize::new(0),
        });
        let tools = ToolRegistry::with_builtin(root.clone(), Vec::new());
        let config = AgentConfig {
            tdd: false,
            review_every_change: true,
            ..Default::default()
        };
        let agent = Agent::builder()
            .llm(llm)
            .tools(tools)
            .config(config)
            .build()
            .expect("agent builds");

        let result = agent
            .run_loop("create hello module", &LoopConfig::default())
            .await
            .expect("loop runs");

        // The loop converged on iteration 1.
        assert!(result.converged, "expected convergence");
        assert_eq!(result.iteration_count(), 1);

        // The ACT phase mutated the filesystem — the file exists on disk.
        let written = std::fs::read_to_string(root.join("src/hello.rs")).expect("file was written");
        assert!(written.contains("hello from the loop"));
    }

    #[tokio::test]
    async fn run_loop_convergence_vetoed_by_deterministic_verifier() {
        // Critic approves immediately, but a deterministic verify step fails.
        // The independent grader must veto convergence regardless of the LLM.
        let llm = ScriptedLlm::approve_after(0);
        let agent = loop_test_agent(llm);
        let failing_step = VerifyStep {
            id: "must_fail".into(),
            command: "git".into(),
            args: vec!["not-a-real-git-subcommand".into()],
            expected: None,
        };
        let cfg = LoopConfig {
            // Oscillation detection off so a repeated verify-failure signature
            // doesn't terminate the loop before we observe the veto.
            detect_oscillation: false,
            verifier: Some(VerifierConfig {
                steps: vec![failing_step],
                options: VerifyOptions::default(),
                workdir: std::env::temp_dir(),
            }),
            ..Default::default()
        };
        let result = agent
            .run_loop("ship the feature", &cfg)
            .await
            .expect("loop runs");

        assert!(!result.converged, "verifier failure must veto convergence");
        assert_ne!(
            result.termination,
            LoopTermination::Approved,
            "must not terminate as Approved when verify fails"
        );
        // Every iteration ran the grader and recorded the failure.
        assert!(result
            .iterations
            .iter()
            .all(|i| !i.verify_failed.is_empty()));
    }

    // --- Guardrail tests ---

    #[tokio::test]
    async fn run_loop_terminates_on_spend_cap() {
        // ScriptedLlm returns small but non-zero usage per call.
        // After one iteration (execute + critique), the estimated cost
        // exceeds a tiny cap → SpendCapExceeded.
        let llm = ScriptedLlm::approve_after(usize::MAX);
        let agent = loop_test_agent(llm);
        let cfg = LoopConfig {
            max_iterations: 5,
            spend_cap_usd: Some(0.00001),
            detect_oscillation: false,
            ..Default::default()
        };
        let result = agent
            .run_loop("ship the feature", &cfg)
            .await
            .expect("loop runs");

        assert!(!result.converged);
        assert!(
            matches!(result.termination, LoopTermination::SpendCapExceeded(cost) if cost > 0.0),
            "expected SpendCapExceeded, got {:?}",
            result.termination
        );
        // Should have terminated before exhausting all 5 iterations.
        assert!(result.iteration_count() < 5);
    }

    #[tokio::test]
    async fn run_loop_detects_oscillation() {
        // ScriptedLlm always rejects with the same issues ("tests missing").
        // After iteration 2, the same critique signature repeats → Oscillation.
        let llm = ScriptedLlm::approve_after(usize::MAX);
        let agent = loop_test_agent(llm);
        let cfg = LoopConfig {
            max_iterations: 5,
            detect_oscillation: true,
            ..Default::default()
        };
        let result = agent
            .run_loop("ship the feature", &cfg)
            .await
            .expect("loop runs");

        assert!(!result.converged);
        assert_eq!(result.termination, LoopTermination::Oscillation);
        // Oscillation is detected at iteration 2 (the first repeat).
        assert_eq!(result.iteration_count(), 2);
        // Both iterations had the same critique issues.
        assert_eq!(
            result.iterations[0].critique_issues,
            result.iterations[1].critique_issues
        );
    }

    // --- Ensemble critic tests (U1) ---

    #[tokio::test]
    async fn ensemble_one_persona_blocks_union_issues_and_min_score() {
        // One persona blocks → ensemble ANDs = false, score = min,
        // issues union, tagged. Three personas approve.
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: false,
                score: 0.2,
                issues: vec!["buffer overflow on empty input".into()],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.8,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec![],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "fix bug".into(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(!result.approved);
        // All personas' issues unioned. The correctness persona's issue is present.
        assert!(result
            .issues
            .contains(&"buffer overflow on empty input".into()));
        // Score is min across personas (blocking 0.2 wins).
        assert!((result.score - 0.2).abs() < f64::EPSILON);
        // persona_breakdown has all four personas.
        assert_eq!(result.persona_breakdown.len(), 4);
        let correctness_result = result
            .persona_breakdown
            .iter()
            .find(|p| p.id == "correctness")
            .expect("correctness persona recorded");
        assert!(!correctness_result.approved);
        assert_eq!(correctness_result.score, 0.2);
        assert!(correctness_result.issues == vec!["buffer overflow on empty input"]);
    }

    #[test]
    fn ensemble_empty_personas_fallback_to_single_critic() {
        // Empty personas → single legacy call with CRITIQUE_SYSTEM_PROMPT,
        // additive fields empty.
        let mut config = AgentConfig::default();
        config.critique_personas.clear();

        let agent = Agent::builder()
            .llm(ScriptedLlm::approve_after(0))
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");

        // Blocking against the empty-personas fallback requires a sync environment.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = agent
                .critique(
                    &Plan {
                        goal: "test".into(),
                        subtasks: vec![],
                        tdd: false,
                        risks: vec![],
                    },
                    &vec![],
                )
                .await
                .expect("critique runs");

            // Approved, no persona_breakdown, no injected_learning_ids.
            assert!(result.approved);
            assert_eq!(result.persona_breakdown, vec![]);
            assert_eq!(result.injected_learning_ids, Vec::<String>::new());
        });
    }

    #[tokio::test]
    async fn ensemble_union_dedup_issues() {
        // Multiple personas report the same semantic issue → union with
        // dedup. Sorted order for determinism.
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 0.9,
                issues: vec!["tests missing".into()],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.8,
                issues: vec!["tests missing".into()],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec!["tests missing".into()],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec!["tests missing".into()],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "test".into(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        // Approved (all personas approve), issues deduped to one.
        assert!(result.approved);
        // Four personas each reported "tests missing" → union has one entry.
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0], "tests missing");
    }

    #[tokio::test]
    async fn ensemble_parallel_dispatch_is_concurrent() {
        // Parallel dispatch: the ensemble uses tokio::JoinSet and all
        // personas run concurrently. Deterministic check: record active
        // concurrency high-water mark.
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.8,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec![],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm.clone())
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "concurrency check".into(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(result.approved);
        // High-water mark >= 2 (both personas ran simultaneously at some point).
        assert!(llm.max_concurrent.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn ensemble_all_personas_approve() {
        // All personas approve → merged approved == true, issues empty, score == 1.0.
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 1.0,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.95,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec![],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "all good".into(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(result.approved);
        assert!(result.issues.is_empty());
        // Score is min across personas (0.85 wins).
        assert!((result.score - 0.85).abs() < f64::EPSILON);
        assert_eq!(result.persona_breakdown.len(), 4);
    }

    #[tokio::test]
    async fn ensemble_score_is_min_not_mean() {
        // Three personas score 1.0, one scores 0.2 → merged score == 0.2 (not mean).
        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 1.0,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 1.0,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 1.0,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.2,
                issues: vec![],
            },
        ]);

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");
        let result = agent
            .critique(
                &Plan {
                    goal: "score check".into(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(result.approved);
        // Min score wins (0.2), not mean (0.8).
        assert!((result.score - 0.2).abs() < f64::EPSILON);
    }

    // --- Memory-in-critique tests (U4) ---

    #[tokio::test]
    async fn critique_injects_guardrail_blind_spots_and_bumps_retrieval_count() {
        // Guardrail learning in memory → appears in critic prompt under
        // "Known blind spots". Playbooks are excluded. Retrieval counters bump.
        let guardrail = LearningEntry::guardrail(
            "boundary crossing added",
            "change crosses forbidden dependency",
            "This change crosses a forbidden dependency boundary. Consider alternative approach.",
        );
        let playbook = LearningEntry::new(
            "pattern works",
            "regex pattern extraction succeeded",
            "Pattern extraction approach is validated.",
        );

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let tempdir = tempfile::tempdir().expect("tempdir");

        let llm = PersonaScriptedLlm::new(vec![]);
        let agent = Agent::builder()
            .llm(llm)
            .tools(ToolRegistry::new())
            .config(config)
            .memory(tempdir.path())
            .build()
            .expect("agent builds");

        agent.memory.as_ref().map(|(_, mem)| {
            mem.lock().unwrap().add_learning(guardrail.clone());
            mem.lock().unwrap().add_learning(playbook.clone());
            mem.lock()
                .unwrap()
                .save(tempdir.path())
                .expect("save memory");
        });

        let result = agent
            .critique(
                &Plan {
                    goal: "boundary crossing".into(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        // The guardrail learning ID is in injected_learning_ids; the playbook is not.
        assert!(result.injected_learning_ids.contains(&guardrail.id));
        assert!(!result.injected_learning_ids.contains(&playbook.id));

        // Retrieval count on the guardrail was bumped (from 0 to 1).
        agent.memory.as_ref().map(|(_, mem)| {
            let mem = mem.lock().unwrap();
            let entry = mem
                .learnings
                .iter()
                .find(|e| e.id == guardrail.id)
                .expect("guardrail added");
            assert_eq!(entry.retrieval_count, 1);
        });
    }

    #[tokio::test]
    async fn critique_no_memory_shows_no_blind_spots_section() {
        // No memory → no blind-spots section in the prompt.
        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();

        let llm = PersonaScriptedLlm::new(vec![PersonaResponse {
            system_prompt_contains: "correctness",
            approved: true,
            score: 0.9,
            issues: vec![],
        }]);
        let agent = Agent::builder()
            .llm(llm.clone())
            .tools(ToolRegistry::new())
            .config(config)
            .build()
            .expect("agent builds");

        let _ = agent
            .critique(
                &Plan {
                    goal: "no memory".into(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        // Prompt received by the mock contains the shared context but no
        // "Known blind spots" section.
        let all_prompts = format!(
            "{}\n{}",
            llm.received_system_prompt(),
            llm.received_user_prompt()
        );
        assert!(!all_prompts.contains("Known blind spots"));
        assert!(all_prompts.contains("Execution Results"));
    }

    #[tokio::test]
    async fn critique_playbooks_excluded_from_blind_spots() {
        // Only guardrail learnings appear in the blind-spots section; playbooks
        // are excluded from injected_learning_ids.
        let guardrail = LearningEntry::guardrail(
            "memory leak on disconnect",
            "connection not closed",
            "Always close connections in a finally block.",
        );
        let playbook = LearningEntry::new(
            "successful pattern",
            "caching worked well",
            "Use Redis for caching.",
        );

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let tempdir = tempfile::tempdir().expect("tempdir");

        let llm = PersonaScriptedLlm::new(vec![PersonaResponse {
            system_prompt_contains: "correctness",
            approved: true,
            score: 0.9,
            issues: vec![],
        }]);
        let agent = Agent::builder()
            .llm(llm.clone())
            .tools(ToolRegistry::new())
            .config(config)
            .memory(tempdir.path())
            .build()
            .expect("agent builds");

        agent.memory.as_ref().map(|(_, mem)| {
            mem.lock().unwrap().add_learning(guardrail.clone());
            mem.lock().unwrap().add_learning(playbook.clone());
            mem.lock()
                .unwrap()
                .save(tempdir.path())
                .expect("save memory");
        });

        let _ = agent
            .critique(
                &Plan {
                    goal: "connection not closed".into(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        // The guardrail is injected; the playbook is not.
        let user_prompt = llm.received_user_prompt();
        assert!(
            user_prompt.contains("Always close connections"),
            "guardrail advice should appear in prompt"
        );
        assert!(
            !user_prompt.contains("Redis for caching"),
            "playbook should not appear in blind-spots prompt"
        );
    }

    #[tokio::test]
    async fn critique_roundtrips_with_ensemble() {
        // With ensemble active, each persona's prompt contains the blind-spots
        // section (not just one persona).
        let guardrail = LearningEntry::guardrail(
            "unchecked unwrap",
            "potential panic",
            "Always handle Result types properly.",
        );

        let mut config = AgentConfig::default();
        config.critique_personas = CritiquePersona::default_personas();
        let tempdir = tempfile::tempdir().expect("tempdir");

        let llm = PersonaScriptedLlm::new(vec![
            PersonaResponse {
                system_prompt_contains: "correctness",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "acceptance criteria",
                approved: true,
                score: 0.8,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "boundary",
                approved: true,
                score: 0.9,
                issues: vec![],
            },
            PersonaResponse {
                system_prompt_contains: "regression",
                approved: true,
                score: 0.85,
                issues: vec![],
            },
        ]);
        let agent = Agent::builder()
            .llm(llm.clone())
            .tools(ToolRegistry::new())
            .config(config)
            .memory(tempdir.path())
            .build()
            .expect("agent builds");

        agent.memory.as_ref().map(|(_, mem)| {
            mem.lock().unwrap().add_learning(guardrail.clone());
            mem.lock()
                .unwrap()
                .save(tempdir.path())
                .expect("save memory");
        });

        let result = agent
            .critique(
                &Plan {
                    goal: "potential panic".into(),
                    subtasks: vec![],
                    tdd: false,
                    risks: vec![],
                },
                &vec![],
            )
            .await
            .expect("critique runs");

        assert!(result.approved);
        // The guardrail was injected.
        assert!(result.injected_learning_ids.contains(&guardrail.id));
        // All four personas were invoked (each sees the blind-spots section
        // because it's appended to shared_user which all personas receive).
        assert_eq!(result.persona_breakdown.len(), 4);
    }

    #[test]
    fn usage_estimated_cost_is_nonzero() {
        let usage = Usage {
            prompt_tokens: 1_000_000,
            completion_tokens: 500_000,
            total_tokens: 1_500_000,
        };
        // gpt-4o-mini: 1M * $0.15/1M + 0.5M * $0.60/1M = $0.15 + $0.30 = $0.45
        let cost = usage.estimated_cost_usd();
        assert!(
            (cost - 0.45).abs() < 0.001,
            "expected ~$0.45, got ${cost:.4}"
        );
    }

    #[test]
    fn critique_signature_normalises_order() {
        let a = critique_signature(&["x".into(), "y".into()]);
        let b = critique_signature(&["y".into(), "x".into()]);
        assert_eq!(a, b);
    }
}

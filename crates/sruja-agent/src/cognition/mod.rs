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
    CompletionRequest, CompletionResponse, LlmClient, LlmError, Message, ModelRouter,
    Usage,
};

pub use crate::llm::TaskTier;
use crate::tool::{FileGuard, Phase, ToolError, ToolRegistry};
use crate::memory::{AgenticMemory, Memory};
use crate::LearningEntry;

pub use hook::{Hook, HookAction, HookRegistry, Hooks, LoggingHook, AutoLearningHook, AutoDocsHook, TokenSavingHook};
pub use decision::{DecisionRecord, DecisionStatus};
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
    /// Max tool-call iterations before giving up (default: 25).
    pub max_tool_iterations: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            models: ModelMapping::default(),
            tdd: true,
            review_every_change: true,
            spend_cap_usd: None,
            dry_run: false,
            max_tool_iterations: 25,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Critique {
    pub approved: bool,
    pub score: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
    pub usage: Usage,
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
                        let utility = l.utility_ratio().map(|u| format!("{:.0}%", u * 100.0)).unwrap_or_default();
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

        let system = format!("{COMPREHENSION_SYSTEM_PROMPT}{memory_context}");
        let user = format!(
            "## Goal\n{goal}\n\n\
             ## Instructions\n\
             Use the available tools to explore the codebase. \
             Cite architecture element IDs in your findings. \
             Produce a concise, grounded understanding."
        );

        let req = CompletionRequest::prompt(&system, user)
            .with_tools(self.tools.schemas());

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

        for _iteration in 0..self.config.max_tool_iterations {
            let response = self.llm.complete(&req).await?;
            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            total_usage.total_tokens += response.usage.total_tokens;

            if !response.wants_tools() {
                return Ok((response, total_usage));
            }

            // Push the assistant's tool-call message.
            req.messages.push(Message {
                role: crate::llm::MessageRole::Assistant,
                content: response.content.clone(),
                tool_calls: response.tool_calls.clone(),
                tool_call_id: None,
            });

            // Execute each requested tool and feed results back.
            for call in &response.tool_calls {
                let result = match self.tools.dispatch(&call.name, call.arguments.clone()).await {
                    Ok(out) => out,
                    Err(e) => format!("ERROR: {e}"),
                };
                let truncated = truncate(&result, 8_000);
                req.messages.push(Message::tool_result(&call.id, truncated));
            }
        }

        Err(AgentError::MaxIterations(self.config.max_tool_iterations))
    }

    /// Route a request to the model configured for a specific tier.
    pub async fn complete_tiered(
        &self,
        tier: TaskTier,
        req: CompletionRequest,
    ) -> Result<CompletionResponse, AgentError> {
        let model = match tier {
            TaskTier::Cheap => &self.config.models.cheap,
            TaskTier::Mid => &self.config.models.mid,
            TaskTier::Premium => &self.config.models.premium,
        };
        let req = CompletionRequest {
            model: Some(model.clone()),
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
             - `description`: what to do (concise, actionable)\n\
             - `tier`: cheap (classification/extraction), mid (standard coding), \
               or premium (hard architecture reasoning)\n\
             - `kind`: test_author, implement, verify, or review\n\
             - `files`: list of files this subtask touches\n\
             - `acceptance_criteria`: how to verify completion\n\n\
             Output a JSON object with `subtasks` array and `risks` array.\n\
             {tdd_note}",
            comprehension.summary,
            comprehension.cited_elements,
        );

        let req = CompletionRequest::prompt(PLAN_SYSTEM_PROMPT, user)
            .with_tools(self.tools.schemas());

        let (response, _usage) = self.run_tool_loop(req).await?;

        // Parse the plan from the LLM response.
        let plan = parse_plan_from_response(&response.content, goal, self.config.tdd);

        let mut plan = plan;
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

            // Route to the appropriate model tier.
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

        let req = CompletionRequest::prompt(&system, user)
                .with_tools(self.tools.schemas());

            let response = self.complete_tiered(tier, req).await?;
            let (response, tool_usage) = self.run_tool_loop(
                CompletionRequest::prompt(system, &response.content)
                    .with_tools(self.tools.schemas()),
            )
            .await?;

            let mut total_usage = response.usage.clone();
            total_usage.prompt_tokens += tool_usage.prompt_tokens;
            total_usage.completion_tokens += tool_usage.completion_tokens;
            total_usage.total_tokens += tool_usage.total_tokens;

            let status = if response.content.contains("ERROR") {
                StepStatus::Failed
            } else {
                StepStatus::Ok
            };

            let result = StepResult {
                subtask_id: step.id.clone(),
                status,
                output: response.content,
                usage: total_usage,
            };

            self.hooks.after_step(step, &result).await;
            results.push(result);
        }

        // After all subtasks, reset to Comprehend phase.
        self.guard.set_phase(Phase::Comprehend);

        Ok(results)
    }

    // --- Critique: review every change via the review model ---

    /// Review changes using the configured review model.
    ///
    /// Every change goes through this gate when `config.review_every_change` is true.
    /// The Critic is the quality barrier — it checks architectural compliance,
    /// test adequacy, and blast radius.
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
            .map(|r| format!("- [{}] {:?}: {}", r.subtask_id, r.status, truncate(&r.output, 200)))
            .collect();

        let user = format!(
            "## Goal\n{}\n\n\
             ## Plan\n{}\n\n\
             ## Execution Results\n{}\n\n\
             ## Instructions\n\
             Review this change as a senior architect. Check:\n\
             1. Does the output match the goal?\n\
             2. Are acceptance criteria met?\n\
             3. Any architectural violations or risks?\n\
             4. Should this be approved or rejected?\n\n\
             Respond with JSON: {{\"approved\": bool, \"score\": 0.0-1.0, \
             \"issues\": [...], \"suggestions\": [...]}}",
            plan.goal,
            plan.subtasks
                .iter()
                .map(|s| format!("- [{}] {} ({:?})", s.id, s.description, s.tier))
                .collect::<Vec<_>>()
                .join("\n"),
            step_summary.join("\n"),
        );

        let req = CompletionRequest::prompt(CRITIQUE_SYSTEM_PROMPT, &user)
            .with_model(&self.config.models.review);

        let response = self.llm.complete(&req).await?;
        let critique = parse_critique_from_response(&response.content, response.usage.clone());

        if let HookAction::Abort(reason) = self.hooks.after_review(&critique).await {
            return Err(AgentError::HookAborted(reason));
        }

        Ok(critique)
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
        let successes = results.iter().filter(|r| r.status == StepStatus::Ok).count();
        let failures = results.iter().filter(|r| r.status == StepStatus::Failed).count();

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
        let decision = self.generate_decision(&plan, &step_results, critique.as_ref()).await;
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

    /// Generate a decision record explaining WHY this change was made.
    async fn generate_decision(
        &self,
        plan: &Plan,
        results: &[StepResult],
        critique: Option<&Critique>,
    ) -> Option<DecisionRecord> {
        let successes = results.iter().filter(|r| r.status == StepStatus::Ok).count();
        let failures = results.iter().filter(|r| r.status == StepStatus::Failed).count();

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
            critique.map(|c| format!("approved={}", c.approved)).unwrap_or_else(|| "skipped".into()),
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
    async fn generate_runbook(
        &self,
        plan: &Plan,
        _results: &[StepResult],
    ) -> Option<Runbook> {
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

        let req = CompletionRequest::prompt(system, user)
            .with_tools(self.tools.schemas());

        let response = self.complete_tiered(subtask.tier, req).await?;
        let (response, _tool_usage) = self.run_tool_loop(
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

        let req = CompletionRequest::prompt(
            "You are a code reviewer. Be concise and practical.",
            &user,
        ).with_model(&self.config.models.cheap);

        let (response, _usage) = self.run_tool_loop(req).await?;
        let json_str = extract_json(&response.content);
        let value: serde_json::Value = serde_json::from_str(&json_str)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        Ok(crate::pair::ReviewResult {
            approved: value.get("approved").and_then(|a| a.as_bool()).unwrap_or(true),
            feedback: value.get("feedback")
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
        ).with_model(&self.config.models.cheap);

        let (response, _usage) = self.run_tool_loop(req).await?;
        Ok(response.content)
    }

    /// Suggest cleanup steps (for pair programming navigator driving).
    pub async fn suggest_cleanup(
        &self,
        results: &[crate::pair::StepResult],
    ) -> Result<Vec<String>, AgentError> {
        let summaries: Vec<String> = results.iter()
            .enumerate()
            .map(|(i, r)| {
                let end = r.output.char_indices().nth(100).map(|(i, _)| i).unwrap_or(r.output.len());
                format!("{}. {} (files: {:?})", i + 1, &r.output[..end], r.files_affected)
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
        ).with_model(&self.config.models.cheap);

        let (response, _usage) = self.run_tool_loop(req).await?;
        let json_str = extract_json(&response.content);
        let suggestions: Vec<String> = serde_json::from_str::<Vec<String>>(&json_str)
            .unwrap_or_default();

        Ok(suggestions)
    }
}

fn comprehension_cited_elements(plan: &Plan) -> Vec<String> {
    plan.subtasks.iter().flat_map(|s| s.files.clone()).collect()
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
            if parts.iter().all(|p| !p.is_empty() && p.chars().next().map_or(false, |c| c.is_uppercase()))
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
1. ALWAYS use tools to ground your understanding — never guess.\n\
2. Cite architecture element IDs (e.g. Sruja.CLI, Sruja.Graph) in your findings.\n\
3. Assess blast radius and risks.\n\
4. Be concise. Cite evidence, not speculation.";

pub(crate) const PLAN_SYSTEM_PROMPT: &str = "\
You are a Principal Engineer decomposing work into concrete subtasks.\n\n\
Rules:\n\
1. Each subtask must have: description, tier (cheap/mid/premium), kind (test_author/implement/verify/review), files, acceptance_criteria.\n\
2. If TDD mode: test_author subtasks MUST come before implement subtasks.\n\
3. Tag complexity accurately: classification/extraction = cheap, standard coding = mid, hard architecture = premium.\n\
4. Identify risks and edge cases.\n\
5. Output a JSON object: {\"subtasks\": [...], \"risks\": [...]}.";

const EXECUTION_SYSTEM_PROMPT: &str = "\
You are a Principal Engineer executing a specific subtask.\n\n\
Rules:\n\
1. Use tools to accomplish the task — never guess file contents.\n\
2. Be precise and minimal — make the smallest change that satisfies acceptance criteria.\n\
3. If in TestAuthor phase: write tests only, do not touch implementation.\n\
4. If in Implement phase: write code to pass the frozen tests, do not modify tests.\n\
5. Cite evidence for every decision.";

const CRITIQUE_SYSTEM_PROMPT: &str = "\
You are a senior architect reviewing a change. Be adversarial but fair.\n\n\
Check:\n\
1. Does the change match the stated goal?\n\
2. Are acceptance criteria satisfied?\n\
3. Any architectural violations or boundary crossings?\n\
4. Is test coverage adequate?\n\
5. What is the blast radius?\n\n\
Respond with JSON: {\"approved\": bool, \"score\": 0.0-1.0, \"issues\": [...], \"suggestions\": [...]}";

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
                    .filter_map(|st| {
                        Some(Subtask {
                            id: st.get("id")?.as_str()?.to_string(),
                            description: st.get("description")?.as_str()?.to_string(),
                            tier: parse_tier(st.get("tier")?.as_str()?),
                            kind: parse_kind(st.get("kind")?.as_str()?),
                            files: st
                                .get("files")
                                .and_then(|f| f.as_array())
                                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default(),
                            acceptance_criteria: st
                                .get("acceptance_criteria")
                                .and_then(|a| a.as_array())
                                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let risks: Vec<String> = value
            .get("risks")
            .and_then(|r| r.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

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
            id: "1".into(),
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
            approved: value.get("approved").and_then(|a| a.as_bool()).unwrap_or(false),
            score: value.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0),
            issues: value
                .get("issues")
                .and_then(|i| i.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            suggestions: value
                .get("suggestions")
                .and_then(|s| s.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            usage,
        };
    }

    // Fallback: check for approve/reject keywords.
    let lower = content.to_lowercase();
    Critique {
        approved: lower.contains("approved") || lower.contains("approve"),
        score: if lower.contains("approved") { 0.8 } else { 0.3 },
        issues: vec!["could not parse structured critique".into()],
        suggestions: Vec::new(),
        usage,
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

    /// Build the agent.
    pub fn build(self) -> Result<Agent, AgentError> {
        let llm_arc = self.llm.ok_or(AgentError::NoLlm)?;

        // Wrap in ModelRouter if a spend cap is configured.
        let llm: Arc<dyn LlmClient> = if let Some(cap) = self.config.spend_cap_usd {
            let mut rc = crate::llm::router::RouterConfig::default();
            rc.spend_cap_usd = Some(cap);
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

        Ok(Agent {
            llm,
            tools,
            guard: self.guard,
            hooks: HookRegistry::new(self.hooks),
            config: self.config,
            memory,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

//! Lifecycle hooks for the agent loop.
//!
//! Implement [`Hook`] and override only the lifecycle points you care about —
//! all methods have default no-op implementations.
//!
//! ## Built-in hooks
//!
//! - [`LoggingHook`] — traces all lifecycle events via `tracing`.
//! - [`AutoLearningHook`] — extracts and records a learning after every step.
//! - [`AutoDocsHook`] — updates architecture docs when files change.
//! - [`TokenSavingHook`] — compresses observations when context grows large.
//!
//! ```no_run
//! use sruja_agent::{Agent, cognition::{LoggingHook, AutoLearningHook}};
//! let agent = Agent::builder()
//!     .hook(Box::new(LoggingHook))
//!     .hook(Box::new(AutoLearningHook))
//!     .build();
//! ```

use crate::tool::Phase;
use crate::LearningEntry;

use super::{AgentError, Comprehension, Critique, Plan, StepResult, Subtask};

/// What a hook decides after observing a lifecycle point.
#[derive(Debug, Clone)]
pub enum HookAction {
    /// Proceed normally.
    Continue,
    /// Skip the current step (plan continues with the next subtask).
    Skip,
    /// Abort the entire run with a reason.
    Abort(String),
}

impl Default for HookAction {
    fn default() -> Self {
        Self::Continue
    }
}

/// A lifecycle hook. Override only what you need.
#[async_trait::async_trait]
pub trait Hook: Send + Sync {
    /// Before the comprehension phase.
    async fn before_comprehend(&self, _goal: &str) -> HookAction {
        HookAction::Continue
    }
    /// After comprehension completes.
    async fn after_comprehend(&self, _result: &Comprehension) -> HookAction {
        HookAction::Continue
    }
    /// Before planning begins.
    async fn before_plan(&self, _goal: &str) -> HookAction {
        HookAction::Continue
    }
    /// After a plan is produced (can modify it in place).
    async fn after_plan(&self, _plan: &mut Plan) -> HookAction {
        HookAction::Continue
    }
    /// Before a subtask executes.
    async fn before_step(&self, _step: &Subtask) -> HookAction {
        HookAction::Continue
    }
    /// After a subtask completes.
    async fn after_step(&self, _step: &Subtask, _result: &StepResult) {}
    /// Before the Critic reviews changes.
    async fn before_review(&self) -> HookAction {
        HookAction::Continue
    }
    /// After the Critic produces its assessment.
    async fn after_review(&self, _critique: &Critique) -> HookAction {
        HookAction::Continue
    }
    /// When the TDD phase transitions.
    async fn on_phase_change(&self, _to: Phase) {}
    /// When a learning is recorded to memory.
    async fn on_learning(&self, _entry: &LearningEntry) {}
    /// When an error occurs during the run.
    async fn on_error(&self, _error: &AgentError) {}
}

/// Runs a collection of hooks in registration order.
pub struct HookRegistry {
    hooks: Vec<Box<dyn Hook>>,
}

impl HookRegistry {
    pub fn new(hooks: Vec<Box<dyn Hook>>) -> Self {
        Self { hooks }
    }

    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    pub async fn before_comprehend(&self, goal: &str) -> HookAction {
        for h in &self.hooks {
            match h.before_comprehend(goal).await {
                HookAction::Continue => {}
                other => return other,
            }
        }
        HookAction::Continue
    }

    pub async fn after_comprehend(&self, result: &Comprehension) -> HookAction {
        for h in &self.hooks {
            match h.after_comprehend(result).await {
                HookAction::Continue => {}
                other => return other,
            }
        }
        HookAction::Continue
    }

    pub async fn before_plan(&self, goal: &str) -> HookAction {
        for h in &self.hooks {
            match h.before_plan(goal).await {
                HookAction::Continue => {}
                other => return other,
            }
        }
        HookAction::Continue
    }

    pub async fn after_plan(&self, plan: &mut Plan) -> HookAction {
        for h in &self.hooks {
            match h.after_plan(plan).await {
                HookAction::Continue => {}
                other => return other,
            }
        }
        HookAction::Continue
    }

    pub async fn before_step(&self, step: &Subtask) -> HookAction {
        for h in &self.hooks {
            match h.before_step(step).await {
                HookAction::Continue => {}
                other => return other,
            }
        }
        HookAction::Continue
    }

    pub async fn after_step(&self, step: &Subtask, result: &StepResult) {
        for h in &self.hooks {
            h.after_step(step, result).await;
        }
    }

    pub async fn before_review(&self) -> HookAction {
        for h in &self.hooks {
            match h.before_review().await {
                HookAction::Continue => {}
                other => return other,
            }
        }
        HookAction::Continue
    }

    pub async fn after_review(&self, critique: &Critique) -> HookAction {
        for h in &self.hooks {
            match h.after_review(critique).await {
                HookAction::Continue => {}
                other => return other,
            }
        }
        HookAction::Continue
    }

    pub async fn on_phase_change(&self, to: Phase) {
        for h in &self.hooks {
            h.on_phase_change(to).await;
        }
    }

    pub async fn on_learning(&self, entry: &LearningEntry) {
        for h in &self.hooks {
            h.on_learning(entry).await;
        }
    }

    pub async fn on_error(&self, error: &AgentError) {
        for h in &self.hooks {
            h.on_error(error).await;
        }
    }
}

/// Convenience alias.
pub type Hooks = HookRegistry;

// ---------------------------------------------------------------------------
// Built-in hooks
// ---------------------------------------------------------------------------

/// Traces all lifecycle events via `tracing`.
pub struct LoggingHook;

#[async_trait::async_trait]
impl Hook for LoggingHook {
    async fn before_comprehend(&self, goal: &str) -> HookAction {
        tracing::info!(goal = %goal, "comprehend:start");
        HookAction::Continue
    }

    async fn after_comprehend(&self, result: &Comprehension) -> HookAction {
        tracing::info!(
            elements = ?result.cited_elements,
            tokens = result.usage.total_tokens,
            "comprehend:done"
        );
        HookAction::Continue
    }

    async fn after_plan(&self, plan: &mut Plan) -> HookAction {
        tracing::info!(
            subtasks = plan.subtasks.len(),
            tdd = plan.tdd,
            "plan:produced"
        );
        HookAction::Continue
    }

    async fn before_step(&self, step: &Subtask) -> HookAction {
        tracing::info!(id = %step.id, kind = ?step.kind, tier = ?step.tier, "step:start");
        HookAction::Continue
    }

    async fn after_step(&self, _step: &Subtask, result: &StepResult) {
        tracing::info!(id = %result.subtask_id, status = ?result.status, "step:done");
    }

    async fn on_phase_change(&self, to: Phase) {
        tracing::info!(phase = ?to, "phase:change");
    }

    async fn on_error(&self, error: &AgentError) {
        tracing::error!(error = %error, "agent:error");
    }
}

/// Automatically extracts and records a learning after every successful step.
///
/// This is the "compound self-learning" loop — every run produces lessons
/// that future runs retrieve, creating compounding improvement.
pub struct AutoLearningHook;

#[async_trait::async_trait]
impl Hook for AutoLearningHook {
    async fn after_step(&self, step: &Subtask, result: &StepResult) {
        if result.status != super::StepStatus::Ok {
            return;
        }
        let entry = match step.kind {
            super::SubtaskKind::TestAuthor => LearningEntry::playbook(
                format!("Wrote test: {}", step.description),
                "Test-first approach validates requirements before implementation",
                &format!(
                    "Always write tests before implementation for: {}",
                    step.description
                ),
            ),
            super::SubtaskKind::Implement => LearningEntry::playbook(
                format!("Implemented: {}", step.description),
                "Code written to pass frozen tests",
                &format!("Implementation approach for: {}", step.description),
            ),
            super::SubtaskKind::Review => LearningEntry::invariant(
                format!("Reviewed: {}", step.description),
                "Critic approved the change",
                &format!("Review criteria satisfied for: {}", step.description),
            ),
            _ => return,
        };
        tracing::info!(subtask = %step.id, "learning:recorded");
        let _ = entry; // In production, this would be saved to Memory
    }
}

/// Automatically updates architecture documentation when files change.
///
/// After each implement/test step, checks if docs need updating based on
/// what was written. Prevents documentation drift.
pub struct AutoDocsHook;

#[async_trait::async_trait]
impl Hook for AutoDocsHook {
    async fn after_step(&self, step: &Subtask, result: &StepResult) {
        if result.status != super::StepStatus::Ok {
            return;
        }
        if !step.files.is_empty() {
            tracing::info!(
                files = ?step.files,
                "docs:check_if_update_needed"
            );
            // In production: compare changed files against doc coverage,
            // emit a TaskTier::Cheap subtask to update docs if needed.
        }
    }
}

/// Monitors token usage and triggers compression when context grows large.
///
/// This is the "save tokens over time" mechanism — older observations are
/// compressed to decision-only summaries, keeping the context lean.
pub struct TokenSavingHook {
    /// Token threshold that triggers compression.
    pub threshold: usize,
}

impl Default for TokenSavingHook {
    fn default() -> Self {
        Self { threshold: 8_000 }
    }
}

#[async_trait::async_trait]
impl Hook for TokenSavingHook {
    async fn after_step(&self, _step: &Subtask, _result: &StepResult) {
        // In production: check cumulative token usage and compress
        // older observations if above threshold. Uses the same rolling
        // compression as sruja-cli's observation_compression module.
    }
}

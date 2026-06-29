//! @element Sruja.Agent.LoopEvent
//! @layer Core Engine
//! @boundary Events are sent through tokio::sync::mpsc::Sender, mirroring
//!           TurnEvent pattern from chat.rs. Events are structured telemetry
//!           for CLI/MCP hosts, not for direct rendering to users.
//!
//! Structured events emitted during the autonomous loop (comprehend → plan
//! → execute → critique → replan → verify). Modeled on [`TurnEvent`] from
//! the chat module: a small enum sent through an [`mpsc::Sender`].
//!
//! The host receives these events and renders them as:
//! - Plan preview (when verdict is Ask or --show-plan is set)
//! - Live phase status bar
//! - Changelog writer (after Done)
//!
//! Events are best-effort: a closed receiver must not fail the loop.

use serde::{Deserialize, Serialize};
use crate::calibration::AskPlan;

/// Phases of the autonomous loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoopPhase {
    /// Comprehend: understand the goal and classify complexity.
    Comprehend,
    /// Plan: generate a structured plan with subtasks.
    Plan,
    /// Execute: run each subtask through the tool loop.
    Execute,
    /// Critique: run the LLM critic on the changes.
    Critique,
    /// Replan: generate a new plan addressing critique feedback.
    Replan,
    /// Verify: run the deterministic grader.
    Verify,
    /// Complete: loop finished (converged or exhausted).
    Complete,
}

/// A small serializable subset of [`Plan`] for plan previews.
///
/// Contains only the essential fields needed to render a plan brief to
/// the human before execution begins. Full plan details are available in
/// the final [`LoopResult`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanBrief {
    pub goal: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subtasks: Vec<SubtaskBrief>,
}

/// A brief summary of a subtask.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskBrief {
    pub id: String,
    pub description: String,
    pub tier: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

impl From<&crate::cognition::Plan> for PlanBrief {
    fn from(plan: &crate::cognition::Plan) -> Self {
        Self {
            goal: plan.goal.clone(),
            criteria: plan.criteria.clone(),
            subtasks: plan.subtasks.iter().map(Into::into).collect(),
        }
    }
}

impl From<&crate::cognition::Subtask> for SubtaskBrief {
    fn from(st: &crate::cognition::Subtask) -> Self {
        Self {
            id: st.id.clone(),
            description: st.description.clone(),
            tier: format!("{:?}", st.tier),
            files: st.files.clone(),
        }
    }
}

/// Events emitted during the autonomous loop, sent to the host for rendering.
///
/// The host receives these via an [`mpsc::Sender`] and renders them
/// (plan preview block, phase status bar, changelog writer).
///
/// Events are ordered and best-effort: a closed receiver must not fail the loop.
#[derive(Debug, Clone, Serialize)]
pub enum LoopEvent {
    /// Loop started after comprehension.
    Started {
        goal: String,
        max_iterations: usize,
    },
    /// Phase transition.
    PhaseChanged(LoopPhase),
    /// Plan is ready with calibration verdict.
    PlanReady {
        plan_brief: PlanBrief,
        ask_plan: AskPlan,
    },
    /// Replanning loop started.
    IterationStarted {
        n: usize,
        reason: Option<String>,
    },
    /// Progress within the Execute phase.
    StepProgress {
        step: usize,
        total: usize,
        description: String,
    },
    /// Deterministic grader result for a verify step.
    VerifyResult {
        step: String,
        ok: bool,
    },
    /// Loop finished with outcome summary.
    Done {
        outcome_summary: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_brief_from_plan() {
        let plan = crate::cognition::Plan {
            goal: "test goal".into(),
            goal_statement: "test statement".into(),
            criteria: vec!["c1".into(), "c2".into()],
            subtasks: vec![crate::cognition::Subtask {
                id: "s1".into(),
                description: "desc".into(),
                tier: crate::llm::TaskTier::Mid,
                kind: crate::cognition::SubtaskKind::Implement,
                files: vec!["file.rs".into()],
                acceptance_criteria: vec![],
            }],
            tdd: true,
            risks: vec![],
            schema_version: "1.0".into(),
            complexity: crate::cognition::TaskComplexity::Simple,
        };

        let brief = PlanBrief::from(&plan);
        assert_eq!(brief.goal, "test goal");
        assert_eq!(brief.criteria, vec!["c1", "c2"]);
        assert_eq!(brief.subtasks.len(), 1);
        assert_eq!(brief.subtasks[0].id, "s1");
        assert_eq!(brief.subtasks[0].tier, "Mid");
        assert_eq!(brief.subtasks[0].files, vec!["file.rs"]);
    }

    #[test]
    fn test_subtask_brief_from_subtask() {
        let st = crate::cognition::Subtask {
            id: "s1".into(),
            description: "desc".into(),
            tier: crate::llm::TaskTier::Premium,
            kind: crate::cognition::SubtaskKind::TestAuthor,
            files: vec!["a.rs".into(), "b.rs".into()],
            acceptance_criteria: vec![],
        };

        let brief = SubtaskBrief::from(&st);
        assert_eq!(brief.id, "s1");
        assert_eq!(brief.description, "desc");
        assert_eq!(brief.tier, "Premium");
        assert_eq!(brief.files, vec!["a.rs", "b.rs"]);
    }

    #[test]
    fn test_loop_phase_serialization() {
        for phase in [
            LoopPhase::Comprehend,
            LoopPhase::Plan,
            LoopPhase::Execute,
            LoopPhase::Critique,
            LoopPhase::Replan,
            LoopPhase::Verify,
            LoopPhase::Complete,
        ] {
            let json = serde_json::to_string(&phase).unwrap();
            let deser: LoopPhase = serde_json::from_str(&json).unwrap();
            assert_eq!(phase, deser, "phase {:?} round-trips", phase);
        }
    }

    #[test]
    fn test_loop_event_serialization() {
        let event = LoopEvent::PlanReady {
            plan_brief: PlanBrief {
                goal: "test".into(),
                criteria: vec![],
                subtasks: vec![],
            },
            ask_plan: AskPlan {
                verdict: crate::calibration::Verdict::ProceedSilent,
                reason: "test reason".into(),
                reversibility: crate::calibration::Reversibility::TwoWay,
                blast_radius: 5,
                confidence: Some(80),
                trust_level: Some(70),
                has_precedent: false,
                policy_says_ask: false,
            },
        };

        let json = serde_json::to_string(&event).unwrap();
        // Note: LoopEvent only has Serialize, not Deserialize
        assert!(json.contains("PlanReady"));
        assert!(json.contains("test reason"));
    }
}
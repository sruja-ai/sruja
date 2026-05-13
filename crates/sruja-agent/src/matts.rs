//! Memory-Aware Test-Time Scaling (MaTTS).
//!
//! Generates multiple parallel evaluation trajectories for the same goal and
//! distills higher-quality guardrails by contrasting success vs. failure outcomes.
//! Each trajectory runs in an isolated sandbox (git worktree), and after all
//! trajectories complete, a self-contrast phase extracts transferable learnings.
//!
//! This is a local-first implementation of the MaTTS concept described in
//! Google Research's ReasoningBank: instead of running a task once and hoping
//! for the best, we run N parallel attempts and distill wisdom from the spread.

use crate::memory::{AgenticMemory, ExperimentOutcome, LearningEntry, LearningKind};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The outcome of a single evaluation trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryOutcome {
    pub trajectory_id: String,
    pub goal: String,
    pub status: TrajectoryStatus,
    pub exit_code: Option<i32>,
    pub summary: String,
    pub elapsed_ms: u128,
    pub affected_elements: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrajectoryStatus {
    Success,
    Failed,
    Timeout,
}

/// The result of contrasting multiple trajectory outcomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastResult {
    pub total_trajectories: usize,
    pub successes: usize,
    pub failures: usize,
    pub distilled_learnings: Vec<LearningEntry>,
    pub confidence: f64,
}

/// Orchestrator for parallel trajectory evaluation and self-contrast distillation.
pub struct TrajectoryRunner {
    num_trajectories: usize,
}

impl TrajectoryRunner {
    pub fn new(num_trajectories: usize) -> Self {
        Self {
            num_trajectories: num_trajectories.max(2),
        }
    }

    pub fn num_trajectories(&self) -> usize {
        self.num_trajectories
    }

    /// Distills learnings by contrasting successful trajectories against failed ones.
    ///
    /// This is the core MaTTS insight: by comparing what worked vs. what didn't
    /// for the same goal, we extract higher-quality guardrails than any single
    /// trajectory could provide.
    pub fn distill_from_contrast(&self, outcomes: &[TrajectoryOutcome]) -> ContrastResult {
        let successes: Vec<&TrajectoryOutcome> = outcomes
            .iter()
            .filter(|o| o.status == TrajectoryStatus::Success)
            .collect();
        let failures: Vec<&TrajectoryOutcome> = outcomes
            .iter()
            .filter(|o| o.status == TrajectoryStatus::Failed)
            .collect();

        let mut distilled = Vec::new();

        if !successes.is_empty() && !failures.is_empty() {
            let success_summaries: Vec<&str> =
                successes.iter().map(|o| o.summary.as_str()).collect();
            let failure_summaries: Vec<&str> =
                failures.iter().map(|o| o.summary.as_str()).collect();

            let all_affected: Vec<String> = outcomes
                .iter()
                .flat_map(|o| o.affected_elements.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let guardrail = format!(
                "MaTTS contrast ({}/{} succeeded): successful paths: [{}]. Failed paths: [{}]. \
                 Prefer the approach from successful trajectories.",
                successes.len(),
                outcomes.len(),
                success_summaries.join("; "),
                failure_summaries.join("; "),
            );

            distilled.push(LearningEntry {
                id: String::new(),
                kind: Some(LearningKind::Playbook),
                timestamp: Utc::now(),
                run_id: None,
                repo: None,
                selector: None,
                context: format!(
                    "MaTTS parallel evaluation: {} trajectories for goal '{}'",
                    outcomes.len(),
                    outcomes
                        .first()
                        .map(|o| o.goal.as_str())
                        .unwrap_or("unknown")
                ),
                hypothesis: format!(
                    "Contrast of {} success vs {} failure trajectories",
                    successes.len(),
                    failures.len()
                ),
                outcome: ExperimentOutcome::Success,
                reason: Some(format!(
                    "Distilled from {} parallel trajectories",
                    outcomes.len()
                )),
                guardrail_advice: guardrail,
                affected_elements: all_affected,
                evidence_refs: Vec::new(),
                confidence: None,
                tags: vec!["matts".to_string(), "contrast".to_string()],
                related_ids: Vec::new(),
            });
        } else if successes.is_empty() && !failures.is_empty() {
            let failure_summaries: Vec<&str> =
                failures.iter().map(|o| o.summary.as_str()).collect();
            let all_affected: Vec<String> = failures
                .iter()
                .flat_map(|o| o.affected_elements.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            distilled.push(LearningEntry {
                id: String::new(),
                kind: Some(LearningKind::Guardrail),
                timestamp: Utc::now(),
                run_id: None,
                repo: None,
                selector: None,
                context: format!(
                    "MaTTS: all {} trajectories failed for goal '{}'",
                    failures.len(),
                    outcomes.first().map(|o| o.goal.as_str()).unwrap_or("unknown")
                ),
                hypothesis: "All parallel trajectories failed — goal may be structurally blocked"
                    .to_string(),
                outcome: ExperimentOutcome::Failed,
                reason: Some(format!("Failures: [{}]", failure_summaries.join("; "))),
                guardrail_advice:
                    "All MaTTS trajectories failed. Investigate structural blockers before retrying."
                        .to_string(),
                affected_elements: all_affected,
                evidence_refs: Vec::new(),
                confidence: None,
                tags: vec!["matts".to_string(), "blocked".to_string()],
                related_ids: Vec::new(),
            });
        }

        let confidence = if outcomes.is_empty() {
            0.0
        } else {
            let success_rate = successes.len() as f64 / outcomes.len() as f64;
            // High confidence when outcomes are clear-cut (all succeed or all fail).
            // Lower confidence when mixed results make the signal ambiguous.
            let clarity = (success_rate - 0.5).abs() * 2.0;
            0.5 + (clarity * 0.5)
        };

        ContrastResult {
            total_trajectories: outcomes.len(),
            successes: successes.len(),
            failures: failures.len(),
            distilled_learnings: distilled,
            confidence,
        }
    }

    /// Records the distilled learnings into agentic memory with cross-linking.
    pub fn record_learnings(contrast: &ContrastResult, memory: &mut AgenticMemory) {
        for learning in &contrast.distilled_learnings {
            memory.add_learning(learning.clone());
        }
    }
}

/// High-level MaTTS helper to distill learnings from available outcomes and
/// optionally persist them to repo-local agentic memory.
///
/// Contract:
/// - Returns `None` when there is no meaningful contrast signal (fewer than 2 outcomes).
/// - If `auto_record` is true and distillation yields learnings, persists them to
///   `.sruja/agent_memory.json` under `repo_root`.
/// - Never "pretends" to do MaTTS when only one trajectory outcome is available.
pub fn maybe_distill_and_record(
    repo_root: &Path,
    requested_trajectories: usize,
    outcomes: &[TrajectoryOutcome],
    auto_record: bool,
) -> (Option<ContrastResult>, Vec<String>) {
    // MaTTS contrast requires >= 2 outcomes. Callers may request more trajectories,
    // but we should not synthesize contrast from a single run.
    if outcomes.len() < 2 {
        if requested_trajectories >= 2 {
            return (
                None,
                vec![format!(
                    "MaTTS requested {} trajectories, but only {} outcome(s) available; skipping contrast distillation.",
                    requested_trajectories,
                    outcomes.len()
                )],
            );
        }
        return (None, Vec::new());
    }

    let runner = TrajectoryRunner::new(requested_trajectories.max(outcomes.len()));
    let contrast = runner.distill_from_contrast(outcomes);

    let mut notes = Vec::new();
    if auto_record && !contrast.distilled_learnings.is_empty() {
        let mut memory = AgenticMemory::load(repo_root).unwrap_or_default();
        TrajectoryRunner::record_learnings(&contrast, &mut memory);
        if memory.save(repo_root).is_ok() {
            for entry in &contrast.distilled_learnings {
                notes.push(format!("MaTTS: {}", entry.guardrail_advice));
            }
        } else {
            notes.push("MaTTS: failed to persist agentic memory (continuing).".to_string());
        }
    }

    (Some(contrast), notes)
}

/// Generates sandbox names for a set of parallel trajectories.
pub fn sandbox_names(goal: &str, count: usize) -> Vec<String> {
    let slug: String = goal
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .chars()
        .take(20)
        .collect();
    let ts = Utc::now().timestamp();
    (0..count)
        .map(|i| format!("matts-{}-{}-{}", slug, ts, i))
        .collect()
}

/// Checks whether sandbox infrastructure (git worktrees) is available.
pub fn is_sandbox_available(repo_path: &Path) -> bool {
    std::process::Command::new("git")
        .args(["worktree", "list"])
        .current_dir(repo_path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_outcome(id: &str, status: TrajectoryStatus, summary: &str) -> TrajectoryOutcome {
        TrajectoryOutcome {
            trajectory_id: id.to_string(),
            goal: "test goal".to_string(),
            status,
            exit_code: Some(if status == TrajectoryStatus::Success {
                0
            } else {
                1
            }),
            summary: summary.to_string(),
            elapsed_ms: 100,
            affected_elements: vec!["API.Routes".to_string()],
        }
    }

    #[test]
    fn distill_mixed_outcomes() {
        let runner = TrajectoryRunner::new(3);
        let outcomes = vec![
            make_outcome(
                "t1",
                TrajectoryStatus::Success,
                "Used service layer pattern",
            ),
            make_outcome(
                "t2",
                TrajectoryStatus::Failed,
                "Direct DB access from routes",
            ),
            make_outcome("t3", TrajectoryStatus::Success, "Used repository pattern"),
        ];

        let result = runner.distill_from_contrast(&outcomes);
        assert_eq!(result.total_trajectories, 3);
        assert_eq!(result.successes, 2);
        assert_eq!(result.failures, 1);
        assert_eq!(result.distilled_learnings.len(), 1);

        let learning = &result.distilled_learnings[0];
        assert!(learning.guardrail_advice.contains("MaTTS contrast"));
        assert!(learning.guardrail_advice.contains("2/3 succeeded"));
        assert!(learning.tags.contains(&"matts".to_string()));
    }

    #[test]
    fn distill_all_failures() {
        let runner = TrajectoryRunner::new(2);
        let outcomes = vec![
            make_outcome("t1", TrajectoryStatus::Failed, "Timeout on lint"),
            make_outcome("t2", TrajectoryStatus::Failed, "Boundary violation"),
        ];

        let result = runner.distill_from_contrast(&outcomes);
        assert_eq!(result.successes, 0);
        assert_eq!(result.failures, 2);
        assert_eq!(result.distilled_learnings.len(), 1);
        assert!(result.distilled_learnings[0]
            .guardrail_advice
            .contains("All MaTTS trajectories failed"));
    }

    #[test]
    fn distill_all_successes() {
        let runner = TrajectoryRunner::new(2);
        let outcomes = vec![
            make_outcome("t1", TrajectoryStatus::Success, "Clean"),
            make_outcome("t2", TrajectoryStatus::Success, "Clean too"),
        ];

        let result = runner.distill_from_contrast(&outcomes);
        assert_eq!(result.successes, 2);
        assert_eq!(result.failures, 0);
        assert!(result.distilled_learnings.is_empty());
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn distill_empty() {
        let runner = TrajectoryRunner::new(2);
        let result = runner.distill_from_contrast(&[]);
        assert_eq!(result.total_trajectories, 0);
        assert!(result.distilled_learnings.is_empty());
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn record_learnings_into_memory() {
        let runner = TrajectoryRunner::new(2);
        let outcomes = vec![
            make_outcome("t1", TrajectoryStatus::Success, "OK"),
            make_outcome("t2", TrajectoryStatus::Failed, "Bad"),
        ];
        let contrast = runner.distill_from_contrast(&outcomes);

        let mut memory = AgenticMemory::default();
        TrajectoryRunner::record_learnings(&contrast, &mut memory);
        assert_eq!(memory.learnings.len(), 1);
        assert!(memory.learnings[0].tags.contains(&"matts".to_string()));
    }

    #[test]
    fn sandbox_names_generates_unique() {
        let names = sandbox_names("Fix boundary violation", 3);
        assert_eq!(names.len(), 3);
        assert!(names[0].starts_with("matts-fix-boundary-viola"));
        let unique: std::collections::HashSet<&String> = names.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn min_trajectories_is_two() {
        let runner = TrajectoryRunner::new(1);
        assert_eq!(runner.num_trajectories(), 2);
    }
}

//! @element Sruja.Agent.Changelog
//! @layer Core Engine
//! @boundary Changelog is a file artifact written to .sruja/changelogs/,
//!           parallel to .sruja/decisions/ and .sruja/runbooks/. Best-effort:
//!           a write failure logs but does not fail the loop.
//!
//! Post-loop changelog writer. Consolidates the loop run into a single
//! human-readable, committable markdown artifact tying together the goal,
//! calibration verdict, per-iteration evidence, decision records, and
//! verify-step results.

use chrono::Utc;
use serde::Serialize;

use crate::calibration::{AskPlan, Verdict};
use crate::cognition::{LoopResult, LoopTermination};

/// A builder that consolidates a `LoopResult` into a markdown changelog.
#[derive(Debug, Serialize)]
pub struct AgentChangelog {
    pub session_id: String,
    pub goal: String,
    pub verdict: Option<Verdict>,
    pub converged: bool,
    pub termination: String,
    pub iterations: Vec<IterationSummary>,
    pub files_changed: Vec<String>,
    pub verify_results: Vec<VerifySummary>,
    pub decisions: Vec<DecisionSummary>,
    pub dry_run: bool,
    pub grader_source: String,
}

#[derive(Debug, Serialize)]
pub struct IterationSummary {
    pub iteration: usize,
    pub replanned: bool,
    pub subtask_count: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub critique_approved: bool,
    pub critique_score: f64,
    pub critique_issues: Vec<String>,
    pub verify_failed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct VerifySummary {
    pub step: String,
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct DecisionSummary {
    pub title: String,
    pub context: String,
    pub decision: String,
}

impl AgentChangelog {
    /// Build from a `LoopResult` and optional calibration `AskPlan`.
    pub fn from_loop(result: &LoopResult, ask_plan: Option<&AskPlan>, dry_run: bool) -> Self {
        let session_id = format!("{}", Utc::now().format("%Y%m%d-%H%M%S"));

        let iterations: Vec<IterationSummary> = result
            .iterations
            .iter()
            .map(|i| IterationSummary {
                iteration: i.iteration,
                replanned: i.replanned,
                subtask_count: i.subtask_count,
                succeeded: i.succeeded,
                failed: i.failed,
                critique_approved: i.critique_approved,
                critique_score: i.critique_score,
                critique_issues: i.critique_issues.clone(),
                verify_failed: i.verify_failed.clone(),
            })
            .collect();

        let files_changed: Vec<String> = {
            let mut files: Vec<String> = result
                .final_result
                .plan
                .subtasks
                .iter()
                .flat_map(|s| s.files.clone())
                .collect();
            files.sort();
            files.dedup();
            files
        };

        let verify_results: Vec<VerifySummary> = result
            .iterations
            .last()
            .map(|i| {
                i.verify_failed
                    .iter()
                    .map(|f| VerifySummary {
                        step: f.clone(),
                        ok: false,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let decisions: Vec<DecisionSummary> = result
            .final_result
            .decision
            .as_ref()
            .map(|d| {
                vec![DecisionSummary {
                    title: d.title.clone(),
                    context: d.context.clone(),
                    decision: d.decision.clone(),
                }]
            })
            .unwrap_or_default();

        let termination = match &result.termination {
            LoopTermination::Approved => "Approved".to_string(),
            LoopTermination::MaxIterations => "MaxIterations".to_string(),
            LoopTermination::NoReplan => "NoReplan".to_string(),
            LoopTermination::SpendCapExceeded(cost) => {
                format!("SpendCapExceeded (${cost:.4})")
            }
            LoopTermination::Oscillation => "Oscillation".to_string(),
            LoopTermination::ModelNotConverging(frac) => {
                format!("ModelNotConverging ({:.0}% non-converged)", frac * 100.0)
            }
            LoopTermination::Aborted(msg) => format!("Aborted: {msg}"),
        };

        Self {
            session_id,
            goal: result.goal.clone(),
            verdict: ask_plan.map(|p| p.verdict),
            converged: result.converged,
            termination,
            iterations,
            files_changed,
            verify_results,
            decisions,
            dry_run,
            grader_source: result.grader_source.clone(),
        }
    }

    /// Render as markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Header
        let status = if self.converged {
            "CONVERGED"
        } else {
            "NOT CONVERGED"
        };
        md.push_str(&format!("# Agent Loop Changelog: {}\n\n", self.session_id));
        md.push_str(&format!("**Status:** {}\n", status));
        md.push_str(&format!("**Termination:** {}\n", self.termination));
        if self.dry_run {
            md.push_str("**Dry run:** true\n");
        }
        md.push_str(&format!("**Grader:** {}\n", self.grader_source));
        md.push_str(&format!("**Goal:** {}\n\n", self.goal));

        // Verdict block
        if let Some(verdict) = self.verdict {
            md.push_str("## Calibration Verdict\n\n");
            md.push_str(&format!("- **Verdict:** {:?}\n", verdict));
            md.push('\n');
        }

        // Iterations table
        if !self.iterations.is_empty() {
            md.push_str("## Iterations\n\n");
            md.push_str("| # | Replanned | Subtasks | Succeeded | Failed | Critique | Score | Verify Failures |\n");
            md.push_str("|---|-----------|----------|-----------|--------|----------|-------|-----------------|\n");
            for iter in &self.iterations {
                let mark = if iter.critique_approved {
                    "PASS"
                } else {
                    "FAIL"
                };
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} | {:.1} | {} |\n",
                    iter.iteration,
                    if iter.replanned { "yes" } else { "no" },
                    iter.subtask_count,
                    iter.succeeded,
                    iter.failed,
                    mark,
                    iter.critique_score,
                    iter.verify_failed.len(),
                ));
            }
            md.push('\n');
        }

        // Files changed
        if !self.files_changed.is_empty() {
            md.push_str("## Files Changed\n\n");
            for f in &self.files_changed {
                md.push_str(&format!("- `{f}`\n"));
            }
            md.push('\n');
        }

        // Verification results
        if !self.verify_results.is_empty() {
            md.push_str("## Verification Results\n\n");
            md.push_str("| Step | Result |\n");
            md.push_str("|------|--------|\n");
            for v in &self.verify_results {
                let res = if v.ok { "PASS" } else { "FAIL" };
                md.push_str(&format!("| {} | {} |\n", v.step, res));
            }
            md.push('\n');
        }

        // Decisions
        if !self.decisions.is_empty() {
            md.push_str("## Decision Records\n\n");
            for d in &self.decisions {
                md.push_str(&format!("### {}\n\n", d.title));
                md.push_str(&format!("**Context:** {}\n\n", d.context));
                md.push_str(&format!("**Decision:** {}\n\n", d.decision));
            }
        }

        md
    }

    /// Filename for this changelog (e.g. `changelog-20260629-143012.md`).
    pub fn filename(&self) -> String {
        format!("changelog-{}.md", self.session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::{AskPlan, Reversibility, Verdict};
    use crate::cognition::{
        AgentRunResult, Comprehension, LoopIteration, LoopResult, LoopTermination, Plan,
        StepResult, StepStatus, Subtask, SubtaskKind, TaskComplexity,
    };
    use crate::llm::{TaskTier, Usage};

    fn fixture_loop_result(converged: bool) -> LoopResult {
        LoopResult {
            goal: "Add observability events".into(),
            iterations: vec![LoopIteration {
                iteration: 1,
                replanned: false,
                plan_goal: "Add observability events".into(),
                subtask_count: 2,
                succeeded: 2,
                failed: 0,
                critique_approved: true,
                critique_score: 0.9,
                critique_issues: vec![],
                verify_failed: vec![],
                injected_learning_ids: vec![],
                usage: Usage::default(),
                cost_usd: 0.0,
                plan_parse_error: None,
                incorporation_gap: None,
            }],
            converged,
            termination: if converged {
                LoopTermination::Approved
            } else {
                LoopTermination::MaxIterations
            },
            total_usage: Usage::default(),
            grader_source: "default".into(),
            final_result: AgentRunResult {
                goal: "Add observability events".into(),
                comprehension: Comprehension {
                    goal: "Add observability events".into(),
                    summary: "Add event stream".into(),
                    cited_elements: vec![],
                    key_findings: vec![],
                    risks: vec![],
                    usage: Usage::default(),
                    retrieved_learning_ids: vec![],
                    complexity: TaskComplexity::Moderate,
                },
                plan: Plan {
                    goal: "Add observability events".into(),
                    goal_statement: "Add observability events".into(),
                    criteria: vec![],
                    subtasks: vec![
                        Subtask {
                            id: "s1".into(),
                            description: "Create loop_event.rs".into(),
                            tier: TaskTier::Mid,
                            kind: SubtaskKind::Implement,
                            files: vec!["src/loop_event.rs".into()],
                            acceptance_criteria: vec![],
                        },
                        Subtask {
                            id: "s2".into(),
                            description: "Wire events into run_loop".into(),
                            tier: TaskTier::Mid,
                            kind: SubtaskKind::Implement,
                            files: vec!["src/mod.rs".into(), "src/loop_event.rs".into()],
                            acceptance_criteria: vec![],
                        },
                    ],
                    tdd: false,
                    risks: vec![],
                    schema_version: "1.0".into(),
                    complexity: TaskComplexity::Moderate,
                },
                step_results: vec![
                    StepResult {
                        subtask_id: "s1".into(),
                        status: StepStatus::Ok,
                        output: "done".into(),
                        usage: Usage::default(),
                        tool_signals: vec![],
                        converged: true,
                    },
                    StepResult {
                        subtask_id: "s2".into(),
                        status: StepStatus::Ok,
                        output: "done".into(),
                        usage: Usage::default(),
                        tool_signals: vec![],
                        converged: true,
                    },
                ],
                critique: None,
                decision: None,
                runbook: None,
                total_usage: Usage::default(),
            },
        }
    }

    fn fixture_ask_plan(verdict: Verdict) -> AskPlan {
        AskPlan {
            verdict,
            reason: "test".into(),
            reversibility: Reversibility::TwoWay,
            blast_radius: 3,
            confidence: Some(80),
            trust_level: Some(70),
            has_precedent: false,
            policy_says_ask: false,
        }
    }

    #[test]
    fn full_run_markdown_contains_all_sections() {
        let result = fixture_loop_result(true);
        let cl = AgentChangelog::from_loop(
            &result,
            Some(&fixture_ask_plan(Verdict::ProceedSilent)),
            false,
        );
        let md = cl.to_markdown();

        assert!(md.contains("Agent Loop Changelog"));
        assert!(md.contains("CONVERGED"));
        assert!(md.contains("Calibration Verdict"));
        assert!(md.contains("Iterations"));
        assert!(md.contains("Files Changed"));
        assert!(md.contains("`src/loop_event.rs`"));
        assert!(md.contains("`src/mod.rs`"));
    }

    #[test]
    fn not_converged_markdown_reflects_status() {
        let result = fixture_loop_result(false);
        let cl = AgentChangelog::from_loop(&result, None, false);
        let md = cl.to_markdown();

        assert!(md.contains("NOT CONVERGED"));
        assert!(md.contains("MaxIterations"));
    }

    #[test]
    fn dry_run_marker_present() {
        let result = fixture_loop_result(true);
        let cl = AgentChangelog::from_loop(&result, None, true);
        let md = cl.to_markdown();

        assert!(md.contains("**Dry run:** true"));
    }

    #[test]
    fn verify_failures_recorded() {
        let mut result = fixture_loop_result(false);
        result.iterations[0].verify_failed = vec!["cargo test failed".into()];
        let cl = AgentChangelog::from_loop(&result, None, false);
        let md = cl.to_markdown();

        assert!(md.contains("Verification Results"));
        assert!(md.contains("cargo test failed"));
        assert!(md.contains("FAIL"));
    }

    #[test]
    fn filename_is_well_formed() {
        let result = fixture_loop_result(true);
        let cl = AgentChangelog::from_loop(&result, None, false);
        let name = cl.filename();

        assert!(name.starts_with("changelog-"));
        assert!(name.ends_with(".md"));
    }

    #[test]
    fn files_changed_are_deduplicated() {
        let result = fixture_loop_result(true);
        let cl = AgentChangelog::from_loop(&result, None, false);

        // src/loop_event.rs appears in two subtasks but should be listed once
        let count = cl
            .files_changed
            .iter()
            .filter(|f| f == &"src/loop_event.rs")
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn no_verdict_omits_verdict_section() {
        let result = fixture_loop_result(true);
        let cl = AgentChangelog::from_loop(&result, None, false);
        let md = cl.to_markdown();

        assert!(!md.contains("Calibration Verdict"));
    }
}

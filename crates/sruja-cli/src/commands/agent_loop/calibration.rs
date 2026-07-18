//! Calibration gate and related utilities for the agent loop.

use std::path::Path;

use sruja_agent::calibration::{self, AskInput, Thresholds};

/// Outcome of the pre-flight calibration gate.
#[derive(Debug)]
pub(crate) enum GateOutcome {
    /// Calibration says halt — human approval required.
    Halt { reason: String },
    /// Calibration says proceed — optional DR already constructed.
    Proceed {
        plan: Box<sruja_agent::AskPlan>,
        record: Option<Box<sruja_agent::cognition::DecisionRecord>>,
    },
}

/// Pure calibration gate: decides Halt vs Proceed from goal scope + thresholds.
///
/// No async, no LLM, no I/O — fully unit-testable.
pub(crate) fn calibration_gate(
    goal: &str,
    target_elements: &[String],
    target_files: &[String],
    has_precedent: bool,
    thresholds: &Thresholds,
    force_proceed: bool,
) -> GateOutcome {
    // Heuristic blast radius: target elements + target files, saturated at u16::MAX.
    let blast_radius = (target_elements.len() + target_files.len()).min(u16::MAX as usize) as u16;

    // Infer reversibility from the goal text (conservative: keywords).
    let reversibility = calibration::infer_reversibility(calibration::TargetHints {
        kind: "Goal",
        label: goal,
    });

    let input = AskInput {
        reversibility,
        blast_radius,
        confidence: None,
        trust_level: None,
        has_precedent,
        policy_says_ask: false,
    };

    let plan = calibration::decide(&input, thresholds);

    if plan.verdict.should_ask() {
        if force_proceed {
            // Forced bypass — proceed but write no calibration DR.
            GateOutcome::Proceed {
                plan: Box::new(plan),
                record: None,
            }
        } else {
            GateOutcome::Halt {
                reason: plan.reason.clone(),
            }
        }
    } else {
        let record = sruja_agent::proceed_decision_record(&plan, goal).map(Box::new);
        GateOutcome::Proceed {
            plan: Box::new(plan),
            record,
        }
    }
}

/// Check whether agentic memory contains a precedent learning *relevant to
/// this goal*. Scoped to avoid a single global precedent from unlocking every
/// one-way-door goal. Relevance is a simple text-contains match on the goal
/// or target element IDs — consistent with `Memory::search` semantics.
pub(crate) fn has_goal_precedent(repo_path: &Path, goal: &str, target_elements: &[String]) -> bool {
    let mem = match sruja_agent::AgenticMemory::load(repo_path) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let goal_lower = goal.to_lowercase();
    mem.learnings.iter().any(|l| {
        if l.hitl_kind.as_deref() != Some("precedent") {
            return false;
        }
        // Precedent is relevant if any target element matches, or if the goal
        // text overlaps with the learning's context/hypothesis.
        let ctx_lower = l.context.to_lowercase();
        let hyp_lower = l.hypothesis.to_lowercase();
        target_elements
            .iter()
            .any(|e| ctx_lower.contains(&e.to_lowercase()) || hyp_lower.contains(&e.to_lowercase()))
            || ctx_lower.contains(&goal_lower)
            || hyp_lower.contains(&goal_lower)
            || goal_lower.contains(&ctx_lower)
    })
}

/// Extract the alphabetic family name from a model identifier.
/// Used for provider prefix routing in the TieredClient.
///   "GLM-5.2" → "glm", "mimo-v2.5-pro" → "mimo",
///   "anthropic/claude-sonnet-4" → "claude"
pub(crate) fn model_family(model: &str) -> String {
    let base = model.rsplit('/').next().unwrap_or(model);
    base.chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_agent::Verdict;

    fn t() -> Thresholds {
        Thresholds::default()
    }

    #[test]
    fn one_way_door_no_precedent_no_force_halts() {
        let goal = "migrate the database schema";
        let outcome = calibration_gate(goal, &[], &[], false, &t(), false);
        match outcome {
            GateOutcome::Halt { reason } => assert!(reason.contains("One-way door")),
            GateOutcome::Proceed { .. } => expected_halt(),
        }
    }

    #[test]
    fn one_way_door_no_precedent_force_proceeds_without_dr() {
        let goal = "migrate the database schema";
        let outcome = calibration_gate(goal, &[], &[], false, &t(), true);
        match outcome {
            GateOutcome::Proceed { plan, record } => {
                assert_eq!(plan.verdict, Verdict::Ask);
                assert!(record.is_none(), "forced bypass should write no DR");
            }
            GateOutcome::Halt { .. } => expected_proceed(),
        }
    }

    #[test]
    fn two_way_door_bounded_blast_proceeds_silent_no_dr() {
        let goal = "rename a variable in the handler";
        let outcome = calibration_gate(goal, &[], &[], false, &t(), false);
        match outcome {
            GateOutcome::Proceed { plan, record } => {
                assert_eq!(plan.verdict, Verdict::ProceedSilent);
                assert!(record.is_none(), "ProceedSilent should write no DR");
            }
            GateOutcome::Halt { .. } => expected_proceed(),
        }
    }

    #[test]
    fn mid_confidence_proceeds_with_flag_and_dr() {
        // Mid-confidence requires a confidence signal; with None (unmeasured)
        // on a two-way door we get ProceedSilent. With precedent we get
        // ProceedCitingPrecedent and a DR. Test the precedent path.
        let goal = "refactor API handler";
        let outcome = calibration_gate(goal, &[], &[], true, &t(), false);
        match outcome {
            GateOutcome::Proceed { plan, record } => {
                assert_eq!(plan.verdict, Verdict::ProceedCitingPrecedent);
                assert!(record.is_some(), "ProceedCitingPrecedent should write a DR");
            }
            GateOutcome::Halt { .. } => expected_proceed(),
        }
    }

    #[test]
    fn precedent_proceeds_with_dr() {
        let goal = "delete old migration files";
        let outcome = calibration_gate(goal, &[], &[], true, &t(), false);
        match outcome {
            GateOutcome::Proceed { plan, record } => {
                assert_eq!(plan.verdict, Verdict::ProceedCitingPrecedent);
                assert!(record.is_some(), "precedent path should write a DR");
            }
            GateOutcome::Halt { .. } => expected_proceed(),
        }
    }

    fn expected_halt() {
        panic!("expected Halt but got Proceed");
    }

    fn expected_proceed() {
        panic!("expected Proceed but got Halt");
    }
}

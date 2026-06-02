//! Health score calculation from violations.
//!
//! Scoring Philosophy:
//! - Score must reflect REALITY: meaningful spread, not "everything above 90."
//! - Red flags (cycles, layer violations) deduct heavily.
//! - Orphans and god modules also deduct proportionally (capped) so that repos
//!   with many issues score lower than clean ones. 90+ means "few issues."
//! - Tests, examples, tools, and doc paths are excluded from violation counting
//!   (see drift.rs is_likely_doc_or_tool_path).
//! - Floor at 30 so truly problematic codebases can be identified;
//!   0-30 = Critical, 31-50 = Poor, 51-65 = Fair, 66-80 = Good, 81+ = Excellent.

use crate::types::{HealthScorePenalties, Severity, Violation, ViolationKind};

const MIN_SCORE: u8 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthGrade {
    Critical,
    Poor,
    Fair,
    Good,
    Excellent,
}

impl HealthGrade {
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=30 => HealthGrade::Critical,
            31..=50 => HealthGrade::Poor,
            51..=65 => HealthGrade::Fair,
            66..=80 => HealthGrade::Good,
            _ => HealthGrade::Excellent,
        }
    }
}

pub struct HealthScoreBreakdown {
    pub score: u8,
    pub grade: HealthGrade,
    pub god_module_penalty: u8,
    pub zone_of_pain_penalty: u8,
    pub coupling_penalty: u8,
    pub cycle_penalty: u8,
    pub orphan_penalty: u8,
    pub other_penalty: u8,
}

pub fn calculate_health_score_from_violations(
    violations: &[Violation],
    penalties: HealthScorePenalties,
) -> u8 {
    let breakdown = calculate_health_score_with_breakdown(violations, penalties);
    breakdown.score
}

pub fn calculate_health_score_with_breakdown(
    violations: &[Violation],
    penalties: HealthScorePenalties,
) -> HealthScoreBreakdown {
    let mut cycle_count: usize = 0;
    let mut orphan_count: usize = 0;
    let mut god_module_count: usize = 0;
    let mut layer_count: usize = 0;
    let mut other_penalty: u8 = 0;

    for v in violations {
        if v.production_relevant == Some(false) {
            continue;
        }

        match v.kind {
            ViolationKind::CircularDependency => cycle_count += 1,
            ViolationKind::OrphanComponent => orphan_count += 1,
            ViolationKind::GodModule => god_module_count += 1,
            ViolationKind::LayerViolation => layer_count += 1,
            _ => {
                let penalty = match v.severity {
                    Severity::Error => penalties.error,
                    Severity::Warning => penalties.warning,
                    Severity::Info => penalties.info,
                };
                other_penalty = other_penalty.saturating_add(penalty);
            }
        }
    }

    let total_issues = cycle_count + orphan_count + god_module_count + layer_count;
    if total_issues == 0 && other_penalty == 0 {
        return HealthScoreBreakdown {
            score: 100,
            grade: HealthGrade::Excellent,
            god_module_penalty: 0,
            zone_of_pain_penalty: 0,
            coupling_penalty: 0,
            cycle_penalty: 0,
            orphan_penalty: 0,
            other_penalty: 0,
        };
    }

    let cycle_penalty = (cycle_count as u8).saturating_mul(2).min(15);
    let layer_penalty = (layer_count as u8).saturating_mul(2).min(15);
    let god_penalty = match god_module_count {
        0 => 0,
        1..=3 => 2,
        4..=10 => 5,
        11..=25 => 8,
        _ => 12,
    };
    let orphan_penalty = match orphan_count {
        0 => 0,
        1..=3 => 1,
        4..=10 => 3,
        _ => 5,
    };
    let other = other_penalty.min(8);

    let total_penalty = cycle_penalty
        .saturating_add(layer_penalty)
        .saturating_add(god_penalty)
        .saturating_add(orphan_penalty)
        .saturating_add(other);

    let score = 100u8.saturating_sub(total_penalty).max(MIN_SCORE);
    let grade = HealthGrade::from_score(score);

    HealthScoreBreakdown {
        score,
        grade,
        god_module_penalty: god_penalty,
        zone_of_pain_penalty: layer_penalty,
        coupling_penalty: 0,
        cycle_penalty,
        orphan_penalty,
        other_penalty: other,
    }
}

#[cfg(test)]
mod tests {
    use super::{calculate_health_score_with_breakdown, HealthGrade};
    use crate::types::{HealthScorePenalties, Severity, Violation};

    #[test]
    fn health_grade_from_score_boundaries() {
        assert_eq!(HealthGrade::from_score(0), HealthGrade::Critical);
        assert_eq!(HealthGrade::from_score(30), HealthGrade::Critical);
        assert_eq!(HealthGrade::from_score(31), HealthGrade::Poor);
        assert_eq!(HealthGrade::from_score(50), HealthGrade::Poor);
        assert_eq!(HealthGrade::from_score(51), HealthGrade::Fair);
        assert_eq!(HealthGrade::from_score(65), HealthGrade::Fair);
        assert_eq!(HealthGrade::from_score(66), HealthGrade::Good);
        assert_eq!(HealthGrade::from_score(80), HealthGrade::Good);
        assert_eq!(HealthGrade::from_score(81), HealthGrade::Excellent);
        assert_eq!(HealthGrade::from_score(100), HealthGrade::Excellent);
    }

    #[test]
    fn calculate_health_score_with_breakdown_empty() {
        let violations: Vec<Violation> = vec![];
        let penalties = HealthScorePenalties::default();
        let b = calculate_health_score_with_breakdown(&violations, penalties);
        assert_eq!(b.score, 100);
        assert_eq!(b.grade, HealthGrade::Excellent);
    }

    #[test]
    fn calculate_health_score_penalizes_cycles_and_orphans() {
        use crate::types::{Violation, ViolationKind};

        fn violation(kind: ViolationKind, message: &str) -> Violation {
            Violation {
                kind,
                severity: Severity::Error,
                message: message.to_string(),
                location: None,
                suggestion: None,
                sources: vec![],
                confidence: None,
                evidence_count: None,
                production_relevant: Some(true),
                baseline_delta: None,
                suppressed: None,
                rule_id: None,
                rationale: None,
            }
        }

        let violations = vec![
            violation(ViolationKind::CircularDependency, "cycle"),
            violation(ViolationKind::OrphanComponent, "orphan"),
        ];
        let breakdown =
            calculate_health_score_with_breakdown(&violations, HealthScorePenalties::default());
        assert!(breakdown.score < 100);
        assert!(breakdown.cycle_penalty > 0);
        assert!(breakdown.orphan_penalty > 0);
    }

    #[test]
    fn non_production_violations_are_excluded_from_score() {
        use crate::types::{Violation, ViolationKind};

        let violations = vec![Violation {
            kind: ViolationKind::CircularDependency,
            severity: Severity::Error,
            message: "test-only cycle".to_string(),
            location: None,
            suggestion: None,
            sources: vec![],
            confidence: None,
            evidence_count: None,
            production_relevant: Some(false),
            baseline_delta: None,
            suppressed: None,
            rule_id: None,
            rationale: None,
        }];
        let breakdown =
            calculate_health_score_with_breakdown(&violations, HealthScorePenalties::default());
        assert_eq!(breakdown.score, 100);
    }
}

//! Health score calculation from violations.
//!
//! Scoring Philosophy:
//! - Score must reflect REALITY: meaningful spread, not "everything above 90."
//! - Red flags (cycles, layer violations) deduct heavily.
//! - Orphans and god modules also deduct proportionally (capped) so that repos
//!   with many issues score lower than clean ones. 90+ means "few issues."
//! - Tests, examples, tools, and doc paths are excluded from violation counting
//!   (see drift.rs is_likely_doc_or_tool_path).
//! - Floor at 40 so truly problematic codebases can be identified;
//!   40-60 = Critical, 60-75 = Poor, 75-85 = Fair, 85-95 = Good, 95+ = Excellent.

use crate::types::{HealthScorePenalties, Severity, Violation, ViolationKind};

const MIN_SCORE: u8 = 30;

const DENSITY_SCALE: usize = 1000;

mod density_thresholds {
    pub const CYCLES_EXCELLENT: f32 = 1.0;
    pub const CYCLES_GOOD: f32 = 5.0;
    pub const CYCLES_FAIR: f32 = 10.0;

    pub const ORPHANS_EXCELLENT: f32 = 10.0;
    pub const ORPHANS_GOOD: f32 = 20.0;
    pub const ORPHANS_FAIR: f32 = 40.0;

    pub const GOD_MODULES_EXCELLENT: f32 = 20.0;
    pub const GOD_MODULES_GOOD: f32 = 75.0;
    pub const GOD_MODULES_FAIR: f32 = 150.0;

    pub const LAYER_EXCELLENT: f32 = 0.5;
    pub const LAYER_GOOD: f32 = 3.0;
    pub const LAYER_FAIR: f32 = 10.0;
}

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

    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            HealthGrade::Critical => "Critical",
            HealthGrade::Poor => "Poor",
            HealthGrade::Fair => "Fair",
            HealthGrade::Good => "Good",
            HealthGrade::Excellent => "Excellent",
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            HealthGrade::Critical => "Immediate attention required",
            HealthGrade::Poor => "Significant issues found",
            HealthGrade::Fair => "Some improvements needed",
            HealthGrade::Good => "Minor issues",
            HealthGrade::Excellent => "Well-architected",
        }
    }
}

#[allow(dead_code)]
pub struct HealthScoreWeights {
    pub god_modules: f64,
    pub zone_of_pain: f64,
    pub coupling: f64,
    pub cycles: f64,
    pub orphaned_modules: f64,
}

impl Default for HealthScoreWeights {
    fn default() -> Self {
        Self {
            god_modules: 15.0,
            zone_of_pain: 5.0,
            coupling: 10.0,
            cycles: 20.0,
            orphaned_modules: 2.0,
        }
    }
}

#[allow(dead_code)]
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

    let _weights = HealthScoreWeights::default();

    let cycle_penalty = (cycle_count as u8).saturating_mul(2).min(15);
    let layer_penalty = (layer_count as u8).saturating_mul(1).min(10);
    let god_penalty = ((god_module_count / 100) as u8).min(10);
    let orphan_penalty = ((orphan_count / 100) as u8).min(5);
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

/// Calculate density (issues per 1000 modules).
#[allow(dead_code)]
fn calculate_density(count: usize, total_modules: usize) -> f32 {
    if total_modules == 0 {
        return 0.0;
    }
    (count as f32 / total_modules as f32) * DENSITY_SCALE as f32
}

/// Score a density value against thresholds.
/// Returns a score component (0-25 for each category).
#[allow(dead_code)]
fn score_density(density: f32, excellent: f32, good: f32, fair: f32) -> u8 {
    if density <= excellent {
        25 // Excellent - no penalty
    } else if density <= good {
        20 // Good - small penalty
    } else if density <= fair {
        15 // Fair - moderate penalty
    } else {
        10 // Needs work - but still give credit
    }
}

/// Extended version that takes module count for density-based scoring.
#[allow(dead_code)]
pub fn calculate_health_score_with_density(
    violations: &[Violation],
    penalties: HealthScorePenalties,
    total_modules: usize,
) -> u8 {
    if total_modules < 100 {
        // For small projects, use simple scoring
        return calculate_health_score_from_violations(violations, penalties);
    }

    // Count violations by kind
    let mut cycle_count: usize = 0;
    let mut orphan_count: usize = 0;
    let mut god_module_count: usize = 0;
    let mut layer_count: usize = 0;

    for v in violations {
        match v.kind {
            ViolationKind::CircularDependency => cycle_count += 1,
            ViolationKind::OrphanComponent => orphan_count += 1,
            ViolationKind::GodModule => god_module_count += 1,
            ViolationKind::LayerViolation => layer_count += 1,
            _ => {}
        }
    }

    // Calculate densities
    let cycle_density = calculate_density(cycle_count, total_modules);
    let orphan_density = calculate_density(orphan_count, total_modules);
    let god_density = calculate_density(god_module_count, total_modules);
    let layer_density = calculate_density(layer_count, total_modules);

    // Score each category (0-25 each, total up to 100)
    let cycle_score = score_density(
        cycle_density,
        density_thresholds::CYCLES_EXCELLENT,
        density_thresholds::CYCLES_GOOD,
        density_thresholds::CYCLES_FAIR,
    );
    let orphan_score = score_density(
        orphan_density,
        density_thresholds::ORPHANS_EXCELLENT,
        density_thresholds::ORPHANS_GOOD,
        density_thresholds::ORPHANS_FAIR,
    );
    let god_score = score_density(
        god_density,
        density_thresholds::GOD_MODULES_EXCELLENT,
        density_thresholds::GOD_MODULES_GOOD,
        density_thresholds::GOD_MODULES_FAIR,
    );
    let layer_score = score_density(
        layer_density,
        density_thresholds::LAYER_EXCELLENT,
        density_thresholds::LAYER_GOOD,
        density_thresholds::LAYER_FAIR,
    );

    let total = cycle_score + orphan_score + god_score + layer_score;

    // Ensure minimum
    total.max(MIN_SCORE)
}

#[cfg(test)]
mod tests {
    use super::{calculate_health_score_with_breakdown, HealthGrade};
    use crate::types::{HealthScorePenalties, Violation};

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
    fn health_grade_label_and_description() {
        assert_eq!(HealthGrade::Critical.label(), "Critical");
        assert_eq!(HealthGrade::Excellent.label(), "Excellent");
        assert!(HealthGrade::Critical.description().len() > 0);
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
    fn calculate_health_score_with_density_small_project() {
        let violations: Vec<Violation> = vec![];
        let penalties = HealthScorePenalties::default();
        let score = super::calculate_health_score_with_density(&violations, penalties, 50);
        assert_eq!(score, 100);
    }
}

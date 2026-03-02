//! Health score calculation from violations.
//!
//! Scoring Philosophy:
//! - Score must reflect REALITY: meaningful spread, not "everything above 90."
//! - Red flags (cycles, layer violations) deduct heavily.
//! - Orphans and god modules also deduct proportionally (capped) so that repos
//!   with many issues score lower than clean ones. 90+ means "few issues."
//! - Tests, examples, tools, and doc paths are excluded from violation counting
//!   (see drift.rs is_likely_doc_or_tool_path).
//! - Floor at 50 so no repo is branded "failed"; but 60–80 = needs work, 80–90 = good, 90+ = excellent.

use crate::types::{HealthScorePenalties, Severity, Violation, ViolationKind};

/// Minimum health score - successful projects deserve respect.
const MIN_SCORE: u8 = 50;

/// Scale factor for density calculation (issues per N modules).
const DENSITY_SCALE: usize = 1000;

/// Density thresholds for scoring (issues per 1000 modules).
/// These are calibrated so that well-maintained projects score 70+.
mod density_thresholds {
    pub const CYCLES_EXCELLENT: f32 = 0.5;
    pub const CYCLES_GOOD: f32 = 2.0;
    pub const CYCLES_FAIR: f32 = 5.0;

    pub const ORPHANS_EXCELLENT: f32 = 1.0;
    pub const ORPHANS_GOOD: f32 = 5.0;
    pub const ORPHANS_FAIR: f32 = 10.0;

    pub const GOD_MODULES_EXCELLENT: f32 = 10.0;
    pub const GOD_MODULES_GOOD: f32 = 50.0;
    pub const GOD_MODULES_FAIR: f32 = 100.0;

    pub const LAYER_EXCELLENT: f32 = 0.0;
    pub const LAYER_GOOD: f32 = 1.0;
    pub const LAYER_FAIR: f32 = 5.0;
}

/// Compute health score using density-based approach.
///
/// Instead of penalizing absolute counts, we calculate issue density
/// (issues per 1000 modules) and score based on how that compares
/// to thresholds calibrated on real successful projects.
pub fn calculate_health_score_from_violations(
    violations: &[Violation],
    penalties: HealthScorePenalties,
) -> u8 {
    // Count violations by kind
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

    // For small projects (< 100 modules), use simpler scoring
    // We can't calculate meaningful density with few modules
    let total_issues = cycle_count + orphan_count + god_module_count + layer_count;
    if total_issues == 0 && other_penalty == 0 {
        return 100;
    }

    // Base 100. Deduct for all issue types so score has REAL SPREAD (not everything 90+).

    // Red flags: cycles (real architectural debt)
    let cycle_penalty = ((cycle_count * 6) as u8).min(30);
    let mut score = 100u8.saturating_sub(cycle_penalty);

    // Red flags: layer violations
    let layer_penalty = ((layer_count * 5) as u8).min(25);
    score = score.saturating_sub(layer_penalty);

    // Orphans: -1 per 2 orphans, max 20. So 0–1=0, 2–3=1, … 40+=20. Creates spread.
    let orphan_penalty = ((orphan_count / 2) as u8).min(20);
    score = score.saturating_sub(orphan_penalty);

    // God modules: -1 per 25, max 20. So 0–24=0, 25–49=1, … 500+=20.
    let god_penalty = ((god_module_count / 25) as u8).min(20);
    score = score.saturating_sub(god_penalty);

    // Other
    score = score.saturating_sub(other_penalty.min(10));

    // Ensure minimum score - respect that this code exists and works
    score.max(MIN_SCORE)
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

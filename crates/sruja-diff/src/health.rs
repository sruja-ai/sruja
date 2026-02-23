//! Health score calculation from violations.

use crate::types::{HealthScorePenalties, Severity, Violation};

/// Compute health score from violations using unified penalty scheme.
pub fn calculate_health_score_from_violations(
    violations: &[Violation],
    penalties: HealthScorePenalties,
) -> u8 {
    let mut score: u8 = 100;
    for v in violations {
        match v.severity {
            Severity::Error => score = score.saturating_sub(penalties.error),
            Severity::Warning => score = score.saturating_sub(penalties.warning),
            Severity::Info => score = score.saturating_sub(penalties.info),
        }
    }
    score
}

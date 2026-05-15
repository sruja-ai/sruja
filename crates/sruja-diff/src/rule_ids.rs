//! Stable rule identifiers for deterministic architecture findings.

use crate::types::ViolationKind;

/// Stable rule code for a violation kind (used in CI, agents, and remediation playbooks).
#[must_use]
pub fn rule_id_for_kind(kind: ViolationKind) -> &'static str {
    match kind {
        ViolationKind::LayerViolation => "SRUJA-LAYER-001",
        ViolationKind::MissingDependency => "SRUJA-DEPS-001",
        ViolationKind::OrphanComponent => "SRUJA-ORPHAN-001",
        ViolationKind::CircularDependency => "SRUJA-CYCLE-001",
        ViolationKind::UndocumentedComponent => "SRUJA-DOC-001",
        ViolationKind::PatternMismatch => "SRUJA-PATTERN-001",
        ViolationKind::GodModule => "SRUJA-GOD-001",
    }
}

/// Default confidence for scan-derived structural findings (0.0–1.0).
#[must_use]
pub fn default_confidence_for_kind(kind: ViolationKind) -> f32 {
    match kind {
        ViolationKind::CircularDependency | ViolationKind::LayerViolation => 0.95,
        ViolationKind::GodModule | ViolationKind::OrphanComponent => 0.85,
        ViolationKind::UndocumentedComponent | ViolationKind::PatternMismatch => 0.75,
        ViolationKind::MissingDependency => 0.80,
    }
}
